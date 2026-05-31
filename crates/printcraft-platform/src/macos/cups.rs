//! macOS CUPS 打印机操作
//!
//! 通过 libcups FFI 调用 CUPS API 实现打印机枚举和查询。

use std::ffi::CStr;

use printcraft_core::error::{PrintCraftError, Result};
use printcraft_core::printer::{PrinterInfo, PrinterStatus};

/// CUPS FFI 绑定 (最小化，仅包含需要的函数)
#[repr(C)]
struct CupsOption {
    name: *mut std::ffi::c_char,
    value: *mut std::ffi::c_char,
}

#[repr(C)]
struct CupsDest {
    name: *mut std::ffi::c_char,
    instance: *mut std::ffi::c_char,
    is_default: std::ffi::c_int,
    num_options: std::ffi::c_int,
    options: *mut CupsOption,
}

#[link(name = "cups")]
extern "C" {
    fn cupsGetDests(dests: *mut *mut CupsDest) -> i32;
    fn cupsFreeDests(num_dests: i32, dests: *mut CupsDest);
    fn cupsGetDefault() -> *const std::ffi::c_char;
}

/// 列出系统所有打印机
pub fn list_printers() -> Result<Vec<PrinterInfo>> {
    unsafe {
        let mut dests: *mut CupsDest = std::ptr::null_mut();
        let count = cupsGetDests(&mut dests);

        if count <= 0 || dests.is_null() {
            tracing::info!("macOS CUPS: 未发现打印机");
            return Ok(vec![]);
        }

        let default_name = get_default_printer_name();
        let mut printers = Vec::with_capacity(count as usize);

        for i in 0..count as usize {
            let dest = &*dests.add(i);
            let name = if dest.name.is_null() {
                continue;
            } else {
                CStr::from_ptr(dest.name)
                    .to_string_lossy()
                    .into_owned()
            };

            let is_default = default_name.as_deref() == Some(&name);

            printers.push(PrinterInfo {
                name,
                is_default,
                status: PrinterStatus::Ready, // CUPS doesn't easily provide status in dests
                paper_sizes: vec![],
                color_support: false,
                duplex: false,
                driver_name: String::new(),
                port: String::new(),
            });
        }

        cupsFreeDests(count, dests);

        tracing::info!("macOS CUPS: 发现 {} 台打印机", printers.len());
        Ok(printers)
    }
}

/// 获取默认打印机
pub fn get_default_printer() -> Result<PrinterInfo> {
    let printers = list_printers()?;
    printers
        .into_iter()
        .find(|p| p.is_default)
        .ok_or_else(|| PrintCraftError::Printer("未找到默认打印机".to_string()))
}

/// 获取打印机支持的纸张尺寸
pub fn get_paper_sizes(_printer_name: &str) -> Result<Vec<printcraft_core::printer::PaperSize>> {
    // TODO: 使用 cupsGetOption 查询 media-supported 选项
    tracing::info!("macOS CUPS: 查询纸张尺寸 {} (stub)", _printer_name);
    Ok(vec![])
}

/// 获取默认打印机名称（内部辅助）
fn get_default_printer_name() -> Option<String> {
    unsafe {
        let ptr = cupsGetDefault();
        if ptr.is_null() {
            return None;
        }
        Some(CStr::from_ptr(ptr).to_string_lossy().into_owned())
    }
}
