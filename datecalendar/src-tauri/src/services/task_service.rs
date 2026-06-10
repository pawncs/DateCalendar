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

    // ==================== 排序 ====================

    /// 重排任务：移动任务到新的父节点和排序位置
    ///
    /// 包含循环引用检测：使用递归 CTE 检查目标父节点是否是被移动任务的子孙节点
    pub fn reorder_task(&self, task_id: &str, new_parent_id: Option<&str>, new_sort_order: i32) -> Result<(), String> {
        // 1. 防循环检测：如果目标父节点存在，检查 task_id 是否是 pid 的祖先
        // （即：新父节点是否是被移动任务的子孙）
        if let Some(pid) = new_parent_id {
            let conn = self.pool.get().map_err(|e| e.to_string())?;
            let is_ancestor: bool = conn
                .query_row(
                    "WITH RECURSIVE ancestors AS (
                        SELECT id, parent_id FROM tasks WHERE id = ?1
                        UNION ALL
                        SELECT t.id, t.parent_id FROM tasks t JOIN ancestors a ON t.id = a.parent_id
                     )
                     SELECT EXISTS(SELECT 1 FROM ancestors WHERE id = ?2)",
                    params![pid, task_id],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;

            if is_ancestor {
                return Err("Cannot move a task into its own descendant".to_string());
            }
        }

        // 2. 在同一事务中更新（使用独立连接避免借用冲突）
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

        // 更新目标任务的 parent_id 和 sort_order
        tx.execute(
            "UPDATE tasks SET parent_id = ?1, sort_order = ?2, updated_at = ?3 WHERE id = ?4",
            params![new_parent_id, new_sort_order, chrono::Utc::now().to_rfc3339(), task_id],
        )
        .map_err(|e| e.to_string())?;

        // 3. 重新编号目标父节点下的所有兄弟（消除间隙）
        let brothers = {
            let brothers_sql = if new_parent_id.is_some() {
                "SELECT id FROM tasks WHERE parent_id = ?1 AND id != ?2 ORDER BY sort_order, created_at"
            } else {
                "SELECT id FROM tasks WHERE parent_id IS NULL AND id != ?1 ORDER BY sort_order, created_at"
            };

            let mut stmt = tx.prepare(brothers_sql).map_err(|e| e.to_string())?;
            let result: Vec<String> = if let Some(pid) = new_parent_id {
                stmt.query_map(params![pid, task_id], |row| row.get::<_, String>(0))
                    .map_err(|e| e.to_string())?
                    .filter_map(|r| r.ok())
                    .collect()
            } else {
                stmt.query_map(params![task_id], |row| row.get::<_, String>(0))
                    .map_err(|e| e.to_string())?
                    .filter_map(|r| r.ok())
                    .collect()
            };
            result
        }; // stmt 在此处 drop，释放 tx 的借用

        // 按 sort_order 为兄弟节点重新编号
        for (i, bro_id) in brothers.iter().enumerate() {
            let mut target_order = i as i32;
            // 如果被移动的任务的 sort_order 小于或等于当前兄弟的新编号，需要跳过
            if target_order >= new_sort_order {
                target_order += 1;
            }
            tx.execute(
                "UPDATE tasks SET sort_order = ?1 WHERE id = ?2",
                params![target_order, bro_id],
            )
            .map_err(|e| e.to_string())?;
        }

        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    // ==================== 批量操作 ====================

    /// 批量更新任务状态
    pub fn batch_update_status(&self, ids: &[String], status: &str) -> Result<(), String> {
        if ids.is_empty() {
            return Ok(());
        }
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();

        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("?{}", i)).collect();
        let sql = format!(
            "UPDATE tasks SET status = ?{}, updated_at = ?{} WHERE id IN ({})",
            ids.len() + 1, ids.len() + 2,
            placeholders.join(", ")
        );

        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        for id in ids {
            params.push(Box::new(id.clone()));
        }
        params.push(Box::new(status.to_string()));
        params.push(Box::new(now));

        // 如果变为 completed，同时更新 completed_at
        let sql_completed = if status == "completed" {
            format!(
                "UPDATE tasks SET status = 'completed', completed_at = ?{}, updated_at = ?{} WHERE id IN ({})",
                ids.len() + 1, ids.len() + 2,
                placeholders.join(", ")
            )
        } else {
            sql
        };

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|v| v.as_ref()).collect();
        conn.execute(&sql_completed, params_refs.as_slice())
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 批量删除任务（含递归删除子任务）
    pub fn batch_delete(&self, ids: &[String]) -> Result<(), String> {
        for id in ids {
            self.delete_task(id)?;
        }
        Ok(())
    }

    /// 批量移动任务到新父节点
    pub fn batch_move(&self, ids: &[String], new_parent_id: Option<&str>) -> Result<(), String> {
        if ids.is_empty() {
            return Ok(());
        }

        let conn = self.pool.get().map_err(|e| e.to_string())?;

        // 计算目标位置的最大 sort_order
        let max_order: i32 = if let Some(pid) = new_parent_id {
            conn.query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM tasks WHERE parent_id = ?1",
                params![pid],
                |row| row.get(0),
            )
            .unwrap_or(-1)
        } else {
            conn.query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM tasks WHERE parent_id IS NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or(-1)
        };

        let now = Utc::now().to_rfc3339();
        for (i, id) in ids.iter().enumerate() {
            conn.execute(
                "UPDATE tasks SET parent_id = ?1, sort_order = ?2, updated_at = ?3 WHERE id = ?4",
                params![new_parent_id, max_order + 1 + i as i32, now, id],
            )
            .map_err(|e| e.to_string())?;
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use r2d2_sqlite::SqliteConnectionManager;

    fn create_test_service() -> TaskService {
        // 使用共享缓存的内存数据库，确保同一 URI 的所有连接共享同一份数据
        let db_path = format!("file:test_{}.db?mode=memory&cache=shared", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let manager = SqliteConnectionManager::file(&db_path);
        let pool = Pool::builder()
            .max_size(2)
            .build(manager)
            .expect("Failed to create test pool");

        // 在第一个连接上初始化表结构
        let conn = pool.get().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                parent_id TEXT,
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'pending',
                priority INTEGER NOT NULL DEFAULT 0,
                sort_order INTEGER NOT NULL DEFAULT 0,
                color TEXT NOT NULL DEFAULT '',
                is_milestone INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                completed_at TEXT
            );
            CREATE TABLE IF NOT EXISTS milestone_risks (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                risk_desc TEXT NOT NULL,
                probability TEXT NOT NULL DEFAULT 'medium',
                mitigation TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                title TEXT NOT NULL DEFAULT '',
                content TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
            );"
        ).expect("Failed to create test tables");

        TaskService::new(pool)
    }

    // ==================== 任务 CRUD 测试 ====================

    #[test]
    fn test_create_task_basic() {
        let service = create_test_service();

        let input = NewTask {
            parent_id: None,
            title: "测试任务".to_string(),
            description: Some("测试描述".to_string()),
            priority: Some(2),
            color: Some("#ff0000".to_string()),
            is_milestone: Some(true),
        };

        let task = service.create_task(input).expect("创建任务失败");
        assert_eq!(task.title, "测试任务");
        assert_eq!(task.description, "测试描述");
        assert_eq!(task.status, "pending");
        assert_eq!(task.priority, 2);
        assert_eq!(task.color, "#ff0000");
        assert_eq!(task.is_milestone, true);
        assert_eq!(task.sort_order, 0); // 第一个同级任务
        assert!(task.parent_id.is_none());
        assert!(!task.id.is_empty());
    }

    #[test]
    fn test_create_task_with_parent() {
        let service = create_test_service();

        let parent = service.create_task(NewTask {
            parent_id: None,
            title: "父任务".to_string(),
            description: None,
            priority: None,
            color: None,
            is_milestone: None,
        }).expect("创建父任务失败");

        let child = service.create_task(NewTask {
            parent_id: Some(parent.id.clone()),
            title: "子任务".to_string(),
            description: None,
            priority: None,
            color: None,
            is_milestone: None,
        }).expect("创建子任务失败");

        assert_eq!(child.parent_id, Some(parent.id.clone()));
        assert_eq!(child.sort_order, 0);
    }

    #[test]
    fn test_create_task_sort_order_increment() {
        let service = create_test_service();

        let task1 = service.create_task(NewTask {
            parent_id: None,
            title: "任务1".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).expect("创建任务1失败");

        let task2 = service.create_task(NewTask {
            parent_id: None,
            title: "任务2".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).expect("创建任务2失败");

        let task3 = service.create_task(NewTask {
            parent_id: None,
            title: "任务3".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).expect("创建任务3失败");

        assert_eq!(task1.sort_order, 0);
        assert_eq!(task2.sort_order, 1);
        assert_eq!(task3.sort_order, 2);
    }

    #[test]
    fn test_get_all_tasks() {
        let service = create_test_service();

        service.create_task(NewTask {
            parent_id: None, title: "任务A".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();
        service.create_task(NewTask {
            parent_id: None, title: "任务B".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();

        let tasks = service.get_all_tasks().expect("获取任务列表失败");
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].title, "任务A");
        assert_eq!(tasks[1].title, "任务B");
    }

    #[test]
    fn test_get_task_by_id() {
        let service = create_test_service();

        let created = service.create_task(NewTask {
            parent_id: None, title: "查找测试".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();

        let found = service.get_task(&created.id).expect("查询任务失败");
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "查找测试");

        let not_found = service.get_task("nonexistent").expect("查询不存在任务失败");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_update_task_fields() {
        let service = create_test_service();

        let task = service.create_task(NewTask {
            parent_id: None, title: "原始标题".to_string(),
            description: None, priority: None, color: None, is_milestone: Some(false),
        }).unwrap();

        let updated = service.update_task(
            &task.id,
            Some("新标题"),
            Some("新描述"),
            Some("in_progress"),
            Some(3),
            Some("#0000ff"),
            Some(true), // is_milestone
            None,        // parent_id
            None,        // sort_order
        ).expect("更新任务失败");

        assert_eq!(updated.title, "新标题");
        assert_eq!(updated.description, "新描述");
        assert_eq!(updated.status, "in_progress");
        assert_eq!(updated.priority, 3);
        assert_eq!(updated.color, "#0000ff");
        assert_eq!(updated.is_milestone, true);
    }

    #[test]
    fn test_update_task_milestone_save() {
        let service = create_test_service();

        // 创建任务，is_milestone = false
        let task = service.create_task(NewTask {
            parent_id: None, title: "里程碑测试".to_string(),
            description: None, priority: None, color: None, is_milestone: Some(false),
        }).unwrap();
        assert_eq!(task.is_milestone, false);

        // 更新为里程碑
        let updated = service.update_task(
            &task.id,
            None, None, None, None, None,
            Some(true), // is_milestone = true
            None, None,
        ).expect("设置里程碑失败");
        assert_eq!(updated.is_milestone, true);

        // 再次读取确认持久化
        let reloaded = service.get_task(&task.id).unwrap().unwrap();
        assert_eq!(reloaded.is_milestone, true);

        // 取消里程碑
        let updated2 = service.update_task(
            &task.id,
            None, None, None, None, None,
            Some(false),
            None, None,
        ).expect("取消里程碑失败");
        assert_eq!(updated2.is_milestone, false);
    }

    #[test]
    fn test_update_task_status_completed_at() {
        let service = create_test_service();

        let task = service.create_task(NewTask {
            parent_id: None, title: "状态测试".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();

        // 初始状态 completed_at 为空
        assert!(task.completed_at.is_none());

        // 设为 completed
        let updated = service.update_task(
            &task.id,
            None, None,
            Some("completed"),
            None, None, None, None, None,
        ).expect("设置完成状态失败");

        assert_eq!(updated.status, "completed");
        assert!(updated.completed_at.is_some());
    }

    #[test]
    fn test_delete_task_cascade() {
        let service = create_test_service();

        let parent = service.create_task(NewTask {
            parent_id: None, title: "父".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();

        let child = service.create_task(NewTask {
            parent_id: Some(parent.id.clone()), title: "子".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();

        let grandchild = service.create_task(NewTask {
            parent_id: Some(child.id.clone()), title: "孙".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();

        service.delete_task(&parent.id).expect("删除父任务失败");

        // 验证递归删除
        let all = service.get_all_tasks().unwrap();
        assert!(all.is_empty());
    }

    // ==================== 里程碑风险测试 ====================

    #[test]
    fn test_add_risk() {
        let service = create_test_service();

        let task = service.create_task(NewTask {
            parent_id: None, title: "风险测试".to_string(),
            description: None, priority: None, color: None, is_milestone: Some(true),
        }).unwrap();

        let risk = service.add_risk(
            &task.id, "进度延迟", "high", "增加人力",
        ).expect("添加风险失败");

        assert_eq!(risk.risk_desc, "进度延迟");
        assert_eq!(risk.probability, "high");
        assert_eq!(risk.mitigation, "增加人力");
        assert_eq!(risk.task_id, task.id);
    }

    #[test]
    fn test_get_and_delete_risks() {
        let service = create_test_service();

        let task = service.create_task(NewTask {
            parent_id: None, title: "风险管理".to_string(),
            description: None, priority: None, color: None, is_milestone: Some(true),
        }).unwrap();

        service.add_risk(&task.id, "风险A", "low", "").unwrap();
        service.add_risk(&task.id, "风险B", "high", "预案").unwrap();

        let risks = service.get_risks(&task.id).expect("获取风险列表失败");
        assert_eq!(risks.len(), 2);

        service.delete_risk(&risks[0].id).expect("删除风险失败");
        let remaining = service.get_risks(&task.id).unwrap();
        assert_eq!(remaining.len(), 1);
    }

    // ==================== 笔记测试 ====================

    #[test]
    fn test_save_note_create_and_update() {
        let service = create_test_service();

        let task = service.create_task(NewTask {
            parent_id: None, title: "笔记测试".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();

        // 创建新笔记
        let note = service.save_note(&task.id, None, "笔记标题", "笔记内容")
            .expect("创建笔记失败");
        assert_eq!(note.title, "笔记标题");
        assert_eq!(note.content, "笔记内容");
        assert!(!note.id.is_empty());

        // 更新笔记
        let updated = service.save_note(&task.id, Some(&note.id), "更新标题", "更新内容")
            .expect("更新笔记失败");
        assert_eq!(updated.title, "更新标题");
        assert_eq!(updated.content, "更新内容");
        assert_eq!(updated.id, note.id);
    }

    #[test]
    fn test_get_and_delete_notes() {
        let service = create_test_service();

        let task = service.create_task(NewTask {
            parent_id: None, title: "笔记删除".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();

        let note = service.save_note(&task.id, None, "测试", "内容").unwrap();
        let notes = service.get_notes(&task.id).unwrap();
        assert_eq!(notes.len(), 1);

        service.delete_note(&note.id).unwrap();
        let after = service.get_notes(&task.id).unwrap();
        assert!(after.is_empty());
    }

    // ==================== 排序测试 ====================

    #[test]
    fn test_reorder_task_same_level() {
        let service = create_test_service();

        let t1 = service.create_task(NewTask {
            parent_id: None, title: "A".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();
        let t2 = service.create_task(NewTask {
            parent_id: None, title: "B".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();
        let t3 = service.create_task(NewTask {
            parent_id: None, title: "C".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();

        // 将 t3 移到 t1 前面（sort_order = 0）
        service.reorder_task(&t3.id, None, 0).expect("重排失败");

        let tasks = service.get_all_tasks().unwrap();
        // 验证 t3 现在是第一个
        let reordered_t3 = tasks.iter().find(|t| t.id == t3.id).unwrap();
        assert_eq!(reordered_t3.sort_order, 0);
    }

    #[test]
    fn test_reorder_task_move_to_parent() {
        let service = create_test_service();

        let parent = service.create_task(NewTask {
            parent_id: None, title: "父".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();
        let child = service.create_task(NewTask {
            parent_id: None, title: "待移动".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();

        service.reorder_task(&child.id, Some(&parent.id), 0).expect("移动到父节点失败");

        let moved = service.get_task(&child.id).unwrap().unwrap();
        assert_eq!(moved.parent_id, Some(parent.id));
    }

    #[test]
    fn test_reorder_task_cycle_detection() {
        let service = create_test_service();

        let parent = service.create_task(NewTask {
            parent_id: None, title: "父".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();
        let child = service.create_task(NewTask {
            parent_id: Some(parent.id.clone()), title: "子".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();

        // 验证子任务确实是父任务的子孙
        let tasks = service.get_all_tasks().unwrap();
        let child_task = tasks.iter().find(|t| t.id == child.id).unwrap();
        assert_eq!(child_task.parent_id, Some(parent.id.clone()));

        // 尝试把父移动到子的下面（循环引用）
        let result = service.reorder_task(&parent.id, Some(&child.id), 0);
        assert!(result.is_err(), "Expected error but got Ok: moving parent into child should be rejected");
        assert!(result.unwrap_err().contains("descendant"), "Error message should mention 'descendant'");
    }

    // ==================== 批量操作测试 ====================

    #[test]
    fn test_batch_update_status() {
        let service = create_test_service();

        let t1 = service.create_task(NewTask {
            parent_id: None, title: "批量1".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();
        let t2 = service.create_task(NewTask {
            parent_id: None, title: "批量2".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();

        service.batch_update_status(&[t1.id.clone(), t2.id.clone()], "completed").unwrap();

        let after1 = service.get_task(&t1.id).unwrap().unwrap();
        let after2 = service.get_task(&t2.id).unwrap().unwrap();
        assert_eq!(after1.status, "completed");
        assert_eq!(after2.status, "completed");
        assert!(after1.completed_at.is_some());
        assert!(after2.completed_at.is_some());
    }

    #[test]
    fn test_batch_delete() {
        let service = create_test_service();

        let t1 = service.create_task(NewTask {
            parent_id: None, title: "删A".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();
        let t2 = service.create_task(NewTask {
            parent_id: None, title: "删B".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();

        service.batch_delete(&[t1.id, t2.id]).unwrap();
        let remaining = service.get_all_tasks().unwrap();
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_batch_move() {
        let service = create_test_service();

        let parent = service.create_task(NewTask {
            parent_id: None, title: "目标父".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();
        let t1 = service.create_task(NewTask {
            parent_id: None, title: "移A".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();
        let t2 = service.create_task(NewTask {
            parent_id: None, title: "移B".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();

        service.batch_move(&[t1.id.clone(), t2.id.clone()], Some(&parent.id)).unwrap();

        let moved1 = service.get_task(&t1.id).unwrap().unwrap();
        let moved2 = service.get_task(&t2.id).unwrap().unwrap();
        assert_eq!(moved1.parent_id, Some(parent.id.clone()));
        assert_eq!(moved2.parent_id, Some(parent.id));
    }

    // ==================== 搜索测试 ====================

    #[test]
    fn test_search_tasks() {
        let service = create_test_service();

        service.create_task(NewTask {
            parent_id: None, title: "前端开发".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();
        service.create_task(NewTask {
            parent_id: None, title: "后端API".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();
        service.create_task(NewTask {
            parent_id: None, title: "部署上线".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();

        let results = service.search_tasks("开发").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "前端开发");

        let results2 = service.search_tasks("不存在的关键词").unwrap();
        assert!(results2.is_empty());
    }

    #[test]
    fn test_search_tasks_case_insensitive() {
        let service = create_test_service();

        service.create_task(NewTask {
            parent_id: None, title: "Hello World".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).unwrap();

        // SQLite LIKE 默认不区分大小写
        let results = service.search_tasks("hello").unwrap();
        assert_eq!(results.len(), 1);
    }
}
