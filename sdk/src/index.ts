/**
 * PrintCraft SDK 入口
 *
 * 暴露全局 LODOP / CLODOP 对象，兼容 Lodop 使用方式。
 * 使用方式: <script src="http://localhost:18000/sdk/printcraft.js"></script>
 */

import { Lodop } from './lodop';

// 创建单例
const lodop = new Lodop();

// 暴露全局变量（Lodop 兼容）
(window as any).LODOP = lodop;
(window as any).CLODOP = lodop;

// Lodop 兼容函数
(window as any).getLodop = () => lodop;

export { Lodop };
export default lodop;
