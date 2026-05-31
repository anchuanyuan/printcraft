//! Chromium CDP 渲染引擎
//!
//! 使用 chromiumoxide 通过 Chrome DevTools Protocol 将 HTML 渲染为 PDF。
//! 适用于 HTM/TABLE/URL 等需要完整 HTML 渲染的元素。

use async_trait::async_trait;
use base64::Engine;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::page::Page;
use futures_util::stream::StreamExt;
use printcraft_core::error::{PrintCraftError, Result};
use printcraft_core::print_job::PrintJob;

use crate::pdf_engine::PdfRenderer;
use crate::template::assemble_html;

/// Chromium CDP 渲染器
///
/// 每次渲染启动 headless Chrome，将 PrintJob 元素组装为 HTML，
/// 通过 CDP Page.printToPDF 输出 PDF。
pub struct ChromiumRenderer {
    chromium_path: String,
}

impl ChromiumRenderer {
    /// 创建渲染器
    pub fn new(chromium_path: Option<String>) -> Self {
        Self {
            chromium_path: chromium_path.unwrap_or_default(),
        }
    }

    /// 启动浏览器并渲染 HTML 为 PDF
    async fn render_html_to_pdf(&self, html: &str) -> Result<Vec<u8>> {
        let config = if self.chromium_path.is_empty() {
            BrowserConfig::builder()
                .no_sandbox()
                .headless_mode(chromiumoxide::browser::HeadlessMode::New)
                .build()
                .map_err(|e| PrintCraftError::Platform(format!("Chrome 配置错误: {}", e)))?
        } else {
            BrowserConfig::builder()
                .chrome_executable(std::path::PathBuf::from(&self.chromium_path))
                .no_sandbox()
                .headless_mode(chromiumoxide::browser::HeadlessMode::New)
                .build()
                .map_err(|e| PrintCraftError::Platform(format!("Chrome 配置错误: {}", e)))?
        };

        let (mut browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| PrintCraftError::Platform(format!("启动 Chrome 失败: {}", e)))?;

        // 在后台处理 CDP 事件
        let handle = tokio::spawn(async move {
            while handler.next().await.is_some() {}
        });

        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|e| PrintCraftError::Platform(format!("创建页面失败: {}", e)))?;

        // 用 data URL 加载 HTML
        let data_url = format!(
            "data:text/html;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(html.as_bytes())
        );
        page.goto(&data_url)
            .await
            .map_err(|e| PrintCraftError::Platform(format!("加载 HTML 失败: {}", e)))?;

        // 等待页面加载
        page.wait_for_navigation()
            .await
            .map_err(|e| PrintCraftError::Platform(format!("等待页面加载失败: {}", e)))?;

        // 渲染 PDF
        let pdf_data = render_page_to_pdf(&page).await?;

        let _ = browser.close().await;
        handle.abort();

        Ok(pdf_data)
    }
}

/// 调用 CDP Page.printToPDF
async fn render_page_to_pdf(page: &Page) -> Result<Vec<u8>> {
    use chromiumoxide_cdp::cdp::browser_protocol::page::PrintToPdfParams;

    let pdf_data = page
        .pdf(PrintToPdfParams {
            print_background: Some(true),
            prefer_css_page_size: Some(true),
            ..Default::default()
        })
        .await
        .map_err(|e| PrintCraftError::Platform(format!("PDF 渲染失败: {}", e)))?;

    Ok(pdf_data)
}

#[async_trait]
impl PdfRenderer for ChromiumRenderer {
    async fn render(&self, job: &PrintJob) -> Result<Vec<u8>> {
        let html = assemble_html(job);
        self.render_html_to_pdf(&html).await
    }

    fn name(&self) -> &str {
        "ChromiumRenderer"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chromium_renderer_name() {
        let renderer = ChromiumRenderer::new(None);
        assert_eq!(renderer.name(), "ChromiumRenderer");
    }

    #[test]
    fn test_chromium_renderer_with_path() {
        let renderer = ChromiumRenderer::new(Some("/usr/bin/chromium".to_string()));
        assert_eq!(renderer.chromium_path, "/usr/bin/chromium");
    }
}
