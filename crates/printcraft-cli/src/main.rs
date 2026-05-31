//! PrintCraft CLI
//!
//! 命令行工具，用于管理和查询 PrintCraft 服务。
//!
//! 用法:
//!   printcraft status      — 检查服务状态
//!   printcraft printers    — 列出打印机
//!   printcraft start       — 启动服务
//!   printcraft config      — 显示配置

mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "printcraft", version, about = "PrintCraft 打印服务 CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// 服务端口
    #[arg(short, long, default_value = "18000")]
    port: u16,
}

#[derive(Subcommand)]
enum Commands {
    /// 检查 PrintCraft 服务状态
    Status,
    /// 列出系统打印机
    Printers,
    /// 启动 PrintCraft 服务
    Start,
    /// 显示当前配置
    Config,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let base_url = format!("http://127.0.0.1:{}", cli.port);

    match cli.command {
        Commands::Status => commands::status(&base_url).await,
        Commands::Printers => commands::printers(&base_url).await,
        Commands::Start => commands::start(cli.port).await,
        Commands::Config => commands::config().await,
    }
}
