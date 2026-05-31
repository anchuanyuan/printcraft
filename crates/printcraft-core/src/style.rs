//! 打印样式系统
//!
//! 对应 Lodop 的 SET_PRINT_STYLE / SET_PRINT_STYLEA。
//! 样式是可合并的（设置某属性不覆盖其他已设属性）。

use serde::{Deserialize, Serialize};

/// 文本对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

impl Default for Alignment {
    fn default() -> Self {
        Alignment::Left
    }
}

/// 打印样式
///
/// 所有字段均为 Option，支持部分设置（合并语义）。
/// `merge()` 方法将非 None 的值覆盖到目标样式上。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintStyle {
    pub font_name: Option<String>,
    pub font_size: Option<f64>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub alignment: Option<Alignment>,
    /// 文字颜色，格式 "#RRGGBB"
    pub color: Option<String>,
    /// 行间距倍数
    pub line_spacing: Option<f64>,
    /// 是否自动换行
    pub word_wrap: Option<bool>,
    /// 旋转角度 (0-360)
    pub rotation: Option<f64>,
    /// 透明度 (0.0-1.0)
    pub opacity: Option<f64>,
}

impl PrintStyle {
    /// 创建空样式（所有字段为 None）
    pub fn new() -> Self {
        Self {
            font_name: None,
            font_size: None,
            bold: None,
            italic: None,
            underline: None,
            alignment: None,
            color: None,
            line_spacing: None,
            word_wrap: None,
            rotation: None,
            opacity: None,
        }
    }

    /// 将 `other` 中非 None 的字段合并到 self
    ///
    /// 对应 Lodop 的行为：SET_PRINT_STYLE 设置的属性影响后续添加的元素，
    /// 且只覆盖被设置的属性，不影响其他属性。
    pub fn merge(&mut self, other: &PrintStyle) {
        if let Some(v) = &other.font_name {
            self.font_name = Some(v.clone());
        }
        if let Some(v) = other.font_size {
            self.font_size = Some(v);
        }
        if let Some(v) = other.bold {
            self.bold = Some(v);
        }
        if let Some(v) = other.italic {
            self.italic = Some(v);
        }
        if let Some(v) = other.underline {
            self.underline = Some(v);
        }
        if let Some(v) = other.alignment {
            self.alignment = Some(v);
        }
        if let Some(v) = &other.color {
            self.color = Some(v.clone());
        }
        if let Some(v) = other.line_spacing {
            self.line_spacing = Some(v);
        }
        if let Some(v) = other.word_wrap {
            self.word_wrap = Some(v);
        }
        if let Some(v) = other.rotation {
            self.rotation = Some(v);
        }
        if let Some(v) = other.opacity {
            self.opacity = Some(v);
        }
    }

    /// 解析 Lodop 样式属性名到 PrintStyle 字段
    ///
    /// Lodop 使用字符串属性名如 "FontSize", "FontName", "Bold" 等。
    pub fn set_by_name(&mut self, name: &str, value: &str) {
        match name {
            "FontName" => self.font_name = Some(value.to_string()),
            "FontSize" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.font_size = Some(v);
                }
            }
            "Bold" => self.bold = Some(value == "1" || value.to_lowercase() == "true"),
            "Italic" => self.italic = Some(value == "1" || value.to_lowercase() == "true"),
            "UnderLine" => self.underline = Some(value == "1" || value.to_lowercase() == "true"),
            "Alignment" => {
                self.alignment = Some(match value {
                    "1" => Alignment::Center,
                    "2" => Alignment::Right,
                    _ => Alignment::Left,
                });
            }
            "FontColor" => self.color = Some(value.to_string()),
            "LineSpacing" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.line_spacing = Some(v);
                }
            }
            "Angle" => {
                if let Ok(v) = value.parse::<f64>() {
                    self.rotation = Some(v);
                }
            }
            _ => tracing::warn!("未知样式属性: {} = {}", name, value),
        }
    }
}

impl Default for PrintStyle {
    fn default() -> Self {
        Self {
            font_name: Some("SimSun".to_string()),
            font_size: Some(12.0),
            bold: Some(false),
            italic: Some(false),
            underline: Some(false),
            alignment: Some(Alignment::Left),
            color: Some("#000000".to_string()),
            line_spacing: Some(1.0),
            word_wrap: Some(true),
            rotation: Some(0.0),
            opacity: Some(1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge() {
        let mut base = PrintStyle::default();
        let mut override_style = PrintStyle::new();
        override_style.font_size = Some(16.0);
        override_style.bold = Some(true);

        base.merge(&override_style);

        assert_eq!(base.font_size, Some(16.0));
        assert_eq!(base.bold, Some(true));
        // 未设置的保持原值
        assert_eq!(base.font_name, Some("SimSun".to_string()));
    }

    #[test]
    fn test_set_by_name() {
        let mut style = PrintStyle::new();
        style.set_by_name("FontSize", "20");
        style.set_by_name("Bold", "1");
        style.set_by_name("Alignment", "1");

        assert_eq!(style.font_size, Some(20.0));
        assert_eq!(style.bold, Some(true));
        assert_eq!(style.alignment, Some(Alignment::Center));
    }

    #[test]
    fn test_merge_doesnt_override_none() {
        let mut base = PrintStyle::default();
        let empty_override = PrintStyle::new();

        let original = base.clone();
        base.merge(&empty_override);

        assert_eq!(base.font_name, original.font_name);
        assert_eq!(base.font_size, original.font_size);
        assert_eq!(base.bold, original.bold);
        assert_eq!(base.italic, original.italic);
        assert_eq!(base.underline, original.underline);
        assert_eq!(base.alignment, original.alignment);
        assert_eq!(base.color, original.color);
        assert_eq!(base.line_spacing, original.line_spacing);
        assert_eq!(base.word_wrap, original.word_wrap);
        assert_eq!(base.rotation, original.rotation);
        assert_eq!(base.opacity, original.opacity);
    }

    #[test]
    fn test_set_by_name_all_attributes() {
        let mut style = PrintStyle::new();

        style.set_by_name("FontName", "Arial");
        assert_eq!(style.font_name, Some("Arial".to_string()));

        style.set_by_name("FontSize", "14.5");
        assert_eq!(style.font_size, Some(14.5));

        style.set_by_name("Bold", "1");
        assert_eq!(style.bold, Some(true));

        style.set_by_name("Italic", "true");
        assert_eq!(style.italic, Some(true));

        style.set_by_name("UnderLine", "1");
        assert_eq!(style.underline, Some(true));

        style.set_by_name("Alignment", "2");
        assert_eq!(style.alignment, Some(Alignment::Right));

        style.set_by_name("FontColor", "#FF0000");
        assert_eq!(style.color, Some("#FF0000".to_string()));

        style.set_by_name("LineSpacing", "1.5");
        assert_eq!(style.line_spacing, Some(1.5));

        style.set_by_name("Angle", "90");
        assert_eq!(style.rotation, Some(90.0));
    }

    #[test]
    fn test_default_style() {
        let style = PrintStyle::default();

        assert_eq!(style.font_name, Some("SimSun".to_string()));
        assert_eq!(style.font_size, Some(12.0));
        assert_eq!(style.bold, Some(false));
        assert_eq!(style.italic, Some(false));
        assert_eq!(style.underline, Some(false));
        assert_eq!(style.alignment, Some(Alignment::Left));
        assert_eq!(style.color, Some("#000000".to_string()));
        assert_eq!(style.line_spacing, Some(1.0));
        assert_eq!(style.word_wrap, Some(true));
        assert_eq!(style.rotation, Some(0.0));
        assert_eq!(style.opacity, Some(1.0));
    }

    #[test]
    fn test_new_style_all_none() {
        let style = PrintStyle::new();

        assert!(style.font_name.is_none());
        assert!(style.font_size.is_none());
        assert!(style.bold.is_none());
        assert!(style.italic.is_none());
        assert!(style.underline.is_none());
        assert!(style.alignment.is_none());
        assert!(style.color.is_none());
        assert!(style.line_spacing.is_none());
        assert!(style.word_wrap.is_none());
        assert!(style.rotation.is_none());
        assert!(style.opacity.is_none());
    }
}
