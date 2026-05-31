//! 静态文件服务
//!
//! 提供嵌入式 SDK JS 和 demo 页面。

use axum::http::StatusCode;

/// 兜底路由处理
pub async fn serve() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "PrintCraft: 资源未找到")
}
