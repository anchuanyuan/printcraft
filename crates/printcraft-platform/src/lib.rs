//! PrintCraft 平台抽象层
//!
//! 提供跨平台的打印机操作接口。
//! macOS 通过 CUPS，Windows 通过 winspool。

pub mod trait_def;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

pub use trait_def::PlatformPrinter;

/// 创建当前平台的打印实现
///
/// 根据编译目标自动选择对应的平台实现。
pub fn create_platform_printer() -> Box<dyn PlatformPrinter> {
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOSPrinter::new())
    }

    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsPrinter::new())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        compile_error!("PrintCraft 目前仅支持 macOS 和 Windows 平台")
    }
}
