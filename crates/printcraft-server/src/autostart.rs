//! 开机自启管理
//!
//! 使用 auto-launch crate 管理开机自启。
//! macOS: Login Items
//! Windows: 注册表 Run key

use auto_launch::AutoLaunch;

const APP_NAME: &str = "PrintCraft";

/// 获取当前可执行文件路径
fn current_exe_path() -> String {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// 创建 AutoLaunch 实例
fn create_auto_launch() -> Option<AutoLaunch> {
    let exe_path = current_exe_path();
    if exe_path.is_empty() {
        tracing::warn!("无法获取可执行文件路径，开机自启不可用");
        return None;
    }
    // macOS: new(name, path, use_launch_agent, args)
    // Windows: new(name, path, args)
    #[cfg(target_os = "macos")]
    {
        Some(AutoLaunch::new(APP_NAME, &exe_path, false, &[] as &[&str]))
    }
    #[cfg(not(target_os = "macos"))]
    {
        Some(AutoLaunch::new(APP_NAME, &exe_path, &[] as &[&str]))
    }
}

/// 检查是否已启用开机自启
pub fn is_enabled() -> bool {
    create_auto_launch()
        .map(|al| al.is_enabled().unwrap_or(false))
        .unwrap_or(false)
}

/// 启用开机自启
pub fn enable() -> Result<(), String> {
    let al = create_auto_launch().ok_or("无法创建 AutoLaunch 实例")?;
    al.enable().map_err(|e| format!("启用开机自启失败: {}", e))
}

/// 禁用开机自启
pub fn disable() -> Result<(), String> {
    let al = create_auto_launch().ok_or("无法创建 AutoLaunch 实例")?;
    al.disable().map_err(|e| format!("禁用开机自启失败: {}", e))
}

/// 切换开机自启状态
pub fn toggle() -> Result<bool, String> {
    if is_enabled() {
        disable()?;
        Ok(false)
    } else {
        enable()?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_enabled_returns_bool() {
        // 不管系统状态如何，应该返回 bool 而不是 panic
        let _enabled = is_enabled();
    }
}
