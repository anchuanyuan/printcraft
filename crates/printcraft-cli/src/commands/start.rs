//! start 命令 — 启动 PrintCraft 服务

pub async fn start(port: u16) -> anyhow::Result<()> {
    println!("启动 PrintCraft 服务 (端口 {})...", port);

    // 检查是否已在运行
    let url = format!("http://127.0.0.1:{}/api/status", port);
    if let Ok(resp) = reqwest::get(&url).await {
        if resp.status().is_success() {
            println!("服务已在运行 (端口 {})", port);
            return Ok(());
        }
    }

    // 启动 printcraft-server
    let exe_path = std::env::current_exe()?;
    let server_path = exe_path
        .parent()
        .map(|p| p.join("printcraft-server"))
        .unwrap_or_default();

    if server_path.exists() {
        println!("启动服务进程: {}", server_path.display());
        let _child = std::process::Command::new(&server_path)
            .spawn()
            .map_err(|e| anyhow::anyhow!("启动服务失败: {}", e))?;

        // 等待服务就绪
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        if let Ok(resp) = reqwest::get(&url).await {
            if resp.status().is_success() {
                println!("服务启动成功！端口: {}", port);
            } else {
                println!("服务已启动但状态异常");
            }
        } else {
            println!("服务启动中，请稍后重试...");
        }
    } else {
        println!("未找到服务可执行文件: {}", server_path.display());
        println!("请直接运行: printcraft-server");
    }

    Ok(())
}
