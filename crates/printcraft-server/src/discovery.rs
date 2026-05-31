//! 端口发现
//!
//! 尝试从指定端口开始绑定，找到第一个可用端口。
//! 类比 C-Lodop 的 8000-8005 端口发现机制。

use printcraft_core::error::Result;

/// 在 [start_port, start_port + 5] 范围内找可用端口
pub async fn find_available_port(start_port: u16) -> Result<u16> {
    for port in start_port..=start_port + 5 {
        match tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
            Ok(_listener) => {
                tracing::info!("可用端口: {}", port);
                return Ok(port);
            }
            Err(_) => {
                tracing::debug!("端口 {} 已占用，尝试下一个", port);
            }
        }
    }

    Err(printcraft_core::error::PrintCraftError::Config(
        format!("未找到可用端口 (范围 {}-{})", start_port, start_port + 5)
    ))
}
