use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use blog_search_service::{SearchEngine, SearchError};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // 获取查询参数
    let url = req.uri();
    let query = url.query()
        .and_then(|q| q.split('=').nth(1))
        .unwrap_or("");
    
    // 初始化搜索引擎
    let search_engine = match SearchEngine::new("./data/search_index") {
        Ok(engine) => engine,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(json!({
                    "error": format!("搜索引擎初始化失败: {}", e)
                }).to_string().into())?);
        }
    };
    
    // 执行搜索
    match search_engine.search(query) {
        Ok(results) => {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&results)?.into())?)
        }
        Err(e) => {
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(json!({
                    "error": format!("搜索失败: {}", e)
                }).to_string().into())?)
        }
    }
} 