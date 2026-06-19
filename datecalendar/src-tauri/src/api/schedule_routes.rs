use actix_web::{web, HttpResponse};
use crate::services::schedule_service::ScheduleService;
use serde::Deserialize;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/api/schedules", web::get().to(get_all_schedules))
        .route("/api/schedules", web::post().to(create_schedule))
        .route("/api/schedules/range", web::get().to(get_schedules_in_range))
        .route("/api/schedules/day/{date}", web::get().to(get_day_schedules))
        .route("/api/schedules/week", web::get().to(get_week_schedules))
        .route("/api/schedules/task/{task_id}", web::get().to(get_schedules_by_task))
        .route("/api/schedules/conflicts", web::get().to(check_conflicts))
        .route("/api/schedules/{id}", web::get().to(get_schedule))
        .route("/api/schedules/{id}", web::put().to(update_schedule))
        .route("/api/schedules/{id}", web::delete().to(delete_schedule))
        .route("/api/schedules/{schedule_id}/status", web::put().to(update_status));
}

// ==================== 日程查询 ====================

async fn get_all_schedules(svc: web::Data<ScheduleService>) -> HttpResponse {
    match svc.get_all_schedules() {
        Ok(schedules) => HttpResponse::Ok().json(schedules),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

async fn get_schedule(svc: web::Data<ScheduleService>, path: web::Path<String>) -> HttpResponse {
    match svc.get_schedule(&path.into_inner()) {
        Ok(schedule) => HttpResponse::Ok().json(schedule),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

#[derive(Deserialize)]
struct RangeQuery {
    start: String,
    end: String,
}

async fn get_schedules_in_range(svc: web::Data<ScheduleService>, query: web::Query<RangeQuery>) -> HttpResponse {
    match svc.get_schedules_in_range(&query.start, &query.end) {
        Ok(schedules) => HttpResponse::Ok().json(schedules),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

async fn get_day_schedules(svc: web::Data<ScheduleService>, path: web::Path<String>) -> HttpResponse {
    match svc.get_day_schedules(&path.into_inner()) {
        Ok(schedules) => HttpResponse::Ok().json(schedules),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

#[derive(Deserialize)]
struct WeekQuery {
    #[serde(rename = "weekStart")]
    week_start: String,
    #[serde(rename = "weekEnd")]
    week_end: String,
}

async fn get_week_schedules(svc: web::Data<ScheduleService>, query: web::Query<WeekQuery>) -> HttpResponse {
    match svc.get_week_schedules(&query.week_start, &query.week_end) {
        Ok(schedules) => HttpResponse::Ok().json(schedules),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

async fn get_schedules_by_task(svc: web::Data<ScheduleService>, path: web::Path<String>) -> HttpResponse {
    match svc.get_schedules_by_task(&path.into_inner()) {
        Ok(schedules) => HttpResponse::Ok().json(schedules),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

// ==================== 日程 CRUD ====================

#[derive(Deserialize)]
struct CreateScheduleInput {
    #[serde(rename = "taskId")]
    task_id: String,
    title: String,
    #[serde(rename = "startTime")]
    start_time: String,
    #[serde(rename = "endTime")]
    end_time: String,
    #[serde(rename = "isAllDay")]
    is_all_day: Option<bool>,
    #[serde(rename = "scheduleType")]
    schedule_type: Option<String>,
    color: Option<String>,
}

async fn create_schedule(svc: web::Data<ScheduleService>, input: web::Json<CreateScheduleInput>) -> HttpResponse {
    let input = input.into_inner();
    match svc.create_schedule(
        &input.task_id,
        &input.title,
        &input.start_time,
        &input.end_time,
        input.is_all_day.unwrap_or(false),
        &input.schedule_type.unwrap_or_else(|| "fixed".to_string()),
        &input.color.unwrap_or_default(),
    ) {
        Ok(schedule) => HttpResponse::Created().json(schedule),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

#[derive(Deserialize)]
struct UpdateScheduleInput {
    title: Option<String>,
    #[serde(rename = "startTime")]
    start_time: Option<String>,
    #[serde(rename = "endTime")]
    end_time: Option<String>,
    #[serde(rename = "isAllDay")]
    is_all_day: Option<bool>,
    #[serde(rename = "scheduleType")]
    schedule_type: Option<String>,
    status: Option<String>,
    color: Option<String>,
    #[serde(rename = "taskId")]
    task_id: Option<String>,
}

async fn update_schedule(svc: web::Data<ScheduleService>, path: web::Path<String>, input: web::Json<UpdateScheduleInput>) -> HttpResponse {
    let input = input.into_inner();
    match svc.update_schedule(
        &path.into_inner(),
        input.title.as_deref(),
        input.start_time.as_deref(),
        input.end_time.as_deref(),
        input.is_all_day,
        input.schedule_type.as_deref(),
        input.status.as_deref(),
        input.color.as_deref(),
        input.task_id.as_deref(),
    ) {
        Ok(schedule) => HttpResponse::Ok().json(schedule),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

async fn delete_schedule(svc: web::Data<ScheduleService>, path: web::Path<String>) -> HttpResponse {
    match svc.delete_schedule(&path.into_inner()) {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

// ==================== 状态同步 ====================

#[derive(Deserialize)]
struct UpdateStatusInput {
    #[serde(rename = "newStatus")]
    new_status: String,
}

async fn update_status(svc: web::Data<ScheduleService>, path: web::Path<String>, input: web::Json<UpdateStatusInput>) -> HttpResponse {
    match svc.update_schedule_status(&path.into_inner(), &input.new_status) {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

// ==================== 冲突检测 ====================

#[derive(Deserialize)]
struct ConflictQuery {
    #[serde(rename = "startTime")]
    start_time: String,
    #[serde(rename = "endTime")]
    end_time: String,
    #[serde(rename = "excludeId")]
    exclude_id: Option<String>,
}

async fn check_conflicts(svc: web::Data<ScheduleService>, query: web::Query<ConflictQuery>) -> HttpResponse {
    match svc.check_conflicts(&query.start_time, &query.end_time, query.exclude_id.as_deref()) {
        Ok(conflicts) => HttpResponse::Ok().json(conflicts),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

// ==================== HTTP API 层白盒测试 ====================
/// 使用 actix_web::test 发送真实 HTTP 请求，验证：
/// - JSON 序列化/反序列化（camelCase ↔ snake_case 转换）
/// - 查询参数（?start=&end=）和路径参数（/{date}）解析
/// - HTTP 状态码
/// - 错误传播

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;
    use uuid::Uuid;
    use chrono::Utc;
    use crate::services::schedule_service::ScheduleService;

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
            rusqlite::params![task_id, "API日程测试任务", now, now],
        ).unwrap();
        drop(conn);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(ScheduleService::new(pool)))
                .configure(configure)
        ).await;

        (app, task_id)
    }

    // ==================== 日程 CRUD HTTP 测试 ====================

    #[actix_web::test]
    async fn api_create_schedule_returns_201_camelcase() {
        let (app, task_id) = create_test_app().await;
        let req = test::TestRequest::post()
            .uri("/api/schedules")
            .set_json(&serde_json::json!({
                "taskId": task_id,
                "title": "API日程",
                "startTime": "2026-06-10T09:00:00",
                "endTime": "2026-06-10T10:00:00",
                "isAllDay": false,
                "scheduleType": "fixed",
                "color": "#3b82f6"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["title"], "API日程");
        assert_eq!(body["task_id"], task_id);
        assert_eq!(body["schedule_type"], "fixed");
        assert_eq!(body["status"], "pending");
    }

    #[actix_web::test]
    async fn api_create_schedule_default_type_and_color() {
        let (app, task_id) = create_test_app().await;
        // 不传 scheduleType 和 color → 使用默认值
        let req = test::TestRequest::post()
            .uri("/api/schedules")
            .set_json(&serde_json::json!({
                "taskId": task_id,
                "title": "默认值",
                "startTime": "2026-06-10T08:00:00",
                "endTime": "2026-06-10T09:00:00"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["schedule_type"], "fixed");  // 默认值
        assert_eq!(body["color"], "");                // 默认值
    }

    #[actix_web::test]
    async fn api_get_all_schedules_returns_200() {
        let (app, task_id) = create_test_app().await;
        // 先创建一条日程
        let req = test::TestRequest::post()
            .uri("/api/schedules")
            .set_json(&serde_json::json!({
                "taskId": task_id,
                "title": "查询日程",
                "startTime": "2026-06-10T09:00:00",
                "endTime": "2026-06-10T10:00:00"
            }))
            .to_request();
        test::call_service(&app, req).await;

        let req = test::TestRequest::get().uri("/api/schedules").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);
    }

    #[actix_web::test]
    async fn api_get_schedule_by_id_returns_200() {
        let (app, task_id) = create_test_app().await;
        let req = test::TestRequest::post()
            .uri("/api/schedules")
            .set_json(&serde_json::json!({
                "taskId": task_id, "title": "单个",
                "startTime": "2026-06-10T09:00:00", "endTime": "2026-06-10T10:00:00"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let s_id = body["id"].as_str().unwrap().to_string();

        let req = test::TestRequest::get().uri(&format!("/api/schedules/{}", s_id)).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["title"], "单个");
    }

    #[actix_web::test]
    async fn api_get_schedule_not_found_returns_null() {
        let (app, _) = create_test_app().await;
        let req = test::TestRequest::get().uri("/api/schedules/nonexistent").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.is_null());
    }

    #[actix_web::test]
    async fn api_update_schedule_partial_returns_200() {
        let (app, task_id) = create_test_app().await;
        let req = test::TestRequest::post()
            .uri("/api/schedules")
            .set_json(&serde_json::json!({
                "taskId": task_id, "title": "原标题",
                "startTime": "2026-06-10T09:00:00", "endTime": "2026-06-10T10:00:00"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let s_id = body["id"].as_str().unwrap().to_string();

        let req = test::TestRequest::put()
            .uri(&format!("/api/schedules/{}", s_id))
            .set_json(&serde_json::json!({"title": "新标题", "status": "completed"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["title"], "新标题");
        assert_eq!(body["status"], "completed");
    }

    #[actix_web::test]
    async fn api_delete_schedule_returns_204() {
        let (app, task_id) = create_test_app().await;
        let req = test::TestRequest::post()
            .uri("/api/schedules")
            .set_json(&serde_json::json!({
                "taskId": task_id, "title": "待删",
                "startTime": "2026-06-10T09:00:00", "endTime": "2026-06-10T10:00:00"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let s_id = body["id"].as_str().unwrap().to_string();

        let req = test::TestRequest::delete().uri(&format!("/api/schedules/{}", s_id)).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);
    }

    // ==================== 查询 HTTP 测试 ====================

    #[actix_web::test]
    async fn api_get_schedules_in_range_with_query_params() {
        let (app, task_id) = create_test_app().await;
        test::call_service(&app, test::TestRequest::post().uri("/api/schedules")
            .set_json(&serde_json::json!({
                "taskId": task_id, "title": "范围内",
                "startTime": "2026-06-10T09:00:00", "endTime": "2026-06-10T10:00:00"
            })).to_request()).await;
        test::call_service(&app, test::TestRequest::post().uri("/api/schedules")
            .set_json(&serde_json::json!({
                "taskId": task_id, "title": "范围外",
                "startTime": "2026-07-01T09:00:00", "endTime": "2026-07-01T10:00:00"
            })).to_request()).await;

        let req = test::TestRequest::get()
            .uri("/api/schedules/range?start=2026-06-09&end=2026-06-11").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);
    }

    #[actix_web::test]
    async fn api_get_day_schedules_with_path_param() {
        let (app, task_id) = create_test_app().await;
        test::call_service(&app, test::TestRequest::post().uri("/api/schedules")
            .set_json(&serde_json::json!({
                "taskId": task_id, "title": "当天",
                "startTime": "2026-06-10T09:00:00", "endTime": "2026-06-10T10:00:00"
            })).to_request()).await;

        let req = test::TestRequest::get().uri("/api/schedules/day/2026-06-10").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);
    }

    #[actix_web::test]
    async fn api_get_week_schedules_with_camelcase_query() {
        let (app, task_id) = create_test_app().await;
        test::call_service(&app, test::TestRequest::post().uri("/api/schedules")
            .set_json(&serde_json::json!({
                "taskId": task_id, "title": "周一",
                "startTime": "2026-06-08T09:00:00", "endTime": "2026-06-08T10:00:00"
            })).to_request()).await;
        test::call_service(&app, test::TestRequest::post().uri("/api/schedules")
            .set_json(&serde_json::json!({
                "taskId": task_id, "title": "周三",
                "startTime": "2026-06-10T09:00:00", "endTime": "2026-06-10T10:00:00"
            })).to_request()).await;

        // 查询参数用 camelCase: weekStart, weekEnd → serde rename 转换
        let req = test::TestRequest::get()
            .uri("/api/schedules/week?weekStart=2026-06-08&weekEnd=2026-06-14").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 2);
    }

    #[actix_web::test]
    async fn api_get_schedules_by_task_with_path_param() {
        let (app, task_id) = create_test_app().await;
        test::call_service(&app, test::TestRequest::post().uri("/api/schedules")
            .set_json(&serde_json::json!({
                "taskId": task_id, "title": "关联",
                "startTime": "2026-06-10T09:00:00", "endTime": "2026-06-10T10:00:00"
            })).to_request()).await;

        let req = test::TestRequest::get().uri(&format!("/api/schedules/task/{}", task_id)).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);
        assert_eq!(body[0]["task_id"], task_id);
    }

    // ==================== 状态同步 HTTP 测试 ====================

    #[actix_web::test]
    async fn api_update_schedule_status_returns_204() {
        let (app, task_id) = create_test_app().await;
        let req = test::TestRequest::post()
            .uri("/api/schedules")
            .set_json(&serde_json::json!({
                "taskId": task_id, "title": "状态测试",
                "startTime": "2026-06-10T09:00:00", "endTime": "2026-06-10T10:00:00"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let s_id = body["id"].as_str().unwrap().to_string();

        // camelCase: newStatus
        let req = test::TestRequest::put()
            .uri(&format!("/api/schedules/{}/status", s_id))
            .set_json(&serde_json::json!({"newStatus": "cancelled"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);

        // 验证状态已更新
        let req = test::TestRequest::get().uri(&format!("/api/schedules/{}", s_id)).to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "cancelled");
    }

    // ==================== 冲突检测 HTTP 测试 ====================

    #[actix_web::test]
    async fn api_check_conflicts_finds_overlap_with_camelcase_query() {
        let (app, task_id) = create_test_app().await;
        // 基线日程 和 冲突日程
        test::call_service(&app, test::TestRequest::post().uri("/api/schedules")
            .set_json(&serde_json::json!({
                "taskId": task_id, "title": "基准",
                "startTime": "2026-06-10T09:00:00", "endTime": "2026-06-10T11:00:00"
            })).to_request()).await;
        test::call_service(&app, test::TestRequest::post().uri("/api/schedules")
            .set_json(&serde_json::json!({
                "taskId": task_id, "title": "冲突",
                "startTime": "2026-06-10T10:00:00", "endTime": "2026-06-10T12:00:00"
            })).to_request()).await;

        // 查询参数用 camelCase: startTime, endTime → serde rename 转换
        let req = test::TestRequest::get()
            .uri("/api/schedules/conflicts?startTime=2026-06-10T09%3A00%3A00&endTime=2026-06-10T11%3A00%3A00")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert!(!body.is_empty());
    }

    #[actix_web::test]
    async fn api_check_conflicts_no_conflict_returns_empty() {
        let (app, task_id) = create_test_app().await;
        test::call_service(&app, test::TestRequest::post().uri("/api/schedules")
            .set_json(&serde_json::json!({
                "taskId": task_id, "title": "独占",
                "startTime": "2026-06-10T09:00:00", "endTime": "2026-06-10T10:00:00"
            })).to_request()).await;

        let req = test::TestRequest::get()
            .uri("/api/schedules/conflicts?startTime=2026-06-10T11%3A00%3A00&endTime=2026-06-10T12%3A00%3A00")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert!(body.is_empty());
    }
}
