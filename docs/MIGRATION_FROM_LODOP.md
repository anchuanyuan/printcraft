# 从 Lodop 迁移到 PrintCraft

本文档帮助已使用 Lodop/C-Lodop 的项目快速迁移到 PrintCraft。

## 迁移概述

PrintCraft 是 Lodop 的开源替代品，API 设计高度兼容。迁移只需 3 步：

1. 替换 JS 引用
2. 修改连接地址（可选）
3. 测试打印功能

## 步骤 1: 替换 JS 引用

**Lodop 原代码：**
```html
<script src="http://localhost:8000/LodopFuncs.js"></script>
```

**PrintCraft 替换为：**
```html
<script src="http://localhost:18000/printcraft.js"></script>
```

> PrintCraft 默认端口 18000（Lodop/C-Lodop 用 8000-8005）

## 步骤 2: 全局变量兼容

PrintCraft 自动暴露以下全局变量（与 Lodop 完全兼容）：

| 变量 | 说明 |
|------|------|
| `window.LODOP` | 主对象 |
| `window.CLODOP` | 兼容别名 |
| `getLodop()` | 获取函数 |

现有代码无需修改，直接使用：
```javascript
var LODOP = getLodop();
LODOP.PRINT_INIT("打印任务");
LODOP.ADD_PRINT_TEXT(50, 50, 300, 30, "Hello");
LODOP.PRINT();
```

## 已支持的 Lodop API

### 打印元素
| Lodop API | PrintCraft | 状态 |
|-----------|-----------|------|
| ADD_PRINT_TEXT | ✅ | 完全兼容 |
| ADD_PRINT_RECT | ✅ | 完全兼容 |
| ADD_PRINT_LINE | ✅ | 完全兼容 |
| ADD_PRINT_ELLIPSE | ✅ | 完全兼容 |
| ADD_PRINT_IMAGE | ✅ | 完全兼容 |
| ADD_PRINT_BARCODE | ✅ | 支持 QR Code/Code128 等 |
| ADD_PRINT_HTM | ✅ | 完全兼容 |
| ADD_PRINT_TABLE | ✅ | 完全兼容 |
| ADD_PRINT_URL | ✅ | 完全兼容 |
| ADD_PRINT_SHAPE | ✅ | 完全兼容 |

### 样式设置
| Lodop API | PrintCraft | 状态 |
|-----------|-----------|------|
| SET_PRINT_STYLE | ✅ | 完全兼容 |
| SET_PRINT_STYLEA | ✅ | 完全兼容 |

### 打印控制
| Lodop API | PrintCraft | 状态 |
|-----------|-----------|------|
| PRINT_INIT | ✅ | 完全兼容 |
| SET_PRINT_PAGESIZE | ✅ | 完全兼容 |
| SET_PRINTER_INDEX | ✅ | 完全兼容 |
| SET_PRINT_COPIES | ✅ | 完全兼容 |
| PRINT | ✅ | 完全兼容 |
| PREVIEW | ✅ | 完全兼容 |

### 打印机查询
| Lodop API | PrintCraft | 状态 |
|-----------|-----------|------|
| GET_PRINTER_COUNT | ✅ | 完全兼容 |
| GET_PRINTER_NAME | ✅ | 完全兼容 |
| GET_PRINT_IN_VALUE | ✅ | 完全兼容 |

## 已知差异

| 项目 | Lodop | PrintCraft |
|------|-------|-----------|
| 端口 | 8000-8005 | 18000-18005 |
| 水印 | 免费版有水印 | 无水印 |
| 打印预览 | 内置预览窗口 | HTML 预览页 |
| HTML 渲染 | IE 内核 | Chromium（可选） |
| 条码类型 | 全部支持 | QR/Code128/EAN13 等 |

## 样式属性对照

```javascript
// 两者完全相同
LODOP.SET_PRINT_STYLE("FontSize", 14);
LODOP.SET_PRINT_STYLE("FontName", "SimSun");
LODOP.SET_PRINT_STYLE("Bold", true);
LODOP.SET_PRINT_STYLE("Alignment", 1); // 0=左 1=中 2=右
LODOP.SET_PRINT_STYLE("FontColor", "#FF0000");
```

## 常见迁移问题

### Q: 服务未连接
A: 确保 PrintCraft 服务已运行。运行 `printcraft status` 检查。

### Q: 打印机列表为空
A: PrintCraft 使用系统打印机，确保系统已安装打印机驱动。

### Q: 打印内容位置偏移
A: PrintCraft 使用 PDF 渲染，坐标系统与 Lodop 相同（0.1mm 单位）。

## 卸载 Lodop

迁移完成后：
1. 删除 Lodop 安装目录
2. 删除 Lodop 的开机自启项
3. 移除代码中的 Lodop 相关引用
