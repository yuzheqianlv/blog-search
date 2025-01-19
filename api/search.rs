use std::collections::HashMap;
use vercel_runtime::{
    Body, Error, Request, Response, StatusCode,
    http::header::{HeaderMap, HeaderValue},
};
use blog_search_service::{SearchEngine, SearchDoc};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Error> {
    Ok(vercel_runtime::run(handler).await?)
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    // 设置 CORS 头
    let mut headers = HeaderMap::new();
    headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
    headers.insert("Access-Control-Allow-Methods", HeaderValue::from_static("GET, OPTIONS"));
    headers.insert("Access-Control-Allow-Headers", HeaderValue::from_static("Content-Type"));

    // 处理 OPTIONS 请求
    if req.method() == "OPTIONS" {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .headers(headers)
            .body(Body::Empty)?);
    }

    // 解析查询参数
    let query_params: HashMap<String, String> = req.uri()
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_default();

    let query = query_params.get("q").cloned().unwrap_or_default();
    
    // 初始化搜索引擎
    let search_engine = match SearchEngine::new("./data/search_index") {
        Ok(engine) => engine,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .headers(headers)
                .body(Body::Text(format!("搜索引擎初始化失败: {}", e)))?);
        }
    };

    // 执行搜索
    match search_engine.search(&query) {
        Ok(results) => {
            let json = json!({
                "results": results,
                "query": query,
            });

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .headers(headers)
                .body(Body::Text(json.to_string()))?)
        }
        Err(e) => {
            Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .headers(headers)
                .body(Body::Text(format!("搜索失败: {}", e)))?)
        }
    }
} 