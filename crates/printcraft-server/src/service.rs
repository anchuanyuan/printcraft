//! 打印服务编排器
//!
//! 接收 WebSocket 命令，构建 PrintJob，调用渲染器和平台层。

use std::collections::HashMap;
use std::sync::Arc;

use printcraft_core::config::PageOrientation;
use printcraft_core::elements::{BarcodeType, ElementPosition, PrintElement, PrintElementKind};
use printcraft_core::error::{PrintCraftError, Result};
use printcraft_core::print_job::PrintJob;
use printcraft_core::queue::PrintQueue;
use printcraft_core::style::PrintStyle;
use printcraft_platform::PlatformPrinter;
use printcraft_render::PdfRenderer;
use tokio::sync::RwLock;

/// 预览数据存储（线程安全）
pub type PreviewStore = Arc<RwLock<HashMap<String, PreviewEntry>>>;

/// 预览条目
pub struct PreviewEntry {
    /// 预览 HTML 内容
    pub html: String,
    /// PDF 字节数据
    pub pdf: Vec<u8>,
    /// 创建时间
    pub created_at: std::time::Instant,
}

/// 创建预览存储
pub fn create_preview_store() -> PreviewStore {
    Arc::new(RwLock::new(HashMap::new()))
}

/// 打印服务编排器
///
/// 连接 WebSocket 命令到渲染引擎和平台打印层。
pub struct PrintService {
    renderer: Arc<dyn PdfRenderer>,
    platform: Arc<dyn PlatformPrinter>,
    queue: PrintQueue,
    current_job: Option<PrintJob>,
    previews: PreviewStore,
}

impl PrintService {
    /// 创建新的打印服务
    pub fn new(renderer: Arc<dyn PdfRenderer>, platform: Arc<dyn PlatformPrinter>, previews: PreviewStore) -> Self {
        Self {
            renderer,
            platform,
            queue: PrintQueue::new(16),
            current_job: None,
            previews,
        }
    }

    /// 处理 Lodop 兼容命令
    ///
    /// 根据 cmd 名称分发到对应的处理逻辑。
    pub async fn handle_command(
        &mut self,
        cmd: &str,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        match cmd {
            "PRINT_INIT" => self.cmd_print_init(args),
            "SET_PRINT_PAGESIZE" => self.cmd_set_print_pagesize(args),
            "ADD_PRINT_TEXT" => self.cmd_add_print_text(args),
            "ADD_PRINT_RECT" => self.cmd_add_print_rect(args),
            "ADD_PRINT_LINE" => self.cmd_add_print_line(args),
            "ADD_PRINT_IMAGE" => self.cmd_add_print_image(args),
            "ADD_PRINT_BARCODE" => self.cmd_add_print_barcode(args),
            "ADD_PRINT_HTM" => self.cmd_add_print_htm(args),
            "ADD_PRINT_TABLE" => self.cmd_add_print_table(args),
            "ADD_PRINT_URL" => self.cmd_add_print_url(args),
            "ADD_PRINT_ELLIPSE" => self.cmd_add_print_ellipse(args),
            "ADD_PRINT_SHAPE" => self.cmd_add_print_shape(args),
            "SET_PRINT_STYLE" => self.cmd_set_print_style(args),
            "SET_PRINTER_INDEX" | "SET_PRINTER_INDEXA" => self.cmd_set_printer_index(args),
            "SET_PRINT_COPIES" => self.cmd_set_print_copies(args),
            "PRINT" => self.cmd_print().await,
            "PREVIEW" => self.cmd_preview().await,
            "GET_PRINTER_COUNT" => self.cmd_get_printer_count().await,
            "GET_PRINTER_NAME" => self.cmd_get_printer_name(args).await,
            "GET_PRINT_IN_VALUE" => self.cmd_get_print_in_value(args).await,
            _ => Err(PrintCraftError::Unsupported(format!("未知命令: {}", cmd))),
        }
    }

    /// PRINT_INIT - 初始化打印任务
    fn cmd_print_init(&mut self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("PrintCraft Job")
            .to_string();

        self.current_job = Some(PrintJob::new(&name));
        tracing::info!("PRINT_INIT: {}", name);
        Ok(serde_json::json!({ "ok": true }))
    }

    /// SET_PRINT_PAGESIZE - 设置页面尺寸
    fn cmd_set_print_pagesize(&mut self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let job = self.current_job.as_mut().ok_or_else(|| {
            PrintCraftError::PrintJob("未初始化打印任务，请先调用 PRINT_INIT".to_string())
        })?;

        // orientation: 1=Portrait, 2=Landscape
        let orientation = args
            .get("orientation")
            .and_then(|v| v.as_i64())
            .unwrap_or(1);
        job.page_config.orientation = match orientation {
            2 => PageOrientation::Landscape,
            _ => PageOrientation::Portrait,
        };

        // width, height in Lodop units (0.1mm)
        if let Some(w) = args.get("width").and_then(|v| v.as_f64()) {
            job.page_config.width = w;
        }
        if let Some(h) = args.get("height").and_then(|v| v.as_f64()) {
            job.page_config.height = h;
        }

        tracing::info!(
            "SET_PRINT_PAGESIZE: orientation={:?}, {}x{}",
            job.page_config.orientation,
            job.page_config.width,
            job.page_config.height
        );
        Ok(serde_json::json!({ "ok": true }))
    }

    /// ADD_PRINT_TEXT - 添加文本元素
    fn cmd_add_print_text(&mut self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let job = self.current_job.as_mut().ok_or_else(|| {
            PrintCraftError::PrintJob("未初始化打印任务，请先调用 PRINT_INIT".to_string())
        })?;

        let position = parse_position(args);
        let content = args
            .get("strContent")
            .or_else(|| args.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let element = PrintElement {
            index: 0,
            position,
            style: None,
            kind: PrintElementKind::Text { content },
        };
        job.add_element(element);

        tracing::debug!("ADD_PRINT_TEXT: added element");
        Ok(serde_json::json!({ "ok": true }))
    }

    /// ADD_PRINT_RECT - 添加矩形元素
    fn cmd_add_print_rect(&mut self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let job = self.current_job.as_mut().ok_or_else(|| {
            PrintCraftError::PrintJob("未初始化打印任务，请先调用 PRINT_INIT".to_string())
        })?;

        let position = parse_position(args);
        let border_width = args
            .get("borderWidth")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        let border_style = args
            .get("borderStyle")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;

        let element = PrintElement {
            index: 0,
            position,
            style: None,
            kind: PrintElementKind::Rect {
                border_width,
                border_style,
            },
        };
        job.add_element(element);

        tracing::debug!("ADD_PRINT_RECT: added element");
        Ok(serde_json::json!({ "ok": true }))
    }

    /// ADD_PRINT_LINE - 添加直线元素
    fn cmd_add_print_line(&mut self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let job = self.current_job.as_mut().ok_or_else(|| {
            PrintCraftError::PrintJob("未初始化打印任务，请先调用 PRINT_INIT".to_string())
        })?;

        let position = parse_position(args);
        let end_top = args
            .get("endTop")
            .and_then(|v| v.as_f64())
            .unwrap_or(position.top + position.height);
        let end_left = args
            .get("endLeft")
            .and_then(|v| v.as_f64())
            .unwrap_or(position.left + position.width);
        let line_style = args
            .get("lineStyle")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;
        let line_width = args
            .get("lineWidth")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);

        let element = PrintElement {
            index: 0,
            position,
            style: None,
            kind: PrintElementKind::Line {
                end_top,
                end_left,
                line_style,
                line_width,
            },
        };
        job.add_element(element);

        tracing::debug!("ADD_PRINT_LINE: added element");
        Ok(serde_json::json!({ "ok": true }))
    }

    /// ADD_PRINT_IMAGE - 添加图片元素
    fn cmd_add_print_image(&mut self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let job = self.current_job.as_mut().ok_or_else(|| {
            PrintCraftError::PrintJob("未初始化打印任务，请先调用 PRINT_INIT".to_string())
        })?;

        let position = parse_position(args);
        let src = args
            .get("src")
            .or_else(|| args.get("strContent"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let element = PrintElement {
            index: 0,
            position,
            style: None,
            kind: PrintElementKind::Image { src },
        };
        job.add_element(element);

        tracing::debug!("ADD_PRINT_IMAGE: added element");
        Ok(serde_json::json!({ "ok": true }))
    }

    /// ADD_PRINT_BARCODE - 添加条码元素
    fn cmd_add_print_barcode(&mut self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let job = self.current_job.as_mut().ok_or_else(|| {
            PrintCraftError::PrintJob("未初始化打印任务，请先调用 PRINT_INIT".to_string())
        })?;

        let position = parse_position(args);
        let code = args
            .get("strContent")
            .or_else(|| args.get("code"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let barcode_type = args
            .get("barcodeType")
            .and_then(|v| v.as_str())
            .map(parse_barcode_type)
            .unwrap_or(BarcodeType::Code128);

        let element = PrintElement {
            index: 0,
            position,
            style: None,
            kind: PrintElementKind::Barcode {
                code,
                barcode_type,
            },
        };
        job.add_element(element);

        tracing::debug!("ADD_PRINT_BARCODE: added element");
        Ok(serde_json::json!({ "ok": true }))
    }

    /// ADD_PRINT_HTM - 添加 HTML 元素
    fn cmd_add_print_htm(&mut self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let job = self.current_job.as_mut().ok_or_else(|| {
            PrintCraftError::PrintJob("未初始化打印任务，请先调用 PRINT_INIT".to_string())
        })?;

        let position = parse_position(args);
        let html_content = args
            .get("strHTML")
            .or_else(|| args.get("strContent"))
            .or_else(|| args.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let element = PrintElement {
            index: 0,
            position,
            style: None,
            kind: PrintElementKind::Html { html_content },
        };
        job.add_element(element);

        tracing::debug!("ADD_PRINT_HTM: added element");
        Ok(serde_json::json!({ "ok": true }))
    }

    /// ADD_PRINT_TABLE - 添加表格元素
    fn cmd_add_print_table(&mut self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let job = self.current_job.as_mut().ok_or_else(|| {
            PrintCraftError::PrintJob("未初始化打印任务，请先调用 PRINT_INIT".to_string())
        })?;

        let position = parse_position(args);
        let html_content = args
            .get("strHTML")
            .or_else(|| args.get("strContent"))
            .or_else(|| args.get("content"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let element = PrintElement {
            index: 0,
            position,
            style: None,
            kind: PrintElementKind::Table { html_content },
        };
        job.add_element(element);

        tracing::debug!("ADD_PRINT_TABLE: added element");
        Ok(serde_json::json!({ "ok": true }))
    }

    /// ADD_PRINT_URL - 添加网页元素
    fn cmd_add_print_url(&mut self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let job = self.current_job.as_mut().ok_or_else(|| {
            PrintCraftError::PrintJob("未初始化打印任务，请先调用 PRINT_INIT".to_string())
        })?;

        let position = parse_position(args);
        let url = args
            .get("strURL")
            .or_else(|| args.get("strUrl"))
            .or_else(|| args.get("url"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let element = PrintElement {
            index: 0,
            position,
            style: None,
            kind: PrintElementKind::Url { url },
        };
        job.add_element(element);

        tracing::debug!("ADD_PRINT_URL: added element");
        Ok(serde_json::json!({ "ok": true }))
    }

    /// ADD_PRINT_ELLIPSE - 添加椭圆元素
    fn cmd_add_print_ellipse(&mut self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let job = self.current_job.as_mut().ok_or_else(|| {
            PrintCraftError::PrintJob("未初始化打印任务，请先调用 PRINT_INIT".to_string())
        })?;

        let position = parse_position(args);
        let border_width = args
            .get("intLineWidth")
            .or_else(|| args.get("lineWidth"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);

        let element = PrintElement {
            index: 0,
            position,
            style: None,
            kind: PrintElementKind::Ellipse { border_width },
        };
        job.add_element(element);

        tracing::debug!("ADD_PRINT_ELLIPSE: added element");
        Ok(serde_json::json!({ "ok": true }))
    }

    /// ADD_PRINT_SHAPE - 添加形状元素
    fn cmd_add_print_shape(&mut self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let job = self.current_job.as_mut().ok_or_else(|| {
            PrintCraftError::PrintJob("未初始化打印任务，请先调用 PRINT_INIT".to_string())
        })?;

        let position = parse_position(args);
        let shape_type = args
            .get("intShapeType")
            .or_else(|| args.get("shapeType"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;
        let border_width = args
            .get("intLineWidth")
            .or_else(|| args.get("lineWidth"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        let border_style = args
            .get("intLineStyle")
            .or_else(|| args.get("lineStyle"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;
        let color = args
            .get("strColor")
            .or_else(|| args.get("color"))
            .and_then(|v| v.as_str())
            .unwrap_or("#000000")
            .to_string();

        let element = PrintElement {
            index: 0,
            position,
            style: None,
            kind: PrintElementKind::Shape {
                shape_type,
                border_width,
                border_style,
                color,
            },
        };
        job.add_element(element);

        tracing::debug!("ADD_PRINT_SHAPE: added element");
        Ok(serde_json::json!({ "ok": true }))
    }

    /// SET_PRINT_STYLE - 设置默认样式
    fn cmd_set_print_style(&mut self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let job = self.current_job.as_mut().ok_or_else(|| {
            PrintCraftError::PrintJob("未初始化打印任务，请先调用 PRINT_INIT".to_string())
        })?;

        let mut style = PrintStyle::new();

        if let Some(v) = args.get("strFontName").and_then(|v| v.as_str()) {
            style.font_name = Some(v.to_string());
        }
        if let Some(v) = args.get("FontSize").or_else(|| args.get("fontSize")).and_then(|v| v.as_f64()) {
            style.font_size = Some(v);
        }
        if let Some(v) = args.get("Bold").and_then(|v| v.as_bool()) {
            style.bold = Some(v);
        }
        if let Some(v) = args.get("Italic").and_then(|v| v.as_bool()) {
            style.italic = Some(v);
        }
        if let Some(v) = args.get("UnderLine").and_then(|v| v.as_bool()) {
            style.underline = Some(v);
        }
        if let Some(v) = args.get("Alignment").and_then(|v| v.as_i64()) {
            use printcraft_core::style::Alignment;
            style.alignment = Some(match v {
                1 => Alignment::Center,
                2 => Alignment::Right,
                _ => Alignment::Left,
            });
        }
        if let Some(v) = args.get("FontColor").and_then(|v| v.as_str()) {
            style.color = Some(v.to_string());
        }

        job.default_style.merge(&style);

        tracing::debug!("SET_PRINT_STYLE: updated default style");
        Ok(serde_json::json!({ "ok": true }))
    }

    /// SET_PRINTER_INDEX / SET_PRINTER_INDEXA - 设置目标打印机
    fn cmd_set_printer_index(&mut self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let job = self.current_job.as_mut().ok_or_else(|| {
            PrintCraftError::PrintJob("未初始化打印任务，请先调用 PRINT_INIT".to_string())
        })?;

        // 支持按名称或索引设置打印机
        if let Some(name) = args.get("printerName").or_else(|| args.get("name")).and_then(|v| v.as_str()) {
            job.set_printer(name);
            tracing::info!("SET_PRINTER_INDEX: printer={}", name);
        } else if let Some(_index) = args.get("index").and_then(|v| v.as_i64()) {
            // Index-based printer lookup: stored as index, resolved at print time
            // For now, store the index string so we can resolve later
            // TODO: resolve index to name via platform.list_printers()
            tracing::info!("SET_PRINTER_INDEX: index={}", _index);
        }

        Ok(serde_json::json!({ "ok": true }))
    }

    /// SET_PRINT_COPIES - 设置打印份数
    fn cmd_set_print_copies(&mut self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let job = self.current_job.as_mut().ok_or_else(|| {
            PrintCraftError::PrintJob("未初始化打印任务，请先调用 PRINT_INIT".to_string())
        })?;

        let copies = args
            .get("copies")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u32;
        job.set_copies(copies);

        tracing::info!("SET_PRINT_COPIES: {}", copies);
        Ok(serde_json::json!({ "ok": true }))
    }

    /// PRINT - 提交打印任务
    async fn cmd_print(&mut self) -> Result<serde_json::Value> {
        let job = self.current_job.take().ok_or_else(|| {
            PrintCraftError::PrintJob("未初始化打印任务，请先调用 PRINT_INIT".to_string())
        })?;

        tracing::info!("PRINT: rendering job '{}' with {} elements", job.name, job.elements.len());

        // 1. Render to PDF
        let pdf_bytes = self.renderer.render(&job).await?;
        if pdf_bytes.is_empty() {
            tracing::warn!("渲染器返回空 PDF，跳过打印");
            return Ok(serde_json::json!({ "ok": true, "warning": "empty PDF" }));
        }

        // 2. Send to platform printer
        self.platform
            .print_pdf(&job.printer, &pdf_bytes, job.copies, &job.name)
            .await?;

        tracing::info!("PRINT: job '{}' sent to printer", job.name);
        Ok(serde_json::json!({
            "ok": true,
            "jobId": job.id,
            "name": job.name,
        }))
    }

    /// PREVIEW - 渲染为 HTML + PDF，存储并返回预览 ID
    async fn cmd_preview(&mut self) -> Result<serde_json::Value> {
        let job = self.current_job.as_ref().ok_or_else(|| {
            PrintCraftError::PrintJob("未初始化打印任务，请先调用 PRINT_INIT".to_string())
        })?;

        tracing::info!("PREVIEW: rendering job '{}' for preview", job.name);

        let html = printcraft_render::template::assemble_html(job);
        let pdf_bytes = self.renderer.render(job).await?;
        let preview_id = uuid::Uuid::new_v4().to_string();

        self.previews.write().await.insert(
            preview_id.clone(),
            PreviewEntry {
                html,
                pdf: pdf_bytes.clone(),
                created_at: std::time::Instant::now(),
            },
        );

        Ok(serde_json::json!({
            "ok": true,
            "previewId": preview_id,
            "size": pdf_bytes.len(),
        }))
    }

    /// GET_PRINTER_COUNT - 获取打印机数量
    async fn cmd_get_printer_count(&self) -> Result<serde_json::Value> {
        let printers = self.platform.list_printers().await?;
        Ok(serde_json::json!({
            "ok": true,
            "count": printers.len(),
        }))
    }

    /// GET_PRINTER_NAME - 获取指定索引的打印机名称
    async fn cmd_get_printer_name(&self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let index = args
            .get("index")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let printers = self.platform.list_printers().await?;
        if index >= printers.len() {
            return Err(PrintCraftError::PrinterNotFound(format!(
                "打印机索引 {} 超出范围 (共 {} 台)",
                index,
                printers.len()
            )));
        }

        Ok(serde_json::json!({
            "ok": true,
            "name": printers[index].name,
            "isDefault": printers[index].is_default,
        }))
    }

    /// GET_PRINT_IN_VALUE - 获取打印机详细信息
    ///
    /// intType:
    ///   0 = 打印机名称
    ///   1 = 纸张名称列表
    ///   2 = 打印机状态
    async fn cmd_get_print_in_value(&self, args: &serde_json::Value) -> Result<serde_json::Value> {
        let printer_name = args
            .get("strPrinterName")
            .or_else(|| args.get("printerName"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let value_type = args
            .get("intType")
            .or_else(|| args.get("type"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        match value_type {
            0 => {
                // 返回打印机名称
                let printers = self.platform.list_printers().await?;
                let name = if printer_name.is_empty() {
                    printers.iter().find(|p| p.is_default).map(|p| p.name.clone()).unwrap_or_default()
                } else {
                    printer_name.to_string()
                };
                Ok(serde_json::json!({ "ok": true, "value": name }))
            }
            1 => {
                // 返回纸张列表
                let paper_sizes = self.platform.get_paper_sizes(printer_name).await?;
                let names: Vec<String> = paper_sizes.iter().map(|p| p.name.clone()).collect();
                Ok(serde_json::json!({ "ok": true, "value": names.join(",") }))
            }
            2 => {
                // 返回打印机状态
                let printers = self.platform.list_printers().await?;
                let printer = if printer_name.is_empty() {
                    printers.iter().find(|p| p.is_default)
                } else {
                    printers.iter().find(|p| p.name == printer_name)
                };
                let status = printer.map(|p| format!("{:?}", p.status)).unwrap_or("Unknown".to_string());
                Ok(serde_json::json!({ "ok": true, "value": status }))
            }
            _ => Ok(serde_json::json!({ "ok": true, "value": "" })),
        }
    }

    /// 列出所有打印机（供 REST API 调用）
    pub async fn list_printers(&self) -> Result<serde_json::Value> {
        let printers = self.platform.list_printers().await?;
        Ok(serde_json::json!(printers))
    }
}

/// 从 JSON args 解析 ElementPosition
fn parse_position(args: &serde_json::Value) -> ElementPosition {
    ElementPosition {
        top: args.get("top").and_then(|v| v.as_f64()).unwrap_or(0.0),
        left: args.get("left").and_then(|v| v.as_f64()).unwrap_or(0.0),
        width: args.get("width").and_then(|v| v.as_f64()).unwrap_or(100.0),
        height: args.get("height").and_then(|v| v.as_f64()).unwrap_or(30.0),
    }
}

/// 解析条码类型字符串
fn parse_barcode_type(s: &str) -> BarcodeType {
    match s.to_uppercase().as_str() {
        "128A" => BarcodeType::Code128A,
        "128B" => BarcodeType::Code128B,
        "128C" => BarcodeType::Code128C,
        "128" => BarcodeType::Code128,
        "39" | "CODE39" => BarcodeType::Code39,
        "EAN13" | "EAN-13" => BarcodeType::EAN13,
        "EAN8" | "EAN-8" => BarcodeType::EAN8,
        "UPCA" | "UPC-A" => BarcodeType::UPCA,
        "QR" | "QRCODE" | "QR_CODE" => BarcodeType::QRCode,
        "PDF417" => BarcodeType::PDF417,
        _ => BarcodeType::Code128,
    }
}

/// 简单的 base64 编码（无外部依赖）
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}
