# PrintCraft

开源无水印 Web 打印服务，Lodop/C-Lodop 替代品。

## 为什么用 PrintCraft？

| 对比 | Lodop 免费版 | PrintCraft |
|------|------------|-----------|
| 水印 | 有水印 | 无水印 |
| 开源 | 否 | MIT 开源 |
| 平台 | Windows | Windows + macOS |
| 技术栈 | C++ (IE 内核) | Rust + TypeScript |
| HTML 渲染 | IE | Chromium（可选） |

## 系统要求

- Windows 10+ x64 / macOS 10.15+
- 已安装打印机驱动

## 快速开始

### 1. 启动服务

**Windows：**
```
双击 printcraft-server.exe
```

**macOS：**
```bash
./printcraft-server
```

服务启动后：
- 监听 `localhost:18000`
- 系统托盘出现 PrintCraft 图标
- 开机自动启动（可在托盘菜单关闭）

### 2. 网页中引入 SDK

```html
<script src="http://localhost:18000/sdk/printcraft.js"></script>
```

### 3. 调用打印

```javascript
// 获取 LODOP 对象（与原版 Lodop 用法完全相同）
var LODOP = getLodop();

// 初始化
LODOP.PRINT_INIT("我的打印任务");

// 设置纸张 A4
LODOP.SET_PRINT_PAGESIZE(1, 0, 0, "A4");

// 添加文字
LODOP.ADD_PRINT_TEXT(50, 50, 300, 30, "Hello PrintCraft!");

// 设置样式
LODOP.SET_PRINT_STYLE("FontSize", 14);
LODOP.SET_PRINT_STYLE("FontName", "SimSun");
LODOP.SET_PRINT_STYLE("Bold", true);

// 打印
LODOP.PRINT();
```

## API 参考

### 打印初始化

```javascript
LODOP.PRINT_INIT(strTaskName)    // 初始化任务，清空之前的元素
```

### 添加打印元素

```javascript
// 纯文本
LODOP.ADD_PRINT_TEXT(top, left, width, height, strContent)

// 矩形
LODOP.ADD_PRINT_RECT(top, left, width, height, intLineStyle, intLineWidth)

// 直线
LODOP.ADD_PRINT_LINE(top1, left1, top2, left2, intLineStyle, intLineWidth)

// 椭圆
LODOP.ADD_PRINT_ELLIPSE(top, left, width, height, intLineStyle, intLineWidth)

// 图片（支持 base64 data-URL）
LODOP.ADD_PRINT_IMAGE(top, left, width, height, strHtmlContent)

// 条码 / 二维码
LODOP.ADD_PRINT_BARCODE(top, left, width, height, strType, strValue)
// strType: "QRCode", "Code128", "EAN13"

// HTML 内容
LODOP.ADD_PRINT_HTM(top, left, width, height, strHtmlContent)

// 表格
LODOP.ADD_PRINT_TABLE(top, left, width, height, strHtmlContent)

// 网页 URL
LODOP.ADD_PRINT_URL(top, left, width, height, strUrl)

// 形状
LODOP.ADD_PRINT_SHAPE(top, left, width, height, intShapeType, intLineStyle, intLineWidth)
```

### 设置样式

```javascript
LODOP.SET_PRINT_STYLE(strName, varValue)

// 常用样式名
LODOP.SET_PRINT_STYLE("FontSize", 14)           // 字号
LODOP.SET_PRINT_STYLE("FontName", "SimSun")     // 字体
LODOP.SET_PRINT_STYLE("Bold", true)             // 加粗
LODOP.SET_PRINT_STYLE("Italic", true)           // 斜体
LODOP.SET_PRINT_STYLE("Underline", true)        // 下划线
LODOP.SET_PRINT_STYLE("Alignment", 1)           // 0=左 1=中 2=右
LODOP.SET_PRINT_STYLE("FontColor", "#FF0000")   // 字体颜色
```

### 打印控制

```javascript
LODOP.SET_PRINT_PAGESIZE(intOrient, PageWidth, PageHeight, strPageName)
// intOrient: 1=纵向 2=横向
// strPageName: "A4", "A5", "Letter" 等

LODOP.SET_PRINTER_INDEX(strPrinterName)  // 指定打印机
LODOP.SET_PRINT_COPIES(intCopies)        // 设置份数

LODOP.PRINT()       // 直接打印
LODOP.PREVIEW()     // 打印预览
```

### 打印机查询

```javascript
LODOP.GET_PRINTER_COUNT()                // 返回打印机数量
LODOP.GET_PRINTER_NAME(intIndex)         // 返回打印机名称
LODOP.GET_PRINT_IN_VALUE(intFlag)        // 查询输入值
```

### 坐标单位

默认使用 Lodop 单位（0.1mm）：

```
坐标值 1000 = 100mm = 10cm
A4 纸: 宽 2100, 高 2970
```

## CLI 命令行工具

```bash
printcraft status       # 查看服务状态
printcraft printers     # 列出系统打印机
printcraft start        # 启动服务
printcraft config       # 查看配置
```

## 从 Lodop 迁移

迁移只需 3 步：

```html
<!-- 1. 替换 script src -->
<!-- 旧: <script src="http://localhost:8000/LodopFuncs.js"></script> -->
<script src="http://localhost:18000/printcraft.js"></script>

<!-- 2. 全局变量自动兼容，代码无需修改 -->
<script>
  var LODOP = getLodop();  // 自动可用
  LODOP.PRINT_INIT("测试");
  LODOP.ADD_PRINT_TEXT(50, 50, 300, 30, "迁移成功");
  LODOP.PRINT();
</script>
```

详细迁移指南见 [docs/MIGRATION_FROM_LODOP.md](docs/MIGRATION_FROM_LODOP.md)

## 项目结构

```
printcraft/
├── crates/
│   ├── printcraft-core/       # 核心类型
│   ├── printcraft-platform/   # 平台层（CUPS / winspool）
│   ├── printcraft-render/     # 渲染引擎（printpdf + Chromium）
│   └── printcraft-server/     # HTTP/WS 服务
├── sdk/                       # 浏览器 JS SDK
├── installers/                # 安装器配置
└── docs/                      # 文档
```

## 构建

```bash
# macOS 构建
cargo build --release -p printcraft-server -p printcraft-cli

# 交叉编译 Windows (需 cargo-xwin)
cargo xwin build --target x86_64-pc-windows-msvc -p printcraft-server -p printcraft-cli --release

# 构建 SDK
cd sdk && npm install && npm run build
```

## 测试

```bash
cargo test --workspace --lib --tests   # 49 Rust 测试
cd sdk && npm test                      # 23 SDK 测试
```

## 技术架构

```
浏览器 ──WebSocket──▶ printcraft-server ──▶ 渲染引擎 ──▶ 平台层 ──▶ 打印机
                                │
                          localhost:18000
```

- **双引擎渲染**：纯元素用 printpdf（快），HTML 用 Chromium CDP（功能全）
- **端口发现**：自动尝试 18000-18005
- **Lodop 兼容**：22 个 API 方法，迁移只改 script src

## License

MIT
