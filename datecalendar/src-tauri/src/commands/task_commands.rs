use tauri::State;
use crate::db::models::{Task, NewTask, MilestoneRisk, Note};
use crate::services::task_service::TaskService;

// ==================== 任务命令 ====================

#[tauri::command]
pub fn get_all_tasks(service: State<'_, TaskService>) -> Result<Vec<Task>, String> {
    service.get_all_tasks()
}

#[tauri::command]
pub fn get_task(service: State<'_, TaskService>, id: String) -> Result<Option<Task>, String> {
    service.get_task(&id)
}

#[tauri::command]
pub fn create_task(service: State<'_, TaskService>, input: NewTask) -> Result<Task, String> {
    service.create_task(input)
}

#[tauri::command]
pub fn update_task(
    service: State<'_, TaskService>,
    id: String,
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    priority: Option<i32>,
    color: Option<String>,
    is_milestone: Option<bool>,
    parent_id: Option<Option<String>>,
    sort_order: Option<i32>,
) -> Result<Task, String> {
    service.update_task(
        &id,
        title.as_deref(),
        description.as_deref(),
        status.as_deref(),
        priority,
        color.as_deref(),
        is_milestone,
        parent_id.as_ref().map(|p| p.as_deref()),
        sort_order,
    )
}

#[tauri::command]
pub fn delete_task(service: State<'_, TaskService>, id: String) -> Result<(), String> {
    service.delete_task(&id)
}

#[tauri::command]
pub fn search_tasks(service: State<'_, TaskService>, query: String) -> Result<Vec<Task>, String> {
    service.search_tasks(&query)
}

// ==================== 里程碑风险命令 ====================

#[tauri::command]
pub fn get_risks(service: State<'_, TaskService>, task_id: String) -> Result<Vec<MilestoneRisk>, String> {
    service.get_risks(&task_id)
}

#[tauri::command]
pub fn add_risk(
    service: State<'_, TaskService>,
    task_id: String,
    risk_desc: String,
    probability: Option<String>,
    mitigation: Option<String>,
) -> Result<MilestoneRisk, String> {
    service.add_risk(
        &task_id,
        &risk_desc,
        &probability.unwrap_or_else(|| "medium".to_string()),
        &mitigation.unwrap_or_default(),
    )
}

#[tauri::command]
pub fn delete_risk(service: State<'_, TaskService>, risk_id: String) -> Result<(), String> {
    service.delete_risk(&risk_id)
}

// ==================== 笔记命令 ====================

#[tauri::command]
pub fn get_notes(service: State<'_, TaskService>, task_id: String) -> Result<Vec<Note>, String> {
    service.get_notes(&task_id)
}

#[tauri::command]
pub fn save_note(
    service: State<'_, TaskService>,
    task_id: String,
    note_id: Option<String>,
    title: String,
    content: String,
) -> Result<Note, String> {
    service.save_note(&task_id, note_id.as_deref(), &title, &content)
}

#[tauri::command]
pub fn delete_note(service: State<'_, TaskService>, note_id: String) -> Result<(), String> {
    service.delete_note(&note_id)
}

// ==================== 排序命令 ====================

#[tauri::command]
pub fn reorder_task(
    service: State<'_, TaskService>,
    task_id: String,
    new_parent_id: Option<String>,
    new_sort_order: i32,
) -> Result<(), String> {
    service.reorder_task(&task_id, new_parent_id.as_deref(), new_sort_order)
}

// ==================== 批量操作命令 ====================

#[tauri::command]
pub fn batch_update_tasks(
    service: State<'_, TaskService>,
    ids: Vec<String>,
    status: String,
) -> Result<(), String> {
    service.batch_update_status(&ids, &status)
}

#[tauri::command]
pub fn batch_delete_tasks(
    service: State<'_, TaskService>,
    ids: Vec<String>,
) -> Result<(), String> {
    service.batch_delete(&ids)
}

#[tauri::command]
pub fn batch_move_tasks(
    service: State<'_, TaskService>,
    ids: Vec<String>,
    new_parent_id: Option<String>,
) -> Result<(), String> {
    service.batch_move(&ids, new_parent_id.as_deref())
}
