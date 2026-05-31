/**
 * WebSocket 连接管理
 *
 * 负责与 PrintCraft 本地服务的 WebSocket 通信。
 * - 自动端口发现（18000-18005）
 * - 断线重连（指数退避）
 * - 请求/响应关联
 * - 心跳保活
 */
export declare class Connection {
    private ws;
    private port;
    private pending;
    private reconnectTimer;
    private reconnectDelay;
    private heartbeatTimer;
    private connected;
    constructor();
    /** 检测 PrintCraft 服务主机（从 script src 或当前页面） */
    private detectHost;
    /** 获取 script 所在端口 */
    private detectPort;
    /** 自动发现端口并连接 */
    private connect;
    /** 尝试连接指定端口 */
    private tryConnect;
    /** 设置消息监听 */
    private setupListeners;
    /** 获取当前连接的端口号 */
    getPort(): number;
    /** 发送命令并等待响应 */
    send(cmd: string, args?: any): Promise<any>;
    /** 安排重连 */
    private scheduleReconnect;
    /** 启动心跳 */
    private startHeartbeat;
    /** 停止心跳 */
    private stopHeartbeat;
}
