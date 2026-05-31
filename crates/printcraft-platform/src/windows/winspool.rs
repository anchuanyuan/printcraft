//! Windows winspool 打印机操作
//!
//! 通过 Win32 API 枚举和查询打印机。
//! 使用 `windows` crate 的 FFI 绑定调用:
//! - EnumPrintersW (PRINTER_INFO_4W)
//! - GetDefaultPrinterW
//! - DeviceCapabilitiesW (DC_PAPERNAMES / DC_PAPERSIZE)

use printcraft_core::error::{PrintCraftError, Result};
use printcraft_core::printer::{PaperSize, PrinterInfo, PrinterStatus};

use windows::Win32::Foundation::BOOL;

// ── Win32 FFI 类型定义 ──────────────────────────────────────

/// PRINTER_INFO_4W — 最轻量的打印机信息结构
/// 只包含名称和属性，不需要连接到打印机
#[repr(C)]
struct PrinterInfo4W {
    p_printer_name: *mut u16,
    p_server_name: *mut u16,
    attributes: u32,
}

// ── 外部 FFI 函数声明 ───────────────────────────────────────

extern "system" {
    /// 枚举系统打印机
    fn EnumPrintersW(
        flags: u32,
        name: *const u16,
        level: u32,
        p_printer_enum: *mut u8,
        cb_buf: u32,
        pcb_needed: *mut u32,
        pc_returned: *mut u32,
    ) -> BOOL;

    /// 获取默认打印机名称
    fn GetDefaultPrinterW(
        printer_name_buffer: *mut u16,
        buffer_size: *mut u32,
    ) -> BOOL;

    /// 查询打印机能力（纸张尺寸等）
    fn DeviceCapabilitiesW(
        p_device: *const u16,
        p_port: *const u16,
        capability: u32,
        p_output: *mut u16,
        p_dev_mode: *const u8,
    ) -> u32;

    /// 释放 Spooler 分配的内存
    fn GlobalFree(h_mem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn GlobalAlloc(flags: u32, bytes: usize) -> *mut std::ffi::c_void;
}

// ── Win32 常量 ──────────────────────────────────────────────

const PRINTER_ENUM_LOCAL: u32 = 0x00000002;
const PRINTER_ENUM_CONNECTIONS: u32 = 0x00000004;
const PRINTER_ATTRIBUTE_NETWORK: u32 = 0x00000010;

const DC_PAPERNAMES: u32 = 16;
const DC_PAPERSIZE: u32 = 6;

const GMEM_FIXED: u32 = 0x0000;

// ── 公开 API ────────────────────────────────────────────────

/// 列出系统所有打印机
pub fn list_printers() -> Result<Vec<PrinterInfo>> {
    unsafe {
        let mut needed: u32 = 0;
        let mut returned: u32 = 0;

        // 第一次调用获取缓冲区大小
        EnumPrintersW(
            PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS,
            std::ptr::null(),
            4, // PRINTER_INFO_4
            std::ptr::null_mut(),
            0,
            &mut needed,
            &mut returned,
        );

        if needed == 0 {
            tracing::info!("Windows winspool: 未发现打印机");
            return Ok(vec![]);
        }

        // 分配缓冲区
        let h_mem = GlobalAlloc(GMEM_FIXED, needed as usize);
        if h_mem.is_null() {
            return Err(PrintCraftError::Platform(
                "分配打印机枚举缓冲区失败 (GlobalAlloc)".to_string(),
            ));
        }
        let buf = h_mem as *mut u8;

        // 第二次调用填充数据
        let success = EnumPrintersW(
            PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS,
            std::ptr::null(),
            4,
            buf,
            needed,
            &mut needed,
            &mut returned,
        );

        if !success.as_bool() {
            let _ = GlobalFree(h_mem);
            return Err(PrintCraftError::Platform(format!(
                "EnumPrintersW 失败: {}",
                last_error_message()
            )));
        }

        // 获取默认打印机名称用于标记
        let default_name = get_default_printer_name();

        // 遍历 PRINTER_INFO_4W 数组
        let info_ptr = buf as *const PrinterInfo4W;
        let mut printers = Vec::with_capacity(returned as usize);

        for i in 0..returned as usize {
            let info = &*info_ptr.add(i);

            if info.p_printer_name.is_null() {
                continue;
            }

            let name = wide_ptr_to_string(info.p_printer_name);
            if name.is_empty() {
                continue;
            }

            let is_network = (info.attributes & PRINTER_ATTRIBUTE_NETWORK) != 0;
            let is_default = default_name.as_ref() == Some(&name);

            printers.push(PrinterInfo {
                name,
                is_default,
                status: PrinterStatus::Ready,
                paper_sizes: vec![],
                color_support: false,
                duplex: false,
                driver_name: String::new(),
                port: if is_network { "network".to_string() } else { "local".to_string() },
            });
        }

        let _ = GlobalFree(h_mem);

        tracing::info!("Windows winspool: 发现 {} 台打印机", printers.len());
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
pub fn get_paper_sizes(printer_name: &str) -> Result<Vec<PaperSize>> {
    let wide_name = string_to_wide(printer_name);

    unsafe {
        // 第一次调用获取纸张数量
        let count = DeviceCapabilitiesW(
            wide_name.as_ptr(),
            std::ptr::null(),
            DC_PAPERNAMES,
            std::ptr::null_mut(),
            std::ptr::null(),
        );

        if count == u32::MAX || count == 0 {
            tracing::warn!(
                "Windows: DeviceCapabilitiesW 查询纸张数量失败 ({})",
                printer_name
            );
            return Ok(vec![]);
        }

        let count = count as usize;

        // 获取纸张名称
        let name_buf: Vec<u16> = vec![0u16; count * 64];
        let names_ret = DeviceCapabilitiesW(
            wide_name.as_ptr(),
            std::ptr::null(),
            DC_PAPERNAMES,
            name_buf.as_ptr() as *mut u16,
            std::ptr::null(),
        );

        if names_ret == u32::MAX {
            return Ok(vec![]);
        }

        // 尝试获取纸张物理尺寸 (0.1mm 单位)
        // 某些打印机不支持 DC_PAPERSIZE，只返回纸张名
        let size_buf: Vec<u8> = vec![0u8; count * 8];
        let sizes_ret = DeviceCapabilitiesW(
            wide_name.as_ptr(),
            std::ptr::null(),
            DC_PAPERSIZE,
            size_buf.as_ptr() as *mut u16,
            std::ptr::null(),
        );

        let has_sizes = sizes_ret != u32::MAX && sizes_ret as usize >= count;
        let size_data = size_buf.as_ptr() as *const i32;

        let mut paper_sizes = Vec::with_capacity(count);

        for i in 0..count {
            let name_start = i * 64;
            let name_wide = &name_buf[name_start..name_start + 64];
            let name = wide_slice_to_string(name_wide);

            if has_sizes {
                let cx = *size_data.add(i * 2);
                let cy = *size_data.add(i * 2 + 1);
                if cx > 0 && cy > 0 {
                    paper_sizes.push(PaperSize {
                        name: name.trim().to_string(),
                        width_mm: cx as f64 / 10.0,
                        height_mm: cy as f64 / 10.0,
                    });
                } else {
                    let (w, h) = parse_size_from_name(&name);
                    paper_sizes.push(PaperSize {
                        name: name.trim().to_string(),
                        width_mm: w,
                        height_mm: h,
                    });
                }
            } else {
                let (w, h) = parse_size_from_name(&name);
                paper_sizes.push(PaperSize {
                    name: name.trim().to_string(),
                    width_mm: w,
                    height_mm: h,
                });
            }
        }

        tracing::info!(
            "Windows: {} 支持 {} 种纸张 (尺寸数据: {})",
            printer_name,
            paper_sizes.len(),
            if has_sizes { "有" } else { "无" }
        );
        Ok(paper_sizes)
    }
}

// ── 内部辅助函数 ────────────────────────────────────────────

/// 获取默认打印机名称（内部用）
fn get_default_printer_name() -> Option<String> {
    unsafe {
        let mut buf = vec![0u16; 256];
        let mut size: u32 = 256;

        let success = GetDefaultPrinterW(buf.as_mut_ptr(), &mut size);
        if success.as_bool() && size > 0 {
            Some(wide_slice_to_string(&buf[..size as usize]))
        } else {
            // 回退到注册表
            get_default_printer_from_registry()
        }
    }
}

/// 从注册表读取默认打印机 (HKCU\...\Windows\Device)
fn get_default_printer_from_registry() -> Option<String> {
    use windows::Win32::System::Registry;

    unsafe {
        let mut key = Registry::HKEY::default();
        let sub_key: Vec<u16> = "Software\\Microsoft\\Windows NT\\CurrentVersion\\Windows"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let value_name: Vec<u16> = "Device".encode_utf16().chain(std::iter::once(0)).collect();

        let result = Registry::RegOpenKeyExW(
            Registry::HKEY_CURRENT_USER,
            windows::core::PCWSTR(sub_key.as_ptr()),
            0,
            Registry::KEY_READ,
            &mut key,
        );

        if result.is_err() {
            return None;
        }

        let mut buf = [0u16; 512];
        let mut buf_size: u32 = 512 * 2; // bytes
        let mut value_type: windows::Win32::System::Registry::REG_VALUE_TYPE = windows::Win32::System::Registry::REG_VALUE_TYPE(0);

        let result = Registry::RegQueryValueExW(
            key,
            windows::core::PCWSTR(value_name.as_ptr()),
            None,
            Some(&mut value_type),
            Some(buf.as_mut_ptr() as *mut u8),
            Some(&mut buf_size),
        );

        let _ = Registry::RegCloseKey(key);

        if result.is_ok() && buf_size > 0 {
            let len = (buf_size as usize / 2).saturating_sub(1); // 去掉 null terminator
            let name = wide_slice_to_string(&buf[..len]);
            // 格式: "打印机名称,winspool,端口:"
            // 取逗号前的部分
            name.split(',').next().map(|s| s.to_string())
        } else {
            None
        }
    }
}

/// 宽字符指针 → Rust String (null-terminated)
unsafe fn wide_ptr_to_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    let slice = std::slice::from_raw_parts(ptr, len);
    String::from_utf16_lossy(slice)
}

/// 宽字符切片 → Rust String (去除尾部 null)
fn wide_slice_to_string(slice: &[u16]) -> String {
    let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
    String::from_utf16_lossy(&slice[..end])
}

/// Rust String → null-terminated Vec<u16>
fn string_to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 从纸张名称中解析尺寸（如 "75mm x 130mm" → (75.0, 130.0)）
fn parse_size_from_name(name: &str) -> (f64, f64) {
    // 匹配 "NNmm x NNmm" 模式
    let lower = name.to_lowercase();
    if let Some(idx) = lower.find("mm") {
        let w_str = lower[..idx].trim();
        // 找到 "x" 分隔符
        let after_w = &lower[idx + 2..];
        if let Some(x_pos) = after_w.find('x') {
            let h_part = after_w[x_pos + 1..].trim();
            if let Some(h_idx) = h_part.find("mm") {
                let h_str = h_part[..h_idx].trim();
                if let (Ok(w), Ok(h)) = (w_str.parse::<f64>(), h_str.parse::<f64>()) {
                    return (w, h);
                }
            }
        }
    }
    (0.0, 0.0)
}

/// 获取 Win32 最后错误的描述
fn last_error_message() -> String {
    unsafe {
        let err = windows::Win32::Foundation::GetLastError();
        format!("Win32 error code: {:08X}", err.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_printers() {
        let printers = list_printers().unwrap();
        println!("发现 {} 台打印机:", printers.len());
        for p in &printers {
            println!("  {} {}", p.name, if p.is_default { "(默认)" } else { "" });
        }
        assert!(!printers.is_empty());
    }

    #[test]
    fn test_get_paper_sizes() {
        let default = get_default_printer().unwrap();
        println!("默认打印机: {}", default.name);

        // 测试所有打印机的纸张查询
        let printers = list_printers().unwrap();
        for p in &printers {
            let papers = get_paper_sizes(&p.name).unwrap();
            println!("{}: {} 种纸张", p.name, papers.len());
            for (i, pp) in papers.iter().enumerate().take(5) {
                println!("  {}. {} ({}x{}mm)", i + 1, pp.name, pp.width_mm, pp.height_mm);
            }
            if papers.len() > 5 {
                println!("  ... 共 {} 种", papers.len());
            }
        }
    }

    #[test]
    fn test_device_capabilities_debug() {
        let printers = list_printers().unwrap();
        for p in &printers {
            let wide_name = string_to_wide(&p.name);
            unsafe {
                let name_count = DeviceCapabilitiesW(
                    wide_name.as_ptr(),
                    std::ptr::null(),
                    DC_PAPERNAMES,
                    std::ptr::null_mut(),
                    std::ptr::null(),
                );
                let size_count = DeviceCapabilitiesW(
                    wide_name.as_ptr(),
                    std::ptr::null(),
                    DC_PAPERSIZE,
                    std::ptr::null_mut(),
                    std::ptr::null(),
                );
                println!(
                    "{}: DC_PAPERNAMES={}, DC_PAPERSIZE={}",
                    p.name, name_count, size_count
                );
            }
        }
    }
}
