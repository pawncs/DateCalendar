/**
 * 浏览器后端 — 日程 CRUD + 状态同步 + 冲突检测
 * 接口签名与 Tauri commands 一致
 */
import { getDatabase } from './db';
import { generateId, now, rowToSchedule } from './utils';
import type { Schedule } from '@/types/schedule';

// ==================== 日程查询 ====================

export async function get_all_schedules(): Promise<Schedule[]> {
  const db = getDatabase();
  const results = db.exec('SELECT * FROM schedules ORDER BY start_time');
  if (results.length === 0) return [];
  return results[0].values.map((row, _i) => {
    const obj: Record<string, unknown> = {};
    results[0].columns.forEach((col, ci) => { obj[col] = row[ci]; });
    return rowToSchedule(obj) as unknown as Schedule;
  });
}

export async function get_schedule(id: string): Promise<Schedule | null> {
  const db = getDatabase();
  const stmt = db.prepare('SELECT * FROM schedules WHERE id = ?');
  stmt.bind([id]);
  if (stmt.step()) {
    const row = stmt.getAsObject();
    stmt.free();
    return rowToSchedule(row) as unknown as Schedule;
  }
  stmt.free();
  return null;
}

export async function get_schedules_in_range(start_date: string, end_date: string): Promise<Schedule[]> {
  const db = getDatabase();
  const results = db.exec(
    'SELECT * FROM schedules WHERE start_time >= ? AND end_time <= ? ORDER BY start_time',
    [start_date, end_date]
  );
  if (results.length === 0) return [];
  return results[0].values.map((row, _i) => {
    const obj: Record<string, unknown> = {};
    results[0].columns.forEach((col, ci) => { obj[col] = row[ci]; });
    return rowToSchedule(obj) as unknown as Schedule;
  });
}

export async function get_day_schedules(date: string): Promise<Schedule[]> {
  const db = getDatabase();
  // 当天 fixed + todo_day；当周 todo_week；排除 cancelled
  const dayStart = `${date}T00:00:00`;
  const dayEnd = `${date}T23:59:59`;

  // 计算周一
  const d = new Date(date);
  const day = d.getDay();
  const monday = new Date(d);
  monday.setDate(d.getDate() - day + (day === 0 ? -6 : 1));
  const weekStart = monday.toISOString().split('T')[0] + 'T00:00:00';

  // 计算周日
  const sunday = new Date(monday);
  sunday.setDate(monday.getDate() + 6);
  const weekEnd = sunday.toISOString().split('T')[0] + 'T23:59:59';

  const results = db.exec(`
    SELECT * FROM schedules
    WHERE status != 'cancelled'
      AND (
        (schedule_type = 'fixed' AND start_time >= ? AND end_time <= ?)
        OR (schedule_type = 'todo_day' AND start_time >= ? AND end_time <= ?)
        OR (schedule_type = 'todo_week' AND start_time >= ? AND end_time <= ?)
      )
    ORDER BY start_time
  `, [dayStart, dayEnd, dayStart, dayEnd, weekStart, weekEnd]);

  if (results.length === 0) return [];
  return results[0].values.map((row, _i) => {
    const obj: Record<string, unknown> = {};
    results[0].columns.forEach((col, ci) => { obj[col] = row[ci]; });
    return rowToSchedule(obj) as unknown as Schedule;
  });
}

export async function get_week_schedules(week_start: string, week_end: string): Promise<Schedule[]> {
  return get_schedules_in_range(week_start, week_end);
}

export async function get_schedules_by_task(task_id: string): Promise<Schedule[]> {
  const db = getDatabase();
  const results = db.exec('SELECT * FROM schedules WHERE task_id = ? ORDER BY start_time', [task_id]);
  if (results.length === 0) return [];
  return results[0].values.map((row, _i) => {
    const obj: Record<string, unknown> = {};
    results[0].columns.forEach((col, ci) => { obj[col] = row[ci]; });
    return rowToSchedule(obj) as unknown as Schedule;
  });
}

// ==================== 日程 CRUD ====================

export async function create_schedule(
  task_id: string, title: string, start_time: string, end_time: string,
  is_all_day?: boolean, schedule_type?: string, color?: string,
): Promise<Schedule> {
  const db = getDatabase();
  const id = generateId();
  const timestamp = now();
  db.run(
    `INSERT INTO schedules (id, task_id, title, start_time, end_time, is_all_day, schedule_type, status, color, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, 'pending', ?, ?, ?)`,
    [id, task_id, title, start_time, end_time, is_all_day ? 1 : 0, schedule_type ?? 'fixed', color ?? '', timestamp, timestamp]
  );
  return (await get_schedule(id))!;
}

export async function update_schedule(
  id: string,
  title?: string, start_time?: string, end_time?: string,
  is_all_day?: boolean, schedule_type?: string, status?: string,
  color?: string, task_id?: string,
): Promise<Schedule> {
  const db = getDatabase();
  const sets: string[] = [];
  const params: unknown[] = [];

  if (title !== undefined) { sets.push('title = ?'); params.push(title); }
  if (start_time !== undefined) { sets.push('start_time = ?'); params.push(start_time); }
  if (end_time !== undefined) { sets.push('end_time = ?'); params.push(end_time); }
  if (is_all_day !== undefined) { sets.push('is_all_day = ?'); params.push(is_all_day ? 1 : 0); }
  if (schedule_type !== undefined) { sets.push('schedule_type = ?'); params.push(schedule_type); }
  if (status !== undefined) { sets.push('status = ?'); params.push(status); }
  if (color !== undefined) { sets.push('color = ?'); params.push(color); }
  if (task_id !== undefined) { sets.push('task_id = ?'); params.push(task_id); }

  if (sets.length > 0) {
    sets.push('updated_at = ?'); params.push(now());
    params.push(id);
    db.run(`UPDATE schedules SET ${sets.join(', ')} WHERE id = ?`, params);
  }

  return (await get_schedule(id))!;
}

export async function delete_schedule(id: string): Promise<void> {
  const db = getDatabase();
  db.run('DELETE FROM schedules WHERE id = ?', [id]);
}

// ==================== 状态同步 ====================

export async function update_schedule_status(schedule_id: string, new_status: string): Promise<void> {
  const db = getDatabase();
  const timestamp = now();

  // 1. 获取日程关联的 task_id
  const r = db.exec('SELECT task_id FROM schedules WHERE id = ?', [schedule_id]);
  if (r.length === 0 || r[0].values.length === 0) return;
  const task_id = r[0].values[0][0] as string;

  // 2. 更新日程状态
  db.run('UPDATE schedules SET status = ?, updated_at = ? WHERE id = ?', [new_status, timestamp, schedule_id]);

  // 3. 同步任务状态
  const taskStatus = new_status === 'completed' ? 'completed' :
                     new_status === 'cancelled' ? 'cancelled' : 'in_progress';
  if (taskStatus === 'completed') {
    db.run('UPDATE tasks SET status = ?, completed_at = ?, updated_at = ? WHERE id = ? AND status != ?',
      [taskStatus, timestamp, timestamp, task_id, taskStatus]);
  } else {
    db.run('UPDATE tasks SET status = ?, updated_at = ? WHERE id = ? AND status != ?',
      [taskStatus, timestamp, task_id, taskStatus]);
  }

  // 4. 同步同任务下其他日程状态
  db.run('UPDATE schedules SET status = ?, updated_at = ? WHERE task_id = ? AND id != ? AND status != ?',
    [new_status, timestamp, task_id, schedule_id, new_status]);
}

// ==================== 冲突检测 ====================

export async function check_conflicts(
  start_time: string, end_time: string, exclude_id?: string | null
): Promise<Schedule[]> {
  const db = getDatabase();
  let sql = `
    SELECT * FROM schedules
    WHERE status != 'cancelled'
      AND schedule_type = 'fixed'
      AND start_time < ? AND end_time > ?
  `;
  const params: unknown[] = [end_time, start_time];

  if (exclude_id) {
    sql += ' AND id != ?';
    params.push(exclude_id);
  }

  const results = db.exec(sql, params);
  if (results.length === 0) return [];
  return results[0].values.map((row, _i) => {
    const obj: Record<string, unknown> = {};
    results[0].columns.forEach((col, ci) => { obj[col] = row[ci]; });
    return rowToSchedule(obj) as unknown as Schedule;
  });
}
