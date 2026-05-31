//! HTML 模板组装
//!
//! 将 PrintJob 中的所有元素转换为 HTML 页面。
//! 使用绝对定位（Lodop 风格坐标），供 Chromium CDP 渲染为 PDF。

use printcraft_core::config::{paper_size, PageOrientation};
use printcraft_core::elements::{PrintElement, PrintElementKind};
use printcraft_core::print_job::PrintJob;
use printcraft_core::style::{Alignment, PrintStyle};

/// 将 PrintJob 组装为完整 HTML 页面
pub fn assemble_html(job: &PrintJob) -> String {
    let (page_w_mm, page_h_mm) = resolve_page_size(&job.page_config);
    let is_landscape = job.page_config.orientation == PageOrientation::Landscape;

    let (w, h) = if is_landscape {
        (page_h_mm, page_w_mm)
    } else {
        (page_w_mm, page_h_mm)
    };

    let mut elements_html = String::new();
    for elem in &job.elements {
        elements_html.push_str(&render_element(elem, &job.default_style));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<style>
  @page {{ size: {w}mm {h}mm; margin: 0; }}
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  html, body {{ width: {w}mm; height: {h}mm; overflow: hidden; position: relative; }}
  .el {{ position: absolute; overflow: hidden; }}
  .el-text {{ white-space: pre-wrap; word-wrap: break-word; }}
  .el-rect {{ border-style: solid; }}
  .el-rect-dashed {{ border-style: dashed; }}
  .el-line {{ position: absolute; }}
  .el-img img {{ width: 100%; height: 100%; object-fit: contain; }}
</style>
</head>
<body>
{elements_html}
</body>
</html>"#,
        w = w,
        h = h,
        elements_html = elements_html,
    )
}

/// 解析页面尺寸 (mm)
fn resolve_page_size(config: &printcraft_core::config::PageConfig) -> (f64, f64) {
    // 用户指定了自定义尺寸
    if config.width > 0.0 && config.height > 0.0 {
        return (lodop_to_mm(config.width), lodop_to_mm(config.height));
    }
    // 按纸张名称查找
    if let Some((w, h)) = paper_size(&config.page_name) {
        return (w, h);
    }
    // 默认 A4
    (210.0, 297.0)
}

/// Lodop 单位 (0.1mm) → mm
fn lodop_to_mm(val: f64) -> f64 {
    val / 10.0
}

/// 渲染单个元素为 HTML
fn render_element(elem: &PrintElement, default_style: &PrintStyle) -> String {
    let style = elem.style.as_ref().unwrap_or(default_style);
    let pos = &elem.position;

    let top = lodop_to_mm(pos.top);
    let left = lodop_to_mm(pos.left);
    let width = lodop_to_mm(pos.width);
    let height = lodop_to_mm(pos.height);

    let base_style = format!(
        "top:{top}mm;left:{left}mm;width:{width}mm;height:{height}mm;",
        top = top,
        left = left,
        width = width,
        height = height,
    );

    match &elem.kind {
        PrintElementKind::Text { content } => {
            let text_style = build_text_style(style);
            format!(
                r#"<div class="el el-text" style="{base}{text}">{content}</div>"#,
                base = base_style,
                text = text_style,
                content = escape_html(content),
            )
        }
        PrintElementKind::Html { html_content } => {
            let text_style = build_text_style(style);
            format!(
                r#"<div class="el" style="{base}{text}">{html}</div>"#,
                base = base_style,
                text = text_style,
                html = html_content,
            )
        }
        PrintElementKind::Table { html_content } => {
            let text_style = build_text_style(style);
            format!(
                r#"<div class="el" style="{base}{text}">{html}</div>"#,
                base = base_style,
                text = text_style,
                html = html_content,
            )
        }
        PrintElementKind::Url { url } => {
            format!(
                r#"<iframe class="el" style="{base}border:none;" src="{url}"></iframe>"#,
                base = base_style,
                url = escape_html(url),
            )
        }
        PrintElementKind::Image { src } => {
            format!(
                r#"<div class="el el-img" style="{base}"><img src="{src}"></div>"#,
                base = base_style,
                src = escape_html(src),
            )
        }
        PrintElementKind::Rect {
            border_width,
            border_style,
        } => {
            let bw = *border_style;
            let cls = if bw == 1 {
                "el-rect-dashed"
            } else {
                "el-rect"
            };
            let lw = lodop_to_mm(*border_width).max(0.1);
            format!(
                r#"<div class="el {cls}" style="{base}border-width:{lw}mm;border-color:black;"></div>"#,
                cls = cls,
                base = base_style,
                lw = lw,
            )
        }
        PrintElementKind::Line {
            end_top: _,
            end_left: _,
            line_style,
            line_width,
        } => {
            // 用 SVG 画直线（支持虚线）
            let lw = lodop_to_mm(*line_width).max(0.1);
            let dash = if *line_style == 1 {
                format!("stroke-dasharray=\"{}\"", lw * 3.0)
            } else {
                String::new()
            };
            format!(
                r#"<svg class="el el-line" style="{base}" xmlns="http://www.w3.org/2000/svg">
  <line x1="0" y1="0" x2="{w}mm" y2="{h}mm" stroke="black" stroke-width="{lw}mm" {dash}/>
</svg>"#,
                base = base_style,
                w = width,
                h = height,
                lw = lw,
                dash = dash,
            )
        }
        PrintElementKind::Ellipse { border_width } => {
            let lw = lodop_to_mm(*border_width).max(0.1);
            format!(
                r#"<div class="el" style="{base}border:{lw}mm solid black;border-radius:50%;"></div>"#,
                base = base_style,
                lw = lw,
            )
        }
        PrintElementKind::Barcode {
            code,
            barcode_type: _,
        } => {
            // 条码作为文字占位符渲染（Chromium 渲染时由 JS 替换为图片）
            format!(
                r#"<div class="el el-text" style="{base}font-family:monospace;font-size:10pt;text-align:center;display:flex;align-items:center;justify-content:center;">[Barcode: {code}]</div>"#,
                base = base_style,
                code = escape_html(code),
            )
        }
        PrintElementKind::Shape {
            shape_type,
            border_width,
            border_style,
            color,
        } => {
            let lw = lodop_to_mm(*border_width).max(0.1);
            let dash = if *border_style == 1 {
                "border-style:dashed;"
            } else {
                "border-style:solid;"
            };
            let border_css = format!("border:{lw}mm {dash}border-color:{color};",);
            let shape_css = match shape_type {
                1 => "border-radius:50%;",           // 椭圆
                2 => "border-radius:0;",             // 矩形
                3 => "clip-path:polygon(50% 0%,0% 100%,100% 100%);", // 三角形
                _ => "",
            };
            format!(
                r#"<div class="el" style="{base}{border}{shape}"></div>"#,
                base = base_style,
                border = border_css,
                shape = shape_css,
            )
        }
    }
}

/// 构建文字相关 CSS
fn build_text_style(style: &PrintStyle) -> String {
    let mut css = String::new();

    if let Some(ref font) = style.font_name {
        css.push_str(&format!("font-family:'{}',sans-serif;", font));
    }
    if let Some(size) = style.font_size {
        css.push_str(&format!("font-size:{}pt;", size));
    }
    if style.bold.unwrap_or(false) {
        css.push_str("font-weight:bold;");
    }
    if style.italic.unwrap_or(false) {
        css.push_str("font-style:italic;");
    }
    if style.underline.unwrap_or(false) {
        css.push_str("text-decoration:underline;");
    }
    if let Some(ref align) = style.alignment {
        let align_str = match align {
            Alignment::Left => "left",
            Alignment::Center => "center",
            Alignment::Right => "right",
        };
        css.push_str(&format!("text-align:{};", align_str));
    }
    if let Some(ref color) = style.color {
        css.push_str(&format!("color:{};", color));
    }
    if let Some(spacing) = style.line_spacing {
        css.push_str(&format!("line-height:{};", spacing));
    }

    css
}

/// HTML 转义
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use printcraft_core::elements::{ElementPosition, PrintElementKind};
    use printcraft_core::style::PrintStyle;

    fn make_job_with_elements() -> PrintJob {
        let mut job = PrintJob::new("test");
        job.add_element(PrintElement {
            index: 0,
            position: ElementPosition {
                top: 50.0,
                left: 50.0,
                width: 300.0,
                height: 30.0,
            },
            style: Some(PrintStyle {
                font_name: Some("SimSun".to_string()),
                font_size: Some(12.0),
                ..Default::default()
            }),
            kind: PrintElementKind::Text {
                content: "Hello World".to_string(),
            },
        });
        job.add_element(PrintElement {
            index: 0,
            position: ElementPosition {
                top: 40.0,
                left: 40.0,
                width: 320.0,
                height: 100.0,
            },
            style: None,
            kind: PrintElementKind::Rect {
                border_width: 1.0,
                border_style: 0,
            },
        });
        job
    }

    #[test]
    fn test_assemble_html_a4() {
        let job = make_job_with_elements();
        let html = assemble_html(&job);

        assert!(html.contains("@page"));
        assert!(html.contains("210mm"));
        assert!(html.contains("297mm"));
        assert!(html.contains("Hello World"));
        assert!(html.contains("el-rect"));
    }

    #[test]
    fn test_assemble_html_landscape() {
        let mut job = make_job_with_elements();
        job.page_config.orientation = PageOrientation::Landscape;
        let html = assemble_html(&job);

        // Landscape: width=297, height=210
        assert!(html.contains("297mm"));
        assert!(html.contains("210mm"));
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>alert('x')</script>"), "&lt;script&gt;alert(&#39;x&#39;)&lt;/script&gt;");
    }

    #[test]
    fn test_text_style() {
        let style = PrintStyle {
            font_name: Some("Arial".to_string()),
            font_size: Some(14.0),
            bold: Some(true),
            alignment: Some(Alignment::Center),
            ..PrintStyle::new()
        };
        let css = build_text_style(&style);
        assert!(css.contains("font-family:'Arial'"));
        assert!(css.contains("font-size:14pt"));
        assert!(css.contains("font-weight:bold"));
        assert!(css.contains("text-align:center"));
    }

    #[test]
    fn test_resolve_page_size_custom() {
        let config = printcraft_core::config::PageConfig {
            width: 1000.0,
            height: 500.0,
            ..Default::default()
        };
        let (w, h) = resolve_page_size(&config);
        assert_eq!(w, 100.0); // 1000/10
        assert_eq!(h, 50.0);  // 500/10
    }

    #[test]
    fn test_resolve_page_size_a4_default() {
        let config = printcraft_core::config::PageConfig::default();
        let (w, h) = resolve_page_size(&config);
        assert_eq!(w, 210.0);
        assert_eq!(h, 297.0);
    }
}
