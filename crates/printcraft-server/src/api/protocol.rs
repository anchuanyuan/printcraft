//! WebSocket 消息协议
//!
//! 定义客户端 ↔ 服务端的 JSON 消息格式。
//! 对应 Lodop 的 API 调用。

use serde::{Deserialize, Serialize};

/// 客户端 → 服务端 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsCommand {
    /// 请求 ID（用于关联响应）
    pub id: String,
    /// 命令名称（对应 Lodop 方法名）
    pub cmd: String,
    /// 命令参数
    #[serde(default)]
    pub args: serde_json::Value,
}

/// 服务端 → 客户端 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsResponse {
    /// 对应请求的 ID
    pub id: String,
    /// 是否成功
    pub ok: bool,
    /// 返回数据（成功时）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// 错误信息（失败时）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl WsResponse {
    /// 成功响应
    pub fn ok(id: String, data: Option<serde_json::Value>) -> Self {
        Self {
            id,
            ok: true,
            data,
            error: None,
        }
    }

    /// 错误响应
    pub fn error(id: String, error: String) -> Self {
        Self {
            id,
            ok: false,
            data: None,
            error: Some(error),
        }
    }
}
