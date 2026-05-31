# PrintCraft 需求文档

## 1. 项目背景

Lodop/C-Lodop 是国内广泛使用的 Web 打印解决方案，但免费版带有水印。
PrintCraft 旨在构建一个无水印、开源、跨平台的替代品。

## 2. 目标平台

- Windows x64 (Windows 10+)
- macOS ARM64 (macOS 12+)

## 3. 核心需求

### 3.1 浏览器打印服务
- 通过 localhost WebSocket/HTTP 提供打印能力
- 浏览器 JS SDK 调用打印 API
- 无需安装浏览器插件（不依赖 ActiveX/NPAPI）

### 3.2 Lodop API 兼容
- 提供与 Lodop 一致的 JS API
- 现有 Lodop 代码只改 script src 即可迁移
- 暴露 window.LODOP / window.CLODOP / getLodop()

### 3.3 打印功能
- 纯文本打印（精确坐标定位）
- 图片打印（base64/URL）
- 矩形/直线/椭圆等形状
- 条码/二维码（Code128/EAN13/QR 等）
- HTML 超文本打印
- 打印预览

### 3.4 打印机管理
- 枚举系统打印机
- 获取默认打印机
- 查询打印机能力和纸张尺寸
- 指定目标打印机
- 设置打印份数

### 3.5 系统集成
- 系统托盘图标（显示运行状态）
- 开机自启（可选）
- 端口自动发现（18000-18005）

## 4. 非功能需求

- 无水印输出
- 开源 MIT 协议
- 安装包 < 10MB（不含 Chromium）
- 打印延迟 < 2 秒（本地简单任务）
- 内存占用 < 50MB（空闲时）

## 5. 不包含（v0.1）

- 移动端支持
- 网络打印（跨机器）
- 多语言 UI（仅中英文）
- 自动更新
