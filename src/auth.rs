use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

const API_KEY: &str = "jjjshop-district-2026";

pub async fn require_api_key(request: Request, next: Next) -> Response {
    let has_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v == API_KEY);

    if !has_key {
        let body = serde_json::json!({"code": -1, "msg": "无效的API Key，请在请求头中传入 X-API-Key"});
        return (StatusCode::UNAUTHORIZED, axum::Json(body)).into_response();
    }

    next.run(request).await
}
