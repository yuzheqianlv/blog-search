use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;
use blog_search_service::create_app;

#[tokio::test]
async fn test_search_api() {
    // 构建测试 app
    let app = create_app();
    
    // 测试搜索请求
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/search?q=rust")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json.as_array().unwrap().len() > 0);
} 