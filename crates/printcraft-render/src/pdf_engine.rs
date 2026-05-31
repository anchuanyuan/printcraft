//! PDF 渲染 trait 定义
//!
//! 所有渲染引擎（printpdf 基础版、Chromium 版）都实现此 trait。

use async_trait::async_trait;
use printcraft_core::error::Result;
use printcraft_core::print_job::PrintJob;

/// PDF 渲染器 trait
///
/// 将 PrintJob 渲染为 PDF 字节流。
#[async_trait]
pub trait PdfRenderer: Send + Sync {
    /// 将打印任务渲染为 PDF
    ///
    /// # Arguments
    /// * `job` - 打印任务（含元素列表和配置）
    ///
    /// # Returns
    /// PDF 文件的字节数据
    async fn render(&self, job: &PrintJob) -> Result<Vec<u8>>;

    /// 渲染器名称
    fn name(&self) -> &str;
}
