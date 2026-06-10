/**
 * Tauri API Shim — Vite 开发模式占位模块
 * 在非 Tauri 环境下提供 @tauri-apps/api/core 的占位实现
 * 实际运行时适配层检测环境后不会调用这里的 invoke()
 */
export async function invoke<T>(_cmd: string, _args?: Record<string, unknown>): Promise<T> {
  throw new Error(
    'Tauri IPC not available. The adapter should have detected the browser environment ' +
    'and used HTTP API or SQL.js fallback instead. This error means the adapter ' +
    'initialization may have failed.'
  )
}
