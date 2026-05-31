import { defineConfig } from 'vite';

export default defineConfig({
  build: {
    lib: {
      entry: 'src/index.ts',
      name: 'PrintCraft',
      formats: ['es', 'umd'],
      fileName: (format) => `printcraft.${format === 'es' ? 'js' : 'umd.js'}`,
      cssFileName: undefined,
    },
    outDir: 'dist',
    sourcemap: true,
    rollupOptions: {
      output: {
        exports: 'named',
      },
    },
  },
  define: {
    'process.env.NODE_ENV': JSON.stringify('production'),
  },
});
