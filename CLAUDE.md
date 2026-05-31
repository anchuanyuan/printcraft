# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

PrintCraft 是开源无水印 Web 打印服务，Lodop/C-Lodop 替代品。Rust 构建服务端 + TypeScript 浏览器 SDK。
目标平台: Windows x64 + macOS ARM64

## 开发命令

```bash
# 编译检查
cargo check --workspace

# 运行测试 (跳过 doctest，thiserror 兼容问题)
cargo test --workspace --lib --tests

# 运行单个测试
cargo test -p printcraft-core test_name -- --nocapture

# 运行服务
cargo run -p printcraft-server

# SDK 开发
cd sdk && npm run dev       # Vite 开发服务器
cd sdk && npm run build     # 构建 SDK
cd sdk && npm test           # Vitest 测试

# 交叉编译 Windows (需 cargo-xwin)
cargo xwin build --target x86_64-pc-windows-msvc -p printcraft-server -p printcraft-cli

# Release 构建
cargo build --release -p printcraft-server -p printcraft-cli
```

## 架构

### 请求流程

```
浏览器 SDK ──WebSocket──> printcraft-server ──> service.rs ──> PdfRenderer ──> PlatformPrinter ──> 打印机
                        localhost:18000
```

### Workspace Crate 说明

- **printcraft-core**: 核心类型 — `PrintJob`, `PrintElement`/`PrintElementKind`, `PrintStyle`, `PageConfig`, `PrintQueue`, `PrinterInfo`。坐标单位为 Lodop 单位 (0.1mm；A4 = 2100×2970)。
- **printcraft-platform**: 平台抽象层，`PlatformPrinter` trait。Windows 使用 winspool FFI (`EnumPrintersW`, `GetDefaultPrinterW`) + SumatraPDF CLI 或 ShellExecuteW "printto" 进行 PDF 打印。
- **printcraft-render**: `PdfRenderer` trait。`SimplePdfRenderer` 用 printpdf 处理纯元素 (text/rect/line/image/barcode/shape)。HTML 元素需要 Chromium CDP (feature-gated)。`template.rs` 为 Chromium/preview 生成 HTML。
- **printcraft-server**: Axum HTTP+WS 服务。`PrintService` (service.rs) 是核心编排器，分发 Lodop 兼容命令。路由: `/ws` (WebSocket), `/api/*` (REST), `/preview/:id` (预览页面), `/sdk/printcraft.js` (SDK 静态文件)。
- **printcraft-cli**: CLI 工具，支持 status/printer/config 管理。

### SDK (TypeScript)

`sdk/src/lodop.ts` 实现 Lodop 兼容 API。`Lodop` 类在客户端累积元素，`PRINT()`/`PREVIEW()` 时通过 WebSocket 发送完整任务。`Connection` 类处理自动端口发现 (18000-18005)、断线重连、心跳保活。

### 关键设计决策

1. **双引擎渲染**: 纯元素 → printpdf (快)，HTML → Chromium CDP (功能全，feature-gated)
2. **Lodop 兼容**: JS SDK 模拟 Lodop API 名称/参数，迁移只改 script src
3. **端口发现**: 自动尝试 18000-18005 (类似 C-Lodop 的 8000-8005)
4. **单位系统**: 默认 Lodop 单位 (0.1mm)，1000 = 100mm = 10cm
5. **服务端状态**: `PRINT_INIT` 在服务端创建 `current_job`，后续 `ADD_*`/`SET_*` 修改它，`PRINT`/`PREVIEW` 消耗或读取它

### 平台打印 (Windows)

`printcraft-platform/src/windows/pdf_print.rs` 的 Windows PDF 打印流程:
1. 写入临时 PDF 文件
2. 优先用 SumatraPDF CLI (`-print-to`, `-silent`) — 推荐，打包到安装器
3. 回退用 ShellExecuteW "printto" verb — 首次可能弹出对话框

## 测试覆盖

49 个 Rust 测试 (core: 29, render: 19, platform: 0 [需硬件], server: 0 [需集成环境]) + 23 个 SDK Vitest 测试。

## 重要文件

- `docs/REQUIREMENTS.md` — 需求文档
- `docs/DESIGN.md` — 设计文档
- `docs/ARCHITECTURE.md` — 技术架构
- `docs/TASKS.md` — 开发任务清单
- `sdk/preview/index.html` — 预览 UI 页面 (嵌入服务端二进制)
- `.claude/plans/floofy-spinning-willow.md` — 完整实施计划

## 跨平台开发环境

- 主开发: macOS ARM64
- 交叉编译: `cargo-xwin` (Windows)
- 辅助验证: Windows VM (原生编译 + 测试打印机)
