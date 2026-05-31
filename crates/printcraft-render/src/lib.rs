//! PrintCraft 渲染引擎
//!
//! 提供 PDF 渲染能力。
//! Phase 1: 基础 printpdf 引擎（文字/形状/图片）
//! Phase 3: Chromium CDP 引擎（HTML/表格/URL）

pub mod pdf_engine;
pub mod simple;
pub mod template;

#[cfg(feature = "chromium")]
pub mod html;

pub mod barcode;

pub use pdf_engine::PdfRenderer;
