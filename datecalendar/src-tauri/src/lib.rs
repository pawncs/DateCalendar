mod db;
mod services;
mod commands;

use db::connection::init_pool;
use db::migrations::run_migrations;
use services::task_service::TaskService;
use tauri::Manager;

/// DateCalendar Tauri 应用入口
///
/// 初始化流程：
/// 1. 解析应用数据目录
/// 2. 初始化 SQLite 连接池
/// 3. 执行数据库迁移
/// 4. 创建服务实例并注入 Tauri 状态
/// 5. 注册 IPC 命令
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // 初始化日志（debug 模式下）
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // 获取应用数据目录
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("无法获取应用数据目录");

            // 初始化数据库连接池并执行迁移
            let pool = init_pool(app_data_dir)
                .expect("数据库初始化失败");

            {
                let conn = pool.get().expect("无法获取数据库连接");
                run_migrations(&conn).expect("数据库迁移失败");
            }

            // 创建服务实例，注入 Tauri 状态管理
            let task_service = TaskService::new(pool);
            app.manage(task_service);

            log::info!("DateCalendar 初始化完成");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::task_commands::get_all_tasks,
            commands::task_commands::get_task,
            commands::task_commands::create_task,
            commands::task_commands::update_task,
            commands::task_commands::delete_task,
            commands::task_commands::search_tasks,
            commands::task_commands::get_risks,
            commands::task_commands::add_risk,
            commands::task_commands::delete_risk,
            commands::task_commands::get_notes,
            commands::task_commands::save_note,
            commands::task_commands::delete_note,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
