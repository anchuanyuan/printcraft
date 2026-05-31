//! PrintCraft 服务入口
//!
//! 启动 HTTP + WebSocket 服务，系统托盘，开机自启管理。

mod api;
mod autostart;
mod discovery;
mod service;
mod tray;

use std::sync::Arc;

use tokio::sync::Mutex;
use tracing_subscriber::EnvFilter;

use service::PrintService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    tracing::info!("PrintCraft 启动中...");

    // 发现可用端口
    let port = discovery::find_available_port(18000).await?;
    tracing::info!("监听端口: {}", port);

    // 创建平台打印实现
    let platform = printcraft_platform::create_platform_printer();
    let platform: Arc<dyn printcraft_platform::PlatformPrinter> = Arc::from(platform);

    // 创建渲染器
    let renderer: Arc<dyn printcraft_render::PdfRenderer> =
        Arc::new(printcraft_render::simple::SimplePdfRenderer::new());

    // 创建预览存储
    let previews = service::create_preview_store();

    // 创建服务并包装为共享状态
    let service = PrintService::new(renderer, platform, previews.clone());
    let state = Arc::new(Mutex::new(service));

    // 初始化系统托盘
    let _tray_icon = match tray::create_tray() {
        Ok((icon, rx)) => {
            tracing::info!("系统托盘已创建");
            // 在后台处理托盘事件
            tokio::spawn(async move {
                loop {
                    if let Ok(event) = rx.try_recv() {
                        match event {
                            tray::TrayEvent::Quit => {
                                tracing::info!("收到退出命令，正在关闭...");
                                std::process::exit(0);
                            }
                            tray::TrayEvent::ToggleAutostart => {
                                match autostart::toggle() {
                                    Ok(enabled) => {
                                        tracing::info!("开机自启: {}", if enabled { "已启用" } else { "已禁用" });
                                    }
                                    Err(e) => tracing::warn!("切换开机自启失败: {}", e),
                                }
                            }
                            _ => {
                                tracing::debug!("托盘事件: {:?}", event);
                            }
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }
            });
            Some(icon)
        }
        Err(e) => {
            tracing::warn!("创建系统托盘失败 (非桌面环境?): {}", e);
            None
        }
    };

    // 显示开机自启状态
    tracing::info!("开机自启: {}", if autostart::is_enabled() { "已启用" } else { "已禁用" });

    // 启动服务
    api::start_server(port, state, previews).await?;

    Ok(())
}
