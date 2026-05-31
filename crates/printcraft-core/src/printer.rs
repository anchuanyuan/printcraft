//! 打印机信息模型

use serde::{Deserialize, Serialize};

/// 纸张尺寸信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperSize {
    /// 纸张名称 (如 "A4", "Letter")
    pub name: String,
    /// 宽度 (mm)
    pub width_mm: f64,
    /// 高度 (mm)
    pub height_mm: f64,
}

/// 打印机状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrinterStatus {
    /// 就绪
    Ready,
    /// 忙碌
    Busy,
    /// 离线
    Offline,
    /// 错误
    Error,
    /// 未知
    Unknown,
}

/// 打印机信息
///
/// 从操作系统获取的打印机详细信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterInfo {
    /// 打印机名称
    pub name: String,
    /// 是否为系统默认打印机
    pub is_default: bool,
    /// 当前状态
    pub status: PrinterStatus,
    /// 支持的纸张尺寸
    pub paper_sizes: Vec<PaperSize>,
    /// 是否支持彩色打印
    pub color_support: bool,
    /// 是否支持双面打印
    pub duplex: bool,
    /// 驱动名称
    pub driver_name: String,
    /// 端口/连接信息
    pub port: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_printer_info() -> PrinterInfo {
        PrinterInfo {
            name: "HP LaserJet Pro".to_string(),
            is_default: true,
            status: PrinterStatus::Ready,
            paper_sizes: vec![
                PaperSize {
                    name: "A4".to_string(),
                    width_mm: 210.0,
                    height_mm: 297.0,
                },
                PaperSize {
                    name: "Letter".to_string(),
                    width_mm: 215.9,
                    height_mm: 279.4,
                },
            ],
            color_support: true,
            duplex: true,
            driver_name: "HP Universal Printing".to_string(),
            port: "USB001".to_string(),
        }
    }

    #[test]
    fn test_printer_info_construction() {
        let info = make_printer_info();

        assert_eq!(info.name, "HP LaserJet Pro");
        assert!(info.is_default);
        assert_eq!(info.status, PrinterStatus::Ready);
        assert_eq!(info.paper_sizes.len(), 2);
        assert_eq!(info.paper_sizes[0].name, "A4");
        assert!(info.color_support);
        assert!(info.duplex);
        assert_eq!(info.driver_name, "HP Universal Printing");
        assert_eq!(info.port, "USB001");
    }

    #[test]
    fn test_printer_info_serialize_roundtrip() {
        let info = make_printer_info();

        let json = serde_json::to_string(&info).expect("serialize");
        let deserialized: PrinterInfo = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.name, info.name);
        assert_eq!(deserialized.is_default, info.is_default);
        assert_eq!(deserialized.status, info.status);
        assert_eq!(deserialized.paper_sizes.len(), info.paper_sizes.len());
        assert_eq!(deserialized.color_support, info.color_support);
        assert_eq!(deserialized.duplex, info.duplex);
        assert_eq!(deserialized.driver_name, info.driver_name);
        assert_eq!(deserialized.port, info.port);
    }
}
