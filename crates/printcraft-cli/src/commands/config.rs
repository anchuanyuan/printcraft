//! config 命令 — 显示当前配置

use printcraft_core::config::AppConfig;

pub async fn config() -> anyhow::Result<()> {
    let config = AppConfig::load_or_default();

    println!("PrintCraft 配置:");
    println!("  端口: {}", config.port);
    println!("  默认打印机: {}", if config.default_printer.is_empty() { "(系统默认)" } else { &config.default_printer });
    println!("  开机自启: {}", if config.autostart { "是" } else { "否" });
    println!("  日志级别: {}", config.log_level);
    println!("  Chromium: {}", if config.enable_chromium { "启用" } else { "禁用" });
    if !config.chromium_path.is_empty() {
        println!("  Chromium 路径: {}", config.chromium_path);
    }

    Ok(())
}
