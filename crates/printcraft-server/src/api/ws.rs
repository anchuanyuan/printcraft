//! WebSocket 处理器
//!
//! 处理浏览器 SDK 的 WebSocket 连接。
//! 接收打印命令，分发到 service 层处理。

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;

use super::protocol::{WsCommand, WsResponse};
use super::AppState;

/// WebSocket 升级处理
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// 处理单个 WebSocket 连接
async fn handle_socket(mut socket: WebSocket, state: AppState) {
    tracing::info!("新 WebSocket 连接");

    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                let response = handle_command(&text, &state).await;
                if let Ok(resp_json) = serde_json::to_string(&response) {
                    if socket.send(Message::Text(resp_json.into())).await.is_err() {
                        break;
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    tracing::info!("WebSocket 连接关闭");
}

/// 解析并执行单条命令
async fn handle_command(text: &str, state: &AppState) -> WsResponse {
    match serde_json::from_str::<WsCommand>(text) {
        Ok(cmd) => {
            tracing::debug!("收到命令: {} (id={})", cmd.cmd, cmd.id);
            let mut service = state.service.lock().await;
            match service.handle_command(&cmd.cmd, &cmd.args).await {
                Ok(data) => WsResponse::ok(cmd.id, Some(data)),
                Err(e) => WsResponse::error(cmd.id, format!("{}", e)),
            }
        }
        Err(e) => WsResponse::error(uuid::Uuid::new_v4().to_string(), format!("解析错误: {}", e)),
    }
}
