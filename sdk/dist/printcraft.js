function p() {
  return typeof crypto < "u" && crypto.randomUUID ? crypto.randomUUID() : "xxxx-xxxx-xxxx".replace(
    /x/g,
    () => Math.floor(Math.random() * 16).toString(16)
  );
}
class d {
  constructor() {
    this.ws = null, this.port = 0, this.pending = /* @__PURE__ */ new Map(), this.reconnectTimer = null, this.reconnectDelay = 1e3, this.heartbeatTimer = null, this.connected = !1, this.connect();
  }
  /** 检测 PrintCraft 服务主机（从 script src 或当前页面） */
  detectHost() {
    if (typeof document < "u") {
      const e = document.querySelectorAll('script[src*="printcraft.js"]');
      if (e.length > 0)
        try {
          return new URL(e[0].src).hostname;
        } catch {
        }
    }
    return "127.0.0.1";
  }
  /** 获取 script 所在端口 */
  detectPort() {
    if (typeof document < "u") {
      const e = document.querySelectorAll('script[src*="printcraft.js"]');
      if (e.length > 0)
        try {
          const t = new URL(e[0].src);
          return parseInt(t.port) || (t.protocol === "https:" ? 443 : 80);
        } catch {
        }
    }
    return null;
  }
  /** 自动发现端口并连接 */
  async connect() {
    const e = this.detectHost(), t = this.detectPort();
    if (t && t >= 3e3 && t <= 65535)
      try {
        await this.tryConnect(e, t), this.port = t, this.connected = !0, this.reconnectDelay = 1e3, this.startHeartbeat(), console.log(`PrintCraft: 已连接 ${e}:${t} (via proxy)`);
        return;
      } catch {
      }
    for (let n = 18e3; n <= 18005; n++)
      try {
        await this.tryConnect(e, n), this.port = n, this.connected = !0, this.reconnectDelay = 1e3, this.startHeartbeat(), console.log(`PrintCraft: 已连接 ${e}:${n}`);
        return;
      } catch {
        continue;
      }
    console.warn("PrintCraft: 未找到服务，将在后台重试"), this.scheduleReconnect();
  }
  /** 尝试连接指定端口 */
  tryConnect(e, t) {
    return new Promise((n, s) => {
      const i = new WebSocket(`ws://${e}:${t}/ws`);
      i.onopen = () => {
        this.ws = i, this.setupListeners(i), n();
      }, i.onerror = () => s(new Error("连接失败")), setTimeout(() => {
        i.readyState !== WebSocket.OPEN && (i.close(), s(new Error("连接超时")));
      }, 2e3);
    });
  }
  /** 设置消息监听 */
  setupListeners(e) {
    e.onmessage = (t) => {
      try {
        const n = JSON.parse(t.data), s = this.pending.get(n.id);
        s && (this.pending.delete(n.id), s.resolve(n));
      } catch (n) {
        console.error("PrintCraft: 解析响应失败", n);
      }
    }, e.onclose = () => {
      this.connected = !1, this.stopHeartbeat(), console.warn("PrintCraft: 连接断开，重连中..."), this.scheduleReconnect();
    };
  }
  /** 获取当前连接的端口号 */
  getPort() {
    return this.port;
  }
  /** 发送命令并等待响应 */
  send(e, t = {}) {
    return new Promise((n, s) => {
      if (!this.connected || !this.ws) {
        s(new Error("PrintCraft 服务未连接"));
        return;
      }
      const i = p();
      this.pending.set(i, { resolve: n, reject: s });
      const r = JSON.stringify({ id: i, cmd: e, args: t });
      this.ws.send(r), setTimeout(() => {
        this.pending.has(i) && (this.pending.delete(i), s(new Error("请求超时")));
      }, 3e4);
    });
  }
  /** 安排重连 */
  scheduleReconnect() {
    if (this.reconnectTimer) return;
    const e = typeof window < "u" ? window : globalThis;
    this.reconnectTimer = e.setTimeout(() => {
      this.reconnectTimer = null, this.reconnectDelay = Math.min(this.reconnectDelay * 2, 3e4), this.connect();
    }, this.reconnectDelay);
  }
  /** 启动心跳 */
  startHeartbeat() {
    const e = typeof window < "u" ? window : globalThis;
    this.heartbeatTimer = e.setInterval(() => {
      var t;
      this.connected && ((t = this.ws) == null ? void 0 : t.readyState) === WebSocket.OPEN && this.ws.send(JSON.stringify({ id: "ping", cmd: "PING", args: {} }));
    }, 3e4);
  }
  /** 停止心跳 */
  stopHeartbeat() {
    this.heartbeatTimer && ((typeof window < "u" ? window : globalThis).clearInterval(this.heartbeatTimer), this.heartbeatTimer = null);
  }
}
class u {
  constructor() {
    this.elements = [], this.currentStyle = {}, this.taskName = "", this.printer = "", this.copies = 1, this.pageSize = { orientation: 1, width: 0, height: 0, name: "A4" }, this.connection = new d();
  }
  /**
   * 初始化打印任务
   * 对应 Lodop: PRINT_INIT(strTaskName)
   */
  PRINT_INIT(e) {
    return this.taskName = e || "", this.elements = [], this.currentStyle = {}, !0;
  }
  /**
   * 添加纯文本打印项
   * 对应 Lodop: ADD_PRINT_TEXT(top, left, width, height, strContent)
   */
  ADD_PRINT_TEXT(e, t, n, s, i) {
    const r = this.elements.length + 1;
    return this.elements.push({
      index: r,
      type: "text",
      position: { top: e, left: t, width: n, height: s },
      style: { ...this.currentStyle },
      content: i
    }), r;
  }
  /**
   * 添加矩形
   * 对应 Lodop: ADD_PRINT_RECT(top, left, width, height, intLineStyle, intLineWidth)
   */
  ADD_PRINT_RECT(e, t, n, s, i = 0, r = 1) {
    const o = this.elements.length + 1;
    return this.elements.push({
      index: o,
      type: "rect",
      position: { top: e, left: t, width: n, height: s },
      style: { ...this.currentStyle },
      lineStyle: i,
      lineWidth: r
    }), o;
  }
  /**
   * 添加直线
   * 对应 Lodop: ADD_PRINT_LINE(top1, left1, top2, left2, intLineStyle, intLineWidth)
   */
  ADD_PRINT_LINE(e, t, n, s, i = 0, r = 1) {
    const o = this.elements.length + 1;
    return this.elements.push({
      index: o,
      type: "line",
      position: { top: e, left: t, width: s - t, height: n - e },
      style: { ...this.currentStyle },
      lineStyle: i,
      lineWidth: r
    }), o;
  }
  /**
   * 添加图片
   * 对应 Lodop: ADD_PRINT_IMAGE(top, left, width, height, strHtmlContent)
   */
  ADD_PRINT_IMAGE(e, t, n, s, i) {
    const r = this.elements.length + 1;
    return this.elements.push({
      index: r,
      type: "image",
      position: { top: e, left: t, width: n, height: s },
      style: { ...this.currentStyle },
      src: i
    }), r;
  }
  /**
   * 添加条码
   * 对应 Lodop: ADD_PRINT_BARCODE(top, left, width, height, strBarCodeType, strBarCodeValue)
   */
  ADD_PRINT_BARCODE(e, t, n, s, i, r) {
    const o = this.elements.length + 1;
    return this.elements.push({
      index: o,
      type: "barcode",
      position: { top: e, left: t, width: n, height: s },
      style: { ...this.currentStyle },
      barType: i,
      code: r
    }), o;
  }
  /**
   * 添加超文本
   * 对应 Lodop: ADD_PRINT_HTM(top, left, width, height, strHtmlContent)
   */
  ADD_PRINT_HTM(e, t, n, s, i) {
    const r = this.elements.length + 1;
    return this.elements.push({
      index: r,
      type: "htm",
      position: { top: e, left: t, width: n, height: s },
      style: { ...this.currentStyle },
      html: i
    }), r;
  }
  /**
   * 添加表格
   * 对应 Lodop: ADD_PRINT_TABLE(top, left, width, height, strHtmlContent)
   */
  ADD_PRINT_TABLE(e, t, n, s, i) {
    const r = this.elements.length + 1;
    return this.elements.push({
      index: r,
      type: "table",
      position: { top: e, left: t, width: n, height: s },
      style: { ...this.currentStyle },
      html: i
    }), r;
  }
  /**
   * 添加网页地址
   * 对应 Lodop: ADD_PRINT_URL(top, left, width, height, strURL)
   */
  ADD_PRINT_URL(e, t, n, s, i) {
    const r = this.elements.length + 1;
    return this.elements.push({
      index: r,
      type: "url",
      position: { top: e, left: t, width: n, height: s },
      style: { ...this.currentStyle },
      url: i
    }), r;
  }
  /**
   * 添加椭圆
   * 对应 Lodop: ADD_PRINT_ELLIPSE(top, left, width, height, intLineStyle, intLineWidth)
   */
  ADD_PRINT_ELLIPSE(e, t, n, s, i = 0, r = 1) {
    const o = this.elements.length + 1;
    return this.elements.push({
      index: o,
      type: "ellipse",
      position: { top: e, left: t, width: n, height: s },
      style: { ...this.currentStyle },
      lineStyle: i,
      lineWidth: r
    }), o;
  }
  /**
   * 添加形状
   * 对应 Lodop: ADD_PRINT_SHAPE(intShapeType, top, left, width, height, intLineStyle, intLineWidth, strColor)
   */
  ADD_PRINT_SHAPE(e, t, n, s, i, r = 0, o = 1, l = "#000000") {
    const h = this.elements.length + 1;
    return this.elements.push({
      index: h,
      type: "shape",
      position: { top: t, left: n, width: s, height: i },
      style: { ...this.currentStyle },
      shapeType: e,
      lineStyle: r,
      lineWidth: o,
      color: l
    }), h;
  }
  /**
   * 设置下一个添加元素的样式
   * 对应 Lodop: SET_PRINT_STYLE(strStyleName, varStyleValue)
   */
  SET_PRINT_STYLE(e, t) {
    this.currentStyle[e] = t;
  }
  /**
   * 设置指定元素的样式
   * 对应 Lodop: SET_PRINT_STYLEA(varItemNameID, strStyleName, varStyleValue)
   */
  SET_PRINT_STYLEA(e, t, n) {
    const s = this.elements.find((i) => i.index === e);
    s && (s.style[t] = n);
  }
  /**
   * 设置纸张大小
   * 对应 Lodop: SET_PRINT_PAGESIZE(intOrient, PageWidth, PageHeight, strPageName)
   */
  SET_PRINT_PAGESIZE(e, t, n, s) {
    this.pageSize = { orientation: e, width: t, height: n, name: s };
  }
  /**
   * 设置目标打印机
   * 对应 Lodop: SET_PRINTER_INDEX(oIndexOrName)
   */
  SET_PRINTER_INDEX(e) {
    return this.printer = String(e), !0;
  }
  /**
   * 设置打印份数
   * 对应 Lodop: SET_PRINT_COPIES(intCopies)
   */
  SET_PRINT_COPIES(e) {
    this.copies = Math.max(1, e);
  }
  /**
   * 直接打印（无预览）
   * 对应 Lodop: PRINT()
   */
  async PRINT() {
    const e = this.buildJob(), t = await this.connection.send("PRINT", e);
    return (t == null ? void 0 : t.ok) ?? !1;
  }
  /**
   * 打印预览
   * 对应 Lodop: PREVIEW()
   *
   * 将当前任务发送到服务端渲染，打开预览窗口。
   */
  async PREVIEW() {
    var n;
    const e = this.buildJob(), t = await this.connection.send("PREVIEW", e);
    if (t != null && t.ok && ((n = t == null ? void 0 : t.data) != null && n.previewId)) {
      const s = t.data.previewId, r = `http://127.0.0.1:${this.connection.getPort()}/preview/${s}`;
      return window.open(r, "_blank", "width=900,height=700,scrollbars=yes"), 1;
    }
    return console.warn("PrintCraft: PREVIEW 失败", t == null ? void 0 : t.error), 0;
  }
  /**
   * 获取打印机数量
   * 对应 Lodop: GET_PRINTER_COUNT()
   */
  async GET_PRINTER_COUNT() {
    var t;
    const e = await this.connection.send("GET_PRINTER_COUNT", {});
    return ((t = e == null ? void 0 : e.data) == null ? void 0 : t.count) ?? 0;
  }
  /**
   * 获取打印机名称
   * 对应 Lodop: GET_PRINTER_NAME(intPrinterIndex)
   */
  async GET_PRINTER_NAME(e) {
    var n;
    const t = await this.connection.send("GET_PRINTER_NAME", { index: e });
    return ((n = t == null ? void 0 : t.data) == null ? void 0 : n.name) ?? "";
  }
  /** 构建打印任务对象 */
  buildJob() {
    return {
      name: this.taskName,
      printer: this.printer,
      copies: this.copies,
      pageSize: this.pageSize,
      elements: this.elements
    };
  }
}
const c = new u();
window.LODOP = c;
window.CLODOP = c;
window.getLodop = () => c;
export {
  u as Lodop,
  c as default
};
//# sourceMappingURL=printcraft.js.map
