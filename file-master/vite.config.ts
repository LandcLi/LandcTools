import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  // Tauri 终端输出不需要 vite 清屏
  clearScreen: false,
  server: {
    port: 5183,
    strictPort: true,
    watch: {
      // src-tauri 由 cargo 自己 watch，vite 不要重复扫描
      ignored: ['**/src-tauri/**'],
    },
  },
  build: {
    // Tauri 2 在 Windows 上使用 WebView2 / Edge Chromium
    target: 'chrome110',
    minify: 'esbuild',
    sourcemap: false,
    chunkSizeWarningLimit: 1200,
  },
})
