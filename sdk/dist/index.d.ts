/**
 * PrintCraft SDK 入口
 *
 * 暴露全局 LODOP / CLODOP 对象，兼容 Lodop 使用方式。
 * 使用方式: <script src="http://localhost:18000/sdk/printcraft.js"></script>
 */
import { Lodop } from './lodop';
declare const lodop: Lodop;
export { Lodop };
export default lodop;
