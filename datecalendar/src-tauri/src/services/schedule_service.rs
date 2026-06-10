use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;
use chrono::Utc;

use crate::db::models::Schedule;

/// 日程服务 — 封装所有日程相关的业务逻辑
pub struct ScheduleService {
    pool: Pool<SqliteConnectionManager>,
}

impl ScheduleService {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Self {
        Self { pool }
    }

    // ==================== 日程 CRUD ====================

    /// 获取所有日程
    pub fn get_all_schedules(&self) -> Result<Vec<Schedule>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, task_id, title, start_time, end_time, is_all_day,
                        schedule_type, status, color, created_at, updated_at
                 FROM schedules ORDER BY start_time"
            )
            .map_err(|e| e.to_string())?;

        let schedules = stmt
            .query_map([], |row| {
                Ok(Schedule {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    title: row.get(2)?,
                    start_time: row.get(3)?,
                    end_time: row.get(4)?,
                    is_all_day: row.get::<_, i32>(5)? != 0,
                    schedule_type: row.get(6)?,
                    status: row.get(7)?,
                    color: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(schedules)
    }

    /// 获取单个日程
    pub fn get_schedule(&self, id: &str) -> Result<Option<Schedule>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, task_id, title, start_time, end_time, is_all_day,
                        schedule_type, status, color, created_at, updated_at
                 FROM schedules WHERE id = ?1"
            )
            .map_err(|e| e.to_string())?;

        stmt.query_row(params![id], |row| {
            Ok(Schedule {
                id: row.get(0)?,
                task_id: row.get(1)?,
                title: row.get(2)?,
                start_time: row.get(3)?,
                end_time: row.get(4)?,
                is_all_day: row.get::<_, i32>(5)? != 0,
                schedule_type: row.get(6)?,
                status: row.get(7)?,
                color: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .optional()
        .map_err(|e| e.to_string())
    }

    /// 按日期范围查询日程
    /// start_date/end_date 格式: "2026-06-10" (date only)
    pub fn get_schedules_in_range(&self, start_date: &str, end_date: &str) -> Result<Vec<Schedule>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let start_iso = format!("{}T00:00:00", start_date);
        let end_iso = format!("{}T23:59:59", end_date);

        let mut stmt = conn
            .prepare(
                "SELECT id, task_id, title, start_time, end_time, is_all_day,
                        schedule_type, status, color, created_at, updated_at
                 FROM schedules
                 WHERE start_time <= ?1 AND end_time >= ?2
                 ORDER BY start_time"
            )
            .map_err(|e| e.to_string())?;

        let schedules = stmt
            .query_map(params![end_iso, start_iso], |row| {
                Ok(Schedule {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    title: row.get(2)?,
                    start_time: row.get(3)?,
                    end_time: row.get(4)?,
                    is_all_day: row.get::<_, i32>(5)? != 0,
                    schedule_type: row.get(6)?,
                    status: row.get(7)?,
                    color: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(schedules)
    }

    /// 获取某天的日程（fixed + todo_day）
    pub fn get_day_schedules(&self, date: &str) -> Result<Vec<Schedule>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let day_start = format!("{}T00:00:00", date);
        let day_end = format!("{}T23:59:59", date);

        let mut stmt = conn
            .prepare(
                "SELECT id, task_id, title, start_time, end_time, is_all_day,
                        schedule_type, status, color, created_at, updated_at
                 FROM schedules
                 WHERE (
                     (schedule_type = 'fixed' AND start_time <= ?1 AND end_time >= ?2)
                     OR
                     (schedule_type = 'todo_day' AND start_time >= ?3 AND start_time <= ?4)
                     OR
                     (schedule_type = 'todo_week' AND start_time >= ?5 AND start_time <= ?6)
                 )
                 AND status != 'cancelled'
                 ORDER BY
                     CASE schedule_type
                         WHEN 'fixed' THEN 0
                         WHEN 'todo_day' THEN 1
                         WHEN 'todo_week' THEN 2
                     END,
                     start_time"
            )
            .map_err(|e| e.to_string())?;

        let schedules = stmt
            .query_map(params![day_end, day_start, day_start, day_end, day_start, day_end], |row| {
                Ok(Schedule {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    title: row.get(2)?,
                    start_time: row.get(3)?,
                    end_time: row.get(4)?,
                    is_all_day: row.get::<_, i32>(5)? != 0,
                    schedule_type: row.get(6)?,
                    status: row.get(7)?,
                    color: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(schedules)
    }

    /// 获取某周的日程（全部类型）
    pub fn get_week_schedules(&self, week_start: &str, week_end: &str) -> Result<Vec<Schedule>, String> {
        self.get_schedules_in_range(week_start, week_end)
    }

    /// 获取关联到某个任务的所有日程
    pub fn get_schedules_by_task(&self, task_id: &str) -> Result<Vec<Schedule>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, task_id, title, start_time, end_time, is_all_day,
                        schedule_type, status, color, created_at, updated_at
                 FROM schedules WHERE task_id = ?1 ORDER BY start_time"
            )
            .map_err(|e| e.to_string())?;

        let schedules = stmt
            .query_map(params![task_id], |row| {
                Ok(Schedule {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    title: row.get(2)?,
                    start_time: row.get(3)?,
                    end_time: row.get(4)?,
                    is_all_day: row.get::<_, i32>(5)? != 0,
                    schedule_type: row.get(6)?,
                    status: row.get(7)?,
                    color: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(schedules)
    }

    /// 创建日程
    pub fn create_schedule(
        &self,
        task_id: &str,
        title: &str,
        start_time: &str,
        end_time: &str,
        is_all_day: bool,
        schedule_type: &str,
        color: &str,
    ) -> Result<Schedule, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO schedules (id, task_id, title, start_time, end_time, is_all_day, schedule_type, status, color, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending', ?8, ?9, ?10)",
            params![id, task_id, title, start_time, end_time, is_all_day as i32, schedule_type, color, now, now],
        )
        .map_err(|e| e.to_string())?;

        self.get_schedule(&id).map(|s| s.unwrap())
    }

    /// 更新日程（动态字段）
    pub fn update_schedule(
        &self,
        id: &str,
        title: Option<&str>,
        start_time: Option<&str>,
        end_time: Option<&str>,
        is_all_day: Option<bool>,
        schedule_type: Option<&str>,
        status: Option<&str>,
        color: Option<&str>,
        task_id: Option<&str>,
    ) -> Result<Schedule, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();

        let mut sets: Vec<String> = Vec::new();
        let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(t) = title {
            sets.push(format!("title = ?{}", sets.len() + 1));
            values.push(Box::new(t.to_string()));
        }
        if let Some(s) = start_time {
            sets.push(format!("start_time = ?{}", sets.len() + 1));
            values.push(Box::new(s.to_string()));
        }
        if let Some(e) = end_time {
            sets.push(format!("end_time = ?{}", sets.len() + 1));
            values.push(Box::new(e.to_string()));
        }
        if let Some(a) = is_all_day {
            sets.push(format!("is_all_day = ?{}", sets.len() + 1));
            values.push(Box::new(a as i32));
        }
        if let Some(st) = schedule_type {
            sets.push(format!("schedule_type = ?{}", sets.len() + 1));
            values.push(Box::new(st.to_string()));
        }
        if let Some(s) = status {
            sets.push(format!("status = ?{}", sets.len() + 1));
            values.push(Box::new(s.to_string()));
        }
        if let Some(c) = color {
            sets.push(format!("color = ?{}", sets.len() + 1));
            values.push(Box::new(c.to_string()));
        }
        if let Some(tid) = task_id {
            sets.push(format!("task_id = ?{}", sets.len() + 1));
            values.push(Box::new(tid.to_string()));
        }

        if sets.is_empty() {
            return self.get_schedule(id).map(|s| s.unwrap());
        }

        sets.push(format!("updated_at = ?{}", sets.len() + 1));
        values.push(Box::new(now));

        let sql = format!(
            "UPDATE schedules SET {} WHERE id = ?{}",
            sets.join(", "),
            sets.len() + 1
        );
        values.push(Box::new(id.to_string()));

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
        conn.execute(&sql, params_refs.as_slice())
            .map_err(|e| e.to_string())?;

        self.get_schedule(id).map(|s| s.unwrap())
    }

    /// 删除日程
    pub fn delete_schedule(&self, id: &str) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM schedules WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 删除某个任务关联的所有日程
    pub fn delete_schedules_by_task(&self, task_id: &str) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM schedules WHERE task_id = ?1", params![task_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ==================== 状态同步 ====================

    /// 更新日程状态，并同步关联任务状态
    pub fn update_schedule_status(&self, schedule_id: &str, new_status: &str) -> Result<(), String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

        // 1. 更新日程状态
        let now = Utc::now().to_rfc3339();
        tx.execute(
            "UPDATE schedules SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![new_status, now, schedule_id],
        )
        .map_err(|e| e.to_string())?;

        // 2. 查询关联的 task_id
        let task_id: Option<String> = tx
            .query_row(
                "SELECT task_id FROM schedules WHERE id = ?1",
                params![schedule_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        // 3. 同步任务状态
        if let Some(tid) = task_id {
            if !tid.is_empty() {
                let task_status = match new_status {
                    "completed" => "completed",
                    "cancelled" => "cancelled",
                    _ => "in_progress",
                };
                tx.execute(
                    "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE id = ?3 AND status != ?4",
                    params![task_status, now, tid, task_status],
                )
                .map_err(|e| e.to_string())?;

                if task_status == "completed" {
                    tx.execute(
                        "UPDATE tasks SET completed_at = ?1 WHERE id = ?2 AND completed_at IS NULL",
                        params![now, tid],
                    )
                    .map_err(|e| e.to_string())?;
                }

                // 4. 同步该任务关联的其他日程
                if new_status == "completed" {
                    tx.execute(
                        "UPDATE schedules SET status = 'completed', updated_at = ?1 WHERE task_id = ?2 AND id != ?3 AND status != 'completed'",
                        params![now, tid, schedule_id],
                    )
                    .map_err(|e| e.to_string())?;
                } else if new_status == "cancelled" {
                    tx.execute(
                        "UPDATE schedules SET status = 'cancelled', updated_at = ?1 WHERE task_id = ?2 AND id != ?3 AND status != 'cancelled'",
                        params![now, tid, schedule_id],
                    )
                    .map_err(|e| e.to_string())?;
                }
            }
        }

        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    // ==================== 冲突检测 ====================

    /// 检查指定时间段是否与其他 fixed 日程冲突
    pub fn check_conflicts(
        &self,
        start_time: &str,
        end_time: &str,
        exclude_id: Option<&str>,
    ) -> Result<Vec<Schedule>, String> {
        let conn = self.pool.get().map_err(|e| e.to_string())?;

        let ids: Vec<String> = if let Some(ex_id) = exclude_id {
            let mut stmt = conn
                .prepare(
                    "SELECT id FROM schedules
                     WHERE schedule_type = 'fixed'
                       AND status != 'cancelled'
                       AND start_time < ?1 AND end_time > ?2
                       AND id != ?3"
                )
                .map_err(|e| e.to_string())?;

            let rows = stmt.query_map(params![end_time, start_time, ex_id], |row| row.get::<_, String>(0))
                .map_err(|e| e.to_string())?;
            rows.filter_map(|r| r.ok()).collect()
        } else {
            let mut stmt = conn
                .prepare(
                    "SELECT id FROM schedules
                     WHERE schedule_type = 'fixed'
                       AND status != 'cancelled'
                       AND start_time < ?1 AND end_time > ?2"
                )
                .map_err(|e| e.to_string())?;

            let rows = stmt.query_map(params![end_time, start_time], |row| row.get::<_, String>(0))
                .map_err(|e| e.to_string())?;
            rows.filter_map(|r| r.ok()).collect()
        };

        let mut result = Vec::new();
        for id in ids {
            if let Some(s) = self.get_schedule(&id)? {
                result.push(s);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use r2d2_sqlite::SqliteConnectionManager;

    fn create_test_service() -> (ScheduleService, String) {
        let db_path = format!("file:test_{}.db?mode=memory&cache=shared", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let manager = SqliteConnectionManager::file(&db_path);
        let pool = Pool::builder().max_size(2).build(manager).expect("Failed to create test pool");
        let conn = pool.get().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY, parent_id TEXT, title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '', status TEXT NOT NULL DEFAULT 'pending',
                priority INTEGER NOT NULL DEFAULT 0, sort_order INTEGER NOT NULL DEFAULT 0,
                color TEXT NOT NULL DEFAULT '', is_milestone INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL, completed_at TEXT
            ); CREATE TABLE IF NOT EXISTS schedules (
                id TEXT PRIMARY KEY, task_id TEXT NOT NULL, title TEXT NOT NULL,
                start_time TEXT NOT NULL, end_time TEXT NOT NULL, is_all_day INTEGER NOT NULL DEFAULT 0,
                schedule_type TEXT NOT NULL DEFAULT 'fixed', status TEXT NOT NULL DEFAULT 'pending',
                color TEXT NOT NULL DEFAULT '', created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
            );"
        ).expect("Failed to create test tables");
        // 同时创建一个测试任务
        let task_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO tasks (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![task_id, "测试任务", now, now],
        ).unwrap();
        (ScheduleService::new(pool), task_id)
    }

    // ==================== 日程 CRUD 测试 ====================

    #[test]
    fn test_create_schedule() {
        let (service, task_id) = create_test_service();

        let schedule = service.create_schedule(
            &task_id, "团队会议", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "#3b82f6",
        ).expect("创建日程失败");

        assert_eq!(schedule.title, "团队会议");
        assert_eq!(schedule.task_id, task_id);
        assert_eq!(schedule.start_time, "2026-06-10T09:00:00");
        assert_eq!(schedule.end_time, "2026-06-10T10:00:00");
        assert_eq!(schedule.is_all_day, false);
        assert_eq!(schedule.schedule_type, "fixed");
        assert_eq!(schedule.status, "pending");
        assert_eq!(schedule.color, "#3b82f6");
    }

    #[test]
    fn test_create_all_day_schedule() {
        let (service, task_id) = create_test_service();

        let schedule = service.create_schedule(
            &task_id, "全天活动", "2026-06-10T00:00:00", "2026-06-10T23:59:59",
            true, "fixed", "",
        ).expect("创建全天日程失败");

        assert_eq!(schedule.is_all_day, true);
    }

    #[test]
    fn test_get_all_schedules() {
        let (service, task_id) = create_test_service();

        service.create_schedule(&task_id, "日程A", "2026-06-10T09:00:00", "2026-06-10T10:00:00", false, "fixed", "").unwrap();
        service.create_schedule(&task_id, "日程B", "2026-06-10T14:00:00", "2026-06-10T15:00:00", false, "fixed", "").unwrap();

        let all = service.get_all_schedules().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_get_schedules_in_range() {
        let (service, task_id) = create_test_service();

        service.create_schedule(&task_id, "6月10日", "2026-06-10T09:00:00", "2026-06-10T10:00:00", false, "fixed", "").unwrap();
        service.create_schedule(&task_id, "6月15日", "2026-06-15T09:00:00", "2026-06-15T10:00:00", false, "fixed", "").unwrap();

        // 查 6/10 ~ 6/11，应只返回第一条
        let results = service.get_schedules_in_range("2026-06-10", "2026-06-11").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "6月10日");
    }

    #[test]
    fn test_get_day_schedules() {
        let (service, task_id) = create_test_service();

        service.create_schedule(&task_id, "fixed日程", "2026-06-10T09:00:00", "2026-06-10T10:00:00", false, "fixed", "").unwrap();
        service.create_schedule(&task_id, "todo_day日程", "2026-06-10T00:00:00", "2026-06-10T23:59:59", true, "todo_day", "").unwrap();

        let day_schedules = service.get_day_schedules("2026-06-10").unwrap();
        assert_eq!(day_schedules.len(), 2);
    }

    #[test]
    fn test_get_schedules_by_task() {
        let (service, task_id) = create_test_service();

        service.create_schedule(&task_id, "日程1", "2026-06-10T09:00:00", "2026-06-10T10:00:00", false, "fixed", "").unwrap();

        let schedules = service.get_schedules_by_task(&task_id).unwrap();
        assert_eq!(schedules.len(), 1);
    }

    #[test]
    fn test_update_schedule() {
        let (service, task_id) = create_test_service();

        let schedule = service.create_schedule(
            &task_id, "原始标题", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "",
        ).unwrap();

        let updated = service.update_schedule(
            &schedule.id,
            Some("更新标题"),
            Some("2026-06-10T10:00:00"),
            Some("2026-06-10T11:00:00"),
            None, None, None, None, None,
        ).expect("更新日程失败");

        assert_eq!(updated.title, "更新标题");
        assert_eq!(updated.start_time, "2026-06-10T10:00:00");
        assert_eq!(updated.end_time, "2026-06-10T11:00:00");
    }

    #[test]
    fn test_delete_schedule() {
        let (service, task_id) = create_test_service();

        let schedule = service.create_schedule(
            &task_id, "待删除", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "",
        ).unwrap();

        service.delete_schedule(&schedule.id).unwrap();
        let all = service.get_all_schedules().unwrap();
        assert!(all.is_empty());
    }

    // ==================== 状态同步测试 ====================

    #[test]
    fn test_update_schedule_status_sync_task() {
        let (service, task_id) = create_test_service();

        let schedule = service.create_schedule(
            &task_id, "同步测试", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "",
        ).unwrap();

        // 完成日程，应同步任务状态
        service.update_schedule_status(&schedule.id, "completed").unwrap();

        let updated = service.get_schedule(&schedule.id).unwrap().unwrap();
        assert_eq!(updated.status, "completed");
    }

    // ==================== 冲突检测测试 ====================

    #[test]
    fn test_check_conflicts_no_conflict() {
        let (service, task_id) = create_test_service();

        service.create_schedule(
            &task_id, "日程A", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "",
        ).unwrap();

        // 检查一个不重叠的时间段
        let conflicts = service.check_conflicts(
            "2026-06-10T10:00:00", "2026-06-10T11:00:00", None,
        ).unwrap();
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_check_conflicts_with_conflict() {
        let (service, task_id) = create_test_service();

        service.create_schedule(
            &task_id, "已存在日程", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "",
        ).unwrap();

        // 检查一个重叠的时间段
        let conflicts = service.check_conflicts(
            "2026-06-10T09:30:00", "2026-06-10T10:30:00", None,
        ).unwrap();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].title, "已存在日程");
    }

    #[test]
    fn test_check_conflicts_exclude_id() {
        let (service, task_id) = create_test_service();

        let schedule = service.create_schedule(
            &task_id, "自身", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "",
        ).unwrap();

        // 排除自身ID，不应报告冲突
        let conflicts = service.check_conflicts(
            "2026-06-10T09:00:00", "2026-06-10T10:00:00", Some(&schedule.id),
        ).unwrap();
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_check_conflicts_ignores_cancelled() {
        let (service, task_id) = create_test_service();

        let cancelled = service.create_schedule(
            &task_id, "已取消", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "",
        ).unwrap();
        service.update_schedule_status(&cancelled.id, "cancelled").unwrap();

        // 已取消的日程不应报告冲突
        let conflicts = service.check_conflicts(
            "2026-06-10T09:00:00", "2026-06-10T10:00:00", None,
        ).unwrap();
        assert!(conflicts.is_empty());
    }
}
