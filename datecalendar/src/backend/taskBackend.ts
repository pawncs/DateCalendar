/**
 * 浏览器后端 — 任务 CRUD + 风险 + 笔记
 * 接口签名与 Tauri commands 一致，底层使用 SQL.js 执行 SQL
 */
import { getDatabase } from './db';
import { generateId, now, rowToTask } from './utils';
import type { Task, CreateTaskInput, MilestoneRisk, Note } from '@/types/task';

// ==================== 任务 CRUD ====================

export async function get_all_tasks(): Promise<Task[]> {
  const db = getDatabase();
  const results = db.exec(`
    WITH RECURSIVE task_tree AS (
      SELECT *, 0 AS depth FROM tasks WHERE parent_id IS NULL
      UNION ALL
      SELECT t.*, tt.depth + 1
      FROM tasks t JOIN task_tree tt ON t.parent_id = tt.id
    )
    SELECT * FROM task_tree ORDER BY depth, sort_order
  `);
  if (results.length === 0) return [];
  return results[0].values.map((row, _i) => {
    const obj: Record<string, unknown> = {};
    results[0].columns.forEach((col, ci) => { obj[col] = row[ci]; });
    return rowToTask(obj) as unknown as Task;
  });
}

export async function get_task(id: string): Promise<Task | null> {
  const db = getDatabase();
  const stmt = db.prepare('SELECT * FROM tasks WHERE id = ?');
  stmt.bind([id]);
  if (stmt.step()) {
    const row = stmt.getAsObject();
    stmt.free();
    return rowToTask(row) as unknown as Task;
  }
  stmt.free();
  return null;
}

export async function create_task(input: CreateTaskInput): Promise<Task> {
  const db = getDatabase();
  const id = generateId();
  const timestamp = now();

  // 计算同级最大 sort_order + 1
  let sortOrder = 0;
  if (input.parent_id) {
    const r = db.exec('SELECT COALESCE(MAX(sort_order), -1) + 1 AS n FROM tasks WHERE parent_id = ?', [input.parent_id]);
    sortOrder = Number(r[0]?.values[0]?.[0] ?? 0);
  } else {
    const r = db.exec('SELECT COALESCE(MAX(sort_order), -1) + 1 AS n FROM tasks WHERE parent_id IS NULL');
    sortOrder = Number(r[0]?.values[0]?.[0] ?? 0);
  }

  db.run(
    `INSERT INTO tasks (id, parent_id, title, description, priority, sort_order, color, is_milestone, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
    [id, input.parent_id ?? null, input.title, input.description ?? '',
     input.priority ?? 0, sortOrder, input.color ?? '',
     input.is_milestone ? 1 : 0, timestamp, timestamp]
  );

  return (await get_task(id))!;
}

export async function update_task(
  id: string,
  title?: string,
  description?: string,
  status?: string,
  priority?: number,
  color?: string,
  is_milestone?: boolean,
  parent_id?: string | null,
  sort_order?: number,
): Promise<Task> {
  const db = getDatabase();
  const sets: string[] = [];
  const params: unknown[] = [];

  if (title !== undefined) { sets.push('title = ?'); params.push(title); }
  if (description !== undefined) { sets.push('description = ?'); params.push(description); }
  if (status !== undefined) {
    sets.push('status = ?'); params.push(status);
    if (status === 'completed') {
      sets.push('completed_at = ?'); params.push(now());
    } else {
      sets.push('completed_at = NULL');
    }
  }
  if (priority !== undefined) { sets.push('priority = ?'); params.push(priority); }
  if (color !== undefined) { sets.push('color = ?'); params.push(color); }
  if (is_milestone !== undefined) { sets.push('is_milestone = ?'); params.push(is_milestone ? 1 : 0); }
  if (parent_id !== undefined) { sets.push('parent_id = ?'); params.push(parent_id); }
  if (sort_order !== undefined) { sets.push('sort_order = ?'); params.push(sort_order); }

  if (sets.length > 0) {
    sets.push('updated_at = ?'); params.push(now());
    params.push(id);
    db.run(`UPDATE tasks SET ${sets.join(', ')} WHERE id = ?`, params);
  }

  return (await get_task(id))!;
}

export async function delete_task(id: string): Promise<void> {
  const db = getDatabase();
  // 递归 CTE 级联删除子孙任务
  db.run(`
    WITH RECURSIVE descendants AS (
      SELECT id FROM tasks WHERE id = ?
      UNION ALL
      SELECT t.id FROM tasks t JOIN descendants d ON t.parent_id = d.id
    )
    DELETE FROM tasks WHERE id IN (SELECT id FROM descendants)
  `, [id]);
}

// ==================== 搜索 ====================

export async function search_tasks(query: string): Promise<Task[]> {
  const db = getDatabase();
  const results = db.exec(
    'SELECT * FROM tasks WHERE title LIKE ? OR description LIKE ?',
    [`%${query}%`, `%${query}%`]
  );
  if (results.length === 0) return [];
  return results[0].values.map((row, _i) => {
    const obj: Record<string, unknown> = {};
    results[0].columns.forEach((col, ci) => { obj[col] = row[ci]; });
    return rowToTask(obj) as unknown as Task;
  });
}

// ==================== 风险 ====================

export async function get_risks(task_id: string): Promise<MilestoneRisk[]> {
  const db = getDatabase();
  const results = db.exec('SELECT * FROM milestone_risks WHERE task_id = ?', [task_id]);
  if (results.length === 0) return [];
  return results[0].values.map((row, _i) => {
    const obj: Record<string, unknown> = {};
    results[0].columns.forEach((col, ci) => { obj[col] = row[ci]; });
    return obj as unknown as MilestoneRisk;
  });
}

export async function add_risk(
  task_id: string, risk_desc: string, probability?: string, mitigation?: string
): Promise<MilestoneRisk> {
  const db = getDatabase();
  const id = generateId();
  const timestamp = now();
  db.run(
    `INSERT INTO milestone_risks (id, task_id, risk_desc, probability, mitigation, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?)`,
    [id, task_id, risk_desc, probability ?? 'medium', mitigation ?? '', timestamp, timestamp]
  );
  const r = db.exec('SELECT * FROM milestone_risks WHERE id = ?', [id]);
  const obj: Record<string, unknown> = {};
  r[0].columns.forEach((col, ci) => { obj[col] = r[0].values[0][ci]; });
  return obj as unknown as MilestoneRisk;
}

export async function delete_risk(risk_id: string): Promise<void> {
  const db = getDatabase();
  db.run('DELETE FROM milestone_risks WHERE id = ?', [risk_id]);
}

// ==================== 笔记 ====================

export async function get_notes(task_id: string): Promise<Note[]> {
  const db = getDatabase();
  const results = db.exec('SELECT * FROM notes WHERE task_id = ?', [task_id]);
  if (results.length === 0) return [];
  return results[0].values.map((row, _i) => {
    const obj: Record<string, unknown> = {};
    results[0].columns.forEach((col, ci) => { obj[col] = row[ci]; });
    return obj as unknown as Note;
  });
}

export async function save_note(
  task_id: string, note_id: string | null, title: string, content: string
): Promise<Note> {
  const db = getDatabase();
  const timestamp = now();

  if (note_id) {
    db.run('UPDATE notes SET title = ?, content = ?, updated_at = ? WHERE id = ?',
      [title, content, timestamp, note_id]);
  } else {
    note_id = generateId();
    db.run(
      `INSERT INTO notes (id, task_id, title, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)`,
      [note_id, task_id, title, content, timestamp, timestamp]
    );
  }

  const r = db.exec('SELECT * FROM notes WHERE id = ?', [note_id]);
  const obj: Record<string, unknown> = {};
  r[0].columns.forEach((col, ci) => { obj[col] = r[0].values[0][ci]; });
  return obj as unknown as Note;
}

export async function delete_note(note_id: string): Promise<void> {
  const db = getDatabase();
  db.run('DELETE FROM notes WHERE id = ?', [note_id]);
}

// ==================== 排序与批量 ====================

export async function reorder_task(
  task_id: string, new_parent_id: string | null, new_sort_order: number
): Promise<void> {
  const db = getDatabase();

  // 循环引用检测：新父节点不能是被移动任务的子孙
  if (new_parent_id) {
    const results = db.exec(`
      WITH RECURSIVE ancestors AS (
        SELECT id FROM tasks WHERE id = ?
        UNION ALL
        SELECT t.parent_id FROM tasks t JOIN ancestors a ON t.id = a.id WHERE t.parent_id IS NOT NULL
      )
      SELECT id FROM ancestors WHERE id = ?
    `, [new_parent_id, task_id]);
    if (results.length > 0 && results[0].values.length > 0) {
      throw new Error('不能将任务移动到其子孙节点下');
    }
  }

  db.run('UPDATE tasks SET parent_id = ?, sort_order = ?, updated_at = ? WHERE id = ?',
    [new_parent_id, new_sort_order, now(), task_id]);
}

export async function batch_update_tasks(ids: string[], status: string): Promise<void> {
  const db = getDatabase();
  const placeholders = ids.map(() => '?').join(',');
  if (status === 'completed') {
    db.run(`UPDATE tasks SET status = ?, completed_at = ?, updated_at = ? WHERE id IN (${placeholders})`,
      [status, now(), now(), ...ids]);
  } else {
    db.run(`UPDATE tasks SET status = ?, updated_at = ? WHERE id IN (${placeholders})`,
      [status, now(), ...ids]);
  }
}

export async function batch_delete_tasks(ids: string[]): Promise<void> {
  for (const id of ids) {
    await delete_task(id);
  }
}

export async function batch_move_tasks(ids: string[], new_parent_id: string | null): Promise<void> {
  const db = getDatabase();

  // 计算目标父节点下当前最大 sort_order
  let maxOrder = -1;
  if (new_parent_id) {
    const r = db.exec('SELECT COALESCE(MAX(sort_order), -1) FROM tasks WHERE parent_id = ?', [new_parent_id]);
    maxOrder = Number(r[0]?.values[0]?.[0] ?? -1);
  } else {
    const r = db.exec('SELECT COALESCE(MAX(sort_order), -1) FROM tasks WHERE parent_id IS NULL');
    maxOrder = Number(r[0]?.values[0]?.[0] ?? -1);
  }

  for (const id of ids) {
    maxOrder++;
    db.run('UPDATE tasks SET parent_id = ?, sort_order = ?, updated_at = ? WHERE id = ?',
      [new_parent_id, maxOrder, now(), id]);
  }
}
