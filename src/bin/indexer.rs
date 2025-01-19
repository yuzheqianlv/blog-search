use anyhow::Result;
use blog_search_service::SearchEngine;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

fn get_content_path() -> PathBuf {
    let mut content_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    content_path.pop(); // 回到项目根目录
    content_path.join("content").join("blog")
}

fn main() -> Result<()> {
    // 使用绝对路径
    let index_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("search_index");
    let content_path = get_content_path();
    
    println!("索引路径: {}", index_path.display());
    println!("内容路径: {}", content_path.display());
    
    // 确保索引目录存在
    fs::create_dir_all(&index_path)?;
    
    if !content_path.exists() {
        return Err(anyhow::anyhow!(
            "博客内容目录不存在: {}",
            content_path.display()
        ));
    }
    
    let engine = SearchEngine::new(index_path.to_str().unwrap())?;
    
    let mut indexed_count = 0;
    // 遍历博客文章
    for entry in WalkDir::new(&content_path) {
        let entry = entry?;
        if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "md") {
            let content = fs::read_to_string(entry.path())?;
            engine.index_document(&content, entry.path())?;
            println!("已索引: {}", entry.path().display());
            indexed_count += 1;
        }
    }
    
    let stats = engine.stats()?;
    println!("\n索引完成!");
    println!("扫描文章: {} 篇", indexed_count);
    println!("索引文档: {} 篇", stats.doc_count);
    println!("索引字段: {} 个", stats.field_count);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_content_path() {
        let path = get_content_path();
        assert!(path.ends_with("content/blog"));
    }
} 