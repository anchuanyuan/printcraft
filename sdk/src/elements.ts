/**
 * 打印元素类型定义
 */

export interface ElementPosition {
  top: number;
  left: number;
  width: number;
  height: number;
}

export type BarcodeType = string;

export interface PrintElement {
  index: number;
  type: 'text' | 'rect' | 'line' | 'image' | 'barcode' | 'htm' | 'table' | 'url';
  position: ElementPosition;
  style: Record<string, any>;
  [key: string]: any;
}
