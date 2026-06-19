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

// ==================== Tauri IPC 命令层白盒测试 ====================
///
/// 测试策略：
/// - 直接实例化 TaskService（使用内存数据库），绕过 Tauri State 机制
/// - 逐条验证命令的参数转换逻辑（Option 解包、默认值、引用转换）
/// - 与 Service 层测试互补：Service 测 SQL 逻辑，Commands 测参数映射

#[cfg(test)]
mod tests {
    use super::*;
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;
    use uuid::Uuid;
    use chrono::Utc;
    use rusqlite;

    /// 创建测试用 TaskService（内存数据库 + 表初始化 + 预置一条任务）
    fn create_test_service() -> (TaskService, String) {
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
            CREATE TABLE IF NOT EXISTS milestone_risks (
                id TEXT PRIMARY KEY, task_id TEXT NOT NULL,
                risk_desc TEXT NOT NULL, probability TEXT NOT NULL DEFAULT 'medium',
                mitigation TEXT NOT NULL DEFAULT '', created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY, task_id TEXT NOT NULL,
                title TEXT NOT NULL, content TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
            );"
        ).expect("Failed to create test tables");

        let task_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO tasks (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![task_id, "IPC测试任务", now, now],
        ).unwrap();

        (TaskService::new(pool), task_id)
    }

    // ==================== 任务 IPC 参数映射测试 ====================

    #[test]
    fn ipc_get_all_tasks_returns_list() {
        let (service, _) = create_test_service();
        let result = service.get_all_tasks().expect("获取任务列表失败");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "IPC测试任务");
    }

    #[test]
    fn ipc_get_task_returns_some_for_valid_id() {
        let (service, task_id) = create_test_service();
        let result = service.get_task(&task_id).expect("获取任务失败");
        assert!(result.is_some());
    }

    #[test]
    fn ipc_get_task_returns_none_for_missing_id() {
        let (service, _) = create_test_service();
        let result = service.get_task("nonexistent-id").expect("查询失败");
        assert!(result.is_none());
    }

    #[test]
    fn ipc_create_task_with_all_fields() {
        let (service, _) = create_test_service();
        let input = NewTask {
            parent_id: None,
            title: "IPC命令创建".to_string(),
            description: Some("通过IPC".to_string()),
            priority: Some(3),
            color: Some("#00ff00".to_string()),
            is_milestone: Some(true),
        };
        let task = service.create_task(input).expect("创建失败");
        assert_eq!(task.title, "IPC命令创建");
        assert_eq!(task.priority, 3);
        assert_eq!(task.color, "#00ff00");
        assert!(task.is_milestone);
        assert_eq!(task.status, "pending");
    }

    #[test]
    fn ipc_update_task_partial_fields() {
        let (service, task_id) = create_test_service();
        // 只更新 title 和 status，其余为 None
        let task = service.update_task(
            &task_id,
            Some("更新标题"),  // title
            None,              // description
            Some("in_progress"), // status
            None,              // priority
            None,              // color
            None,              // is_milestone
            None,              // parent_id
            None,              // sort_order
        ).expect("更新失败");
        assert_eq!(task.title, "更新标题");
        assert_eq!(task.status, "in_progress");
        // 未改变的字段保持原样
        assert_eq!(task.priority, 0);
    }

    #[test]
    fn ipc_update_task_change_parent() {
        let (service, task_id) = create_test_service();
        // 先创建父任务
        let parent = service.create_task(NewTask {
            parent_id: None,
            title: "父任务".to_string(),
            description: None,
            priority: None,
            color: None,
            is_milestone: None,
        }).expect("创建父任务失败");

        // 将任务移到父任务下 (parent_id = Some(Some("...")))
        let task = service.update_task(
            &task_id,
            None, None, None, None, None, None,
            Some(Some(parent.id.as_str())), // parent_id 的参数转换: Option<Option<&str>>
            None,
        ).expect("移动失败");
        assert_eq!(task.parent_id, Some(parent.id));
    }

    #[test]
    fn ipc_update_task_clear_parent() {
        let (service, task_id) = create_test_service();
        // 先设 parent_id 再清除 (parent_id = Some(None) → unwrap parent)
        let task = service.update_task(
            &task_id,
            None, None, None, None, None, None,
            Some(None), // 清除父任务
            None,
        ).expect("清除父任务失败");
        assert_eq!(task.parent_id, None);
    }

    #[test]
    fn ipc_delete_task_removes_from_list() {
        let (service, task_id) = create_test_service();
        service.delete_task(&task_id).expect("删除失败");
        let result = service.get_task(&task_id).expect("查询失败");
        assert!(result.is_none());
    }

    #[test]
    fn ipc_search_tasks_finds_match() {
        let (service, _) = create_test_service();
        let results = service.search_tasks("IPC测试").expect("搜索失败");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn ipc_search_tasks_no_match() {
        let (service, _) = create_test_service();
        let results = service.search_tasks("不存在的关键词").expect("搜索失败");
        assert!(results.is_empty());
    }

    // ==================== 风险 IPC 参数映射测试 ====================

    #[test]
    fn ipc_add_risk_with_defaults() {
        let (service, task_id) = create_test_service();
        let risk = service.add_risk(
            &task_id, "进度延期", "medium", "",
        ).expect("添加风险失败");
        assert_eq!(risk.risk_desc, "进度延期");
        assert_eq!(risk.probability, "medium");
    }

    #[test]
    fn ipc_add_risk_with_custom_probability() {
        let (service, task_id) = create_test_service();
        let risk = service.add_risk(
            &task_id, "高风险项", "high", "加人处理",
        ).expect("添加风险失败");
        assert_eq!(risk.probability, "high");
        assert_eq!(risk.mitigation, "加人处理");
    }

    #[test]
    fn ipc_get_and_delete_risk_roundtrip() {
        let (service, task_id) = create_test_service();
        let risk = service.add_risk(&task_id, "测试风险", "low", "").expect("添加失败");
        
        let risks = service.get_risks(&task_id).expect("查询失败");
        assert_eq!(risks.len(), 1);
        
        service.delete_risk(&risk.id).expect("删除失败");
        let after = service.get_risks(&task_id).expect("查询失败");
        assert!(after.is_empty());
    }

    // ==================== 笔记 IPC 参数映射测试 ====================

    #[test]
    fn ipc_save_note_create_new() {
        let (service, task_id) = create_test_service();
        // note_id = None → 创建新笔记
        let note = service.save_note(&task_id, None, "会议纪要", "讨论下期计划")
            .expect("保存笔记失败");
        assert_eq!(note.title, "会议纪要");
        assert_eq!(note.content, "讨论下期计划");
        assert_eq!(note.task_id, task_id);
    }

    #[test]
    fn ipc_save_note_update_existing() {
        let (service, task_id) = create_test_service();
        let note = service.save_note(&task_id, None, "草稿", "旧内容").expect("创建失败");
        
        // note_id = Some(...) → 更新已有笔记
        let updated = service.save_note(&task_id, Some(&note.id), "已修改", "新内容")
            .expect("更新失败");
        assert_eq!(updated.id, note.id);
        assert_eq!(updated.title, "已修改");
        assert_eq!(updated.content, "新内容");
    }

    #[test]
    fn ipc_delete_note_removes_it() {
        let (service, task_id) = create_test_service();
        let note = service.save_note(&task_id, None, "待删除", "").expect("创建失败");
        service.delete_note(&note.id).expect("删除失败");
        
        let notes = service.get_notes(&task_id).expect("查询失败");
        assert!(notes.is_empty());
    }

    // ==================== 排序 IPC 参数映射测试 ====================

    #[test]
    fn ipc_reorder_task_same_level() {
        let (service, _) = create_test_service();
        // 创建第二个任务
        let task2 = service.create_task(NewTask {
            parent_id: None, title: "任务2".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).expect("创建失败");
        
        // 重排序（同级移动）
        service.reorder_task(&task2.id, None, 0).expect("重排序失败");
        
        let tasks = service.get_all_tasks().expect("查询失败");
        let t = tasks.iter().find(|t| t.id == task2.id).unwrap();
        assert_eq!(t.sort_order, 0);
    }

    // ==================== 批量操作 IPC 参数映射测试 ====================

    #[test]
    fn ipc_batch_update_status() {
        let (service, task_id) = create_test_service();
        let task2 = service.create_task(NewTask {
            parent_id: None, title: "任务2".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).expect("创建失败");
        
        service.batch_update_status(&[task_id.clone(), task2.id.clone()], "completed")
            .expect("批量更新失败");
        
        let t1 = service.get_task(&task_id).expect("查询失败").unwrap();
        let t2 = service.get_task(&task2.id).expect("查询失败").unwrap();
        assert_eq!(t1.status, "completed");
        assert_eq!(t2.status, "completed");
    }

    #[test]
    fn ipc_batch_delete_removes_all() {
        let (service, task_id) = create_test_service();
        let task2 = service.create_task(NewTask {
            parent_id: None, title: "任务2".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).expect("创建失败");
        
        let ids = vec![task_id, task2.id];
        service.batch_delete(&ids).expect("批量删除失败");
        
        let remaining = service.get_all_tasks().expect("查询失败");
        assert!(remaining.is_empty());
    }

    #[test]
    fn ipc_batch_move_to_parent() {
        let (service, _) = create_test_service();
        // 创建目标父任务
        let parent = service.create_task(NewTask {
            parent_id: None, title: "父节点".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).expect("创建父任务失败");
        
        // 创建两个子任务
        let child1 = service.create_task(NewTask {
            parent_id: None, title: "子1".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).expect("创建失败");
        let child2 = service.create_task(NewTask {
            parent_id: None, title: "子2".to_string(),
            description: None, priority: None, color: None, is_milestone: None,
        }).expect("创建失败");
        
        service.batch_move(&[child1.id.clone(), child2.id.clone()], Some(&parent.id))
            .expect("批量移动失败");
        
        let c1 = service.get_task(&child1.id).expect("查询失败").unwrap();
        let c2 = service.get_task(&child2.id).expect("查询失败").unwrap();
        assert_eq!(c1.parent_id, Some(parent.id.clone()));
        assert_eq!(c2.parent_id, Some(parent.id));
    }

    #[test]
    fn ipc_batch_move_empty_ids_is_noop() {
        let (service, _) = create_test_service();
        let result = service.batch_move(&[], None);
        assert!(result.is_ok());
    }
}
