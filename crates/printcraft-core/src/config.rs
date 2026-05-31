//! 页面配置和应用配置

use serde::{Deserialize, Serialize};

/// 页面方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageOrientation {
    /// 纵向 (默认)
    Portrait,
    /// 横向
    Landscape,
}

impl Default for PageOrientation {
    fn default() -> Self {
        PageOrientation::Portrait
    }
}

/// 页面配置
///
/// 对应 Lodop 的 SET_PRINT_PAGESIZE。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageConfig {
    /// 方向
    pub orientation: PageOrientation,
    /// 页面宽度 (Lodop 单位)，0 表示使用纸张默认
    pub width: f64,
    /// 页面高度，0 表示使用纸张默认
    pub height: f64,
    /// 纸张名称 (如 "A4", "Letter")
    pub page_name: String,
    /// 上边距
    pub margin_top: f64,
    /// 下边距
    pub margin_bottom: f64,
    /// 左边距
    pub margin_left: f64,
    /// 右边距
    pub margin_right: f64,
}

impl Default for PageConfig {
    fn default() -> Self {
        Self {
            orientation: PageOrientation::Portrait,
            width: 0.0,
            height: 0.0,
            page_name: "A4".to_string(),
            margin_top: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            margin_right: 0.0,
        }
    }
}

/// 常用纸张尺寸 (宽x高, mm)
pub fn paper_size(name: &str) -> Option<(f64, f64)> {
    match name.to_uppercase().as_str() {
        "A4" => Some((210.0, 297.0)),
        "A5" => Some((148.0, 210.0)),
        "A3" => Some((297.0, 420.0)),
        "B5" => Some((176.0, 250.0)),
        "LETTER" => Some((215.9, 279.4)),
        "LEGAL" => Some((215.9, 355.6)),
        _ => None,
    }
}

/// 应用全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 服务监听端口 (默认 18000)
    pub port: u16,
    /// 默认打印机名称 (空 = 系统默认)
    pub default_printer: String,
    /// 是否开机自启
    pub autostart: bool,
    /// 日志级别
    pub log_level: String,
    /// 是否启用 Chromium 渲染
    pub enable_chromium: bool,
    /// Chromium 可执行路径 (空 = 自动检测)
    pub chromium_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 18000,
            default_printer: String::new(),
            autostart: false,
            log_level: "info".to_string(),
            enable_chromium: false,
            chromium_path: String::new(),
        }
    }
}

impl AppConfig {
    /// 从配置文件加载，不存在则用默认值
    pub fn load_or_default() -> Self {
        // TODO: 读取 ~/.printcraft/config.json
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_page_config() {
        let config = PageConfig::default();
        assert_eq!(config.orientation, PageOrientation::Portrait);
        assert_eq!(config.page_name, "A4");
        assert_eq!(config.width, 0.0);
        assert_eq!(config.height, 0.0);
        assert_eq!(config.margin_top, 0.0);
        assert_eq!(config.margin_bottom, 0.0);
        assert_eq!(config.margin_left, 0.0);
        assert_eq!(config.margin_right, 0.0);
    }

    #[test]
    fn test_paper_size_known() {
        assert_eq!(paper_size("A4"), Some((210.0, 297.0)));
        assert_eq!(paper_size("A5"), Some((148.0, 210.0)));
        assert_eq!(paper_size("A3"), Some((297.0, 420.0)));
        assert_eq!(paper_size("B5"), Some((176.0, 250.0)));
        assert_eq!(paper_size("Letter"), Some((215.9, 279.4)));
        assert_eq!(paper_size("Legal"), Some((215.9, 355.6)));
    }

    #[test]
    fn test_paper_size_unknown() {
        assert_eq!(paper_size("A6"), None);
        assert_eq!(paper_size("CustomSize"), None);
        assert_eq!(paper_size(""), None);
    }

    #[test]
    fn test_paper_size_case_insensitive() {
        assert_eq!(paper_size("a4"), Some((210.0, 297.0)));
        assert_eq!(paper_size("a4"), paper_size("A4"));
        assert_eq!(paper_size("LETTER"), paper_size("Letter"));
        assert_eq!(paper_size("letter"), paper_size("Letter"));
    }

    #[test]
    fn test_default_app_config() {
        let config = AppConfig::default();
        assert_eq!(config.port, 18000);
        assert_eq!(config.default_printer, "");
        assert!(!config.autostart);
        assert_eq!(config.log_level, "info");
        assert!(!config.enable_chromium);
        assert_eq!(config.chromium_path, "");
    }
}
