//! Windows PDF 打印
//!
//! 打印策略（按优先级）:
//! 1. Windows Spooler API — 直接发送 PDF 字节到打印队列（静默，无外部依赖）
//! 2. SumatraPDF CLI — 静默打印，无弹窗
//! 3. ShellExecuteW "print" verb — 弹出系统打印对话框
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
type HANDLE = *mut std::ffi::c_void;
type PCWSTR = *const u16;
type PWSTR = *mut u16;
type DWORD = u32;
type BOOL = i32;

/// DOC_INFO_1W — 文档信息结构
#[repr(C)]
struct DocInfo1W {
    p_doc_name: PWSTR,
    p_output_file: PWSTR,
    p_datatype: PWSTR,
}

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

    // ── Print Spooler API ──
    fn OpenPrinterW(p_printer_name: PCWSTR, ph_printer: *mut HANDLE, p_default: *mut u8) -> BOOL;
    fn StartDocPrinterW(h_printer: HANDLE, level: DWORD, p_doc_info: *const u8) -> DWORD;
    fn WritePrinter(h_printer: HANDLE, p_buf: *const u8, cb_buf: DWORD, pc_written: *mut DWORD) -> BOOL;
    fn EndDocPrinter(h_printer: HANDLE) -> BOOL;
    fn ClosePrinter(h_printer: HANDLE) -> BOOL;
}

const SW_HIDE: i32 = 0;
const SW_SHOWNORMAL: i32 = 1;
const WM_CLOSE: u32 = 0x0010;

// ── 公开 API ────────────────────────────────────────────────

/// 将 PDF 发送到指定打印机
///
/// 策略:
/// 1. Windows Spooler API 直接发送 PDF 字节（静默，无需外部程序）
/// 2. SumatraPDF CLI（静默，需打包）
/// 3. ShellExecuteW "print"（弹出对话框）
///
/// 如果 printer_name 为空，自动使用系统默认打印机。
pub fn print_pdf(
    printer_name: &str,
    pdf_data: &[u8],
    copies: u32,
    job_name: &str,
) -> Result<()> {
    // 解析打印机名称: 空则使用默认打印机
    let actual_printer = if printer_name.is_empty() {
        let default = super::winspool::get_default_printer()?;
        tracing::info!("未指定打印机，使用默认: {}", default.name);
        default.name
    } else {
        printer_name.to_string()
    };

    // 1. 优先尝试 Windows Spooler API（最可靠，静默打印）
    tracing::info!("尝试 Spooler API 直接打印到 '{}'", actual_printer);
    match print_with_spooler(&actual_printer, pdf_data, job_name) {
        Ok(()) => return Ok(()),
        Err(e) => tracing::warn!("Spooler API 失败: {}, 尝试其他方式", e),
    }

    // 2. 写入临时文件，尝试其他方式
    let tmp_path = create_temp_pdf_path(job_name)?;
    std::fs::write(&tmp_path, pdf_data).map_err(|e| {
        PrintCraftError::Platform(format!("写入临时 PDF 文件失败: {}", e))
    })?;

    // 3. 尝试 SumatraPDF 静默打印
    if let Some(sumatra) = find_sumatra_pdf() {
        let result = print_with_sumatra(&sumatra, &tmp_path, &actual_printer, copies);
        let _ = std::fs::remove_file(&tmp_path);
        return result;
    }

    // 4. 回退: ShellExecuteW（可能弹出对话框）
    tracing::warn!("未找到 SumatraPDF.exe，使用 ShellExecuteW（可能弹出对话框）");
    let result = print_with_shell_execute(&tmp_path, &actual_printer);
    let _ = std::fs::remove_file(&tmp_path);
    result
}

// ── Spooler API 打印 ────────────────────────────────────────

/// 使用 Windows Print Spooler API 直接发送 PDF 字节
///
/// 通过 OpenPrinter → StartDocPrinter → WritePrinter → EndDocPrinter 流程，
/// 将 PDF 原始字节发送到打印队列。无需外部 PDF 阅读器。
fn print_with_spooler(printer_name: &str, pdf_data: &[u8], job_name: &str) -> Result<()> {
    let wide_printer = to_wide(printer_name);
    let wide_job_name = to_wide(job_name);
    let wide_datatype = to_wide("RAW");

    unsafe {
        // 打开打印机
        let mut h_printer: HANDLE = std::ptr::null_mut();
        let ok = OpenPrinterW(wide_printer.as_ptr(), &mut h_printer, std::ptr::null_mut());
        if ok == 0 || h_printer.is_null() {
            return Err(PrintCraftError::Platform(format!(
                "OpenPrinterW 失败: 无法打开打印机 '{}'",
                printer_name
            )));
        }

        // 准备文档信息
        let doc_info = DocInfo1W {
            p_doc_name: wide_job_name.as_ptr() as PWSTR,
            p_output_file: std::ptr::null_mut(),
            p_datatype: wide_datatype.as_ptr() as PWSTR,
        };

        // 开始文档
        let doc_id = StartDocPrinterW(
            h_printer,
            1, // DOC_INFO_1
            &doc_info as *const DocInfo1W as *const u8,
        );
        if doc_id == 0 {
            ClosePrinter(h_printer);
            return Err(PrintCraftError::Platform("StartDocPrinterW 失败".to_string()));
        }

        // 写入 PDF 数据
        let mut written: DWORD = 0;
        let ok = WritePrinter(
            h_printer,
            pdf_data.as_ptr(),
            pdf_data.len() as DWORD,
            &mut written,
        );
        if ok == 0 {
            EndDocPrinter(h_printer);
            ClosePrinter(h_printer);
            return Err(PrintCraftError::Platform("WritePrinter 失败".to_string()));
        }

        // 结束文档
        EndDocPrinter(h_printer);
        ClosePrinter(h_printer);

        tracing::info!(
            "Spooler API: PDF ({} bytes) 已发送到 '{}' (job: {})",
            written,
            printer_name,
            job_name
        );
        Ok(())
    }
}

// ── SumatraPDF 打印 ─────────────────────────────────────────

/// 在常见位置查找 SumatraPDF.exe
fn find_sumatra_pdf() -> Option<PathBuf> {
    let candidates = [
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("SumatraPDF.exe"))),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("tools").join("SumatraPDF.exe"))),
        std::env::var("ProgramFiles")
            .ok()
            .map(|p| PathBuf::from(p).join("SumatraPDF").join("SumatraPDF.exe")),
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

/// 使用 ShellExecuteW 打印 PDF
///
/// 先尝试 "printto" verb（指定打印机），失败则用 "print"（弹出打印对话框）。
fn print_with_shell_execute(pdf_path: &PathBuf, printer_name: &str) -> Result<()> {
    let wide_file = to_wide(&pdf_path.to_string_lossy());

    // 优先用 "printto" 指定打印机
    if !printer_name.is_empty() {
        let wide_operation = to_wide("printto");
        let wide_printer = to_wide(printer_name);

        unsafe {
            let result = ShellExecuteW(
                std::ptr::null_mut(),
                wide_operation.as_ptr(),
                wide_file.as_ptr(),
                wide_printer.as_ptr(),
                std::ptr::null_mut(),
                SW_SHOWNORMAL,
            );

            if result as usize > 32 {
                tracing::info!("ShellExecuteW printto: PDF 已发送到 '{}'", printer_name);
                std::thread::sleep(std::time::Duration::from_millis(500));
                return Ok(());
            }

            tracing::warn!(
                "ShellExecuteW printto 失败 ({}), 尝试 print verb",
                shell_error_description(result as usize)
            );
        }
    }

    // 回退: "print" verb（弹出系统打印对话框）
    let wide_operation = to_wide("print");

    unsafe {
        let result = ShellExecuteW(
            std::ptr::null_mut(),
            wide_operation.as_ptr(),
            wide_file.as_ptr(),
            std::ptr::null(),
            std::ptr::null_mut(),
            SW_SHOWNORMAL,
        );

        let code = result as usize;
        if code > 32 {
            tracing::info!("ShellExecuteW print: 已打开打印对话框 (code={})", code);
            std::thread::sleep(std::time::Duration::from_secs(2));
            close_pdf_viewer_window();
            Ok(())
        } else {
            tracing::error!("ShellExecuteW print 失败: 返回码 {} ({})", code, shell_error_description(code));
            Err(PrintCraftError::Platform(format!(
                "ShellExecuteW print 失败: 返回码 {} ({})",
                code,
                shell_error_description(code)
            )))
        }
    }
}

/// 尝试关闭 PDF 阅读器弹出的窗口
fn close_pdf_viewer_window() {
    unsafe {
        let viewers = ["AcrobatSDIWindow", "ATL:00007FF..."];
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
