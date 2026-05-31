//! 基于 printpdf 的简单渲染器
//!
//! 处理纯元素类型：TEXT, RECT, LINE, ELLIPSE, IMAGE, BARCODE, SHAPE。
//! 不支持 HTML 类型（需要 Chromium 引擎）。

use async_trait::async_trait;
use printcraft_core::config::{paper_size, PageOrientation};
use printcraft_core::elements::{BarcodeType, ElementPosition, PrintElement, PrintElementKind};
use printcraft_core::error::{PrintCraftError, Result};
use printcraft_core::print_job::PrintJob;
use printcraft_core::style::{Alignment, PrintStyle};
use printpdf::path::PaintMode;
use printpdf::{
    BuiltinFont, Color, Image, ImageTransform, IndirectFontRef, Line, LineDashPattern, Mm,
    PdfDocument, PdfLayerReference, Point, Rect, Rgb,
};
use std::io::BufWriter;

use super::barcode::generate_barcode_image;
use super::pdf_engine::PdfRenderer;

const DEFAULT_PAPER: (&str, f64, f64) = ("A4", 210.0, 297.0);

pub struct SimplePdfRenderer;

impl SimplePdfRenderer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PdfRenderer for SimplePdfRenderer {
    async fn render(&self, job: &PrintJob) -> Result<Vec<u8>> {
        let (page_w, page_h) = resolve_page_size(&job.page_config);

        let (doc, page_idx, layer_idx) =
            PdfDocument::new(&job.name, Mm(page_w as f32), Mm(page_h as f32), "Layer 1");
        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| PrintCraftError::Render(format!("Failed to load font: {e}")))?;

        for elem in &job.elements {
            let layer = doc.get_page(page_idx).get_layer(layer_idx);
            let style = effective_style(&job.default_style, elem);

            let x_mm = lodop_to_mm(elem.position.left);
            let w_mm = lodop_to_mm(elem.position.width);
            let h_mm = lodop_to_mm(elem.position.height);
            let pdf_y = page_h - lodop_to_mm(elem.position.top) - h_mm;

            match &elem.kind {
                PrintElementKind::Text { content } => {
                    render_text(&layer, &font, content, &style, &elem.position, page_h);
                }
                PrintElementKind::Rect {
                    border_width,
                    border_style,
                } => {
                    render_rect(&layer, &style, x_mm, pdf_y, w_mm, h_mm, *border_width, *border_style);
                }
                PrintElementKind::Line {
                    end_top,
                    end_left,
                    line_style,
                    line_width,
                } => {
                    render_line(
                        &layer,
                        &style,
                        &elem.position,
                        *end_top,
                        *end_left,
                        *line_style,
                        *line_width,
                        page_h,
                    );
                }
                PrintElementKind::Ellipse { border_width } => {
                    render_ellipse(&layer, &style, x_mm, pdf_y, w_mm, h_mm, *border_width);
                }
                PrintElementKind::Image { src } => {
                    render_image(&layer, src, x_mm, pdf_y, w_mm, h_mm)
                        .unwrap_or_else(|e| tracing::warn!("Image render failed: {e}"));
                }
                PrintElementKind::Barcode {
                    code,
                    barcode_type,
                } => {
                    render_barcode(&layer, &font, code, *barcode_type, &style, x_mm, pdf_y, w_mm, h_mm)
                        .unwrap_or_else(|e| tracing::warn!("Barcode render failed: {e}"));
                }
                PrintElementKind::Shape {
                    border_width,
                    border_style,
                    color,
                    ..
                } => {
                    render_shape(&layer, x_mm, pdf_y, w_mm, h_mm, *border_width, *border_style, color);
                }
                PrintElementKind::Html { html_content }
                | PrintElementKind::Table { html_content } => {
                    // HTML 降级为纯文本渲染（无 Chromium 时的回退）
                    let text = strip_html_tags(html_content);
                    render_text(&layer, &font, &text, &style, &elem.position, page_h);
                }
                PrintElementKind::Url { url } => {
                    render_text(&layer, &font, url, &style, &elem.position, page_h);
                }
            }
        }

        let mut buf = BufWriter::new(Vec::new());
        doc.save(&mut buf)
            .map_err(|e| PrintCraftError::Render(format!("PDF save failed: {e}")))?;
        buf.into_inner()
            .map_err(|e| PrintCraftError::Render(format!("Buffer flush failed: {e}")))
    }

    fn name(&self) -> &str {
        "printpdf-simple"
    }
}

fn resolve_page_size(config: &printcraft_core::config::PageConfig) -> (f64, f64) {
    let (mut w, mut h) = if config.width > 0.0 && config.height > 0.0 {
        (lodop_to_mm(config.width), lodop_to_mm(config.height))
    } else if let Some((pw, ph)) = paper_size(&config.page_name) {
        (pw, ph)
    } else {
        (DEFAULT_PAPER.1, DEFAULT_PAPER.2)
    };

    if config.orientation == PageOrientation::Landscape {
        std::mem::swap(&mut w, &mut h);
    }
    (w, h)
}

fn lodop_to_mm(v: f64) -> f64 {
    v / 10.0
}

fn effective_style(default: &PrintStyle, elem: &PrintElement) -> PrintStyle {
    let mut s = default.clone();
    if let Some(ref es) = elem.style {
        s.merge(es);
    }
    s
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    let h = hex.strip_prefix('#').unwrap_or(hex);
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()?;
    let g = u8::from_str_radix(&h[2..4], 16).ok()?;
    let b = u8::from_str_radix(&h[4..6], 16).ok()?;
    Some(Color::Rgb(Rgb::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        None,
    )))
}

fn apply_text_color(layer: &PdfLayerReference, style: &PrintStyle) {
    if let Some(ref color_str) = style.color {
        if let Some(color) = parse_hex_color(color_str) {
            layer.set_fill_color(color);
        }
    }
}

fn apply_outline_color(layer: &PdfLayerReference, style: &PrintStyle) {
    if let Some(ref color_str) = style.color {
        if let Some(color) = parse_hex_color(color_str) {
            layer.set_outline_color(color);
        }
    }
}

fn render_text(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    content: &str,
    style: &PrintStyle,
    pos: &ElementPosition,
    page_h_mm: f64,
) {
    let font_size = style.font_size.unwrap_or(12.0) as f32;
    let x_mm = lodop_to_mm(pos.left) as f32;
    let w_mm = lodop_to_mm(pos.width) as f32;

    apply_text_color(layer, style);

    let lines = if style.word_wrap.unwrap_or(true) && pos.width > 0.0 {
        wrap_text(content, w_mm, font_size)
    } else {
        content.lines().map(String::from).collect::<Vec<_>>()
    };

    let line_spacing = style.line_spacing.unwrap_or(1.0) as f32;
    let line_height = font_size * line_spacing * 0.3528; // pt -> mm

    for (i, line) in lines.iter().enumerate() {
        let line_y_offset = i as f32 * line_height;
        let pdf_y = (page_h_mm as f32 - lodop_to_mm(pos.top) as f32 - line_y_offset) - font_size * 0.3528;

        let aligned_x = match style.alignment {
            Some(Alignment::Center) => x_mm + (w_mm - estimate_text_width_mm(line, font_size)) / 2.0,
            Some(Alignment::Right) => x_mm + w_mm - estimate_text_width_mm(line, font_size),
            _ => x_mm,
        };

        layer.use_text(line.as_str(), font_size, Mm(aligned_x), Mm(pdf_y), font);
    }
}

fn wrap_text(content: &str, width_mm: f32, font_size: f32) -> Vec<String> {
    let avg_char_width = font_size * 0.5 * 0.3528; // rough: 0.5em per char, pt->mm
    if width_mm <= 0.0 || avg_char_width <= 0.0 {
        return content.lines().map(String::from).collect();
    }
    let chars_per_line = (width_mm / avg_char_width).floor() as usize;
    if chars_per_line == 0 {
        return vec![content.to_string()];
    }

    let mut result = Vec::new();
    for paragraph in content.lines() {
        if paragraph.is_empty() {
            result.push(String::new());
            continue;
        }
        let mut remaining = paragraph;
        while !remaining.is_empty() {
            if remaining.len() <= chars_per_line {
                result.push(remaining.to_string());
                break;
            }
            let mut split_at = chars_per_line;
            while split_at > 0 && !remaining.is_char_boundary(split_at) {
                split_at -= 1;
            }
            result.push(remaining[..split_at].to_string());
            remaining = &remaining[split_at..];
        }
    }
    result
}

fn estimate_text_width_mm(text: &str, font_size: f32) -> f32 {
    text.len() as f32 * font_size * 0.5 * 0.3528
}

/// 去除 HTML 标签，提取纯文本内容（简单实现）
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    // 解码常见 HTML 实体
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&nbsp;", " ")
        .replace("&#39;", "'")
}

fn render_rect(
    layer: &PdfLayerReference,
    style: &PrintStyle,
    x_mm: f64,
    y_mm: f64,
    w_mm: f64,
    h_mm: f64,
    border_width: f64,
    border_style: u8,
) {
    if border_style == 1 {
        layer.set_line_dash_pattern(LineDashPattern {
            offset: 0,
            dash_1: Some(3),
            gap_1: Some(3),
            dash_2: None,
            gap_2: None,
            dash_3: None,
            gap_3: None,
        });
    }

    apply_outline_color(layer, style);
    layer.set_outline_thickness(border_width as f32);

    let paint_mode = if border_width <= 0.0 {
        PaintMode::Fill
    } else {
        PaintMode::Stroke
    };

    let rect = Rect::new(
        Mm(x_mm as f32),
        Mm(y_mm as f32),
        Mm((x_mm + w_mm) as f32),
        Mm((y_mm + h_mm) as f32),
    )
    .with_mode(paint_mode);

    layer.add_rect(rect);

    if border_style == 1 {
        layer.set_line_dash_pattern(LineDashPattern::default());
    }
}

fn render_line(
    layer: &PdfLayerReference,
    style: &PrintStyle,
    pos: &ElementPosition,
    end_top: f64,
    end_left: f64,
    line_style: u8,
    line_width: f64,
    page_h_mm: f64,
) {
    if line_style == 1 {
        layer.set_line_dash_pattern(LineDashPattern {
            offset: 0,
            dash_1: Some(3),
            gap_1: Some(3),
            dash_2: None,
            gap_2: None,
            dash_3: None,
            gap_3: None,
        });
    }

    apply_outline_color(layer, style);
    layer.set_outline_thickness(line_width as f32);

    let start_x = lodop_to_mm(pos.left) as f32;
    let start_y = page_h_mm as f32 - lodop_to_mm(pos.top) as f32;
    let end_x = lodop_to_mm(end_left) as f32;
    let end_y = page_h_mm as f32 - lodop_to_mm(end_top) as f32;

    let line = Line::from_iter(vec![
        (Point::new(Mm(start_x), Mm(start_y)), false),
        (Point::new(Mm(end_x), Mm(end_y)), false),
    ]);
    layer.add_line(line);

    if line_style == 1 {
        layer.set_line_dash_pattern(LineDashPattern::default());
    }
}

fn render_ellipse(
    layer: &PdfLayerReference,
    style: &PrintStyle,
    x_mm: f64,
    y_mm: f64,
    w_mm: f64,
    h_mm: f64,
    border_width: f64,
) {
    apply_outline_color(layer, style);
    layer.set_outline_thickness(border_width as f32);

    let cx = x_mm + w_mm / 2.0;
    let cy = y_mm + h_mm / 2.0;
    let rx = w_mm / 2.0;
    let ry = h_mm / 2.0;

    // Approximate ellipse with cubic Bezier curves
    let k = 0.5522847498; // magic number for circle approximation
    let points = vec![
        // Bottom
        (Point::new(Mm((cx - rx) as f32), Mm(cy as f32)), false),
        (
            Point::new(Mm((cx - rx) as f32), Mm((cy - ry * k) as f32)),
            false,
        ),
        (
            Point::new(Mm((cx - rx * k) as f32), Mm((cy - ry) as f32)),
            false,
        ),
        (Point::new(Mm(cx as f32), Mm((cy - ry) as f32)), false),
        // Top-right
        (
            Point::new(Mm((cx + rx * k) as f32), Mm((cy - ry) as f32)),
            false,
        ),
        (
            Point::new(Mm((cx + rx) as f32), Mm((cy - ry * k) as f32)),
            false,
        ),
        (Point::new(Mm((cx + rx) as f32), Mm(cy as f32)), false),
        // Top-left
        (
            Point::new(Mm((cx + rx) as f32), Mm((cy + ry * k) as f32)),
            false,
        ),
        (
            Point::new(Mm((cx + rx * k) as f32), Mm((cy + ry) as f32)),
            false,
        ),
        (Point::new(Mm(cx as f32), Mm((cy + ry) as f32)), false),
        // Bottom-left
        (
            Point::new(Mm((cx - rx * k) as f32), Mm((cy + ry) as f32)),
            false,
        ),
        (
            Point::new(Mm((cx - rx) as f32), Mm((cy + ry * k) as f32)),
            false,
        ),
        (Point::new(Mm((cx - rx) as f32), Mm(cy as f32)), false),
    ];

    let mut bezier = Line::from_iter(points);
    bezier.set_closed(true);
    layer.add_line(bezier);
}

fn render_image(
    layer: &PdfLayerReference,
    src: &str,
    x_mm: f64,
    y_mm: f64,
    w_mm: f64,
    h_mm: f64,
) -> Result<()> {
    let img = load_image_from_src(src)?;
    let pdf_image = Image::from_dynamic_image(&img);

    let transform = ImageTransform {
        translate_x: Some(Mm(x_mm as f32)),
        translate_y: Some(Mm(y_mm as f32)),
        scale_x: Some(w_mm as f32 / img.width() as f32 * 300.0 / 25.4),
        scale_y: Some(h_mm as f32 / img.height() as f32 * 300.0 / 25.4),
        dpi: Some(300.0),
        ..Default::default()
    };

    pdf_image.add_to_layer(layer.clone(), transform);
    Ok(())
}

fn load_image_from_src(src: &str) -> Result<::image::DynamicImage> {
    if let Some(b64) = src.strip_prefix("data:").and_then(|s| s.find(",").map(|i| &s[i + 1..])) {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| PrintCraftError::Render(format!("Base64 decode failed: {e}")))?;
        return ::image::load_from_memory(&bytes)
            .map_err(|e| PrintCraftError::Render(format!("Image decode failed: {e}")));
    }
    Err(PrintCraftError::Unsupported(
        "URL image loading not yet implemented".into(),
    ))
}

fn render_barcode(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    code: &str,
    barcode_type: BarcodeType,
    style: &PrintStyle,
    x_mm: f64,
    y_mm: f64,
    w_mm: f64,
    h_mm: f64,
) -> Result<()> {
    let img = generate_barcode_image(code, barcode_type)?;
    let pdf_image = Image::from_dynamic_image(&img);

    let transform = ImageTransform {
        translate_x: Some(Mm(x_mm as f32)),
        translate_y: Some(Mm(y_mm as f32)),
        scale_x: Some(w_mm as f32 / img.width() as f32 * 300.0 / 25.4),
        scale_y: Some(h_mm as f32 / img.height() as f32 * 300.0 / 25.4),
        dpi: Some(300.0),
        ..Default::default()
    };

    pdf_image.add_to_layer(layer.clone(), transform);

    let font_size = style.font_size.unwrap_or(8.0) as f32;
    apply_text_color(layer, style);
    let text_y = y_mm as f32 - font_size * 0.3528;
    layer.use_text(code, font_size, Mm(x_mm as f32), Mm(text_y), font);

    Ok(())
}

fn render_shape(
    layer: &PdfLayerReference,
    x_mm: f64,
    y_mm: f64,
    w_mm: f64,
    h_mm: f64,
    border_width: f64,
    border_style: u8,
    color: &str,
) {
    if let Some(c) = parse_hex_color(color) {
        layer.set_outline_color(c);
    }

    if border_style == 1 {
        layer.set_line_dash_pattern(LineDashPattern {
            offset: 0,
            dash_1: Some(3),
            gap_1: Some(3),
            dash_2: None,
            gap_2: None,
            dash_3: None,
            gap_3: None,
        });
    }

    layer.set_outline_thickness(border_width as f32);

    let rect = Rect::new(
        Mm(x_mm as f32),
        Mm(y_mm as f32),
        Mm((x_mm + w_mm) as f32),
        Mm((y_mm + h_mm) as f32),
    )
    .with_mode(PaintMode::Stroke);

    layer.add_rect(rect);

    if border_style == 1 {
        layer.set_line_dash_pattern(LineDashPattern::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use printcraft_core::config::PageConfig;
    use printcraft_core::elements::{ElementPosition, PrintElement, PrintElementKind};
    use printcraft_core::print_job::PrintJob;

    fn make_element(kind: PrintElementKind) -> PrintElement {
        PrintElement {
            index: 1,
            position: ElementPosition {
                top: 100.0,
                left: 100.0,
                width: 500.0,
                height: 200.0,
            },
            style: None,
            kind,
        }
    }

    #[tokio::test]
    async fn test_empty_render() {
        let job = PrintJob::new("test");
        let renderer = SimplePdfRenderer::new();
        let pdf = renderer.render(&job).await.unwrap();
        assert!(pdf.len() > 0);
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[tokio::test]
    async fn test_render_text() {
        let mut job = PrintJob::new("text-test");
        job.add_element(make_element(PrintElementKind::Text {
            content: "Hello World".into(),
        }));
        let renderer = SimplePdfRenderer::new();
        let pdf = renderer.render(&job).await.unwrap();
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[tokio::test]
    async fn test_render_rect() {
        let mut job = PrintJob::new("rect-test");
        job.add_element(make_element(PrintElementKind::Rect {
            border_width: 1.0,
            border_style: 0,
        }));
        let renderer = SimplePdfRenderer::new();
        let pdf = renderer.render(&job).await.unwrap();
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[tokio::test]
    async fn test_render_line() {
        let mut job = PrintJob::new("line-test");
        job.add_element(PrintElement {
            index: 1,
            position: ElementPosition {
                top: 100.0,
                left: 100.0,
                width: 0.0,
                height: 0.0,
            },
            style: None,
            kind: PrintElementKind::Line {
                end_top: 500.0,
                end_left: 1000.0,
                line_style: 0,
                line_width: 1.0,
            },
        });
        let renderer = SimplePdfRenderer::new();
        let pdf = renderer.render(&job).await.unwrap();
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[tokio::test]
    async fn test_html_fallback_to_text() {
        let mut job = PrintJob::new("html-test");
        job.add_element(make_element(PrintElementKind::Html {
            html_content: "<b>Hello</b> <i>World</i>".into(),
        }));
        let renderer = SimplePdfRenderer::new();
        let pdf = renderer.render(&job).await.unwrap();
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[tokio::test]
    async fn test_render_barcode_qr() {
        let mut job = PrintJob::new("barcode-test");
        job.add_element(make_element(PrintElementKind::Barcode {
            code: "https://example.com".into(),
            barcode_type: BarcodeType::QRCode,
        }));
        let renderer = SimplePdfRenderer::new();
        let pdf = renderer.render(&job).await.unwrap();
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn test_resolve_page_size_a4() {
        let config = PageConfig::default();
        let (w, h) = resolve_page_size(&config);
        assert!((w - 210.0).abs() < 0.1);
        assert!((h - 297.0).abs() < 0.1);
    }

    #[test]
    fn test_resolve_page_size_landscape() {
        let mut config = PageConfig::default();
        config.orientation = PageOrientation::Landscape;
        let (w, h) = resolve_page_size(&config);
        assert!((w - 297.0).abs() < 0.1);
        assert!((h - 210.0).abs() < 0.1);
    }

    #[test]
    fn test_parse_hex_color() {
        let color = parse_hex_color("#FF0000").unwrap();
        match color {
            Color::Rgb(rgb) => {
                assert!((rgb.r - 1.0).abs() < 0.01);
                assert!((rgb.g - 0.0).abs() < 0.01);
                assert!((rgb.b - 0.0).abs() < 0.01);
            }
            _ => panic!("Expected RGB color"),
        }
    }

    #[test]
    fn test_parse_hex_color_no_hash() {
        let color = parse_hex_color("00FF00").unwrap();
        match color {
            Color::Rgb(rgb) => {
                assert!((rgb.g - 1.0).abs() < 0.01);
            }
            _ => panic!("Expected RGB color"),
        }
    }

    #[test]
    fn test_wrap_text() {
        let lines = wrap_text("Hello World Test", 10.0, 12.0);
        assert!(lines.len() >= 2);
    }
}
