//! 静态文件服务
//!
//! 提供嵌入式 SDK JS 和 demo 页面。

use axum::http::StatusCode;
use axum::response::IntoResponse;

/// SDK 文件（编译时嵌入）
const PRINTCRAFT_JS: &str = include_str!("../../../../sdk/dist/printcraft.js");

/// SDK JS 文件服务
pub async fn serve_sdk_js() -> impl IntoResponse {
    tracing::info!("SDK JS 文件被请求，大小: {} bytes", PRINTCRAFT_JS.len());
    (
        StatusCode::OK,
        [("content-type", "application/javascript; charset=utf-8")],
        PRINTCRAFT_JS,
    )
}

/// 兜底路由处理
pub async fn serve() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "PrintCraft: 资源未找到")
}
