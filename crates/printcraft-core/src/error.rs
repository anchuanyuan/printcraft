//! PrintCraft 错误类型定义
//!
//! 统一的错误枚举，涵盖所有模块可能产生的错误。

use thiserror::Error;

/// PrintCraft 全局错误类型
#[derive(Error, Debug)]
pub enum PrintCraftError {
    #[error("打印任务错误: {0}")]
    PrintJob(String),

    #[error("打印机错误: {0}")]
    Printer(String),

    #[error("渲染错误: {0}")]
    Render(String),

    #[error("平台错误: {0}")]
    Platform(String),

    #[error("配置错误: {0}")]
    Config(String),

    #[error("WebSocket 错误: {0}")]
    WebSocket(String),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON 序列化错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("打印任务未找到: {0}")]
    JobNotFound(String),

    #[error("打印机未找到: {0}")]
    PrinterNotFound(String),

    #[error("不支持的操作: {0}")]
    Unsupported(String),
}

/// 便捷 Result 类型
pub type Result<T> = std::result::Result<T, PrintCraftError>;
