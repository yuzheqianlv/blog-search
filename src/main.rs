use axum::{
    extract::Query,
    http::{HeaderName, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, get_service},
    Router,
    Json,
};
use serde::Deserialize;
use std::{
    net::SocketAddr,
    time::Duration,
};
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::{info, error};
use serde_json::json;

use blog_search_service::{SearchEngine, SearchDoc};

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_page")]
    page: usize,
    #[serde(default = "default_size")]
    size: usize,
}

fn default_page() -> usize {
    1
}

fn default_size() -> usize {
    10
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[derive(Debug)]
pub enum AppError {
    SearchEngine(blog_search_service::SearchError),
    InvalidRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::SearchEngine(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("搜索引擎错误: {}", err),
            ),
            AppError::InvalidRequest(err) => (
                StatusCode::BAD_REQUEST,
                format!("无效的请求: {}", err),
            ),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub fn create_app() -> Router {
    let cors = CorsLayer::new()
        .allow_origin([
            "https://your-blog-domain.com".parse::<HeaderValue>().unwrap(),
            "http://localhost:3000".parse::<HeaderValue>().unwrap(),
        ])
        .allow_methods([Method::GET])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("accept"),
        ])
        .max_age(Duration::from_secs(3600));

    Router::new()
        .route("/api/search", get(handle_search))
        .route("/health", get(health_check))
        .nest_service(
            "/static",
            get_service(ServeDir::new("static"))
                .handle_error(|err| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("静态文件服务错误: {}", err),
                    ).into_response()
                }),
        )
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 确保索引目录存在
    let index_path = "./data/search_index";
    std::fs::create_dir_all(index_path).expect("Failed to create index directory");

    // 初始化搜索引擎并建立索引
    info!("初始化搜索引擎...");
    let engine = SearchEngine::new(index_path).expect("Failed to create search engine");
    
    // 索引测试文章
    let content_dir = "./content/blog";
    if std::path::Path::new(content_dir).exists() {
        info!("索引文章目录: {}", content_dir);
        for entry in std::fs::read_dir(content_dir).expect("Failed to read content directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                let content = std::fs::read_to_string(&path).expect("Failed to read file");
                engine.index_document(&content, &path).expect("Failed to index document");
                info!("已索引: {}", path.display());
            }
        }
    }

    let app = create_app();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Starting server on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_search(
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<SearchDoc>>, AppError> {
    info!("收到搜索请求: {:?}", params);

    let search_engine = SearchEngine::new("./data/search_index")
        .map_err(|e| {
            error!("搜索引擎初始化失败: {}", e);
            AppError::SearchEngine(e)
        })?;

    let mut results = search_engine
        .search(&params.q)
        .map_err(|e| {
            error!("搜索失败: {}", e);
            AppError::SearchEngine(e)
        })?;

    info!("搜索结果数量: {}", results.len());

    // 实现分页
    let start = (params.page - 1) * params.size;
    let end = start + params.size;
    results.truncate(end);
    results = results.into_iter().skip(start).collect();

    Ok(Json(results))
} 