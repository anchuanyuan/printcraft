/**
 * 打印样式定义
 *
 * 对应 Lodop 的 SET_PRINT_STYLE 属性名。
 */
export interface PrintStyle {
    FontName?: string;
    FontSize?: number;
    Bold?: boolean | number;
    Italic?: boolean | number;
    UnderLine?: boolean | number;
    Alignment?: number;
    FontColor?: string;
    LineSpacing?: number;
    Angle?: number;
    [key: string]: any;
}
