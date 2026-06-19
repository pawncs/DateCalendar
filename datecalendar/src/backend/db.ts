/**
 * SQL.js 数据库初始化
 * 浏览器离线降级模式下的 SQLite 引擎
 *
 * sql.js 的模块导出格式与 Vite ESM 不兼容（UMD/IIFE），
 * 改用 CDN script 标签方式加载，使用全局 initSqlJs 函数。
 */
import type { Database, SqlJsStatic } from 'sql.js';
import { SCHEMA_SQL } from './schema';

let SQL: SqlJsStatic | null = null;
let db: Database | null = null;

/** 动态加载 sql.js CDN 脚本 */
function loadSqlJsScript(): Promise<void> {
  return new Promise((resolve, reject) => {
    // 如果已经加载过，直接返回
    if ((window as Record<string, unknown>).initSqlJs) {
      resolve();
      return;
    }
    const script = document.createElement('script');
    script.src = 'https://sql.js.org/dist/sql-wasm.js';
    script.onload = () => resolve();
    script.onerror = () => reject(new Error('Failed to load sql.js from CDN'));
    document.head.appendChild(script);
  });
}

/** 初始化 SQL.js 数据库（内存模式，刷新后数据丢失） */
export async function initDatabase(): Promise<Database> {
  if (db) return db;

  await loadSqlJsScript();

  const initSqlJs = (window as Record<string, unknown>).initSqlJs as (config?: Record<string, unknown>) => Promise<SqlJsStatic>;
  if (!initSqlJs) throw new Error('initSqlJs not found on window');

  SQL = await initSqlJs({
    locateFile: (file: string) => `https://sql.js.org/dist/${file}`,
  });

  db = new SQL.Database();
  db.run(SCHEMA_SQL);
  return db;
}

/** 获取数据库实例（需先调用 initDatabase） */
export function getDatabase(): Database {
  if (!db) throw new Error('Database not initialized. Call initDatabase() first.');
  return db;
}

/** 关闭数据库 */
export function closeDatabase(): void {
  if (db) {
    db.close();
    db = null;
  }
}
