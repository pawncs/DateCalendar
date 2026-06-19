/**
 * 适配层 — 统一入口
 * 根据运行环境自动选择接入方式：
 *   1. Tauri 环境 → IPC (动态加载 tauriBackend)
 *   2. HTTP API 可达 → HTTP (httpBackend)
 *   3. 以上皆不可达 → SQL.js 降级 (sqljsBackend)
 * 
 * 关键：tauriBackend 使用动态 import，避免浏览器环境下
 * 静态导入 @tauri-apps/api/core 导致模块加载失败。
 */
import type { BackendInterface } from './types';
import { httpBackend } from './httpBackend';
import { sqljsBackend } from './sqljsBackend';

export type BackendMode = 'tauri' | 'http' | 'sqljs';

let mode: BackendMode | null = null;
let backend: BackendInterface | null = null;

/** 初始化适配层，检测环境并选择后端 */
export async function initAdapter(): Promise<BackendMode> {
  // 1. 检测 Tauri 环境——动态加载 Tauri IPC
  if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    mode = 'tauri';
    const { createTauriBackend } = await import('./tauriBackend');
    backend = await createTauriBackend();
    console.log('[Adapter] Mode: tauri (IPC)');
    return mode;
  }

  // 2. 检测 HTTP API 是否可达
  try {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), 2000);
    const res = await fetch('http://localhost:9876/api/health', { signal: controller.signal });
    clearTimeout(timeout);
    if (res.ok) {
      mode = 'http';
      backend = httpBackend;
      console.log('[Adapter] Mode: http (HTTP API :9876)');
      return mode;
    }
  } catch {
    // HTTP API 不可达，降级
  }

  // 3. 降级到 SQL.js
  mode = 'sqljs';
  const { initDatabase } = await import('../backend/db');
  await initDatabase();
  backend = sqljsBackend;
  console.log('[Adapter] Mode: sqljs (offline fallback)');
  return mode;
}

/** 获取当前运行模式 */
export function getMode(): BackendMode {
  if (!mode) throw new Error('Adapter not initialized. Call initAdapter() first.');
  return mode;
}

/** 是否处于离线降级模式 */
export function isOffline(): boolean {
  return mode === 'sqljs';
}

/** 获取后端接口（需先调用 initAdapter） */
export function getBackend(): BackendInterface {
  if (!backend) throw new Error('Adapter not initialized. Call initAdapter() first.');
  return backend;
}

/**
 * 统一适配器 — 通过 Proxy 自动路由所有调用
 * 使用方式：adapter.get_all_tasks() / adapter.create_task(input) 等
 */
/** 等待适配层初始化完成的 Promise */
let initPromise: Promise<BackendMode> | null = null;

export function waitForAdapter(): Promise<BackendMode> {
  if (!initPromise) {
    initPromise = initAdapter();
  }
  return initPromise;
}

export const adapter: BackendInterface = new Proxy({} as BackendInterface, {
  get(_target, prop: string) {
    return (...args: unknown[]) => {
      // 如果后端还没初始化，等待初始化完成后再调用
      if (!backend) {
        if (!initPromise) {
          initPromise = initAdapter();
        }
        return initPromise.then(() => {
          const fn = (backend as Record<string, Function>)[prop];
          if (!fn) throw new Error(`Method ${prop} not found on backend`);
          return fn(...args);
        });
      }
      const fn = (backend as Record<string, Function>)[prop];
      if (!fn) throw new Error(`Method ${prop} not found on backend`);
      return fn(...args);
    };
  },
});
