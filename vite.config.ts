import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],

  // 防止 Vite 清除 Tauri 相关的环境变量
  clearScreen: false,

  server: {
    // Tauri 期望在端口 5173
    port: 5173,
    strictPort: true,
    // 允许 Tauri 访问
    host: true,
  },

  // 环境变量前缀
  envPrefix: ['VITE_', 'TAURI_'],
})
