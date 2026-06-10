/**
 * SQL.js 数据库初始化
 * 浏览器离线降级模式下的 SQLite 引擎
 */
import initSqlJs, { type Database, type SqlJsStatic } from 'sql.js';
import { SCHEMA_SQL } from './schema';

let SQL: SqlJsStatic | null = null;
let db: Database | null = null;

/** 初始化 SQL.js 数据库（内存模式，刷新后数据丢失） */
export async function initDatabase(): Promise<Database> {
  if (db) return db;

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
