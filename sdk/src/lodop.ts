/**
 * Lodop 兼容 API 类
 *
 * 提供与 Lodop 完全兼容的 API 接口。
 * 所有方法名、参数顺序、返回值都与原版 Lodop 一致。
 *
 * 使用方式（与 Lodop 完全相同）:
 *   LODOP.PRINT_INIT("打印任务名");
 *   LODOP.SET_PRINT_PAGESIZE(1, 0, 0, "A4");
 *   LODOP.ADD_PRINT_TEXT(50, 50, 300, 30, "Hello World");
 *   LODOP.PRINT();
 */

import { Connection } from './connection';
import { PrintStyle } from './style';
import { PrintElement, BarcodeType } from './elements';

export class Lodop {
  private connection: Connection;
  private elements: PrintElement[] = [];
  private currentStyle: PrintStyle = {};
  private taskName: string = '';
  private printer: string = '';
  private copies: number = 1;
  private pageSize = { orientation: 1, width: 0, height: 0, name: 'A4' };

  constructor() {
    this.connection = new Connection();
  }

  /**
   * 初始化打印任务
   * 对应 Lodop: PRINT_INIT(strTaskName)
   */
  PRINT_INIT(strTaskName: string): boolean {
    this.taskName = strTaskName || '';
    this.elements = [];
    this.currentStyle = {};
    this.printer = '';
    this.copies = 1;
    this.pageSize = { orientation: 1, width: 0, height: 0, name: 'A4' };
    return true;
  }

  /**
   * 添加纯文本打印项
   * 对应 Lodop: ADD_PRINT_TEXT(top, left, width, height, strContent)
   */
  ADD_PRINT_TEXT(top: number, left: number, width: number, height: number, content: string): number {
    const index = this.elements.length + 1;
    this.elements.push({
      index,
      type: 'text',
      position: { top, left, width, height },
      style: { ...this.currentStyle },
      content,
    });
    return index;
  }

  /**
   * 添加矩形
   * 对应 Lodop: ADD_PRINT_RECT(top, left, width, height, intLineStyle, intLineWidth)
   */
  ADD_PRINT_RECT(top: number, left: number, width: number, height: number, lineStyle = 0, lineWidth = 1): number {
    const index = this.elements.length + 1;
    this.elements.push({
      index,
      type: 'rect',
      position: { top, left, width, height },
      style: { ...this.currentStyle },
      lineStyle,
      lineWidth,
    });
    return index;
  }

  /**
   * 添加直线
   * 对应 Lodop: ADD_PRINT_LINE(top1, left1, top2, left2, intLineStyle, intLineWidth)
   */
  ADD_PRINT_LINE(top1: number, left1: number, top2: number, left2: number, lineStyle = 0, lineWidth = 1): number {
    const index = this.elements.length + 1;
    this.elements.push({
      index,
      type: 'line',
      position: { top: top1, left: left1, width: left2 - left1, height: top2 - top1 },
      style: { ...this.currentStyle },
      lineStyle,
      lineWidth,
    });
    return index;
  }

  /**
   * 添加图片
   * 对应 Lodop: ADD_PRINT_IMAGE(top, left, width, height, strHtmlContent)
   */
  ADD_PRINT_IMAGE(top: number, left: number, width: number, height: number, src: string): number {
    const index = this.elements.length + 1;
    this.elements.push({
      index,
      type: 'image',
      position: { top, left, width, height },
      style: { ...this.currentStyle },
      src,
    });
    return index;
  }

  /**
   * 添加条码
   * 对应 Lodop: ADD_PRINT_BARCODE(top, left, width, height, strBarCodeType, strBarCodeValue)
   */
  ADD_PRINT_BARCODE(top: number, left: number, width: number, height: number, barType: string, code: string): number {
    const index = this.elements.length + 1;
    this.elements.push({
      index,
      type: 'barcode',
      position: { top, left, width, height },
      style: { ...this.currentStyle },
      barType,
      code,
    });
    return index;
  }

  /**
   * 添加超文本
   * 对应 Lodop: ADD_PRINT_HTM(top, left, width, height, strHtmlContent)
   */
  ADD_PRINT_HTM(top: number, left: number, width: number, height: number, html: string): number {
    const index = this.elements.length + 1;
    this.elements.push({
      index,
      type: 'htm',
      position: { top, left, width, height },
      style: { ...this.currentStyle },
      html,
    });
    return index;
  }

  /**
   * 添加表格
   * 对应 Lodop: ADD_PRINT_TABLE(top, left, width, height, strHtmlContent)
   */
  ADD_PRINT_TABLE(top: number, left: number, width: number, height: number, html: string): number {
    const index = this.elements.length + 1;
    this.elements.push({
      index,
      type: 'table',
      position: { top, left, width, height },
      style: { ...this.currentStyle },
      html,
    });
    return index;
  }

  /**
   * 添加网页地址
   * 对应 Lodop: ADD_PRINT_URL(top, left, width, height, strURL)
   */
  ADD_PRINT_URL(top: number, left: number, width: number, height: number, url: string): number {
    const index = this.elements.length + 1;
    this.elements.push({
      index,
      type: 'url',
      position: { top, left, width, height },
      style: { ...this.currentStyle },
      url,
    });
    return index;
  }

  /**
   * 添加椭圆
   * 对应 Lodop: ADD_PRINT_ELLIPSE(top, left, width, height, intLineStyle, intLineWidth)
   */
  ADD_PRINT_ELLIPSE(top: number, left: number, width: number, height: number, lineStyle = 0, lineWidth = 1): number {
    const index = this.elements.length + 1;
    this.elements.push({
      index,
      type: 'ellipse',
      position: { top, left, width, height },
      style: { ...this.currentStyle },
      lineStyle,
      lineWidth,
    });
    return index;
  }

  /**
   * 添加形状
   * 对应 Lodop: ADD_PRINT_SHAPE(intShapeType, top, left, width, height, intLineStyle, intLineWidth, strColor)
   */
  ADD_PRINT_SHAPE(shapeType: number, top: number, left: number, width: number, height: number, lineStyle = 0, lineWidth = 1, color = '#000000'): number {
    const index = this.elements.length + 1;
    this.elements.push({
      index,
      type: 'shape',
      position: { top, left, width, height },
      style: { ...this.currentStyle },
      shapeType,
      lineStyle,
      lineWidth,
      color,
    });
    return index;
  }

  /**
   * 设置下一个添加元素的样式
   * 对应 Lodop: SET_PRINT_STYLE(strStyleName, varStyleValue)
   */
  SET_PRINT_STYLE(name: string, value: any): void {
    (this.currentStyle as any)[name] = value;
  }

  /**
   * 设置指定元素的样式
   * 对应 Lodop: SET_PRINT_STYLEA(varItemNameID, strStyleName, varStyleValue)
   */
  SET_PRINT_STYLEA(itemId: number | string, name: string, value: any): void {
    const el = this.elements.find(e => e.index === itemId);
    if (el) {
      (el.style as any)[name] = value;
    }
  }

  /**
   * 设置纸张大小
   * 对应 Lodop: SET_PRINT_PAGESIZE(intOrient, PageWidth, PageHeight, strPageName)
   */
  SET_PRINT_PAGESIZE(orient: number, width: number, height: number, pageName: string): void {
    this.pageSize = { orientation: orient, width, height, name: pageName };
  }

  /**
   * 设置目标打印机
   * 对应 Lodop: SET_PRINTER_INDEX(oIndexOrName)
   */
  SET_PRINTER_INDEX(printer: string | number): boolean {
    this.printer = String(printer);
    return true;
  }

  /**
   * 设置打印份数
   * 对应 Lodop: SET_PRINT_COPIES(intCopies)
   */
  SET_PRINT_COPIES(copies: number): void {
    this.copies = Math.max(1, copies);
  }

  /**
   * 直接打印（无预览）
   * 对应 Lodop: PRINT()
   */
  async PRINT(): Promise<boolean> {
    const job = this.buildJob();
    const result = await this.connection.send('PRINT', job);
    if (result?.ok) return true;
    console.warn('PrintCraft: PRINT 失败', result?.error);
    return false;
  }

  /**
   * 打印预览
   * 对应 Lodop: PREVIEW()
   *
   * 将当前任务发送到服务端渲染，打开预览窗口。
   */
  async PREVIEW(): Promise<number> {
    const job = this.buildJob();
    const result = await this.connection.send('PREVIEW', job);

    if (result?.ok && result?.data?.previewId) {
      const previewId = result.data.previewId;
      const port = this.connection.getPort();
      const url = `http://127.0.0.1:${port}/preview/${previewId}`;
      window.open(url, '_blank', 'width=900,height=700,scrollbars=yes');
      return 1;
    }

    console.warn('PrintCraft: PREVIEW 失败', result?.error);
    return 0;
  }

  /**
   * 获取打印机数量
   * 对应 Lodop: GET_PRINTER_COUNT()
   */
  async GET_PRINTER_COUNT(): Promise<number> {
    const result = await this.connection.send('GET_PRINTER_COUNT', {});
    return result?.data?.count ?? 0;
  }

  /**
   * 获取打印机名称
   * 对应 Lodop: GET_PRINTER_NAME(intPrinterIndex)
   */
  async GET_PRINTER_NAME(index: number): Promise<string> {
    const result = await this.connection.send('GET_PRINTER_NAME', { index });
    return result?.data?.name ?? '';
  }

  /**
   * 获取打印机支持的纸张列表
   * PrintCraft 扩展 API
   */
  async GET_PAPER_SIZES(printerName?: string): Promise<Array<{name: string, width_mm: number, height_mm: number}>> {
    const result = await this.connection.send('GET_PAPER_SIZES', { printerName: printerName || '' });
    return result?.data?.paperSizes ?? [];
  }

  /** 构建打印任务对象 */
  private buildJob() {
    return {
      name: this.taskName,
      printer: this.printer,
      copies: this.copies,
      pageSize: this.pageSize,
      elements: this.elements,
    };
  }
}
