//! macOS PDF 打印
//!
//! 通过 CUPS API 将 PDF 文件发送到打印机。

use std::ffi::CString;
use std::path::PathBuf;

use printcraft_core::error::{PrintCraftError, Result};

/// CUPS FFI 绑定
#[repr(C)]
struct CupsOption {
    name: *mut std::ffi::c_char,
    value: *mut std::ffi::c_char,
}

#[link(name = "cups")]
extern "C" {
    fn cupsPrintFile(
        name: *const std::ffi::c_char,
        filename: *const std::ffi::c_char,
        title: *const std::ffi::c_char,
        num_options: std::ffi::c_int,
        options: *const CupsOption,
    ) -> i32;
}

/// 将 PDF 发送到指定打印机
///
/// 1. 将 PDF 字节写入临时文件
/// 2. 调用 cupsPrintFile 发送到打印机
/// 3. 清理临时文件
pub fn print_pdf(
    printer_name: &str,
    pdf_data: &[u8],
    copies: u32,
    job_name: &str,
) -> Result<()> {
    // 写入临时文件
    let tmp_path = create_temp_pdf_path(job_name)?;
    std::fs::write(&tmp_path, pdf_data).map_err(|e| {
        PrintCraftError::Platform(format!("写入临时 PDF 文件失败: {}", e))
    })?;

    // 准备 CUPS 参数
    let c_printer = CString::new(printer_name).map_err(|_| {
        PrintCraftError::Printer("打印机名称包含无效字符".to_string())
    })?;
    let c_filename = CString::new(tmp_path.to_string_lossy().as_bytes()).map_err(|_| {
        PrintCraftError::Platform("临时文件路径包含无效字符".to_string())
    })?;
    let c_title = CString::new(job_name).map_err(|_| {
        PrintCraftError::PrintJob("任务名称包含无效字符".to_string())
    })?;

    // 构造 copies 选项
    let opt_name = CString::new("copies").unwrap();
    let opt_value = CString::new(copies.to_string()).unwrap();
    let option = CupsOption {
        name: opt_name.into_raw(),
        value: opt_value.into_raw(),
    };

    // 调用 CUPS 打印
    let result = unsafe {
        cupsPrintFile(
            c_printer.as_ptr(),
            c_filename.as_ptr(),
            c_title.as_ptr(),
            1, // num_options
            &option,
        )
    };

    // 回收 CString 防止泄漏
    unsafe {
        let _ = CString::from_raw(option.name);
        let _ = CString::from_raw(option.value);
    }

    // 清理临时文件
    let _ = std::fs::remove_file(&tmp_path);

    if result == 0 {
        Err(PrintCraftError::Platform(format!(
            "CUPS 打印失败: cupsPrintFile 返回 0 (printer={}, file={})",
            printer_name,
            tmp_path.display()
        )))
    } else {
        tracing::info!(
            "macOS CUPS: PDF 已发送到 '{}' (job_id={}, {} bytes, {} 份)",
            printer_name,
            result,
            pdf_data.len(),
            copies
        );
        Ok(())
    }
}

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
