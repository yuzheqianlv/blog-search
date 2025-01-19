use serde::{Deserialize, Serialize};
use std::path::Path;
use tantivy::{
    collector::TopDocs,
    directory::MmapDirectory,
    doc,
    query::{QueryParser, QueryParserError},
    schema::{Schema, STORED, TEXT},
    Index,
    TantivyError,
    directory::error::OpenDirectoryError,
};
use thiserror::Error;

/// 默认的写入器内存限制（50MB）
const DEFAULT_WRITER_MEMORY: usize = 50_000_000;
/// 默认的搜索结果数量限制
const DEFAULT_SEARCH_LIMIT: usize = 10;
/// 默认的摘要长度（单词数）
const DEFAULT_EXCERPT_LENGTH: usize = 50;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("索引错误: {0}")]
    IndexError(#[from] TantivyError),
    #[error("解析错误: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
    #[error("查询解析错误: {0}")]
    QueryError(#[from] QueryParserError),
    #[error("目录错误: {0}")]
    DirectoryError(#[from] OpenDirectoryError),
    #[error("无效的文档格式")]
    InvalidDocument,
    #[error("字段不存在: {0}")]
    FieldNotFound(String),
}

pub type SearchResult<T> = std::result::Result<T, SearchError>;

/// 搜索结果文档
#[derive(Serialize, Deserialize)]
pub struct SearchDoc {
    /// 文档标题
    pub title: String,
    /// 文档路径
    pub path: String,
    /// 文档摘要
    pub excerpt: String,
    /// 文档标签
    pub tags: Vec<String>,
}

/// 搜索引擎核心结构
pub struct SearchEngine {
    index: Index,
    schema: Schema,
    title_field: tantivy::schema::Field,
    content_field: tantivy::schema::Field,
    path_field: tantivy::schema::Field,
    tags_field: tantivy::schema::Field,
}

impl SearchEngine {
    /// 创建新的搜索引擎实例
    /// 
    /// # Arguments
    /// * `index_path` - 索引文件存储路径
    /// 
    /// # Returns
    /// * `SearchResult<Self>` - 搜索引擎实例或错误
    pub fn new(index_path: &str) -> SearchResult<Self> {
        let mut schema_builder = Schema::builder();
        let title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let content_field = schema_builder.add_text_field("content", TEXT);
        let path_field = schema_builder.add_text_field("path", STORED);
        let tags_field = schema_builder.add_text_field("tags", STORED);
        let schema = schema_builder.build();
        
        // 创建索引目录
        let index_path = Path::new(index_path);
        if !index_path.exists() {
            std::fs::create_dir_all(index_path)?;
        }
        
        // 使用 MmapDirectory
        let mmap_dir = MmapDirectory::open(index_path)?;
        
        let index = if index_path.join("meta.json").exists() {
            Index::open(mmap_dir)?
        } else {
            // 使用 create_in_dir 替代 create_with_settings
            Index::create_in_dir(index_path, schema.clone())?
        };
        
        // 确保索引可用
        let _reader = index.reader()?;
        
        Ok(SearchEngine { 
            index,
            schema,
            title_field,
            content_field,
            path_field,
            tags_field,
        })
    }

    /// 索引一篇文档
    /// 
    /// # Arguments
    /// * `content` - 文档内容，包含 front matter
    /// * `file_path` - 文档路径
    /// 
    /// # Returns
    /// * `SearchResult<()>` - 成功或错误
    pub fn index_document(&self, content: &str, file_path: &Path) -> SearchResult<()> {
        let mut writer = self.index.writer(DEFAULT_WRITER_MEMORY)?;
        
        // 解析 Markdown 文件的 front matter
        let (front_matter, content) = if let Some(_) = content.strip_prefix("+++") {
            if let Some(end) = content[3..].find("+++") {
                let front_matter = &content[3..end + 3];
                let content = &content[end + 6..];
                (front_matter.to_string(), content.to_string())
            } else {
                return Err(SearchError::InvalidDocument);
            }
        } else {
            return Err(SearchError::InvalidDocument);
        };
        
        // 解析 front matter
        let front_matter: toml::Value = toml::from_str(&front_matter)?;
        
        let title = front_matter
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SearchError::FieldNotFound("title".to_string()))?;
            
        let tags = front_matter
            .get("taxonomies")
            .and_then(|v| v.get("tags"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_default();

        writer.add_document(doc!(
            self.title_field => title,
            self.content_field => content,
            self.path_field => file_path.to_string_lossy().to_string(),
            self.tags_field => tags
        ))?;
        
        writer.commit()?;
        
        // 重新加载读取器
        let _reader = self.index.reader()?;
        
        Ok(())
    }

    /// 搜索文档
    /// 
    /// # Arguments
    /// * `query_str` - 搜索查询字符串
    /// 
    /// # Returns
    /// * `SearchResult<Vec<SearchDoc>>` - 搜索结果或错误
    pub fn search(&self, query_str: &str) -> SearchResult<Vec<SearchDoc>> {
        if query_str.is_empty() {
            return Ok(Vec::new());
        }

        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![self.title_field, self.content_field],
        );
        
        let query = query_parser.parse_query(query_str)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(DEFAULT_SEARCH_LIMIT))?;
        
        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let doc = searcher.doc(doc_address)?;
            
            let result = SearchDoc {
                title: doc.get_first(self.title_field)
                    .and_then(|f| f.as_text())
                    .unwrap_or("")
                    .to_string(),
                path: doc.get_first(self.path_field)
                    .and_then(|f| f.as_text())
                    .unwrap_or("")
                    .to_string(),
                excerpt: doc.get_first(self.content_field)
                    .and_then(|f| f.as_text())
                    .map(|content| {
                        // 使用常量定义摘要长度
                        let words: Vec<&str> = content.split_whitespace().collect();
                        if words.len() > DEFAULT_EXCERPT_LENGTH {
                            words[..DEFAULT_EXCERPT_LENGTH].join(" ") + "..."
                        } else {
                            words.join(" ")
                        }
                    })
                    .unwrap_or_default(),
                tags: doc.get_first(self.tags_field)
                    .and_then(|f| f.as_text())
                    .map(|t| t.split(',').map(|s| s.to_string()).collect())
                    .unwrap_or_default(),
            };
            results.push(result);
        }
        
        Ok(results)
    }

    /// 清空索引
    /// 
    /// 删除索引中的所有文档。这个操作不可撤销。
    /// 
    /// # Returns
    /// * `SearchResult<()>` - 成功或错误
    pub fn clear(&self) -> SearchResult<()> {
        let mut writer = self.index.writer(DEFAULT_WRITER_MEMORY)?;
        writer.delete_all_documents()?;
        writer.commit()?;
        Ok(())
    }

    /// 获取索引中的文档数量
    /// 
    /// # Returns
    /// * `SearchResult<u64>` - 文档数量或错误
    pub fn doc_count(&self) -> SearchResult<u64> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        Ok(searcher.num_docs())
    }

    /// 检查索引是否存在
    pub fn exists(&self) -> bool {
        self.index.reader().is_ok()
    }

    /// 获取索引统计信息
    pub fn stats(&self) -> SearchResult<IndexStats> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        Ok(IndexStats {
            doc_count: searcher.num_docs(),
            field_count: self.schema.fields().count() as u64,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct IndexStats {
    pub doc_count: u64,
    pub field_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// 清理重试次数
    const CLEANUP_RETRY_COUNT: u32 = 3;
    /// 清理重试间隔（毫秒）
    const CLEANUP_RETRY_INTERVAL: u64 = 100;

    mod setup {
        use super::*;

        pub(crate) fn setup_test_dir(name: &str) -> SearchResult<PathBuf> {
            let test_dir = PathBuf::from("target").join("test_indexes").join(name);
            if test_dir.exists() {
                cleanup_test_dir(&test_dir)?;
            }
            fs::create_dir_all(&test_dir)?;
            Ok(test_dir)
        }

        pub(crate) fn cleanup_test_dir(dir: &Path) -> SearchResult<()> {
            if dir.exists() {
                for _ in 0..CLEANUP_RETRY_COUNT {
                    match fs::remove_dir_all(dir) {
                        Ok(_) => break,
                        Err(e) if e.kind() == std::io::ErrorKind::DirectoryNotEmpty => {
                            std::thread::sleep(std::time::Duration::from_millis(CLEANUP_RETRY_INTERVAL));
                            continue;
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
            }
            Ok(())
        }
    }

    mod indexing {
        use super::*;
        use super::setup::*;

        #[test]
        fn test_document_indexing() -> SearchResult<()> {
            let test_dir = setup_test_dir("index_doc")?;
            let engine = SearchEngine::new(test_dir.to_str().unwrap())?;
            
            let content = r#"+++
            title = "Test Document"
            [taxonomies]
            tags = ["test", "rust"]
            +++
            This is a test document."#;
            
            engine.index_document(content, &PathBuf::from("test.md"))?;
            
            let results = engine.search("test")?;
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].title, "Test Document");
            
            cleanup_test_dir(&test_dir)?;
            Ok(())
        }

        #[test]
        fn test_invalid_front_matter() -> SearchResult<()> {
            let test_dir = setup_test_dir("index_invalid")?;
            let engine = SearchEngine::new(test_dir.to_str().unwrap())?;
            
            let content = r#"+++
            invalid toml content
            +++
            Test content"#;
            
            assert!(engine.index_document(content, &PathBuf::from("invalid.md")).is_err());
            
            cleanup_test_dir(&test_dir)?;
            Ok(())
        }
    }

    mod searching {
        use super::*;
        use super::setup::*;

        #[test]
        fn test_search_with_empty_query() -> SearchResult<()> {
            let test_dir = setup_test_dir("index_empty")?;
            let engine = SearchEngine::new(test_dir.to_str().unwrap())?;
            
            let results = engine.search("")?;
            assert!(results.is_empty());
            
            cleanup_test_dir(&test_dir)?;
            Ok(())
        }

        #[test]
        fn test_search_with_limit() -> SearchResult<()> {
            let test_dir = setup_test_dir("index_limit")?;
            let engine = SearchEngine::new(test_dir.to_str().unwrap())?;
            
            for i in 1..=15 {
                let content = format!(
                    r#"+++
                    title = "Test Document {}"
                    [taxonomies]
                    tags = ["test"]
                    +++
                    This is test document {}."#,
                    i, i
                );
                engine.index_document(&content, &PathBuf::from(format!("test{}.md", i)))?;
            }
            
            let results = engine.search("test")?;
            assert_eq!(results.len(), DEFAULT_SEARCH_LIMIT);
            
            drop(engine);
            cleanup_test_dir(&test_dir)?;
            Ok(())
        }

        #[test]
        fn test_search_by_content() -> SearchResult<()> {
            let test_dir = setup_test_dir("index_content")?;
            let engine = SearchEngine::new(test_dir.to_str().unwrap())?;
            
            let content = r#"+++
            title = "Test Document"
            [taxonomies]
            tags = ["test"]
            +++
            This document contains unique_keyword."#;
            
            engine.index_document(content, &PathBuf::from("test.md"))?;
            
            let results = engine.search("unique_keyword")?;
            assert_eq!(results.len(), 1);
            
            cleanup_test_dir(&test_dir)?;
            Ok(())
        }
    }

    mod maintenance {
        use super::*;
        use super::setup::*;

        #[test]
        fn test_clear_index() -> SearchResult<()> {
            let test_dir = setup_test_dir("index_clear")?;
            let engine = SearchEngine::new(test_dir.to_str().unwrap())?;
            
            let content = r#"+++
            title = "Test Document"
            [taxonomies]
            tags = ["test"]
            +++
            Test content"#;
            
            engine.index_document(content, &PathBuf::from("test.md"))?;
            assert_eq!(engine.doc_count()?, 1);
            
            engine.clear()?;
            assert_eq!(engine.doc_count()?, 0);
            
            cleanup_test_dir(&test_dir)?;
            Ok(())
        }

        #[test]
        fn test_index_stats() -> SearchResult<()> {
            let test_dir = setup_test_dir("index_stats")?;
            let engine = SearchEngine::new(test_dir.to_str().unwrap())?;
            
            let stats = engine.stats()?;
            assert_eq!(stats.doc_count, 0);
            assert_eq!(stats.field_count, 4); // title, content, path, tags
            
            cleanup_test_dir(&test_dir)?;
            Ok(())
        }
    }
} 