//! 打印任务模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::PageConfig;
use crate::elements::PrintElement;
use crate::style::PrintStyle;

/// 打印任务
///
/// 一个 PrintJob 代表一次完整的打印操作，包含：
/// - 全局打印设置（打印机、份数、页面配置）
/// - 待打印元素列表
/// - 全局默认样式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJob {
    /// 唯一 ID
    pub id: String,
    /// 任务名称（对应 Lodop PRINT_INIT 的 name 参数）
    pub name: String,
    /// 目标打印机（空 = 使用默认打印机）
    pub printer: String,
    /// 打印份数
    pub copies: u32,
    /// 页面配置
    pub page_config: PageConfig,
    /// 全局默认样式
    pub default_style: PrintStyle,
    /// 元素列表
    pub elements: Vec<PrintElement>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl PrintJob {
    /// 创建新的打印任务
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            printer: String::new(),
            copies: 1,
            page_config: PageConfig::default(),
            default_style: PrintStyle::default(),
            elements: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// 添加打印元素
    pub fn add_element(&mut self, element: PrintElement) {
        let mut element = element;
        // 自动分配序号
        element.index = self.elements.len() as u32 + 1;
        self.elements.push(element);
    }

    /// 设置目标打印机
    pub fn set_printer(&mut self, printer: impl Into<String>) {
        self.printer = printer.into();
    }

    /// 设置打印份数
    pub fn set_copies(&mut self, copies: u32) {
        self.copies = copies.max(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{ElementPosition, PrintElementKind};

    fn make_text_element() -> PrintElement {
        PrintElement {
            index: 0,
            position: ElementPosition {
                top: 0.0,
                left: 0.0,
                width: 100.0,
                height: 20.0,
            },
            style: None,
            kind: PrintElementKind::Text {
                content: "test".to_string(),
            },
        }
    }

    #[test]
    fn test_new_job() {
        let job = PrintJob::new("test_job");

        assert!(!job.id.is_empty());
        assert_eq!(job.name, "test_job");
        assert_eq!(job.printer, "");
        assert_eq!(job.copies, 1);
        assert!(job.elements.is_empty());
    }

    #[test]
    fn test_add_element_auto_index() {
        let mut job = PrintJob::new("test_job");
        job.add_element(make_text_element());
        job.add_element(make_text_element());
        job.add_element(make_text_element());

        assert_eq!(job.elements[0].index, 1);
        assert_eq!(job.elements[1].index, 2);
        assert_eq!(job.elements[2].index, 3);
    }

    #[test]
    fn test_set_printer() {
        let mut job = PrintJob::new("test_job");
        job.set_printer("HP LaserJet");
        assert_eq!(job.printer, "HP LaserJet");
    }

    #[test]
    fn test_set_copies_min_1() {
        let mut job = PrintJob::new("test_job");

        job.set_copies(0);
        assert_eq!(job.copies, 1);

        job.set_copies(5);
        assert_eq!(job.copies, 5);
    }

    #[test]
    fn test_add_multiple_elements() {
        let mut job = PrintJob::new("test_job");

        for _ in 0..10 {
            job.add_element(make_text_element());
        }

        assert_eq!(job.elements.len(), 10);
        for (i, elem) in job.elements.iter().enumerate() {
            assert_eq!(elem.index, (i + 1) as u32);
        }
    }
}
