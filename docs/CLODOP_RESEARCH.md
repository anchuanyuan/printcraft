# C-Lodop 功能调研报告

> 调研日期: 2026-05-30

## 1. C-Lodop 概述

C-Lodop 是 LODOP 打印控件的"云打印"版本，解决传统 Lodop 依赖 ActiveX/IE 内核的问题。
它在用户本地运行一个轻量 HTTP+WebSocket 服务，浏览器通过 JS API 调用打印功能。

### 与 Lodop 的关系

| 特性 | Lodop (旧) | C-Lodop (新) |
|------|-----------|-------------|
| 浏览器兼容 | 仅 IE (ActiveX) | 全浏览器 |
| 运行方式 | 浏览器插件 | 本地后台服务 |
| 通信方式 | COM 调用 | HTTP / WebSocket |
| 默认端口 | 8000-8005 | 8000-8005 |
| 安装包 | Lodop.ocx + InstallLodop.exe | CLodop.exe (独立) |
| 远程打印 | 不支持 | 支持 (云打印) |

## 2. 通信协议

### 2.1 服务端口

C-Lodop 在本地启动 HTTP 服务，监听多个端口（自动发现）：
- **HTTP**: `localhost:8000` (首选) / `8001`-`8005` (备选)
- **HTTPS**: `localhost:8443` (可选)
- **WebSocket**: `ws://localhost:8000/c_lodop` (长连接模式)

### 2.2 JS 文件加载方式

```html
<!-- 方式1: 从 C-Lodop 服务加载 -->
<script src="http://localhost:8000/CLodopfuncs.js"></script>

<!-- 方式2: 独立 getCLodop.js (自动探测端口) -->
<script src="getCLodop.js"></script>
```

`CLodopfuncs.js` 是 C-Lodop 服务**动态生成**的 JS 文件，包含完整的 API 实现。
`getCLodop.js` 是独立的客户端 JS，自动探测 C-Lodop 服务端口。

### 2.3 端口探测逻辑

客户端按以下顺序尝试连接：
1. `8000` → `8001` → `8002` → `8003` → `8004` → `8005`
2. 如果全部失败，尝试 `18000`-`18005` (PrintCraft 端口)
3. WebSocket 模式: `ws://localhost:{port}/c_lodop`

### 2.4 通信模式

| 模式 | 说明 | 优点 |
|------|------|------|
| HTTP 短连接 | 每次调用发 HTTP POST | 简单可靠 |
| WebSocket 长连接 | 建立 WS 后复用连接 | 低延迟，支持异步回调 |

```javascript
// 切换为 WebSocket 模式
CLODOP.WebSocket = 1;
```

## 3. API 完整清单

### 3.1 任务管理

| API | 参数 | 返回值 | 说明 |
|-----|------|--------|------|
| `PRINT_INIT(strTaskName)` | 任务名 | `boolean` | 初始化任务，清空元素 |
| `PRINT_INITA(Top, Left, Width, Height, strTaskName)` | 偏移+尺寸+任务名 | `boolean` | 带区域的初始化 |

### 3.2 添加打印元素

| API | 参数 | 说明 |
|-----|------|------|
| `ADD_PRINT_TEXT(Top, Left, Width, Height, strContent)` | 坐标+文本 | 纯文本 |
| `ADD_PRINT_HTM(Top, Left, Width, Height, strHtml)` | 坐标+HTML | 超文本 |
| `ADD_PRINT_TABLE(Top, Left, Width, Height, strHtml)` | 坐标+HTML | 表格（自动分页） |
| `ADD_PRINT_URL(Top, Left, Width, Height, strURL)` | 坐标+URL | 网页内容 |
| `ADD_PRINT_IMAGE(Top, Left, Width, Height, strHtmlContent)` | 坐标+图片 | 图片（base64/URL） |
| `ADD_PRINT_BARCODE(Top, Left, Width, Height, strType, strValue)` | 坐标+类型+值 | 条码 |
| `ADD_PRINT_QRCode(Top, Left, Width, Height, strValue)` | 坐标+值 | 二维码（专用） |
| `ADD_PRINT_RECT(Top, Left, Width, Height, intLineStyle, intLineWidth)` | 坐标+线型+线宽 | 矩形 |
| `ADD_PRINT_LINE(Top1, Left1, Top2, Left2, intLineStyle, intLineWidth)` | 起止坐标+线型+线宽 | 直线 |
| `ADD_PRINT_ELLIPSE(Top, Left, Width, Height, intLineStyle, intLineWidth)` | 坐标+线型+线宽 | 椭圆 |
| `ADD_PRINT_SHAPE(ShapeType, Top, Left, Width, Height, LineStyle, LineWidth, LineColor, FillColor)` | 类型+坐标+样式+颜色 | 形状 |

**坐标单位**: 0.1mm（也可用字符串如 `"10mm"`, `"1cm"`, `"1in"`）

**ShapeType 值**:
- `0` = 矩形
- `1` = 圆角矩形
- `2` = 椭圆
- `3` = 实心矩形（填充）
- `4` = 实心椭圆（填充）

**LineStyle 值**:
- `0` = 实线
- `1` = 虚线
- `2` = 点线
- `3` = 点划线

### 3.3 样式设置

| API | 参数 | 说明 |
|-----|------|------|
| `SET_PRINT_STYLE(strStyleName, varStyleValue)` | 样式名+值 | 设置全局默认样式 |
| `SET_PRINT_STYLEA(varItemName, strStyleName, varStyleValue)` | 元素ID+样式名+值 | 设置指定元素样式 |

**样式属性名**:

| 属性 | 类型 | 说明 |
|------|------|------|
| `FontSize` | number | 字号(pt) |
| `FontName` | string | 字体名 |
| `FontColor` | string | 字体颜色 |
| `Bold` | boolean | 加粗 |
| `Italic` | boolean | 斜体 |
| `Underline` | boolean | 下划线 |
| `Alignment` | number | 0=左 1=中 2=右 |
| `Angle` | number | 旋转角度 |
| `Alpha` | number | 透明度 0-255 |
| `LineSpacing` | number | 行距 |
| `PageIndex` | number | 指定页码 |
| `PreviewOnly` | boolean | 仅预览不打印 |
| `ReadOnly` | boolean | 只读 |
| `Top`/`Left`/`Width`/`Height` | number | 位置调整 |

### 3.4 打印控制

| API | 参数 | 说明 |
|-----|------|------|
| `SET_PRINT_PAGESIZE(intOrient, PageWidth, PageHeight, strPageName)` | 方向+尺寸+纸张名 | 设置纸张 |
| `SET_PRINTER_INDEX(oIndexOrName)` | 索引或名称 | 指定打印机 |
| `SET_PRINT_COPIES(intCopies)` | 份数 | 设置份数 |
| `PRINT()` | 无 | 直接打印 |
| `PREVIEW()` | 无 | 打印预览 |
| `PRINT_SETUP()` | 无 | 打印维护 |
| `PRINT_DESIGN()` | 无 | 打印设计（可视化拖拽） |

**intOrient 值**:
- `1` = 纵向（默认）
- `2` = 横向
- `3` = 旋转180度
- `0` = 不旋转

### 3.5 打印机查询

| API | 参数 | 返回值 | 说明 |
|-----|------|--------|------|
| `GET_PRINTER_COUNT()` | 无 | number | 打印机数量 |
| `GET_PRINTER_NAME(intIndex)` | 索引 | string | 打印机名称 |
| `GET_PRINT_IN_VALUE(intFlag)` | 类型标志 | string | 打印机信息 |

**GET_PRINT_IN_VALUE 的 intFlag**:
- `0` = 打印机名称
- `1` = 纸张名称列表
- `2` = 打印机状态

### 3.6 C-Lodop 特有功能

| API | 参数 | 说明 |
|-----|------|------|
| `Create_PrintTask()` | 无 | 创建云打印任务 |
| `SET_PRINT_MODE("NETWORK", true)` | 模式 | 启用网络打印 |
| `SET_PRINT_MODE("CLODOP_ServiceAddr", "x.x.x.x")` | 地址 | 指定远程 C-Lodop |
| `SET_PRINT_MODE("CLODOP_ServicePort", 8000)` | 端口 | 远程端口 |
| `SEND_PRINT_RAWDATA(strData)` | 原始数据 | 发送原始打印指令 |
| `WRITE_FILE_TEXT(strMode, strFileName, strText)` | 模式+文件名+内容 | 写文件 |
| `READ_FILE_TEXT(strFileName)` | 文件名 | 读文件 |
| `GET_SYSTEM_INFO()` | 无 | 获取系统信息 |
| `GET_TASK_STATE()` | 无 | 获取任务状态 |

## 4. PrintCraft 当前覆盖情况

### 已实现的 API (22 个)

| 类别 | API | 状态 |
|------|-----|------|
| 初始化 | `PRINT_INIT` | ✅ |
| 元素 | `ADD_PRINT_TEXT` | ✅ |
| 元素 | `ADD_PRINT_HTM` | ✅ (降级纯文本) |
| 元素 | `ADD_PRINT_TABLE` | ✅ (降级纯文本) |
| 元素 | `ADD_PRINT_URL` | ✅ (降级纯文本) |
| 元素 | `ADD_PRINT_IMAGE` | ✅ (仅 base64) |
| 元素 | `ADD_PRINT_BARCODE` | ✅ (QR/Code128/EAN13) |
| 元素 | `ADD_PRINT_RECT` | ✅ |
| 元素 | `ADD_PRINT_LINE` | ✅ |
| 元素 | `ADD_PRINT_ELLIPSE` | ✅ |
| 元素 | `ADD_PRINT_SHAPE` | ✅ |
| 样式 | `SET_PRINT_STYLE` | ✅ |
| 样式 | `SET_PRINT_STYLEA` | ✅ (SDK 端) |
| 控制 | `SET_PRINT_PAGESIZE` | ✅ |
| 控制 | `SET_PRINTER_INDEX` | ✅ |
| 控制 | `SET_PRINT_COPIES` | ✅ |
| 控制 | `PRINT` | ✅ |
| 控制 | `PREVIEW` | ✅ |
| 查询 | `GET_PRINTER_COUNT` | ✅ |
| 查询 | `GET_PRINTER_NAME` | ✅ |
| 查询 | `GET_PRINT_IN_VALUE` | ✅ |

### 未实现的 API (待开发)

| API | 优先级 | 说明 |
|-----|--------|------|
| `PRINT_INITA` | 中 | 带区域偏移的初始化 |
| `ADD_PRINT_QRCode` | 低 | 二维码专用 API（已有 BARCODE QRCode） |
| `PRINT_SETUP` | 低 | 打印维护界面 |
| `PRINT_DESIGN` | 低 | 可视化设计界面 |
| `SET_PRINT_MODE` | 中 | 打印模式设置 |
| `SEND_PRINT_RAWDATA` | 低 | 原始打印数据 |
| `WRITE_FILE_TEXT` | 低 | 文件写入 |
| `READ_FILE_TEXT` | 低 | 文件读取 |
| `GET_SYSTEM_INFO` | 低 | 系统信息 |
| `GET_TASK_STATE` | 低 | 任务状态 |
| `Create_PrintTask` | 低 | 云打印任务 |

## 5. 技术差异与兼容性问题

### 5.1 字符串单位支持

Lodop/C-Lodop 支持字符串形式的坐标:
```javascript
LODOP.ADD_PRINT_TEXT("10mm", "20mm", "50mm", "10mm", "Hello");
LODOP.ADD_PRINT_TEXT("1cm", "2cm", "5cm", "1cm", "Hello");
LODOP.ADD_PRINT_TEXT("1in", "2in", "3in", "0.5in", "Hello");
```

PrintCraft 目前仅支持数值（0.1mm 单位），需要增加字符串解析。

### 5.2 图片格式支持

| 格式 | Lodop | PrintCraft |
|------|-------|-----------|
| Base64 data-URL | ✅ | ✅ |
| HTTP/HTTPS URL | ✅ | ❌ (需实现) |
| 本地文件路径 | ✅ | ❌ (不适用) |

### 5.3 HTML 渲染能力

| 能力 | Lodop (IE) | C-Lodop | PrintCraft |
|------|-----------|---------|-----------|
| CSS 2.1 | ✅ | ✅ | ⚠️ (降级纯文本) |
| CSS 3 | ❌ | ✅ (Chromium) | ❌ (需 Chromium) |
| JavaScript 执行 | ✅ | ✅ | ❌ |
| 表格自动分页 | ✅ | ✅ | ❌ |

### 5.4 预览功能对比

| 功能 | Lodop | PrintCraft |
|------|-------|-----------|
| 预览窗口 | 内置 Windows 窗口 | 浏览器 HTML 页面 |
| 翻页 | ✅ | ❌ (单页) |
| 缩放 | ✅ | ✅ |
| 打印按钮 | ✅ | ✅ |
| 打印设计 | ✅ | ❌ |

### 5.5 通信协议差异

| 方面 | C-Lodop | PrintCraft |
|------|---------|-----------|
| 默认端口 | 8000-8005 | 18000-18005 |
| 协议 | HTTP POST + WebSocket | WebSocket only |
| 消息格式 | 自定义文本协议 | JSON |
| 心跳 | 有 | 有 (30s) |
| 重连 | 自动 | 自动 (指数退避) |

## 6. 开发建议

### 高优先级
1. **字符串单位解析** — 提升 Lodop 迁移兼容性
2. **URL 图片加载** — 从 HTTP URL 加载图片
3. **错误反馈优化** — 打印失败时给用户明确提示

### 中优先级
4. **PRINT_INITA** — 带区域偏移的初始化
5. **SET_PRINT_MODE** — 打印模式设置
6. **PREVIEW 多页支持** — 预览页面翻页
7. **HTTP API 补充** — 除 WebSocket 外支持 HTTP POST 调用

### 低优先级
8. **PRINT_DESIGN** — 可视化设计界面
9. **SEND_PRINT_RAWDATA** — ESC/POS 等原始数据
10. **云打印** — 跨网络打印任务分发
