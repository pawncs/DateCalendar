use rusqlite::Connection;

/// 执行数据库迁移，创建所有必要的表
///
/// 采用幂等设计（IF NOT EXISTS），多次执行安全
pub fn run_migrations(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute_batch(
        "
        -- 任务表：树形结构通过 parent_id 自引用实现
        CREATE TABLE IF NOT EXISTS tasks (
            id            TEXT PRIMARY KEY,
            parent_id     TEXT REFERENCES tasks(id) ON DELETE SET NULL,
            title         TEXT NOT NULL,
            description   TEXT NOT NULL DEFAULT '',
            status        TEXT NOT NULL DEFAULT 'pending',
            priority      INTEGER NOT NULL DEFAULT 0,
            sort_order    INTEGER NOT NULL DEFAULT 0,
            color         TEXT NOT NULL DEFAULT '',
            is_milestone  INTEGER NOT NULL DEFAULT 0,
            created_at    TEXT NOT NULL,
            updated_at    TEXT NOT NULL,
            completed_at  TEXT
        );

        -- 索引：加速按父节点查询子任务
        CREATE INDEX IF NOT EXISTS idx_tasks_parent ON tasks(parent_id);
        -- 索引：加速按状态筛选
        CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);

        -- 里程碑风险表
        CREATE TABLE IF NOT EXISTS milestone_risks (
            id            TEXT PRIMARY KEY,
            task_id       TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
            risk_desc     TEXT NOT NULL,
            probability   TEXT NOT NULL DEFAULT 'medium',
            mitigation    TEXT NOT NULL DEFAULT '',
            created_at    TEXT NOT NULL,
            updated_at    TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_risks_task ON milestone_risks(task_id);

        -- 笔记表
        CREATE TABLE IF NOT EXISTS notes (
            id            TEXT PRIMARY KEY,
            task_id       TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
            title         TEXT NOT NULL,
            content       TEXT NOT NULL DEFAULT '',
            created_at    TEXT NOT NULL,
            updated_at    TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_notes_task ON notes(task_id);

        -- 日程安排表
        CREATE TABLE IF NOT EXISTS schedules (
            id            TEXT PRIMARY KEY,
            task_id       TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
            title         TEXT NOT NULL,
            start_time    TEXT NOT NULL,
            end_time      TEXT NOT NULL,
            is_all_day    INTEGER NOT NULL DEFAULT 0,
            schedule_type TEXT NOT NULL DEFAULT 'fixed',
            status        TEXT NOT NULL DEFAULT 'pending',
            color         TEXT NOT NULL DEFAULT '',
            created_at    TEXT NOT NULL,
            updated_at    TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_schedules_task ON schedules(task_id);
        CREATE INDEX IF NOT EXISTS idx_schedules_time ON schedules(start_time, end_time);

        -- 应用设置表
        CREATE TABLE IF NOT EXISTS settings (
            key           TEXT PRIMARY KEY,
            value         TEXT NOT NULL
        );

        -- 启用外键约束（SQLite 默认关闭）
        PRAGMA foreign_keys = ON;
        "
    )?;

    Ok(())
}
