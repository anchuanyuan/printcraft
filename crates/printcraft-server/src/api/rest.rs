//! REST API 端点

use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

use super::AppState;

/// GET /api/printers - 列出打印机
pub async fn list_printers(
    State(state): State<AppState>,
) -> Json<Value> {
    let service = state.service.lock().await;
    match service.list_printers().await {
        Ok(printers) => Json(json!({
            "printers": printers,
        })),
        Err(e) => Json(json!({
            "printers": [],
            "error": format!("{}", e),
        })),
    }
}

/// GET /api/status - 服务状态
pub async fn get_status() -> Json<Value> {
    Json(json!({
        "status": "running",
        "version": env!("CARGO_PKG_VERSION"),
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
    }))
}

/// GET /api/preview/{id}/html - 获取预览 HTML
pub async fn preview_html(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    let store = state.previews.read().await;
    match store.get(&id) {
        Some(entry) => {
            (
                [(header::CONTENT_TYPE, "text/html; charset=utf-8".to_string())],
                entry.html.clone(),
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, "预览不存在或已过期").into_response(),
    }
}

/// GET /api/preview/{id}/pdf - 获取预览 PDF
pub async fn preview_pdf(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    let store = state.previews.read().await;
    match store.get(&id) {
        Some(entry) => {
            let disposition = format!("inline; filename=\"preview_{}.pdf\"", id);
            (
                [
                    (header::CONTENT_TYPE, "application/pdf".to_string()),
                    (header::CONTENT_DISPOSITION, disposition),
                ],
                entry.pdf.clone(),
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, "预览不存在或已过期").into_response(),
    }
}
