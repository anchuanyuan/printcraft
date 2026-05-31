//! PrintCraft - 开源 Web 打印服务核心库
//!
//! 提供打印任务、元素、样式、打印机信息等核心类型定义。

pub mod config;
pub mod elements;
pub mod error;
pub mod print_job;
pub mod printer;
pub mod queue;
pub mod style;
pub mod units;

pub use config::{AppConfig, PageConfig, PageOrientation};
pub use elements::{BarcodeType, ElementPosition, PrintElement};
pub use error::PrintCraftError;
pub use print_job::PrintJob;
pub use printer::PrinterInfo;
pub use queue::PrintQueue;
pub use style::{Alignment, PrintStyle};
pub use units::Unit;
