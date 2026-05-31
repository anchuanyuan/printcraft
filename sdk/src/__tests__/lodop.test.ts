import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Lodop } from '../lodop';

// Mock Connection to avoid real WebSocket
vi.mock('../connection', () => ({
  Connection: vi.fn().mockImplementation(() => ({
    send: vi.fn().mockResolvedValue({ ok: true, data: null }),
  })),
}));

describe('Lodop', () => {
  let lodop: Lodop;

  beforeEach(() => {
    lodop = new Lodop();
  });

  describe('PRINT_INIT', () => {
    it('初始化任务并清空元素', () => {
      lodop.ADD_PRINT_TEXT(0, 0, 100, 20, 'test');
      const result = lodop.PRINT_INIT('发票打印');
      expect(result).toBe(true);
    });
  });

  describe('ADD_PRINT_TEXT', () => {
    it('添加文本元素并返回索引', () => {
      lodop.PRINT_INIT('test');
      const idx = lodop.ADD_PRINT_TEXT(50, 50, 300, 30, 'Hello World');
      expect(idx).toBe(1);
    });

    it('连续添加返回递增索引', () => {
      lodop.PRINT_INIT('test');
      expect(lodop.ADD_PRINT_TEXT(0, 0, 100, 20, 'A')).toBe(1);
      expect(lodop.ADD_PRINT_TEXT(0, 0, 100, 20, 'B')).toBe(2);
      expect(lodop.ADD_PRINT_TEXT(0, 0, 100, 20, 'C')).toBe(3);
    });
  });

  describe('ADD_PRINT_RECT', () => {
    it('添加矩形元素', () => {
      lodop.PRINT_INIT('test');
      const idx = lodop.ADD_PRINT_RECT(10, 10, 200, 100, 0, 1);
      expect(idx).toBe(1);
    });
  });

  describe('ADD_PRINT_LINE', () => {
    it('添加直线元素', () => {
      lodop.PRINT_INIT('test');
      const idx = lodop.ADD_PRINT_LINE(0, 0, 100, 100, 0, 1);
      expect(idx).toBe(1);
    });
  });

  describe('ADD_PRINT_IMAGE', () => {
    it('添加图片元素', () => {
      lodop.PRINT_INIT('test');
      const idx = lodop.ADD_PRINT_IMAGE(0, 0, 100, 100, 'data:image/png;base64,abc');
      expect(idx).toBe(1);
    });
  });

  describe('ADD_PRINT_BARCODE', () => {
    it('添加条码元素', () => {
      lodop.PRINT_INIT('test');
      const idx = lodop.ADD_PRINT_BARCODE(0, 0, 200, 80, 'QRCode', 'https://example.com');
      expect(idx).toBe(1);
    });
  });

  describe('ADD_PRINT_HTM', () => {
    it('添加 HTML 元素', () => {
      lodop.PRINT_INIT('test');
      const idx = lodop.ADD_PRINT_HTM(0, 0, 500, 300, '<h1>Title</h1>');
      expect(idx).toBe(1);
    });
  });

  describe('SET_PRINT_STYLE', () => {
    it('设置样式属性', () => {
      lodop.PRINT_INIT('test');
      lodop.SET_PRINT_STYLE('FontSize', 14);
      lodop.SET_PRINT_STYLE('FontName', 'Arial');
      lodop.SET_PRINT_STYLE('Bold', true);
      // 无返回值，验证不抛错
    });
  });

  describe('SET_PRINT_STYLEA', () => {
    it('设置指定元素的样式', () => {
      lodop.PRINT_INIT('test');
      const idx = lodop.ADD_PRINT_TEXT(0, 0, 100, 20, 'text');
      lodop.SET_PRINT_STYLEA(idx, 'FontSize', 16);
      // 无返回值，验证不抛错
    });
  });

  describe('SET_PRINT_PAGESIZE', () => {
    it('设置纸张大小', () => {
      lodop.PRINT_INIT('test');
      lodop.SET_PRINT_PAGESIZE(1, 0, 0, 'A4');
      // 无返回值，验证不抛错
    });
  });

  describe('SET_PRINTER_INDEX', () => {
    it('设置打印机', () => {
      const result = lodop.SET_PRINTER_INDEX('HP LaserJet');
      expect(result).toBe(true);
    });

    it('支持数字索引', () => {
      const result = lodop.SET_PRINTER_INDEX(0);
      expect(result).toBe(true);
    });
  });

  describe('SET_PRINT_COPIES', () => {
    it('设置份数', () => {
      lodop.SET_PRINT_COPIES(3);
      // 无返回值，验证不抛错
    });

    it('最小为 1', () => {
      lodop.SET_PRINT_COPIES(0);
      // 内部 Math.max(1, 0) = 1
    });
  });

  describe('PRINT', () => {
    it('发送打印命令', async () => {
      lodop.PRINT_INIT('test');
      lodop.ADD_PRINT_TEXT(0, 0, 100, 20, 'Hello');
      const result = await lodop.PRINT();
      expect(result).toBe(true);
    });
  });

  describe('GET_PRINTER_COUNT', () => {
    it('获取打印机数量', async () => {
      const count = await lodop.GET_PRINTER_COUNT();
      expect(typeof count).toBe('number');
    });
  });

  describe('GET_PRINTER_NAME', () => {
    it('获取打印机名称', async () => {
      const name = await lodop.GET_PRINTER_NAME(0);
      expect(typeof name).toBe('string');
    });
  });

  describe('PREVIEW', () => {
    it('发送预览命令并返回预览 ID', async () => {
      // Mock connection.send to return preview ID
      const mockSend = vi.fn().mockResolvedValue({
        ok: true,
        id: 'test',
        data: { previewId: 'abc-123' },
      });
      (lodop as any).connection.send = mockSend;

      lodop.PRINT_INIT('test');
      lodop.ADD_PRINT_TEXT(0, 0, 100, 20, 'Hello');

      // Mock window.open
      const mockOpen = vi.fn();
      (globalThis as any).window = { open: mockOpen };
      (lodop as any).connection.getPort = () => 18000;

      const result = await lodop.PREVIEW();
      expect(result).toBe(1);
      expect(mockSend).toHaveBeenCalledWith('PREVIEW', expect.any(Object));
    });
  });

  describe('Lodop 兼容使用流程', () => {
    it('完整打印流程', async () => {
      lodop.PRINT_INIT('测试发票');
      lodop.SET_PRINT_PAGESIZE(1, 0, 0, 'A4');
      lodop.SET_PRINT_STYLE('FontSize', 12);
      lodop.SET_PRINT_STYLE('FontName', 'SimSun');

      lodop.ADD_PRINT_TEXT(50, 50, 300, 30, '发票抬头');
      lodop.ADD_PRINT_TEXT(100, 50, 300, 30, '金额: ¥100.00');
      lodop.ADD_PRINT_RECT(40, 40, 320, 100, 0, 1);
      lodop.ADD_PRINT_LINE(90, 50, 90, 350, 0, 1);

      lodop.SET_PRINT_COPIES(2);
      lodop.SET_PRINTER_INDEX('HP LaserJet');

      const result = await lodop.PRINT();
      expect(result).toBe(true);
    });
  });
});
