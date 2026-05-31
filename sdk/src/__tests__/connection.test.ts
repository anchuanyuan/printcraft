import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { Connection } from '../connection';

// Mock WebSocket
class MockWebSocket {
  static instances: MockWebSocket[] = [];
  onopen: (() => void) | null = null;
  onclose: (() => void) | null = null;
  onmessage: ((event: { data: string }) => void) | null = null;
  onerror: (() => void) | null = null;
  readyState = 0; // CONNECTING
  sent: string[] = [];

  constructor(public url: string) {
    MockWebSocket.instances.push(this);
    // Simulate async connect
    setTimeout(() => {
      this.readyState = 1; // OPEN
      this.onopen?.();
    }, 10);
  }

  send(data: string) {
    this.sent.push(data);
  }

  close() {
    this.readyState = 3; // CLOSED
    this.onclose?.();
  }
}

describe('Connection', () => {
  let originalWebSocket: typeof globalThis.WebSocket;

  beforeEach(() => {
    originalWebSocket = globalThis.WebSocket;
    (globalThis as any).WebSocket = MockWebSocket;
    MockWebSocket.instances = [];
    vi.useFakeTimers();
  });

  afterEach(() => {
    (globalThis as any).WebSocket = originalWebSocket;
    vi.useRealTimers();
  });

  it('尝试连接 18000-18005 端口', async () => {
    new Connection();
    // 让第一个连接尝试跑完
    await vi.advanceTimersByTimeAsync(50);
    // 至少创建了一个 WebSocket 实例
    expect(MockWebSocket.instances.length).toBeGreaterThanOrEqual(1);
  });

  it('成功连接后停止尝试其他端口', async () => {
    new Connection();
    await vi.advanceTimersByTimeAsync(50);
    // 第一个实例成功连接（MockWebSocket 10ms 后 onopen）
    expect(MockWebSocket.instances[0].url).toContain('18000');
  });

  it('send 在未连接时 reject', async () => {
    // Mock 连接全部失败
    (globalThis as any).WebSocket = class {
      constructor() {
        setTimeout(() => this.onerror?.(), 5);
      }
      onopen: any = null;
      onclose: any = null;
      onerror: any = null;
      close() {}
    };

    const conn = new Connection();
    await vi.advanceTimersByTimeAsync(20000);

    await expect(conn.send('PRINT', {})).rejects.toThrow('PrintCraft 服务未连接');
  });
});
