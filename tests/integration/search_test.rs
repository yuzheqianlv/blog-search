use blog_search_service::{SearchEngine, SearchResult};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_index_and_search() -> SearchResult<()> {
    // 设置测试索引目录
    let index_path = "target/test_index";
    fs::create_dir_all(index_path)?;
    
    // 初始化搜索引擎
    let engine = SearchEngine::new(index_path)?;
    
    // 索引测试文章
    let test_files = [
        "test_data/content/blog/test1.md",
        "test_data/content/blog/test2.md",
    ];
    
    for file in test_files {
        let content = fs::read_to_string(file)?;
        engine.index_document(&content, &PathBuf::from(file))?;
    }
    
    // 测试搜索
    let results = engine.search("rust")?;
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|r| r.title == "Rust 编程入门"));
    assert!(results.iter().any(|r| r.title == "搜索引擎实现"));
    
    let results = engine.search("搜索")?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "搜索引擎实现");
    
    Ok(())
}

#[test]
fn test_pagination() -> SearchResult<()> {
    let index_path = "target/test_index_pagination";
    fs::create_dir_all(index_path)?;
    
    let engine = SearchEngine::new(index_path)?;
    
    // 添加多篇测试文章
    for i in 1..=20 {
        let content = format!(
            r#"+++
            title = "Test Article {}"
            [taxonomies]
            tags = ["test"]
            +++
            Content for article {}"#,
            i, i
        );
        engine.index_document(&content, &PathBuf::from(format!("test{}.md", i)))?;
    }
    
    // 测试分页
    let results = engine.search("test")?;
    assert_eq!(results.len(), 20);
    
    Ok(())
} 