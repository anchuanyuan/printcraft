# Windows 静默打印方案 — 排查与解决记录

> 日期: 2026-05-31

## 问题

发送打印命令后，系统打印机没有收到任务。服务端日志显示 ShellExecuteW 失败，错误码 31（SE_ERR_NOASSOC = 没有关联的应用程序）。

## 环境

- Windows 11 ARM64
- 默认 PDF 打开方式：Microsoft Edge（UWP 应用）
- 打印机：DL-888D(NEW)（热敏标签打印机）

## 排查过程

### 第1步：定位 ShellExecuteW 失败原因

最初代码用 `ShellExecuteW("printto", ...)` 发送 PDF 到打印机。

```
ShellExecuteW printto 失败 (没有关联的应用程序)
```

错误码 31 表示 Windows 找不到能处理该文件类型的程序。

**原因**：Microsoft Edge 是 UWP/打包应用，不向 ShellExecuteW 注册 `printto` verb。
传统 Win32 PDF 阅读器（Adobe Reader、Foxit、SumatraPDF）会注册这些 verb，但 Edge 不会。

### 第2步：尝试 ShellExecuteW("print")

回退到 `print` verb（不指定打印机，弹出系统打印对话框）。

同样失败，错误码 31。Edge 也不注册 `print` verb 给 ShellExecuteW。

### 第3步：调研开源方案

调研了 QZ Tray、Electron、C-Lodop 等开源项目的静默打印方案：

| 方案 | 原理 |
|------|------|
| **QZ Tray** (Java) | `javax.print` 的 `DocPrintJob.print()`，绕过对话框直接调系统 API |
| **Electron** | `webContents.print({ silent: true })`，Chromium 内部直接发到 OS 打印队列 |
| **C-Lodop** | 本地服务直接调 winspool/CUPS，不经过 PDF 阅读器 |

**关键发现**：所有方案都不依赖 PDF 阅读器，而是直接调用操作系统的打印 API。

### 第4步：采用 Windows Print Spooler API

Windows 提供了 Print Spooler API，可以直接往打印队列发送原始字节：

```
OpenPrinterW → StartDocPrinterW → WritePrinter → EndDocPrinter → ClosePrinter
```

这是 Windows 最底层的打印接口，不依赖任何应用程序。只要打印机驱动存在就能用。

## 最终方案

三重策略，按优先级降级：

```
1. Spooler API（首选）  — 直接发送 PDF 字节到打印队列，静默，无外部依赖
2. SumatraPDF（备选）   — 如果打包了 SumatraPDF.exe，用 CLI 静默打印
3. ShellExecuteW（兜底）— 弹出系统打印对话框
```

### Spooler API 核心代码

```rust
// 打开打印机
OpenPrinterW(printer_name, &mut h_printer, null_mut());

// 开始文档（数据类型 "RAW" = 原始字节直通）
StartDocPrinterW(h_printer, 1, &doc_info);

// 写入 PDF 字节
WritePrinter(h_printer, pdf_data.as_ptr(), pdf_data.len(), &mut written);

// 结束
EndDocPrinter(h_printer);
ClosePrinter(h_printer);
```

数据类型设为 `"RAW"` 表示不对数据做任何处理，直接传给打印机驱动。

## 关键教训

1. **不要依赖 ShellExecuteW 打印** — 现代 Windows 的默认 PDF 处理器是 Edge（UWP），不注册 print/printto verb
2. **不要依赖 PDF 阅读器** — 直接用操作系统底层 API（Spooler）才是正道
3. **RAW 数据类型** — StartDocPrinterW 的数据类型设为 "RAW" 可以绕过驱动的数据转换，直接发送原始字节
4. **热敏打印机注意** — DL-888D 等标签打印机收到 PDF 后可能打出来是乱码（它们通常只支持 ESC/POS 指令），需要后续支持图片渲染

## 文件

- `crates/printcraft-platform/src/windows/pdf_print.rs` — 打印实现
- `crates/printcraft-server/src/service.rs` — SDK args 解析
- `crates/printcraft-render/src/simple.rs` — HTML 降级纯文本
- `sdk/src/lodop.ts` — PRINT 错误日志
- `sdk/preview/index.html` — 预览页打印按钮修复
