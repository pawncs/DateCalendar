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
