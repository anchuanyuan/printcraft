import { defineConfig } from 'vite';

// const TARGET = 'http://10.211.55.3:18000';
const TARGET = 'http://localhost:18000';

export default defineConfig({
  root: '.',
  server: {
    port: 3000,
    open: true,
    proxy: {
      // 代理 SDK 和 API 到 Windows VM
      '/sdk': TARGET,
      '/api': TARGET,
      '/preview': TARGET,
      '/ws': {
        target: TARGET.replace('http', 'ws'),
        ws: true,
      },
    },
  },
});
