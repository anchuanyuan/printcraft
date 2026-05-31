# PrintCraft 技术架构

## 系统分层

```
浏览器 JS SDK (Lodop 兼容 API)
        │
        │ WebSocket / HTTP
        ▼
  printcraft-server (axum)
  ┌─────────────────────────┐
  │ API Layer (ws/rest)     │
  │ Service Layer (编排)     │
  │ Render Engine (PDF)     │
  │ Platform Layer (OS打印) │
  │ System Tray             │
  └─────────────────────────┘
        │
        ▼
  OS 打印 API (CUPS/winspool)
        │
        ▼
     打印机
```

## Crate 依赖关系

```
printcraft-server
  ├── printcraft-core (类型定义)
  ├── printcraft-platform (平台层)
  │     └── printcraft-core
  └── printcraft-render (渲染引擎)
        └── printcraft-core
```

## 渲染策略

**双引擎**：
1. **printpdf** — 处理 TEXT/RECT/LINE/IMAGE/BARCODE，零外部依赖
2. **Chromium CDP** (可选) — 处理 HTM/TABLE/URL，需系统安装 Chrome

引擎选择：有 HTML 元素 → Chromium，否则 → printpdf

## 通信协议

WebSocket JSON：
- 客户端 → 服务端: `{id, cmd, args}`
- 服务端 → 客户端: `{id, ok, data?, error?}`

## 单位系统

Lodop 单位 = 0.1mm。渲染阶段转为 PDF 点 (1pt = 1/72 inch)。

详见 `src/units.rs`。
