# PrintCraft 开发任务清单

## Phase 1: 核心基础

### 1.1 项目初始化 ✅
- [x] 创建 Cargo workspace
- [x] 创建 4 个 crate 骨架 (core/platform/render/server)
- [x] 配置 workspace 依赖
- [x] 创建 SDK 项目 (Vite + TypeScript)
- [x] 创建文档骨架
- [x] 配置 CI
- [x] .gitignore + LICENSE
- [x] CLAUDE.md

### 1.2 核心类型 (printcraft-core) ✅
- [x] `error.rs` — 错误类型
- [x] `units.rs` — 单位转换系统
- [x] `style.rs` — 样式系统
- [x] `elements.rs` — 打印元素定义
- [x] `config.rs` — 页面/应用配置
- [x] `print_job.rs` — 打印任务模型
- [x] `printer.rs` — 打印机信息
- [x] `queue.rs` — 任务队列
- [x] 单元测试 (29 tests)

### 1.3 平台层 (printcraft-platform) ✅
- [x] `trait_def.rs` — PlatformPrinter trait
- [x] macOS 模块骨架 (cups.rs + pdf_print.rs)
- [x] Windows 模块骨架 (winspool.rs + pdf_print.rs)
- [x] macOS CUPS FFI 实现 (cupsGetDests/cupsGetDefault/cupsPrintFile)
- [x] Windows winspool API 实现 (EnumPrintersW/GetDefaultPrinterW/DeviceCapabilitiesW)
- [x] Windows PDF 打印实现 (SumatraPDF CLI + ShellExecuteW fallback)
- [ ] 集成测试 (需要实机验证)

### 1.4 渲染引擎 (printcraft-render) ✅
- [x] `pdf_engine.rs` — PdfRenderer trait
- [x] `simple.rs` — 基础渲染器完整实现 (Text/Rect/Line/Ellipse/Image/Barcode/Shape)
- [x] `barcode.rs` — QR码生成 + 条码占位
- [x] printpdf 文字渲染实现 (换行/对齐/字号)
- [x] printpdf 形状渲染实现 (矩形/直线/椭圆)
- [x] printpdf 图片渲染实现 (base64 data-URL)
- [x] 条码生成实现 (QR + 占位)
- [x] 单元测试 (13 tests)

## Phase 2: 服务 + SDK

### 2.1 服务端 (printcraft-server) ✅
- [x] `main.rs` — 入口
- [x] `api/ws.rs` — WebSocket 处理
- [x] `api/protocol.rs` — 协议定义
- [x] `api/rest.rs` — REST 端点
- [x] `api/static_files.rs` — 静态文件
- [x] `discovery.rs` — 端口发现
- [x] `service.rs` — 服务编排完整实现 (PRINT_INIT/ADD_PRINT_*/SET_PRINT_*/PRINT/PREVIEW/GET_PRINTER_*)
- [ ] rust-embed 静态文件嵌入
- [ ] 集成测试

### 2.2 JS SDK ✅
- [x] `connection.ts` — 连接管理 (端口发现/重连/心跳)
- [x] `lodop.ts` — Lodop 类核心 API (14 个方法)
- [x] `style.ts` — 样式类型
- [x] `elements.ts` — 元素类型
- [x] `index.ts` — 全局暴露 (LODOP/CLODOP/getLodop)
- [x] 单元测试 (22 tests)
- [x] SDK 构建验证 (ES + UMD, 8KB gzip 2.5KB)

## Phase 3: 丰富内容 + 预览 ✅
- [x] HTML 模板组装 (template.rs) — 全元素 HTML 绝对定位渲染
- [x] Chromium CDP 渲染 (html.rs) — chromiumoxide Page.printToPDF
- [x] ADD_PRINT_HTM/TABLE/URL 实现 (service.rs 已分发)
- [x] 打印预览窗口 — /preview/{id} 页面 + API + SDK PREVIEW()
- [x] 预览 UI 页面 — HTML/PDF 双视图 + 缩放 + 打印

## Phase 4: 托盘 + 安装器
- [x] 系统托盘图标 (tray.rs) — tray-icon + muda, 右键菜单
- [x] 开机自启 (autostart.rs) — auto-launch crate, macOS/Windows 兼容
- [ ] NSIS Windows 安装器
- [ ] macOS DMG 打包
- [ ] 代码签名

## Phase 5: 完整兼容 + 发布 ✅
- [x] Lodop API 补全 (ADD_PRINT_TABLE/URL/ELLIPSE/SHAPE, GET_PRINT_IN_VALUE)
- [x] CLI 工具 (printcraft-cli: status/printers/start/config)
- [x] 迁移文档 (MIGRATION_FROM_LODOP.md)
- [x] 发布 v0.1.0 预备 (release.yml + NSIS + Info.plist)
