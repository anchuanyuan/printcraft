//! printers 命令 — 列出系统打印机

pub async fn printers(base_url: &str) -> anyhow::Result<()> {
    let url = format!("{}/api/printers", base_url);

    match reqwest::get(&url).await {
        Ok(resp) if resp.status().is_success() => {
            let data: serde_json::Value = resp.json().await?;
            let printers = data["printers"].as_array();

            match printers {
                Some(list) if !list.is_empty() => {
                    println!("系统打印机 ({}):", list.len());
                    for (i, p) in list.iter().enumerate() {
                        let name = p["name"].as_str().unwrap_or("?");
                        let is_default = p["is_default"].as_bool().unwrap_or(false);
                        let status = p["status"].as_str().unwrap_or("Ready");
                        let marker = if is_default { " ← 默认" } else { "" };
                        println!("  {}. {} [{}]{}", i + 1, name, status, marker);
                    }
                }
                _ => {
                    println!("未发现打印机");
                }
            }
        }
        Ok(resp) => {
            println!("请求失败: HTTP {}", resp.status());
        }
        Err(_) => {
            println!("无法连接到 {}，服务可能未启动", base_url);
        }
    }

    Ok(())
}
