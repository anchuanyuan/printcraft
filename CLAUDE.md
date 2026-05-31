# PrintCraft - 开源 Web 打印服务

## 项目概述

用 Rust 构建的无水印 Web 打印服务，替代 Lodop/C-Lodop。
目标平台: Windows x64 + macOS ARM64

## 技术栈

- **语言**: Rust (workspace) + TypeScript (SDK)
- **构建**: Cargo workspace, Vite (JS SDK)
- **异步运行时**: tokio
- **Web 框架**: axum (HTTP + WebSocket)
- **PDF 渲染**: printpdf (基础元素) + chromiumoxide (HTML, 可选)
- **平台打印**: macOS CUPS / Windows winspool
- **系统托盘**: tray-icon + muda

## 项目结构

```
printcraft/
├── crates/
│   ├── printcraft-core/       # 核心类型（错误/元素/样式/任务/打印机/队列/单位/配置）
│   ├── printcraft-platform/   # 平台抽象层（PlatformPrinter trait + macOS/Windows 实现）
│   ├── printcraft-render/     # 渲染引擎（PdfRenderer trait + printpdf + 条码）
│   └── printcraft-server/     # HTTP/WS 服务 + 托盘 + 静态文件
├── sdk/                       # 浏览器 JS SDK（Lodop 兼容 API）
├── docs/                      # 文档
└── installers/                # 安装器脚本
```

## 开发命令

```bash
# 编译检查
cargo check --workspace

# 运行测试 (跳过 doctest，thiserror 兼容问题)
cargo test --workspace --lib --tests

# 运行服务
cargo run -p printcraft-server

# SDK 开发
cd sdk && npm run dev
cd sdk && npm run build

# 交叉编译 Windows (需 cargo-xwin)
cargo xwin build --target x86_64-pc-windows-msvc -p printcraft-server
```

## 测试覆盖

| Crate | 测试数 | 覆盖范围 |
|-------|--------|---------|
| printcraft-core | 29 | units, style, config, print_job, printer, queue, elements |
| printcraft-render | 19 | PDF渲染, 条码, HTML模板, 颜色解析, 文字换行 |
| printcraft-platform | 0 | 需要真实硬件 (CUPS/winspool) |
| printcraft-server | 0 | 需要集成测试环境 |
| SDK (vitest) | 23 | Lodop类API, Connection连接管理, PREVIEW |

## 开发阶段

| Phase | 内容 | 状态 |
|-------|------|------|
| 1.1 | 项目初始化 (workspace + crate 骨架) | ✅ 完成 |
| 1.2 | 核心类型 (printcraft-core) | ✅ 完成 (29 tests) |
| 1.3 | 平台层 macOS CUPS FFI | ✅ 完成 |
| 1.4 | 渲染引擎 printpdf + template | ✅ 完成 (19 tests) |
| 2.1 | 服务端 服务编排 + WS 命令处理 | ✅ 完成 |
| 2.2 | JS SDK (Lodop 兼容 API) | ✅ 完成 (22 tests) |
| 3 | HTML 模板 + Chromium CDP 渲染 + 预览窗口 | ✅ 完成 |
| 4 | 系统托盘 + 开机自启 | ✅ 核心完成 |
| 5 | Lodop API 补全 + CLI + 发布准备 | ✅ 完成 |

## 关键设计决策

1. **双引擎渲染**: 纯元素用 printpdf (快)，HTML 用 Chromium CDP (功能全)
2. **Lodop 兼容**: JS API 模拟 Lodop 语法，迁移只改 script src
3. **端口发现**: 自动尝试 18000-18005（类比 C-Lodop 的 8000-8005）
4. **单位系统**: 默认 Lodop 单位 (0.1mm)，兼容现有 Lodop 代码

## 重要文件

- `docs/REQUIREMENTS.md` - 需求文档
- `docs/DESIGN.md` - 设计文档
- `docs/ARCHITECTURE.md` - 技术架构
- `docs/TASKS.md` - 开发任务清单
- `.claude/plans/floofy-spinning-willow.md` - 完整实施计划

## 跨平台开发环境

- 主开发: macOS ARM64
- 交叉编译: `cargo-xwin` (Windows)
- 辅助验证: Windows VM (原生编译 + 测试打印机)
