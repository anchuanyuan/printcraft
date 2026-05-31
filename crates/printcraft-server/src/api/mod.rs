//! API 模块
//!
//! 包含 WebSocket 处理、REST 端点、预览页、静态文件服务。

pub mod ws;
pub mod protocol;
pub mod rest;
pub mod static_files;

use std::sync::Arc;

use axum::{Router, routing::get};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

use crate::service::{PreviewStore, PrintService};

/// 组合应用状态
#[derive(Clone)]
pub struct AppState {
    pub service: Arc<Mutex<PrintService>>,
    pub previews: PreviewStore,
}

/// 启动 HTTP + WebSocket 服务
pub async fn start_server(port: u16, service: Arc<Mutex<PrintService>>, previews: PreviewStore) -> anyhow::Result<()> {
    let state = AppState {
        service,
        previews,
    };

    let app = Router::new()
        // REST API
        .route("/api/printers", get(rest::list_printers))
        .route("/api/status", get(rest::get_status))
        // 预览 API
        .route("/api/preview/{id}/html", get(rest::preview_html))
        .route("/api/preview/{id}/pdf", get(rest::preview_pdf))
        // 预览 UI 页面
        .route("/preview/{id}", get(preview_page))
        // SDK JS 文件
        .route("/sdk/printcraft.js", get(static_files::serve_sdk_js))
        // WebSocket
        .route("/ws", get(ws::websocket_handler))
        // 静态文件兜底
        .fallback(static_files::serve)
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("PrintCraft 服务启动: http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 预览 UI 页面
async fn preview_page() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../../../../sdk/preview/index.html"))
}
