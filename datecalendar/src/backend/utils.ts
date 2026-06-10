/**
 * 浏览器后端工具函数
 * 对应 Rust 端的 UUID 生成和 chrono::Utc::now()
 */
export function generateId(): string {
  return crypto.randomUUID();
}

export function now(): string {
  return new Date().toISOString();
}

/** 将 SQL.js 返回的 INTEGER 字段转为 boolean */
export function intToBool(val: unknown): boolean {
  return val === 1 || val === true;
}

/** 将 SQL.js 返回的行转为 Task（处理 is_milestone boolean） */
export function rowToTask(row: Record<string, unknown>): Record<string, unknown> {
  return {
    ...row,
    is_milestone: intToBool(row.is_milestone),
    priority: Number(row.priority),
    sort_order: Number(row.sort_order),
  };
}

/** 将 SQL.js 返回的行转为 Schedule（处理 is_all_day boolean） */
export function rowToSchedule(row: Record<string, unknown>): Record<string, unknown> {
  return {
    ...row,
    is_all_day: intToBool(row.is_all_day),
  };
}
