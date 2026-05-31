//! 打印元素定义
//!
//! 对应 Lodop 的 ADD_PRINT_* 系列方法。
//! 每个元素有位置信息和元素特有数据。

use serde::{Deserialize, Serialize};

use crate::style::PrintStyle;

/// 元素位置和尺寸
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementPosition {
    /// 距页面顶部 (Lodop 单位: 0.1mm)
    pub top: f64,
    /// 距页面左侧
    pub left: f64,
    /// 宽度
    pub width: f64,
    /// 高度
    pub height: f64,
}

/// 条码类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarcodeType {
    /// Code 128 自动
    Code128,
    /// Code 128A
    Code128A,
    /// Code 128B
    Code128B,
    /// Code 128C
    Code128C,
    /// Code 39
    Code39,
    /// EAN-13
    EAN13,
    /// EAN-8
    EAN8,
    /// UPC-A
    UPCA,
    /// QR 二维码
    QRCode,
    /// PDF417
    PDF417,
}

/// 打印元素
///
/// 对应 Lodop 的 ADD_PRINT_TEXT / ADD_PRINT_IMAGE 等方法创建的内容。
/// 每个元素携带位置、关联样式和元素类型特有数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintElement {
    /// 元素序号（从 1 开始，Lodop 兼容）
    pub index: u32,
    /// 位置和尺寸
    pub position: ElementPosition,
    /// 元素专属样式（可选，覆盖全局样式）
    pub style: Option<PrintStyle>,
    /// 元素类型
    pub kind: PrintElementKind,
}

/// 元素类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrintElementKind {
    /// 纯文本: ADD_PRINT_TEXT
    Text {
        content: String,
    },
    /// 图片: ADD_PRINT_IMAGE
    Image {
        /// 图片数据 (base64 或 URL)
        src: String,
    },
    /// 矩形: ADD_PRINT_RECT
    Rect {
        /// 边框宽度 (0 = 实心填充)
        border_width: f64,
        /// 边框样式: 0=实线, 1=虚线
        border_style: u8,
    },
    /// 直线: ADD_PRINT_LINE
    Line {
        /// 终点位置
        end_top: f64,
        end_left: f64,
        line_style: u8,
        line_width: f64,
    },
    /// 椭圆: ADD_PRINT_ELLIPSE
    Ellipse {
        border_width: f64,
    },
    /// 条码: ADD_PRINT_BARCODE
    Barcode {
        code: String,
        barcode_type: BarcodeType,
    },
    /// 超文本: ADD_PRINT_HTM
    Html {
        html_content: String,
    },
    /// 表格: ADD_PRINT_TABLE
    Table {
        html_content: String,
    },
    /// URL 网页: ADD_PRINT_URL
    Url {
        url: String,
    },
    /// 形状: ADD_PRINT_SHAPE
    Shape {
        shape_type: u8,
        border_width: f64,
        border_style: u8,
        color: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_text_element() -> PrintElement {
        PrintElement {
            index: 1,
            position: ElementPosition {
                top: 10.0,
                left: 20.0,
                width: 200.0,
                height: 50.0,
            },
            style: None,
            kind: PrintElementKind::Text {
                content: "Hello, PrintCraft!".to_string(),
            },
        }
    }

    fn make_barcode_element() -> PrintElement {
        PrintElement {
            index: 2,
            position: ElementPosition {
                top: 100.0,
                left: 50.0,
                width: 150.0,
                height: 80.0,
            },
            style: None,
            kind: PrintElementKind::Barcode {
                code: "123456789".to_string(),
                barcode_type: BarcodeType::Code128,
            },
        }
    }

    #[test]
    fn test_text_element_serialize_roundtrip() {
        let elem = make_text_element();

        let json = serde_json::to_string(&elem).expect("serialize");
        let deserialized: PrintElement = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.index, 1);
        assert_eq!(deserialized.position.top, 10.0);
        assert_eq!(deserialized.position.left, 20.0);
        assert_eq!(deserialized.position.width, 200.0);
        assert_eq!(deserialized.position.height, 50.0);
        assert!(deserialized.style.is_none());

        if let PrintElementKind::Text { content } = &deserialized.kind {
            assert_eq!(content, "Hello, PrintCraft!");
        } else {
            panic!("expected Text variant");
        }
    }

    #[test]
    fn test_barcode_element_serialize_roundtrip() {
        let elem = make_barcode_element();

        let json = serde_json::to_string(&elem).expect("serialize");
        let deserialized: PrintElement = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.index, 2);
        assert_eq!(deserialized.position.top, 100.0);

        if let PrintElementKind::Barcode { code, barcode_type } = &deserialized.kind {
            assert_eq!(code, "123456789");
            assert_eq!(*barcode_type, BarcodeType::Code128);
        } else {
            panic!("expected Barcode variant");
        }
    }

    #[test]
    fn test_element_position_construction() {
        let pos = ElementPosition {
            top: 50.0,
            left: 100.0,
            width: 300.0,
            height: 200.0,
        };

        assert_eq!(pos.top, 50.0);
        assert_eq!(pos.left, 100.0);
        assert_eq!(pos.width, 300.0);
        assert_eq!(pos.height, 200.0);
    }
}
