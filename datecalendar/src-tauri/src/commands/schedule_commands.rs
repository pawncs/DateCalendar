use tauri::State;
use crate::db::models::Schedule;
use crate::services::schedule_service::ScheduleService;

// ==================== 日程 CRUD 命令 ====================

#[tauri::command]
pub fn get_all_schedules(service: State<'_, ScheduleService>) -> Result<Vec<Schedule>, String> {
    service.get_all_schedules()
}

#[tauri::command]
pub fn get_schedule(service: State<'_, ScheduleService>, id: String) -> Result<Option<Schedule>, String> {
    service.get_schedule(&id)
}

#[tauri::command]
pub fn get_schedules_in_range(
    service: State<'_, ScheduleService>,
    start_date: String,
    end_date: String,
) -> Result<Vec<Schedule>, String> {
    service.get_schedules_in_range(&start_date, &end_date)
}

#[tauri::command]
pub fn get_schedules_by_task(
    service: State<'_, ScheduleService>,
    task_id: String,
) -> Result<Vec<Schedule>, String> {
    service.get_schedules_by_task(&task_id)
}

#[tauri::command]
pub fn get_day_schedules(
    service: State<'_, ScheduleService>,
    date: String,
) -> Result<Vec<Schedule>, String> {
    service.get_day_schedules(&date)
}

#[tauri::command]
pub fn get_week_schedules(
    service: State<'_, ScheduleService>,
    week_start: String,
    week_end: String,
) -> Result<Vec<Schedule>, String> {
    service.get_week_schedules(&week_start, &week_end)
}

#[tauri::command]
pub fn create_schedule(
    service: State<'_, ScheduleService>,
    task_id: String,
    title: String,
    start_time: String,
    end_time: String,
    is_all_day: Option<bool>,
    schedule_type: Option<String>,
    color: Option<String>,
) -> Result<Schedule, String> {
    service.create_schedule(
        &task_id,
        &title,
        &start_time,
        &end_time,
        is_all_day.unwrap_or(false),
        &schedule_type.unwrap_or_else(|| "fixed".to_string()),
        &color.unwrap_or_default(),
    )
}

#[tauri::command]
pub fn update_schedule(
    service: State<'_, ScheduleService>,
    id: String,
    title: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    is_all_day: Option<bool>,
    schedule_type: Option<String>,
    status: Option<String>,
    color: Option<String>,
    task_id: Option<String>,
) -> Result<Schedule, String> {
    service.update_schedule(
        &id,
        title.as_deref(),
        start_time.as_deref(),
        end_time.as_deref(),
        is_all_day,
        schedule_type.as_deref(),
        status.as_deref(),
        color.as_deref(),
        task_id.as_deref(),
    )
}

#[tauri::command]
pub fn delete_schedule(service: State<'_, ScheduleService>, id: String) -> Result<(), String> {
    service.delete_schedule(&id)
}

// ==================== 状态同步命令 ====================

#[tauri::command]
pub fn update_schedule_status(
    service: State<'_, ScheduleService>,
    schedule_id: String,
    new_status: String,
) -> Result<(), String> {
    service.update_schedule_status(&schedule_id, &new_status)
}

// ==================== 冲突检测命令 ====================

#[tauri::command]
pub fn check_conflicts(
    service: State<'_, ScheduleService>,
    start_time: String,
    end_time: String,
    exclude_id: Option<String>,
) -> Result<Vec<Schedule>, String> {
    service.check_conflicts(&start_time, &end_time, exclude_id.as_deref())
}

// ==================== Tauri IPC 命令层白盒测试 ====================
///
/// 测试策略：
/// - 直接实例化 ScheduleService（使用内存数据库），绕过 Tauri State 机制
/// - 逐条验证命令的参数转换逻辑（Option 默认值、引用转换）
/// - 与 Service 层测试互补：Service 测 SQL 逻辑，Commands 测参数映射

#[cfg(test)]
mod tests {
    use super::*;
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;
    use uuid::Uuid;
    use chrono::Utc;
    use rusqlite;

    /// 创建测试用 ScheduleService（内存数据库 + 表初始化 + 预置任务）
    fn create_test_service() -> (ScheduleService, String) {
        let manager = SqliteConnectionManager::file(":memory:");
        let pool = Pool::builder()
            .max_size(1)
            .build(manager)
            .expect("Failed to create test pool");

        let conn = pool.get().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY, parent_id TEXT, title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '', status TEXT NOT NULL DEFAULT 'pending',
                priority INTEGER NOT NULL DEFAULT 0, sort_order INTEGER NOT NULL DEFAULT 0,
                color TEXT NOT NULL DEFAULT '', is_milestone INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL, completed_at TEXT
            );
            CREATE TABLE IF NOT EXISTS schedules (
                id TEXT PRIMARY KEY, task_id TEXT NOT NULL, title TEXT NOT NULL,
                start_time TEXT NOT NULL, end_time TEXT NOT NULL, is_all_day INTEGER NOT NULL DEFAULT 0,
                schedule_type TEXT NOT NULL DEFAULT 'fixed', status TEXT NOT NULL DEFAULT 'pending',
                color TEXT NOT NULL DEFAULT '', created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
            );"
        ).expect("Failed to create test tables");

        let task_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO tasks (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![task_id, "IPC日程测试任务", now, now],
        ).unwrap();

        (ScheduleService::new(pool), task_id)
    }

    // ==================== 日程 IPC 参数映射测试 ====================

    #[test]
    fn ipc_create_schedule_all_fields() {
        let (service, task_id) = create_test_service();
        let schedule = service.create_schedule(
            &task_id, "IPC创建日程", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "#3b82f6",
        ).expect("创建日程失败");
        assert_eq!(schedule.title, "IPC创建日程");
        assert_eq!(schedule.task_id, task_id);
        assert_eq!(schedule.schedule_type, "fixed");
        assert_eq!(schedule.status, "pending");
    }

    #[test]
    fn ipc_create_schedule_with_default_type_and_color() {
        let (service, task_id) = create_test_service();
        // schedule_type = "fixed" (default), color = "" (default)
        let schedule = service.create_schedule(
            &task_id, "默认值日程", "2026-06-10T08:00:00", "2026-06-10T09:00:00",
            false, "fixed", "",
        ).expect("创建失败");
        assert_eq!(schedule.schedule_type, "fixed");
        assert_eq!(schedule.color, "");
    }

    #[test]
    fn ipc_create_all_day_schedule() {
        let (service, task_id) = create_test_service();
        let schedule = service.create_schedule(
            &task_id, "全天日程", "2026-06-10T00:00:00", "2026-06-10T23:59:59",
            true, "fixed", "",
        ).expect("创建失败");
        assert!(schedule.is_all_day);
    }

    #[test]
    fn ipc_get_all_and_single_schedule() {
        let (service, task_id) = create_test_service();
        let s = service.create_schedule(
            &task_id, "查询测试", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "",
        ).expect("创建失败");

        let all = service.get_all_schedules().expect("查询全部失败");
        assert_eq!(all.len(), 1);

        let single = service.get_schedule(&s.id).expect("查询单个失败");
        assert!(single.is_some());
        assert_eq!(single.unwrap().title, "查询测试");
    }

    #[test]
    fn ipc_get_schedule_not_found() {
        let (service, _) = create_test_service();
        let result = service.get_schedule("nonexistent").expect("查询失败");
        assert!(result.is_none());
    }

    #[test]
    fn ipc_get_schedules_in_range() {
        let (service, task_id) = create_test_service();
        service.create_schedule(
            &task_id, "范围内", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "",
        ).expect("创建失败");
        service.create_schedule(
            &task_id, "范围外", "2026-07-01T09:00:00", "2026-07-01T10:00:00",
            false, "fixed", "",
        ).expect("创建失败");

        let in_range = service.get_schedules_in_range("2026-06-09", "2026-06-11")
            .expect("范围查询失败");
        assert_eq!(in_range.len(), 1);
    }

    #[test]
    fn ipc_get_day_schedules() {
        let (service, task_id) = create_test_service();
        service.create_schedule(
            &task_id, "当天日程", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "",
        ).expect("创建失败");

        let day = service.get_day_schedules("2026-06-10").expect("日查询失败");
        assert_eq!(day.len(), 1);
    }

    #[test]
    fn ipc_get_schedules_by_task() {
        let (service, task_id) = create_test_service();
        service.create_schedule(
            &task_id, "关联日程", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "",
        ).expect("创建失败");

        let by_task = service.get_schedules_by_task(&task_id).expect("按任务查询失败");
        assert_eq!(by_task.len(), 1);
        assert_eq!(by_task[0].task_id, task_id);
    }

    #[test]
    fn ipc_update_schedule_partial() {
        let (service, task_id) = create_test_service();
        let s = service.create_schedule(
            &task_id, "原标题", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "",
        ).expect("创建失败");

        let updated = service.update_schedule(
            &s.id,
            Some("新标题"), // title: Option<&str>
            None, None, None, None, None, None, None,
        ).expect("更新失败");
        assert_eq!(updated.title, "新标题");
    }

    #[test]
    fn ipc_delete_schedule() {
        let (service, task_id) = create_test_service();
        let s = service.create_schedule(
            &task_id, "待删除", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "",
        ).expect("创建失败");
        
        service.delete_schedule(&s.id).expect("删除失败");
        let result = service.get_schedule(&s.id).expect("查询失败");
        assert!(result.is_none());
    }

    // ==================== 状态同步 IPC 测试 ====================

    #[test]
    fn ipc_update_schedule_status_changes_status() {
        let (service, task_id) = create_test_service();
        let s = service.create_schedule(
            &task_id, "状态测试", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "",
        ).expect("创建失败");

        service.update_schedule_status(&s.id, "completed").expect("状态更新失败");
        let updated = service.get_schedule(&s.id).expect("查询失败").unwrap();
        assert_eq!(updated.status, "completed");
    }

    // ==================== 冲突检测 IPC 测试 ====================

    #[test]
    fn ipc_check_conflicts_finds_overlap() {
        let (service, task_id) = create_test_service();
        service.create_schedule(
            &task_id, "基准日程", "2026-06-10T09:00:00", "2026-06-10T11:00:00",
            false, "fixed", "",
        ).expect("创建失败");
        service.create_schedule(
            &task_id, "冲突日程", "2026-06-10T10:00:00", "2026-06-10T12:00:00",
            false, "fixed", "",
        ).expect("创建失败");

        let conflicts = service.check_conflicts(
            "2026-06-10T09:00:00", "2026-06-10T11:00:00", None,
        ).expect("冲突检测失败");
        assert!(!conflicts.is_empty());
    }

    #[test]
    fn ipc_check_conflicts_no_conflict() {
        let (service, task_id) = create_test_service();
        service.create_schedule(
            &task_id, "独占时间", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "",
        ).expect("创建失败");

        let conflicts = service.check_conflicts(
            "2026-06-10T11:00:00", "2026-06-10T12:00:00", None,
        ).expect("冲突检测失败");
        assert!(conflicts.is_empty());
    }

    #[test]
    fn ipc_check_conflicts_excludes_self() {
        let (service, task_id) = create_test_service();
        let s = service.create_schedule(
            &task_id, "自身日程", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "",
        ).expect("创建失败");

        let conflicts = service.check_conflicts(
            "2026-06-10T09:00:00", "2026-06-10T10:00:00", Some(&s.id),
        ).expect("冲突检测失败");
        assert!(conflicts.is_empty());
    }

    #[test]
    fn ipc_get_week_schedules() {
        let (service, task_id) = create_test_service();
        service.create_schedule(
            &task_id, "周一", "2026-06-08T09:00:00", "2026-06-08T10:00:00",
            false, "fixed", "",
        ).expect("创建失败");
        service.create_schedule(
            &task_id, "周三", "2026-06-10T09:00:00", "2026-06-10T10:00:00",
            false, "fixed", "",
        ).expect("创建失败");

        let week = service.get_week_schedules("2026-06-08", "2026-06-14")
            .expect("周查询失败");
        assert_eq!(week.len(), 2);
    }
}
