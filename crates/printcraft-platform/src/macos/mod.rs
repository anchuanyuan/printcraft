//! macOS 平台实现
//!
//! 通过 CUPS API 进行打印机操作。

pub mod cups;
pub mod pdf_print;

use async_trait::async_trait;
use printcraft_core::error::Result;
use printcraft_core::printer::PrinterInfo;
use printcraft_core::printer::PaperSize;

use crate::trait_def::PlatformPrinter;

/// macOS 打印机实现
pub struct MacOSPrinter {
    // CUPS 连接状态等
}

impl MacOSPrinter {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl PlatformPrinter for MacOSPrinter {
    async fn list_printers(&self) -> Result<Vec<PrinterInfo>> {
        cups::list_printers()
    }

    async fn get_default_printer(&self) -> Result<PrinterInfo> {
        cups::get_default_printer()
    }

    async fn print_pdf(
        &self,
        printer_name: &str,
        pdf_data: &[u8],
        copies: u32,
        job_name: &str,
    ) -> Result<()> {
        pdf_print::print_pdf(printer_name, pdf_data, copies, job_name)
    }

    async fn get_paper_sizes(&self, printer_name: &str) -> Result<Vec<PaperSize>> {
        cups::get_paper_sizes(printer_name)
    }
}
