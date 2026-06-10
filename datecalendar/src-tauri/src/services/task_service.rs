use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;
use chrono::Utc;

use crate::db::models::{Task, NewTask, MilestoneRisk, Note};

/// 任务服务 — 封装所有任务相关的业务逻辑
///
/// 设计要点：
/// - 使用连接池而非单连接，避免 SQLite 锁竞争
/// - 所有时间戳用 ISO 8601 格式存储
/// - UUID v4 作为主键，避免自增 ID 在分布式/导出场景的问题
pub struct TaskService {
    pool: Pool<SqliteConnectionManager>,
}

impl TaskService {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    // ==================== 任务 CRUD ====================

    /// 获取所有任务，以平铺列表返回（前端负责构建树）
    pub fn get_all_tasks(&self) -> Result<Vec<Task>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, parent_id, title, description, status, priority,
                        sort_order, color, is_milestone, created_at, updated_at, completed_at
                 FROM tasks ORDER BY sort_order, created_at"
            )
            .map_err(|e| e.to_string())?;

        let tasks = stmt
            .query_map([], |row| {
                Ok(Task {
                    id: row.get(0)?,
                    parent_id: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    status: row.get(4)?,
                    priority: row.get(5)?,
                    sort_order: row.get(6)?,
                    color: row.get(7)?,
                    is_milestone: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                    completed_at: row.get(11)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(tasks)
    }

    /// 获取单个任务
    pub fn get_task(&self, id: &str) -> Result<Option<Task>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, parent_id, title, description, status, priority,
                        sort_order, color, is_milestone, created_at, updated_at, completed_at
                 FROM tasks WHERE id = ?1"
            )
            .map_err(|e| e.to_string())?;

        let task = stmt
            .query_row(params![id], |row| {
                Ok(Task {
                    id: row.get(0)?,
                    parent_id: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    status: row.get(4)?,
                    priority: row.get(5)?,
                    sort_order: row.get(6)?,
                    color: row.get(7)?,
                    is_milestone: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                    completed_at: row.get(11)?,
                })
            })
            .optional()
            .map_err(|e| e.to_string())?;

        Ok(task)
    }

    /// 创建新任务
    pub fn create_task(&self, input: NewTask) -> Result<Task, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let title = input.title;
        let description = input.description.unwrap_or_default();
        let priority = input.priority.unwrap_or(0);
        let color = input.color.unwrap_or_default();
        let is_milestone = input.is_milestone.unwrap_or(false);
        let parent_id = input.parent_id;

        // 计算同级最大的 sort_order
        let max_order: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM tasks WHERE parent_id IS ?1",
                params![parent_id],
                |row| row.get(0),
            )
            .unwrap_or(-1);

        conn.execute(
            "INSERT INTO tasks (id, parent_id, title, description, status, priority, sort_order, color, is_milestone, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'pending', ?5, ?6, ?7, ?8, ?9, ?10)",
            params![id, parent_id, title, description, priority, max_order + 1, color, is_milestone as i32, now, now],
        )
        .map_err(|e| e.to_string())?;

        self.get_task(&id).map(|t| t.unwrap())
    }

    /// 更新任务
    pub fn update_task(&self, id: &str, title: Option<&str>, description: Option<&str>,
        status: Option<&str>, priority: Option<i32>, color: Option<&str>,
        is_milestone: Option<bool>, parent_id: Option<Option<&str>>,
        sort_order: Option<i32>) -> Result<Task, String>
    {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();

        // 构建动态 SQL
        let mut sets: Vec<String> = Vec::new();
        let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(t) = title {
            sets.push(format!("title = ?{}", sets.len() + 1));
            values.push(Box::new(t.to_string()));
        }
        if let Some(d) = description {
            sets.push(format!("description = ?{}", sets.len() + 1));
            values.push(Box::new(d.to_string()));
        }
        if let Some(s) = status {
            sets.push(format!("status = ?{}", sets.len() + 1));
            values.push(Box::new(s.to_string()));
            // 如果状态变为 completed，记录完成时间
            if s == "completed" {
                sets.push(format!("completed_at = ?{}", sets.len() + 1));
                values.push(Box::new(now.clone()));
            }
        }
        if let Some(p) = priority {
            sets.push(format!("priority = ?{}", sets.len() + 1));
            values.push(Box::new(p));
        }
        if let Some(c) = color {
            sets.push(format!("color = ?{}", sets.len() + 1));
            values.push(Box::new(c.to_string()));
        }
        if let Some(m) = is_milestone {
            sets.push(format!("is_milestone = ?{}", sets.len() + 1));
            values.push(Box::new(m as i32));
        }
        if let Some(pid) = parent_id {
            sets.push(format!("parent_id = ?{}", sets.len() + 1));
            values.push(Box::new(pid.map(|s| s.to_string())));
        }
        if let Some(so) = sort_order {
            sets.push(format!("sort_order = ?{}", sets.len() + 1));
            values.push(Box::new(so));
        }

        // 总是更新 updated_at
        sets.push(format!("updated_at = ?{}", sets.len() + 1));
        values.push(Box::new(now));

        if sets.is_empty() {
            return self.get_task(id).map(|t| t.unwrap());
        }

        let sql = format!(
            "UPDATE tasks SET {} WHERE id = ?{}",
            sets.join(", "),
            sets.len() + 1
        );
        values.push(Box::new(id.to_string()));

        // 转换参数为引用列表
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
        conn.execute(&sql, params_refs.as_slice())
            .map_err(|e| e.to_string())?;

        self.get_task(id).map(|t| t.unwrap())
    }

    /// 删除任务（及其所有子任务，通过 ON DELETE CASCADE 或手动递归）
    pub fn delete_task(&self, id: &str) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        // 递归删除所有子孙任务
        conn.execute(
            "WITH RECURSIVE descendants AS (
                SELECT id FROM tasks WHERE id = ?1
                UNION ALL
                SELECT t.id FROM tasks t JOIN descendants d ON t.parent_id = d.id
             )
             DELETE FROM tasks WHERE id IN (SELECT id FROM descendants)",
            params![id],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    // ==================== 里程碑风险 ====================

    /// 获取任务的所有风险
    pub fn get_risks(&self, task_id: &str) -> Result<Vec<MilestoneRisk>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, task_id, risk_desc, probability, mitigation, created_at, updated_at
                 FROM milestone_risks WHERE task_id = ?1 ORDER BY created_at"
            )
            .map_err(|e| e.to_string())?;

        let risks = stmt
            .query_map(params![task_id], |row| {
                Ok(MilestoneRisk {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    risk_desc: row.get(2)?,
                    probability: row.get(3)?,
                    mitigation: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(risks)
    }

    /// 添加风险备注
    pub fn add_risk(&self, task_id: &str, risk_desc: &str, probability: &str, mitigation: &str) -> Result<MilestoneRisk, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO milestone_risks (id, task_id, risk_desc, probability, mitigation, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, task_id, risk_desc, probability, mitigation, now, now.clone()],
        )
        .map_err(|e| e.to_string())?;

        Ok(MilestoneRisk {
            id,
            task_id: task_id.to_string(),
            risk_desc: risk_desc.to_string(),
            probability: probability.to_string(),
            mitigation: mitigation.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// 删除风险备注
    pub fn delete_risk(&self, risk_id: &str) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM milestone_risks WHERE id = ?1", params![risk_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ==================== 笔记 ====================

    /// 获取任务的所有笔记
    pub fn get_notes(&self, task_id: &str) -> Result<Vec<Note>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, task_id, title, content, created_at, updated_at
                 FROM notes WHERE task_id = ?1 ORDER BY updated_at DESC"
            )
            .map_err(|e| e.to_string())?;

        let notes = stmt
            .query_map(params![task_id], |row| {
                Ok(Note {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    title: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(notes)
    }

    /// 创建或更新笔记（upsert 模式，方便自动保存）
    pub fn save_note(&self, task_id: &str, note_id: Option<&str>, title: &str, content: &str) -> Result<Note, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();

        if let Some(nid) = note_id {
            // 更新已有笔记
            conn.execute(
                "UPDATE notes SET title = ?1, content = ?2, updated_at = ?3 WHERE id = ?4",
                params![title, content, now, nid],
            )
            .map_err(|e| e.to_string())?;

            Ok(Note {
                id: nid.to_string(),
                task_id: task_id.to_string(),
                title: title.to_string(),
                content: content.to_string(),
                created_at: String::new(), // 不读取旧值
                updated_at: now,
            })
        } else {
            // 创建新笔记
            let id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO notes (id, task_id, title, content, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, task_id, title, content, now, now],
            )
            .map_err(|e| e.to_string())?;

            Ok(Note {
                id,
                task_id: task_id.to_string(),
                title: title.to_string(),
                content: content.to_string(),
                created_at: now.clone(),
                updated_at: now,
            })
        }
    }

    /// 删除笔记
    pub fn delete_note(&self, note_id: &str) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM notes WHERE id = ?1", params![note_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ==================== 搜索 ====================

    /// 搜索任务（标题和描述）
    pub fn search_tasks(&self, query: &str) -> Result<Vec<Task>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let pattern = format!("%{}%", query);
        let mut stmt = conn
            .prepare(
                "SELECT id, parent_id, title, description, status, priority,
                        sort_order, color, is_milestone, created_at, updated_at, completed_at
                 FROM tasks WHERE title LIKE ?1 OR description LIKE ?1
                 ORDER BY priority DESC, created_at DESC"
            )
            .map_err(|e| e.to_string())?;

        let tasks = stmt
            .query_map(params![pattern], |row| {
                Ok(Task {
                    id: row.get(0)?,
                    parent_id: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    status: row.get(4)?,
                    priority: row.get(5)?,
                    sort_order: row.get(6)?,
                    color: row.get(7)?,
                    is_milestone: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                    completed_at: row.get(11)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(tasks)
    }
}
