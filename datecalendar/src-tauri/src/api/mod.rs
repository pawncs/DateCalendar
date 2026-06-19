pub mod task_routes;
pub mod schedule_routes;

use actix_cors::Cors;
use actix_web::{web, HttpResponse};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use crate::services::task_service::TaskService;
use crate::services::schedule_service::ScheduleService;

/// 启动 HTTP API 服务器（后台线程）
pub fn start_api_server(pool: Pool<SqliteConnectionManager>) {
    std::thread::spawn(move || {
        let task_service = web::Data::new(TaskService::new(pool.clone()));
        let schedule_service = web::Data::new(ScheduleService::new(pool));

        let server = actix_web::HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600);

            actix_web::App::new()
                .wrap(cors)
                .app_data(task_service.clone())
                .app_data(schedule_service.clone())
                // 健康检查
                .route("/api/health", web::get().to(health))
                // 任务路由
                .configure(task_routes::configure)
                // 日程路由
                .configure(schedule_routes::configure)
        })
        .bind("127.0.0.1:9876")
        .expect("HTTP API 绑定 127.0.0.1:9876 失败")
        .run();

        log::info!("HTTP API 已启动: http://127.0.0.1:9876");

        actix_web::rt::System::new().block_on(server)
            .expect("HTTP API 服务器运行失败");
    });
}

async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}
