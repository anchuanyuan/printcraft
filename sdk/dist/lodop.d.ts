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
export declare class Lodop {
    private connection;
    private elements;
    private currentStyle;
    private taskName;
    private printer;
    private copies;
    private pageSize;
    constructor();
    /**
     * 初始化打印任务
     * 对应 Lodop: PRINT_INIT(strTaskName)
     */
    PRINT_INIT(strTaskName: string): boolean;
    /**
     * 添加纯文本打印项
     * 对应 Lodop: ADD_PRINT_TEXT(top, left, width, height, strContent)
     */
    ADD_PRINT_TEXT(top: number, left: number, width: number, height: number, content: string): number;
    /**
     * 添加矩形
     * 对应 Lodop: ADD_PRINT_RECT(top, left, width, height, intLineStyle, intLineWidth)
     */
    ADD_PRINT_RECT(top: number, left: number, width: number, height: number, lineStyle?: number, lineWidth?: number): number;
    /**
     * 添加直线
     * 对应 Lodop: ADD_PRINT_LINE(top1, left1, top2, left2, intLineStyle, intLineWidth)
     */
    ADD_PRINT_LINE(top1: number, left1: number, top2: number, left2: number, lineStyle?: number, lineWidth?: number): number;
    /**
     * 添加图片
     * 对应 Lodop: ADD_PRINT_IMAGE(top, left, width, height, strHtmlContent)
     */
    ADD_PRINT_IMAGE(top: number, left: number, width: number, height: number, src: string): number;
    /**
     * 添加条码
     * 对应 Lodop: ADD_PRINT_BARCODE(top, left, width, height, strBarCodeType, strBarCodeValue)
     */
    ADD_PRINT_BARCODE(top: number, left: number, width: number, height: number, barType: string, code: string): number;
    /**
     * 添加超文本
     * 对应 Lodop: ADD_PRINT_HTM(top, left, width, height, strHtmlContent)
     */
    ADD_PRINT_HTM(top: number, left: number, width: number, height: number, html: string): number;
    /**
     * 添加表格
     * 对应 Lodop: ADD_PRINT_TABLE(top, left, width, height, strHtmlContent)
     */
    ADD_PRINT_TABLE(top: number, left: number, width: number, height: number, html: string): number;
    /**
     * 添加网页地址
     * 对应 Lodop: ADD_PRINT_URL(top, left, width, height, strURL)
     */
    ADD_PRINT_URL(top: number, left: number, width: number, height: number, url: string): number;
    /**
     * 添加椭圆
     * 对应 Lodop: ADD_PRINT_ELLIPSE(top, left, width, height, intLineStyle, intLineWidth)
     */
    ADD_PRINT_ELLIPSE(top: number, left: number, width: number, height: number, lineStyle?: number, lineWidth?: number): number;
    /**
     * 添加形状
     * 对应 Lodop: ADD_PRINT_SHAPE(intShapeType, top, left, width, height, intLineStyle, intLineWidth, strColor)
     */
    ADD_PRINT_SHAPE(shapeType: number, top: number, left: number, width: number, height: number, lineStyle?: number, lineWidth?: number, color?: string): number;
    /**
     * 设置下一个添加元素的样式
     * 对应 Lodop: SET_PRINT_STYLE(strStyleName, varStyleValue)
     */
    SET_PRINT_STYLE(name: string, value: any): void;
    /**
     * 设置指定元素的样式
     * 对应 Lodop: SET_PRINT_STYLEA(varItemNameID, strStyleName, varStyleValue)
     */
    SET_PRINT_STYLEA(itemId: number | string, name: string, value: any): void;
    /**
     * 设置纸张大小
     * 对应 Lodop: SET_PRINT_PAGESIZE(intOrient, PageWidth, PageHeight, strPageName)
     */
    SET_PRINT_PAGESIZE(orient: number, width: number, height: number, pageName: string): void;
    /**
     * 设置目标打印机
     * 对应 Lodop: SET_PRINTER_INDEX(oIndexOrName)
     */
    SET_PRINTER_INDEX(printer: string | number): boolean;
    /**
     * 设置打印份数
     * 对应 Lodop: SET_PRINT_COPIES(intCopies)
     */
    SET_PRINT_COPIES(copies: number): void;
    /**
     * 直接打印（无预览）
     * 对应 Lodop: PRINT()
     */
    PRINT(): Promise<boolean>;
    /**
     * 打印预览
     * 对应 Lodop: PREVIEW()
     *
     * 将当前任务发送到服务端渲染，打开预览窗口。
     */
    PREVIEW(): Promise<number>;
    /**
     * 获取打印机数量
     * 对应 Lodop: GET_PRINTER_COUNT()
     */
    GET_PRINTER_COUNT(): Promise<number>;
    /**
     * 获取打印机名称
     * 对应 Lodop: GET_PRINTER_NAME(intPrinterIndex)
     */
    GET_PRINTER_NAME(index: number): Promise<string>;
    /**
     * 获取打印机支持的纸张列表
     * PrintCraft 扩展 API
     */
    GET_PAPER_SIZES(printerName?: string): Promise<Array<{
        name: string;
        width_mm: number;
        height_mm: number;
    }>>;
    /** 构建打印任务对象 */
    private buildJob;
}
