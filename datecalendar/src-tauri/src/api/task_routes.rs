use actix_web::{web, HttpResponse};
use crate::db::models::NewTask;
use crate::services::task_service::TaskService;
use serde::Deserialize;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/api/tasks", web::get().to(get_all_tasks))
        .route("/api/tasks", web::post().to(create_task))
        .route("/api/tasks/reorder", web::put().to(reorder_task))
        .route("/api/tasks/batch/status", web::put().to(batch_update_status))
        .route("/api/tasks/batch/delete", web::post().to(batch_delete))
        .route("/api/tasks/batch/move", web::put().to(batch_move))
        .route("/api/tasks/search", web::get().to(search_tasks))
        .route("/api/tasks/{id}", web::get().to(get_task))
        .route("/api/tasks/{id}", web::put().to(update_task))
        .route("/api/tasks/{id}", web::delete().to(delete_task))
        .route("/api/tasks/{task_id}/risks", web::get().to(get_risks))
        .route("/api/tasks/{task_id}/risks", web::post().to(add_risk))
        .route("/api/tasks/{task_id}/notes", web::get().to(get_notes))
        .route("/api/tasks/{task_id}/notes", web::put().to(save_note))
        .route("/api/risks/{risk_id}", web::delete().to(delete_risk))
        .route("/api/notes/{note_id}", web::delete().to(delete_note));
}

// ==================== 任务 CRUD ====================

async fn get_all_tasks(svc: web::Data<TaskService>) -> HttpResponse {
    match svc.get_all_tasks() {
        Ok(tasks) => HttpResponse::Ok().json(tasks),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

async fn get_task(svc: web::Data<TaskService>, path: web::Path<String>) -> HttpResponse {
    match svc.get_task(&path.into_inner()) {
        Ok(task) => HttpResponse::Ok().json(task),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

async fn create_task(svc: web::Data<TaskService>, input: web::Json<NewTask>) -> HttpResponse {
    match svc.create_task(input.into_inner()) {
        Ok(task) => HttpResponse::Created().json(task),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

#[derive(Deserialize)]
struct UpdateTaskInput {
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    priority: Option<i32>,
    color: Option<String>,
    #[serde(rename = "isMilestone")]
    is_milestone: Option<bool>,
    #[serde(rename = "parentId")]
    parent_id: Option<Option<String>>,
    #[serde(rename = "sortOrder")]
    sort_order: Option<i32>,
}

async fn update_task(svc: web::Data<TaskService>, path: web::Path<String>, input: web::Json<UpdateTaskInput>) -> HttpResponse {
    let id = path.into_inner();
    let input = input.into_inner();
    let parent_id: Option<Option<&str>> = input.parent_id.as_ref().map(|p| p.as_deref());
    match svc.update_task(
        &id,
        input.title.as_deref(),
        input.description.as_deref(),
        input.status.as_deref(),
        input.priority,
        input.color.as_deref(),
        input.is_milestone,
        parent_id,
        input.sort_order,
    ) {
        Ok(task) => HttpResponse::Ok().json(task),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

async fn delete_task(svc: web::Data<TaskService>, path: web::Path<String>) -> HttpResponse {
    match svc.delete_task(&path.into_inner()) {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

// ==================== 搜索 ====================

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

async fn search_tasks(svc: web::Data<TaskService>, query: web::Query<SearchQuery>) -> HttpResponse {
    match svc.search_tasks(&query.q) {
        Ok(tasks) => HttpResponse::Ok().json(tasks),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

// ==================== 风险 ====================

async fn get_risks(svc: web::Data<TaskService>, path: web::Path<String>) -> HttpResponse {
    match svc.get_risks(&path.into_inner()) {
        Ok(risks) => HttpResponse::Ok().json(risks),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

#[derive(Deserialize)]
struct AddRiskInput {
    #[serde(rename = "riskDesc")]
    risk_desc: String,
    probability: Option<String>,
    mitigation: Option<String>,
}

async fn add_risk(svc: web::Data<TaskService>, path: web::Path<String>, input: web::Json<AddRiskInput>) -> HttpResponse {
    let input = input.into_inner();
    match svc.add_risk(
        &path.into_inner(),
        &input.risk_desc,
        &input.probability.unwrap_or_else(|| "medium".to_string()),
        &input.mitigation.unwrap_or_default(),
    ) {
        Ok(risk) => HttpResponse::Created().json(risk),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

async fn delete_risk(svc: web::Data<TaskService>, path: web::Path<String>) -> HttpResponse {
    match svc.delete_risk(&path.into_inner()) {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

// ==================== 笔记 ====================

async fn get_notes(svc: web::Data<TaskService>, path: web::Path<String>) -> HttpResponse {
    match svc.get_notes(&path.into_inner()) {
        Ok(notes) => HttpResponse::Ok().json(notes),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

#[derive(Deserialize)]
struct SaveNoteInput {
    #[serde(rename = "noteId")]
    note_id: Option<String>,
    title: String,
    content: String,
}

async fn save_note(svc: web::Data<TaskService>, path: web::Path<String>, input: web::Json<SaveNoteInput>) -> HttpResponse {
    let input = input.into_inner();
    match svc.save_note(
        &path.into_inner(),
        input.note_id.as_deref(),
        &input.title,
        &input.content,
    ) {
        Ok(note) => HttpResponse::Ok().json(note),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

async fn delete_note(svc: web::Data<TaskService>, path: web::Path<String>) -> HttpResponse {
    match svc.delete_note(&path.into_inner()) {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

// ==================== 排序与批量 ====================

#[derive(Deserialize)]
struct ReorderInput {
    #[serde(rename = "taskId")]
    task_id: String,
    #[serde(rename = "newParentId")]
    new_parent_id: Option<String>,
    #[serde(rename = "newSortOrder")]
    new_sort_order: i32,
}

async fn reorder_task(svc: web::Data<TaskService>, input: web::Json<ReorderInput>) -> HttpResponse {
    let input = input.into_inner();
    match svc.reorder_task(&input.task_id, input.new_parent_id.as_deref(), input.new_sort_order) {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

#[derive(Deserialize)]
struct BatchStatusInput {
    ids: Vec<String>,
    status: String,
}

async fn batch_update_status(svc: web::Data<TaskService>, input: web::Json<BatchStatusInput>) -> HttpResponse {
    let input = input.into_inner();
    match svc.batch_update_status(&input.ids, &input.status) {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

#[derive(Deserialize)]
struct BatchIdsInput {
    ids: Vec<String>,
}

async fn batch_delete(svc: web::Data<TaskService>, input: web::Json<BatchIdsInput>) -> HttpResponse {
    match svc.batch_delete(&input.ids) {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

#[derive(Deserialize)]
struct BatchMoveInput {
    ids: Vec<String>,
    #[serde(rename = "newParentId")]
    new_parent_id: Option<String>,
}

async fn batch_move(svc: web::Data<TaskService>, input: web::Json<BatchMoveInput>) -> HttpResponse {
    let input = input.into_inner();
    match svc.batch_move(&input.ids, input.new_parent_id.as_deref()) {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

// ==================== HTTP API 层白盒测试 ====================
/// 使用 actix_web::test 发送真实 HTTP 请求，验证：
/// - JSON 序列化/反序列化（camelCase ↔ snake_case 转换）
/// - HTTP 状态码
/// - 路径参数、查询参数、请求体解析
/// - 错误传播

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;
    use uuid::Uuid;
    use chrono::Utc;
    use crate::services::task_service::TaskService;

    /// 创建测试用 App（内存数据库 + 表初始化 + 注册路由）
    async fn create_test_app() -> (impl actix_web::dev::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >, String) {
        let manager = SqliteConnectionManager::file(":memory:");
        let pool = Pool::builder()
            .max_size(2) // 避免嵌套 pool.get() 死锁
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
            rusqlite::params![task_id, "APITestTask", now, now],
        ).unwrap();
        drop(conn);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(TaskService::new(pool)))
                .configure(configure)
        ).await;

        (app, task_id)
    }

    // ==================== 任务 CRUD HTTP 测试 ====================

    #[actix_web::test]
    async fn api_get_all_tasks_returns_200() {
        let (app, _) = create_test_app().await;
        let req = test::TestRequest::get().uri("/api/tasks").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);
    }

    #[actix_web::test]
    async fn api_get_task_by_id_returns_200() {
        let (app, task_id) = create_test_app().await;
        let req = test::TestRequest::get().uri(&format!("/api/tasks/{}", task_id)).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["title"], "APITestTask");
    }

    #[actix_web::test]
    async fn api_get_task_not_found_returns_null() {
        let (app, _) = create_test_app().await;
        let req = test::TestRequest::get().uri("/api/tasks/nonexistent").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.is_null());
    }

    #[actix_web::test]
    async fn api_create_task_returns_201_with_camelcase_input() {
        let (app, _) = create_test_app().await;
        // 用 camelCase JSON（前端浏览器模式发送的格式）
        let req = test::TestRequest::post()
            .uri("/api/tasks")
            .set_json(&serde_json::json!({
                "title": "HTTP创建",
                "priority": 3,
                "isMilestone": true,
                "description": "通过HTTP API"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["title"], "HTTP创建");
        assert_eq!(body["priority"], 3);
        assert!(body["is_milestone"].as_bool().unwrap());
        assert_eq!(body["status"], "pending");
    }

    #[actix_web::test]
    async fn api_create_task_with_parent_uses_parentid() {
        let (app, task_id) = create_test_app().await;
        // parentId = snake_case: parent_id
        let req = test::TestRequest::post()
            .uri("/api/tasks")
            .set_json(&serde_json::json!({
                "title": "子任务",
                "parentId": task_id
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["parent_id"], task_id);
    }

    #[actix_web::test]
    async fn api_update_task_returns_200_camelcase() {
        let (app, task_id) = create_test_app().await;
        // JSON key 用 camelCase → serde rename 转换为 snake_case
        let req = test::TestRequest::put()
            .uri(&format!("/api/tasks/{}", task_id))
            .set_json(&serde_json::json!({
                "title": "已更新",
                "status": "in_progress",
                "priority": 1
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["title"], "已更新");
        assert_eq!(body["status"], "in_progress");
    }

    #[actix_web::test]
    async fn api_delete_task_returns_204() {
        let (app, task_id) = create_test_app().await;
        let req = test::TestRequest::delete().uri(&format!("/api/tasks/{}", task_id)).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);
    }

    // ==================== 搜索 HTTP 测试 ====================

    #[actix_web::test]
    async fn api_search_tasks_with_query_param() {
        let (app, _) = create_test_app().await;
        let req = test::TestRequest::get().uri("/api/tasks/search?q=APITest").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);
    }

    #[actix_web::test]
    async fn api_search_tasks_no_match_returns_empty() {
        let (app, _) = create_test_app().await;
        let req = test::TestRequest::get().uri("/api/tasks/search?q=NoSuchKeyword").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert!(body.is_empty());
    }

    // ==================== 风险 HTTP 测试 ====================

    #[actix_web::test]
    async fn api_add_risk_with_camelcase_json() {
        let (app, task_id) = create_test_app().await;
        let req = test::TestRequest::post()
            .uri(&format!("/api/tasks/{}/risks", task_id))
            .set_json(&serde_json::json!({
                "riskDesc": "进度风险",
                "probability": "high",
                "mitigation": "加人"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["risk_desc"], "进度风险");
        assert_eq!(body["probability"], "high");
    }

    #[actix_web::test]
    async fn api_get_and_delete_risk_roundtrip() {
        let (app, task_id) = create_test_app().await;
        // 先创建 risk
        let req = test::TestRequest::post()
            .uri(&format!("/api/tasks/{}/risks", task_id))
            .set_json(&serde_json::json!({"riskDesc": "测试风险"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let risk_id = body["id"].as_str().unwrap().to_string();

        // 查风险列表
        let req = test::TestRequest::get().uri(&format!("/api/tasks/{}/risks", task_id)).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let risks: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert_eq!(risks.len(), 1);

        // 删除风险
        let req = test::TestRequest::delete().uri(&format!("/api/risks/{}", risk_id)).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);
    }

    // ==================== 笔记 HTTP 测试 ====================

    #[actix_web::test]
    async fn api_save_note_create_new() {
        let (app, task_id) = create_test_app().await;
        // noteId = None → 创建新笔记
        let req = test::TestRequest::put()
            .uri(&format!("/api/tasks/{}/notes", task_id))
            .set_json(&serde_json::json!({
                "title": "API笔记",
                "content": "HTTP层测试内容"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["title"], "API笔记");
        assert_eq!(body["content"], "HTTP层测试内容");
        assert_eq!(body["task_id"], task_id);
    }

    #[actix_web::test]
    async fn api_save_note_update_existing_with_noteid() {
        let (app, task_id) = create_test_app().await;
        // 先创建笔记
        let req = test::TestRequest::put()
            .uri(&format!("/api/tasks/{}/notes", task_id))
            .set_json(&serde_json::json!({"title": "原始", "content": "旧"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let note_id = body["id"].as_str().unwrap().to_string();

        // 更新已有笔记 (noteId 不为 None)
        let req = test::TestRequest::put()
            .uri(&format!("/api/tasks/{}/notes", task_id))
            .set_json(&serde_json::json!({
                "noteId": note_id,
                "title": "已修改",
                "content": "新内容"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["id"], note_id);
        assert_eq!(body["title"], "已修改");
    }

    #[actix_web::test]
    async fn api_delete_note_returns_204() {
        let (app, task_id) = create_test_app().await;
        let req = test::TestRequest::put()
            .uri(&format!("/api/tasks/{}/notes", task_id))
            .set_json(&serde_json::json!({"title": "待删", "content": ""}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let note_id = body["id"].as_str().unwrap().to_string();

        let req = test::TestRequest::delete().uri(&format!("/api/notes/{}", note_id)).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);
    }

    // ==================== 批量操作 HTTP 测试 ====================

    #[actix_web::test]
    async fn api_batch_update_status_returns_204() {
        let (app, task_id) = create_test_app().await;
        let req = test::TestRequest::put()
            .uri("/api/tasks/batch/status")
            .set_json(&serde_json::json!({
                "ids": [task_id],
                "status": "completed"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);

        // 验证状态已更新
        let req = test::TestRequest::get().uri(&format!("/api/tasks/{}", task_id)).to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "completed");
    }

    #[actix_web::test]
    async fn api_batch_move_with_newparentid() {
        let (app, task_id) = create_test_app().await;
        // 创建第二个任务作为子节点
        let req = test::TestRequest::post()
            .uri("/api/tasks")
            .set_json(&serde_json::json!({"title": "待移动"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let child_id = body["id"].as_str().unwrap().to_string();

        // 批量移动到父任务下 (camelCase: newParentId)
        let req = test::TestRequest::put()
            .uri("/api/tasks/batch/move")
            .set_json(&serde_json::json!({
                "ids": [child_id],
                "newParentId": task_id
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);

        // 验证子任务已移动
        let req = test::TestRequest::get().uri(&format!("/api/tasks/{}", child_id)).to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["parent_id"], task_id);
    }

    #[actix_web::test]
    async fn api_reorder_task_with_camelcase_input() {
        let (app, _) = create_test_app().await;
        // 创建第二个任务
        let req = test::TestRequest::post()
            .uri("/api/tasks")
            .set_json(&serde_json::json!({"title": "重排任务"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let task2_id = body["id"].as_str().unwrap().to_string();

        let req = test::TestRequest::put()
            .uri("/api/tasks/reorder")
            .set_json(&serde_json::json!({
                "taskId": task2_id,
                "newSortOrder": 0
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);
    }
}
