//! 单位转换系统
//!
//! Lodop 默认使用 0.1mm 为单位（称为 Lodop 单位）。
//! 本模块提供各单位间的转换。
//!
//! 换算关系:
//! - 1 Lodop 单位 = 0.1mm
//! - 1mm = 10 Lodop 单位
//! - 1pt (点) = 0.3528mm = 3.528 Lodop 单位
//! - 1px (96dpi) = 0.2646mm = 2.646 Lodop 单位

use serde::{Deserialize, Serialize};

/// 度量单位枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unit {
    /// Lodop 单位 (0.1mm)，Lodop API 默认单位
    Lodop,
    /// 毫米
    Mm,
    /// 像素 (基于 96dpi)
    Px,
    /// 点 (PDF 标准单位，1pt = 1/72 inch)
    Pt,
}

impl Unit {
    /// 将该单位下的 1 个单位转换为毫米
    pub fn to_mm_factor(&self) -> f64 {
        match self {
            Unit::Lodop => 0.1,
            Unit::Mm => 1.0,
            Unit::Px => 25.4 / 96.0, // 1px = 25.4/96 mm at 96dpi
            Unit::Pt => 0.3528,       // 1pt = 0.3528mm
        }
    }
}

/// 单位转换函数
///
/// # Arguments
/// * `value` - 待转换的数值
/// * `from` - 源单位
/// * `to` - 目标单位
///
/// # Returns
/// 转换后的数值
///
/// # Example
/// ```
/// use printcraft_core::units::{convert, Unit};
///
/// // 1000 Lodop 单位 = 100mm
/// let mm = convert(1000.0, Unit::Lodop, Unit::Mm);
/// assert!((mm - 100.0).abs() < 0.001);
/// ```
pub fn convert(value: f64, from: Unit, to: Unit) -> f64 {
    if from == to {
        return value;
    }
    // 先转为 mm，再转为目标单位
    let mm = value * from.to_mm_factor();
    mm / to.to_mm_factor()
}

/// 将 Lodop 单位转为 PDF 点（1/72 inch）
///
/// PDF 内部使用点作为坐标单位，此函数用于渲染阶段。
pub fn lodop_to_pt(value: f64) -> f64 {
    convert(value, Unit::Lodop, Unit::Pt)
}

/// 将毫米转为 PDF 点
pub fn mm_to_pt(value: f64) -> f64 {
    convert(value, Unit::Mm, Unit::Pt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lodop_to_mm() {
        let result = convert(1000.0, Unit::Lodop, Unit::Mm);
        assert!((result - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_mm_to_lodop() {
        let result = convert(100.0, Unit::Mm, Unit::Lodop);
        assert!((result - 1000.0).abs() < 0.001);
    }

    #[test]
    fn test_px_to_mm() {
        // 96px ≈ 25.4mm (1 inch)
        let result = convert(96.0, Unit::Px, Unit::Mm);
        assert!((result - 25.4).abs() < 0.01);
    }

    #[test]
    fn test_pt_to_mm() {
        // 72pt = 1 inch = 25.4mm
        let result = convert(72.0, Unit::Pt, Unit::Mm);
        assert!((result - 25.4016).abs() < 0.01);
    }

    #[test]
    fn test_same_unit() {
        let result = convert(42.0, Unit::Mm, Unit::Mm);
        assert!((result - 42.0).abs() < 0.001);
    }

    #[test]
    fn test_lodop_to_pt() {
        // 1000 Lodop = 100mm ≈ 283.46pt
        let result = lodop_to_pt(1000.0);
        assert!((result - 283.46).abs() < 0.5);
    }
}
