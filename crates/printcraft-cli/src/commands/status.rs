//! status 命令 — 检查 PrintCraft 服务状态

use reqwest;

pub async fn status(base_url: &str) -> anyhow::Result<()> {
    let url = format!("{}/api/status", base_url);

    match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => {
            let data: serde_json::Value = resp.json().await?;
            println!("PrintCraft 服务运行中");
            println!("  版本: {}", data["version"]);
            println!("  平台: {} {}", data["platform"], data["arch"]);
            println!("  状态: {}", data["status"]);
        }
        Ok(resp) => {
            println!("服务响应异常: HTTP {}", resp.status());
        }
        Err(_) => {
            println!("无法连接到 {}，服务可能未启动", base_url);
            println!("尝试: printcraft start");
        }
    }

    Ok(())
}
