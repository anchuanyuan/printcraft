/**
 * WebSocket 连接管理
 *
 * 负责与 PrintCraft 本地服务的 WebSocket 通信。
 * - 自动端口发现（18000-18005）
 * - 断线重连（指数退避）
 * - 请求/响应关联
 * - 心跳保活
 */

interface PendingRequest {
  resolve: (value: any) => void;
  reject: (reason: any) => void;
}

export class Connection {
  private ws: WebSocket | null = null;
  private port: number = 0;
  private pending = new Map<string, PendingRequest>();
  private reconnectTimer: number | null = null;
  private reconnectDelay: number = 1000;
  private heartbeatTimer: number | null = null;
  private connected: boolean = false;

  constructor() {
    this.connect();
  }

  /** 自动发现端口并连接 */
  private async connect(): Promise<void> {
    for (let port = 18000; port <= 18005; port++) {
      try {
        await this.tryConnect(port);
        this.port = port;
        this.connected = true;
        this.reconnectDelay = 1000;
        this.startHeartbeat();
        console.log(`PrintCraft: 已连接 localhost:${port}`);
        return;
      } catch {
        continue;
      }
    }
    console.warn('PrintCraft: 未找到服务，将在后台重试');
    this.scheduleReconnect();
  }

  /** 尝试连接指定端口 */
  private tryConnect(port: number): Promise<void> {
    return new Promise((resolve, reject) => {
      const ws = new WebSocket(`ws://127.0.0.1:${port}/ws`);
      ws.onopen = () => {
        this.ws = ws;
        this.setupListeners(ws);
        resolve();
      };
      ws.onerror = () => reject(new Error('连接失败'));
      // 超时
      setTimeout(() => {
        if (ws.readyState !== WebSocket.OPEN) {
          ws.close();
          reject(new Error('连接超时'));
        }
      }, 2000);
    });
  }

  /** 设置消息监听 */
  private setupListeners(ws: WebSocket): void {
    ws.onmessage = (event) => {
      try {
        const resp = JSON.parse(event.data);
        const pending = this.pending.get(resp.id);
        if (pending) {
          this.pending.delete(resp.id);
          pending.resolve(resp);
        }
      } catch (e) {
        console.error('PrintCraft: 解析响应失败', e);
      }
    };

    ws.onclose = () => {
      this.connected = false;
      this.stopHeartbeat();
      console.warn('PrintCraft: 连接断开，重连中...');
      this.scheduleReconnect();
    };
  }

  /** 获取当前连接的端口号 */
  getPort(): number {
    return this.port;
  }

  /** 发送命令并等待响应 */
  send(cmd: string, args: any = {}): Promise<any> {
    return new Promise((resolve, reject) => {
      if (!this.connected || !this.ws) {
        reject(new Error('PrintCraft 服务未连接'));
        return;
      }

      const id = crypto.randomUUID();
      this.pending.set(id, { resolve, reject });

      const msg = JSON.stringify({ id, cmd, args });
      this.ws.send(msg);

      // 超时处理
      setTimeout(() => {
        if (this.pending.has(id)) {
          this.pending.delete(id);
          reject(new Error('请求超时'));
        }
      }, 30000);
    });
  }

  /** 安排重连 */
  private scheduleReconnect(): void {
    if (this.reconnectTimer) return;
    const g = typeof window !== 'undefined' ? window : globalThis;
    this.reconnectTimer = (g as any).setTimeout(() => {
      this.reconnectTimer = null;
      this.reconnectDelay = Math.min(this.reconnectDelay * 2, 30000);
      this.connect();
    }, this.reconnectDelay);
  }

  /** 启动心跳 */
  private startHeartbeat(): void {
    const g = typeof window !== 'undefined' ? window : globalThis;
    this.heartbeatTimer = (g as any).setInterval(() => {
      if (this.connected && this.ws?.readyState === WebSocket.OPEN) {
        this.ws.send(JSON.stringify({ id: 'ping', cmd: 'PING', args: {} }));
      }
    }, 30000);
  }

  /** 停止心跳 */
  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      const g = typeof window !== 'undefined' ? window : globalThis;
      (g as any).clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }
}
