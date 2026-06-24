pub mod db;
pub mod services;
mod commands;
mod api;
mod floating_window;
mod global_hotkey;

use db::connection::init_pool;
use db::migrations::run_migrations;
use services::task_service::TaskService;
use services::schedule_service::ScheduleService;
use tauri::Manager;
use tauri::Emitter;
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState};
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};

/// DateCalendar Tauri 应用入口
///
/// 初始化流程：
/// 1. 解析应用数据目录
/// 2. 初始化 SQLite 连接池
/// 3. 执行数据库迁移
/// 4. 创建服务实例并注入 Tauri 状态
/// 5. 创建悬浮窗（D-13）
/// 6. 创建系统托盘（D-17）
/// 7. 注册 IPC 命令
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
            let task_service = TaskService::new(pool.clone());
            let schedule_service = ScheduleService::new(pool.clone());
            app.manage(task_service);
            app.manage(schedule_service);

            // 启动 HTTP API 服务器（后台线程，供浏览器前端调用）
            api::start_api_server(pool);

            // === D-13: 创建悬浮窗 ===
            match floating_window::create_floating_window(&app.handle()) {
                Ok(_) => log::info!("悬浮窗创建成功"),
                Err(e) => log::error!("悬浮窗创建失败: {}", e),
            }

            // === D-17: 创建系统托盘 ===
            create_system_tray(app.handle())?;

            // === D-15: 注册全局热键插件 ===
            // 使用 Builder 模式初始化插件，并在 with_handler 中设置热键事件处理
            match app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, shortcut, event| {
                        if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                            let shortcut_str = shortcut.to_string();
                            match shortcut_str.as_str() {
                                "Ctrl+Shift+D" => {
                                    log::info!("热键触发: Ctrl+Shift+D");
                                    // 直接操作窗口位置（不依赖前端事件）  
                                    let _ = floating_window::do_toggle_floating(app);
                                    // 同步发射事件给前端更新内部状态
                                    app.emit_to("floating", "floating:toggle", ()).ok();
                                }
                                "Ctrl+Shift+T" => {
                                    log::info!("热键触发: Ctrl+Shift+T");
                                    app.emit_to("floating", "floating:cycle_transparency", ()).ok();
                                }
                                _ => {}
                            }
                        }
                    })
                    .build()
            ) {
                Ok(_) => {
                    log::info!("全局热键插件注册成功");
                    // 插件注册成功后，注册具体热键
                    global_hotkey::register_global_hotkeys(&app.handle());
                }
                Err(e) => {
                    log::warn!("全局热键插件注册失败: {}", e);
                }
            }

            // === D-17: 主窗口关闭行为 → 隐藏而非退出 ===
            // 同时监听焦点事件：用户点击任务栏图标时，自动弹出悬浮窗
            if let Some(main_window) = app.get_webview_window("main") {
                let window_clone = main_window.clone();
                let app_handle = app.handle().clone();
                main_window.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::CloseRequested { api, .. } => {
                            // 阻止默认关闭行为 → 最小化窗口（而非隐藏）
                            // 最小化后用户可点击任务栏恢复，触发 Focused 事件弹出悬浮窗
                            api.prevent_close();
                            let _ = window_clone.minimize();
                        }
                        tauri::WindowEvent::Focused(true) => {
                            // 主窗口获得焦点（如点击任务栏图标）→ 弹出悬浮窗
                            log::info!("主窗口获得焦点，弹出悬浮窗");
                            let _ = floating_window::force_show_floating(&app_handle);
                        }
                        _ => {}
                    }
                });
            }

            log::info!("DateCalendar 初始化完成");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 任务命令
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
            commands::task_commands::reorder_task,
            commands::task_commands::batch_update_tasks,
            commands::task_commands::batch_delete_tasks,
            commands::task_commands::batch_move_tasks,
            // 日程命令
            commands::schedule_commands::get_all_schedules,
            commands::schedule_commands::get_schedule,
            commands::schedule_commands::get_schedules_in_range,
            commands::schedule_commands::get_schedules_by_task,
            commands::schedule_commands::get_day_schedules,
            commands::schedule_commands::get_week_schedules,
            commands::schedule_commands::create_schedule,
            commands::schedule_commands::update_schedule,
            commands::schedule_commands::delete_schedule,
            commands::schedule_commands::update_schedule_status,
            commands::schedule_commands::check_conflicts,
            // D-13: 悬浮窗控制命令
            floating_window::toggle_floating_visibility,
            floating_window::set_floating_position,
            floating_window::get_cursor_position,
            floating_window::is_floating_visible,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 创建系统托盘（D-17）
///
/// 托盘菜单：
/// - 显示主窗口
/// - 切换悬浮窗
/// - ───
/// - 设置...
/// - ───
/// - 退出 DateCalendar
///
/// 左键单击托盘图标 → 显示主窗口
fn create_system_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // 构建托盘菜单
    let show_main = MenuItemBuilder::with_id("show_main", "显示主窗口").build(app)?;
    let toggle_floating = MenuItemBuilder::with_id("toggle_floating", "切换悬浮窗").build(app)?;
    let separator1 = PredefinedMenuItem::separator(app)?;
    let settings = MenuItemBuilder::with_id("settings", "设置...").build(app)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出 DateCalendar").build(app)?;

    let menu = MenuBuilder::new(app)
        .items(&[&show_main, &toggle_floating, &separator1, &settings, &separator2, &quit])
        .build()?;

    // 创建托盘图标
    let app_handle = app.clone();
    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("DateCalendar")
        .on_menu_event(move |_tray_app, event| {
            match event.id().as_ref() {
                "show_main" => {
                    // 显示主窗口
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    // 同步弹出悬浮窗
                    let _ = crate::floating_window::force_show_floating(&app_handle);
                }
                "toggle_floating" => {
                    // 直接切换悬浮窗位置（不依赖前端事件）
                    let _ = crate::floating_window::do_toggle_floating(&app_handle);
                }
                "settings" => {
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        app_handle.emit("navigate:settings", ()).ok();
                    }
                    // 同步弹出悬浮窗
                    let _ = crate::floating_window::force_show_floating(&app_handle);
                }
                "quit" => {
                    app_handle.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(move |tray, event| {
            // 左键单击托盘图标 → 显示主窗口 + 弹出悬浮窗
            if let tauri::tray::TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
                let _ = crate::floating_window::force_show_floating(app);
            }
        })
        .build(app)?;

    Ok(())
}
