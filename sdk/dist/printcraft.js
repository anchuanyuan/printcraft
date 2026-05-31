class p {
  constructor() {
    this.ws = null, this.port = 0, this.pending = /* @__PURE__ */ new Map(), this.reconnectTimer = null, this.reconnectDelay = 1e3, this.heartbeatTimer = null, this.connected = !1, this.connect();
  }
  /** 自动发现端口并连接 */
  async connect() {
    for (let t = 18e3; t <= 18005; t++)
      try {
        await this.tryConnect(t), this.port = t, this.connected = !0, this.reconnectDelay = 1e3, this.startHeartbeat(), console.log(`PrintCraft: 已连接 localhost:${t}`);
        return;
      } catch {
        continue;
      }
    console.warn("PrintCraft: 未找到服务，将在后台重试"), this.scheduleReconnect();
  }
  /** 尝试连接指定端口 */
  tryConnect(t) {
    return new Promise((e, n) => {
      const s = new WebSocket(`ws://127.0.0.1:${t}/ws`);
      s.onopen = () => {
        this.ws = s, this.setupListeners(s), e();
      }, s.onerror = () => n(new Error("连接失败")), setTimeout(() => {
        s.readyState !== WebSocket.OPEN && (s.close(), n(new Error("连接超时")));
      }, 2e3);
    });
  }
  /** 设置消息监听 */
  setupListeners(t) {
    t.onmessage = (e) => {
      try {
        const n = JSON.parse(e.data), s = this.pending.get(n.id);
        s && (this.pending.delete(n.id), s.resolve(n));
      } catch (n) {
        console.error("PrintCraft: 解析响应失败", n);
      }
    }, t.onclose = () => {
      this.connected = !1, this.stopHeartbeat(), console.warn("PrintCraft: 连接断开，重连中..."), this.scheduleReconnect();
    };
  }
  /** 获取当前连接的端口号 */
  getPort() {
    return this.port;
  }
  /** 发送命令并等待响应 */
  send(t, e = {}) {
    return new Promise((n, s) => {
      if (!this.connected || !this.ws) {
        s(new Error("PrintCraft 服务未连接"));
        return;
      }
      const r = crypto.randomUUID();
      this.pending.set(r, { resolve: n, reject: s });
      const i = JSON.stringify({ id: r, cmd: t, args: e });
      this.ws.send(i), setTimeout(() => {
        this.pending.has(r) && (this.pending.delete(r), s(new Error("请求超时")));
      }, 3e4);
    });
  }
  /** 安排重连 */
  scheduleReconnect() {
    if (this.reconnectTimer) return;
    const t = typeof window < "u" ? window : globalThis;
    this.reconnectTimer = t.setTimeout(() => {
      this.reconnectTimer = null, this.reconnectDelay = Math.min(this.reconnectDelay * 2, 3e4), this.connect();
    }, this.reconnectDelay);
  }
  /** 启动心跳 */
  startHeartbeat() {
    const t = typeof window < "u" ? window : globalThis;
    this.heartbeatTimer = t.setInterval(() => {
      var e;
      this.connected && ((e = this.ws) == null ? void 0 : e.readyState) === WebSocket.OPEN && this.ws.send(JSON.stringify({ id: "ping", cmd: "PING", args: {} }));
    }, 3e4);
  }
  /** 停止心跳 */
  stopHeartbeat() {
    this.heartbeatTimer && ((typeof window < "u" ? window : globalThis).clearInterval(this.heartbeatTimer), this.heartbeatTimer = null);
  }
}
class d {
  constructor() {
    this.elements = [], this.currentStyle = {}, this.taskName = "", this.printer = "", this.copies = 1, this.pageSize = { orientation: 1, width: 0, height: 0, name: "A4" }, this.connection = new p();
  }
  /**
   * 初始化打印任务
   * 对应 Lodop: PRINT_INIT(strTaskName)
   */
  PRINT_INIT(t) {
    return this.taskName = t || "", this.elements = [], this.currentStyle = {}, !0;
  }
  /**
   * 添加纯文本打印项
   * 对应 Lodop: ADD_PRINT_TEXT(top, left, width, height, strContent)
   */
  ADD_PRINT_TEXT(t, e, n, s, r) {
    const i = this.elements.length + 1;
    return this.elements.push({
      index: i,
      type: "text",
      position: { top: t, left: e, width: n, height: s },
      style: { ...this.currentStyle },
      content: r
    }), i;
  }
  /**
   * 添加矩形
   * 对应 Lodop: ADD_PRINT_RECT(top, left, width, height, intLineStyle, intLineWidth)
   */
  ADD_PRINT_RECT(t, e, n, s, r = 0, i = 1) {
    const o = this.elements.length + 1;
    return this.elements.push({
      index: o,
      type: "rect",
      position: { top: t, left: e, width: n, height: s },
      style: { ...this.currentStyle },
      lineStyle: r,
      lineWidth: i
    }), o;
  }
  /**
   * 添加直线
   * 对应 Lodop: ADD_PRINT_LINE(top1, left1, top2, left2, intLineStyle, intLineWidth)
   */
  ADD_PRINT_LINE(t, e, n, s, r = 0, i = 1) {
    const o = this.elements.length + 1;
    return this.elements.push({
      index: o,
      type: "line",
      position: { top: t, left: e, width: s - e, height: n - t },
      style: { ...this.currentStyle },
      lineStyle: r,
      lineWidth: i
    }), o;
  }
  /**
   * 添加图片
   * 对应 Lodop: ADD_PRINT_IMAGE(top, left, width, height, strHtmlContent)
   */
  ADD_PRINT_IMAGE(t, e, n, s, r) {
    const i = this.elements.length + 1;
    return this.elements.push({
      index: i,
      type: "image",
      position: { top: t, left: e, width: n, height: s },
      style: { ...this.currentStyle },
      src: r
    }), i;
  }
  /**
   * 添加条码
   * 对应 Lodop: ADD_PRINT_BARCODE(top, left, width, height, strBarCodeType, strBarCodeValue)
   */
  ADD_PRINT_BARCODE(t, e, n, s, r, i) {
    const o = this.elements.length + 1;
    return this.elements.push({
      index: o,
      type: "barcode",
      position: { top: t, left: e, width: n, height: s },
      style: { ...this.currentStyle },
      barType: r,
      code: i
    }), o;
  }
  /**
   * 添加超文本
   * 对应 Lodop: ADD_PRINT_HTM(top, left, width, height, strHtmlContent)
   */
  ADD_PRINT_HTM(t, e, n, s, r) {
    const i = this.elements.length + 1;
    return this.elements.push({
      index: i,
      type: "htm",
      position: { top: t, left: e, width: n, height: s },
      style: { ...this.currentStyle },
      html: r
    }), i;
  }
  /**
   * 添加表格
   * 对应 Lodop: ADD_PRINT_TABLE(top, left, width, height, strHtmlContent)
   */
  ADD_PRINT_TABLE(t, e, n, s, r) {
    const i = this.elements.length + 1;
    return this.elements.push({
      index: i,
      type: "table",
      position: { top: t, left: e, width: n, height: s },
      style: { ...this.currentStyle },
      html: r
    }), i;
  }
  /**
   * 添加网页地址
   * 对应 Lodop: ADD_PRINT_URL(top, left, width, height, strURL)
   */
  ADD_PRINT_URL(t, e, n, s, r) {
    const i = this.elements.length + 1;
    return this.elements.push({
      index: i,
      type: "url",
      position: { top: t, left: e, width: n, height: s },
      style: { ...this.currentStyle },
      url: r
    }), i;
  }
  /**
   * 添加椭圆
   * 对应 Lodop: ADD_PRINT_ELLIPSE(top, left, width, height, intLineStyle, intLineWidth)
   */
  ADD_PRINT_ELLIPSE(t, e, n, s, r = 0, i = 1) {
    const o = this.elements.length + 1;
    return this.elements.push({
      index: o,
      type: "ellipse",
      position: { top: t, left: e, width: n, height: s },
      style: { ...this.currentStyle },
      lineStyle: r,
      lineWidth: i
    }), o;
  }
  /**
   * 添加形状
   * 对应 Lodop: ADD_PRINT_SHAPE(intShapeType, top, left, width, height, intLineStyle, intLineWidth, strColor)
   */
  ADD_PRINT_SHAPE(t, e, n, s, r, i = 0, o = 1, a = "#000000") {
    const c = this.elements.length + 1;
    return this.elements.push({
      index: c,
      type: "shape",
      position: { top: e, left: n, width: s, height: r },
      style: { ...this.currentStyle },
      shapeType: t,
      lineStyle: i,
      lineWidth: o,
      color: a
    }), c;
  }
  /**
   * 设置下一个添加元素的样式
   * 对应 Lodop: SET_PRINT_STYLE(strStyleName, varStyleValue)
   */
  SET_PRINT_STYLE(t, e) {
    this.currentStyle[t] = e;
  }
  /**
   * 设置指定元素的样式
   * 对应 Lodop: SET_PRINT_STYLEA(varItemNameID, strStyleName, varStyleValue)
   */
  SET_PRINT_STYLEA(t, e, n) {
    const s = this.elements.find((r) => r.index === t);
    s && (s.style[e] = n);
  }
  /**
   * 设置纸张大小
   * 对应 Lodop: SET_PRINT_PAGESIZE(intOrient, PageWidth, PageHeight, strPageName)
   */
  SET_PRINT_PAGESIZE(t, e, n, s) {
    this.pageSize = { orientation: t, width: e, height: n, name: s };
  }
  /**
   * 设置目标打印机
   * 对应 Lodop: SET_PRINTER_INDEX(oIndexOrName)
   */
  SET_PRINTER_INDEX(t) {
    return this.printer = String(t), !0;
  }
  /**
   * 设置打印份数
   * 对应 Lodop: SET_PRINT_COPIES(intCopies)
   */
  SET_PRINT_COPIES(t) {
    this.copies = Math.max(1, t);
  }
  /**
   * 直接打印（无预览）
   * 对应 Lodop: PRINT()
   */
  async PRINT() {
    const t = this.buildJob(), e = await this.connection.send("PRINT", t);
    return (e == null ? void 0 : e.ok) ?? !1;
  }
  /**
   * 打印预览
   * 对应 Lodop: PREVIEW()
   *
   * 将当前任务发送到服务端渲染，打开预览窗口。
   */
  async PREVIEW() {
    var n;
    const t = this.buildJob(), e = await this.connection.send("PREVIEW", t);
    if (e != null && e.ok && ((n = e == null ? void 0 : e.data) != null && n.previewId)) {
      const s = e.data.previewId, i = `http://127.0.0.1:${this.connection.getPort()}/preview/${s}`;
      return window.open(i, "_blank", "width=900,height=700,scrollbars=yes"), 1;
    }
    return console.warn("PrintCraft: PREVIEW 失败", e == null ? void 0 : e.error), 0;
  }
  /**
   * 获取打印机数量
   * 对应 Lodop: GET_PRINTER_COUNT()
   */
  async GET_PRINTER_COUNT() {
    var e;
    const t = await this.connection.send("GET_PRINTER_COUNT", {});
    return ((e = t == null ? void 0 : t.data) == null ? void 0 : e.count) ?? 0;
  }
  /**
   * 获取打印机名称
   * 对应 Lodop: GET_PRINTER_NAME(intPrinterIndex)
   */
  async GET_PRINTER_NAME(t) {
    var n;
    const e = await this.connection.send("GET_PRINTER_NAME", { index: t });
    return ((n = e == null ? void 0 : e.data) == null ? void 0 : n.name) ?? "";
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
const h = new d();
window.LODOP = h;
window.CLODOP = h;
window.getLodop = () => h;
export {
  d as Lodop,
  h as default
};
//# sourceMappingURL=printcraft.js.map
