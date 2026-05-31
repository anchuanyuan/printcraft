//! Windows PDF 打印
//!
//! 打印策略（按优先级）:
//! 1. SumatraPDF CLI — 静默打印，无弹窗，最可控
//! 2. ShellExecuteW "printto" verb — 用系统注册的 PDF 处理器打印
//!
//! 推荐: 打包 SumatraPDF.exe 以获得最佳体验。
//! 下载: https://www.sumatrapdfreader.org/download-free-pdf-viewer

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;

use printcraft_core::error::{PrintCraftError, Result};

// ── Win32 FFI ───────────────────────────────────────────────

type HWND = *mut std::ffi::c_void;
type HINSTANCE = *mut std::ffi::c_void;
type PCWSTR = *const u16;
type PWSTR = *mut u16;

extern "system" {
    fn ShellExecuteW(
        hwnd: HWND,
        lp_operation: PCWSTR,
        lp_file: PCWSTR,
        lp_parameters: PCWSTR,
        lp_directory: PCWSTR,
        n_show_cmd: i32,
    ) -> HINSTANCE;

    fn FindWindowW(lp_class_name: PCWSTR, lp_window_name: PCWSTR) -> HWND;
    fn SendMessageW(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> isize;
}

const SW_HIDE: i32 = 0;
const SW_SHOWNORMAL: i32 = 1;
const WM_CLOSE: u32 = 0x0010;

// ── 公开 API ────────────────────────────────────────────────

/// 将 PDF 发送到指定打印机
///
/// 策略:
/// 1. 查找内嵌的 SumatraPDF.exe → 静默打印
/// 2. 回退到 ShellExecuteW "printto" → 弹出选择对话框（仅首次）
pub fn print_pdf(
    printer_name: &str,
    pdf_data: &[u8],
    copies: u32,
    job_name: &str,
) -> Result<()> {
    // 1. 写入临时文件
    let tmp_path = create_temp_pdf_path(job_name)?;
    std::fs::write(&tmp_path, pdf_data).map_err(|e| {
        PrintCraftError::Platform(format!("写入临时 PDF 文件失败: {}", e))
    })?;

    // 2. 尝试 SumatraPDF 静默打印
    if let Some(sumatra) = find_sumatra_pdf() {
        let result = print_with_sumatra(&sumatra, &tmp_path, printer_name, copies);
        let _ = std::fs::remove_file(&tmp_path);
        return result;
    }

    // 3. 回退: ShellExecuteW "printto"
    tracing::warn!(
        "未找到 SumatraPDF.exe，使用 ShellExecuteW printto（可能弹出对话框）"
    );
    let result = print_with_shell_execute(&tmp_path, printer_name);
    let _ = std::fs::remove_file(&tmp_path);
    result
}

// ── SumatraPDF 打印 ─────────────────────────────────────────

/// 在常见位置查找 SumatraPDF.exe
fn find_sumatra_pdf() -> Option<PathBuf> {
    // 搜索顺序: 同目录 → Program Files → PATH
    let candidates = [
        // 与 printcraft.exe 同目录
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("SumatraPDF.exe"))),
        // 安装目录下的 tools 子目录
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("tools").join("SumatraPDF.exe"))),
        // Program Files
        std::env::var("ProgramFiles")
            .ok()
            .map(|p| PathBuf::from(p).join("SumatraPDF").join("SumatraPDF.exe")),
        // Program Files (x86)
        std::env::var("ProgramFiles(x86)")
            .ok()
            .map(|p| PathBuf::from(p).join("SumatraPDF").join("SumatraPDF.exe")),
    ];

    for candidate in candidates.iter().flatten() {
        if candidate.exists() {
            tracing::info!("找到 SumatraPDF: {}", candidate.display());
            return Some(candidate.clone());
        }
    }

    None
}

/// 使用 SumatraPDF CLI 静默打印
///
/// 命令行: SumatraPDF.exe -print-to "printer" -print-settings "Nx" file.pdf
fn print_with_sumatra(
    sumatra_path: &PathBuf,
    pdf_path: &PathBuf,
    printer_name: &str,
    copies: u32,
) -> Result<()> {
    let settings = format!("{}x", copies);

    let output = std::process::Command::new(sumatra_path)
        .args([
            "-print-to",
            printer_name,
            "-print-settings",
            &settings,
            "-silent",
            &pdf_path.to_string_lossy(),
        ])
        .output()
        .map_err(|e| PrintCraftError::Platform(format!("启动 SumatraPDF 失败: {}", e)))?;

    if output.status.success() {
        tracing::info!(
            "SumatraPDF: PDF 已发送到 '{}' ({} 份)",
            printer_name,
            copies
        );
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(PrintCraftError::Platform(format!(
            "SumatraPDF 打印失败: {}",
            stderr
        )))
    }
}

// ── ShellExecuteW 打印 ──────────────────────────────────────

/// 使用 ShellExecuteW "printto" verb 打印
///
/// Windows 会使用系统注册的 PDF 处理器（如 Edge、Adobe Reader）执行打印。
/// 首次使用可能弹出打印机选择对话框。
fn print_with_shell_execute(pdf_path: &PathBuf, printer_name: &str) -> Result<()> {
    let wide_file = to_wide(&pdf_path.to_string_lossy());
    let wide_operation = to_wide("printto");
    let wide_printer = to_wide(printer_name);

    unsafe {
        let result = ShellExecuteW(
            std::ptr::null_mut(),
            wide_operation.as_ptr(),
            wide_file.as_ptr(),
            wide_printer.as_ptr(),
            std::ptr::null_mut(),
            SW_SHOWNORMAL, // printto 需要可见窗口
        );

        // ShellExecuteW 返回值 > 32 表示成功
        if result as usize > 32 {
            tracing::info!(
                "ShellExecuteW printto: PDF 已发送到 '{}'",
                printer_name
            );

            // 等待一小段时间让打印任务提交
            std::thread::sleep(std::time::Duration::from_millis(500));

            // 尝试关闭可能弹出的 PDF 查看器窗口
            close_pdf_viewer_window();

            Ok(())
        } else {
            let error_code = result as usize;
            Err(PrintCraftError::Platform(format!(
                "ShellExecuteW printto 失败: 返回码 {} ({})",
                error_code,
                shell_error_description(error_code)
            )))
        }
    }
}

/// 尝试关闭 PDF 阅读器弹出的窗口
fn close_pdf_viewer_window() {
    unsafe {
        // 尝试查找常见的 PDF 查看器窗口
        let viewers = ["AcrobatSDIWindow", "ATL:00007FF..."]; // Adobe Reader class names
        for class_name in &viewers {
            let wide_class = to_wide(class_name);
            let hwnd = FindWindowW(wide_class.as_ptr(), std::ptr::null());
            if !hwnd.is_null() {
                SendMessageW(hwnd, WM_CLOSE, 0, 0);
            }
        }
    }
}

// ── 辅助函数 ────────────────────────────────────────────────

/// 生成临时 PDF 文件路径
fn create_temp_pdf_path(job_name: &str) -> Result<PathBuf> {
    let sanitized: String = job_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let filename = format!("printcraft_{}_{}.pdf", sanitized, timestamp);
    Ok(std::env::temp_dir().join(filename))
}

/// 字符串 → null-terminated wide string
fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// ShellExecuteW 错误码描述
fn shell_error_description(code: usize) -> &'static str {
    match code {
        0 => "内存不足",
        2 => "文件未找到",
        3 => "路径未找到",
        5 => "访问被拒绝",
        8 => "内存不足",
        26 => "DLL 未找到",
        27 => "无法注册文件类型",
        31 => "没有关联的应用程序",
        32 => "DLL 协作错误",
        _ => "未知错误",
    }
}
