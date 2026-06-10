import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import path from 'path'

// https://vite.dev/config/
export default defineConfig(({ command }) => {
  // 开发模式（非 Tauri 环境）：为 @tauri-apps/api/core 提供占位模块
  // 实际运行时适配层检测到非 Tauri 环境后不会调用 invoke()
  const isDev = command === 'serve'
  const tauriShimAlias = isDev
    ? {
        '@tauri-apps/api/core': path.resolve(__dirname, './src/adapters/tauriShim.ts'),
      }
    : {}

  return {
    plugins: [
      react(),
      tailwindcss(),
    ],
    resolve: {
      alias: {
        '@': path.resolve(__dirname, './src'),
        ...tauriShimAlias,
      },
    },
    // 优化：排除 SQL.js WASM 的预构建
    optimizeDeps: {
      exclude: ['sql.js'],
    },
  }
})
