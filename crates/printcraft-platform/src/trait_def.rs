//! 平台打印机操作 trait 定义
//!
//! 所有平台（macOS/Windows）都必须实现此 trait。
//! 提供统一的打印机枚举、查询和打印提交接口。

use async_trait::async_trait;

use printcraft_core::printer::PrinterInfo;
use printcraft_core::error::Result;

/// 平台打印操作 trait
///
/// # 实现要求
/// - `list_printers`: 返回系统所有安装的打印机
/// - `get_default_printer`: 返回默认打印机信息
/// - `print_pdf`: 将 PDF 字节流发送到指定打印机
/// - 所有方法都是 async 的，允许异步 I/O
#[async_trait]
pub trait PlatformPrinter: Send + Sync {
    /// 列出系统所有打印机
    async fn list_printers(&self) -> Result<Vec<PrinterInfo>>;

    /// 获取默认打印机
    async fn get_default_printer(&self) -> Result<PrinterInfo>;

    /// 将 PDF 文件发送到打印机打印
    ///
    /// # Arguments
    /// * `printer_name` - 目标打印机名称
    /// * `pdf_data` - PDF 文件字节
    /// * `copies` - 打印份数
    /// * `job_name` - 打印任务显示名称
    async fn print_pdf(
        &self,
        printer_name: &str,
        pdf_data: &[u8],
        copies: u32,
        job_name: &str,
    ) -> Result<()>;

    /// 获取打印机支持的纸张尺寸列表
    async fn get_paper_sizes(&self, printer_name: &str) -> Result<Vec<printcraft_core::printer::PaperSize>>;
}
