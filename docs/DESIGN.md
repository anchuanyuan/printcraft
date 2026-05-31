# PrintCraft 设计文档

## 设计原则

1. **Lodop 兼容优先** — 现有 Lodop 代码零修改迁移
2. **渐进增强** — 基础功能无外部依赖，HTML 渲染可选
3. **单二进制分发** — Rust 静态编译，无需运行时
4. **安全默认** — 仅监听 localhost，无远程暴露风险

## 模块设计

### printcraft-core
纯数据类型，无平台依赖。所有其他 crate 依赖此模块。

关键类型：
- `PrintJob` — 一次完整的打印任务
- `PrintElement` — 单个打印元素（文字/图片/条码等）
- `PrintStyle` — 样式（可合并语义）
- `PrinterInfo` — 打印机信息

### printcraft-platform
平台抽象层。通过 `PlatformPrinter` trait 隔离 OS 差异。

macOS: CUPS FFI
Windows: winspool API (windows-rs)

### printcraft-render
渲染引擎。将 PrintJob 转为 PDF。

`PdfRenderer` trait 实现：
- `SimplePdfRenderer` — printpdf，处理基础元素
- `ChromiumRenderer` — CDP，处理 HTML 元素（可选）

### printcraft-server
入口服务。axum HTTP + WebSocket，编排渲染和打印。

### sdk/
浏览器 JS SDK。TypeScript 编写，Vite 构建，输出 UMD 单文件。

## 关键决策

| 决策 | 选择 | 理由 |
|------|------|------|
| 语言 | Rust | FFI 能力强，无 GC，单二进制 |
| Web 框架 | axum | tokio 生态，WebSocket 支持好 |
| PDF 生成 | printpdf | 纯 Rust，无 C 依赖 |
| HTML 渲染 | chromiumoxide (可选) | 功能最全，按需启用 |
| 端口范围 | 18000-18005 | 类比 C-Lodop 8000-8005 |
