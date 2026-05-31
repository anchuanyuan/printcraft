//! 系统托盘图标
//!
//! 在系统托盘显示 PrintCraft 图标，提供右键菜单。
//! 菜单项：显示/隐藏窗口、查看打印机、开机自启切换、退出。
//!
//! 使用 tray-icon + muda 实现跨平台支持。

use muda::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};
use std::sync::mpsc;

/// 托盘菜单事件
#[derive(Debug, Clone)]
pub enum TrayEvent {
    /// 显示主窗口
    Show,
    /// 隐藏主窗口
    Hide,
    /// 切换开机自启
    ToggleAutostart,
    /// 退出应用
    Quit,
}

/// 创建系统托盘
///
/// 返回 (TrayIcon, 事件接收器)。
/// TrayIcon 必须保持存活，否则托盘图标消失。
pub fn create_tray() -> Result<(TrayIcon, mpsc::Receiver<TrayEvent>), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel();

    // 构建菜单
    let menu = Menu::new();
    let show_item = MenuItem::with_id("show", "显示窗口", true, None);
    let hide_item = MenuItem::with_id("hide", "隐藏窗口", true, None);
    let autostart_item = MenuItem::with_id("autostart", "开机自启", true, None);
    let quit_item = MenuItem::with_id("quit", "退出", true, None);

    menu.append(&show_item)?;
    menu.append(&hide_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&autostart_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&quit_item)?;

    // 创建托盘图标
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("PrintCraft - 本地打印服务")
        .build()?;

    // 启动菜单事件监听线程
    std::thread::spawn(move || {
        // muda 的菜单事件需要在主线程或专用线程处理
        // 这里用轮询方式检查菜单点击
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            // 事件通过 muda 的 MenuEvent 通道传递
            // 实际使用中需要接入事件循环
        }
    });

    // 设置菜单事件处理器
    let tx_clone = tx.clone();
    muda::MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
        let event = match event.id().0.as_str() {
            "show" => Some(TrayEvent::Show),
            "hide" => Some(TrayEvent::Hide),
            "autostart" => Some(TrayEvent::ToggleAutostart),
            "quit" => Some(TrayEvent::Quit),
            _ => None,
        };
        if let Some(e) = event {
            let _ = tx_clone.send(e);
        }
    }));

    Ok((tray_icon, rx))
}
