// 悬浮窗窗口管理模块
//
// 职责：
// 1. 创建悬浮窗 WebView 窗口（无边框、置顶、透明、跳过任务栏）
// 2. 计算屏幕右侧停靠位置（显示/隐藏两个位置）
// 3. 提供窗口显隐切换的 IPC 命令
// 4. 提供光标位置查询命令（供前端边缘检测轮询）
//
// 设计文档：D-13 悬浮窗窗口创建与停靠
// 测试计划：09-floating-window.md

use tauri::{AppHandle, Emitter, Manager, WebviewWindow, WebviewUrl};

const FLOATING_LABEL: &str = "floating";
const WINDOW_WIDTH: f64 = 340.0;
const WINDOW_HEIGHT: f64 = 560.0;
const HIDDEN_EDGE_PX: f64 = 24.0;  // 隐藏时露出 24px 边缘（足够宽可点击）
const VISIBLE_GAP_PX: f64 = 4.0;  // 显示时距右边缘 4px

/// 创建悬浮窗窗口。
///
/// 窗口初始位置在屏幕外（隐藏状态），仅露出 HIDDEN_EDGE_PX 边缘。
/// 通过 `set_floating_position` 命令切换显示/隐藏。
pub fn create_floating_window(app: &AppHandle) -> Result<WebviewWindow, Box<dyn std::error::Error>> {
    let monitor = app.primary_monitor()?.ok_or("无法获取主显示器")?;
    let screen_size = monitor.size();
    let scale = monitor.scale_factor();

    let screen_w = screen_size.width as f64 / scale;
    let screen_h = screen_size.height as f64 / scale;

    // 隐藏位置：窗口在屏幕外，仅露 HIDDEN_EDGE_PX（用于后续隐藏操作）
    let _hidden_x = screen_w - HIDDEN_EDGE_PX;
    // 可见位置：贴右边缘，距边缘 VISIBLE_GAP_PX
    let shown_x = screen_w - WINDOW_WIDTH - VISIBLE_GAP_PX;
    // 垂直居中
    let window_y = (screen_h - WINDOW_HEIGHT) / 2.0;

    let floating = tauri::WebviewWindowBuilder::new(
        app,
        FLOATING_LABEL,
        WebviewUrl::App("/floating".into()),
    )
    .title("")
    .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT)
    .position(shown_x, window_y)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .shadow(false)
    .visible(true)
    .build()?;

    Ok(floating)
}

/// 计算显示位置（贴右边缘）
fn get_shown_x(app: &AppHandle) -> Result<f64, String> {
    let monitor = app.primary_monitor()
        .map_err(|e| format!("获取显示器失败: {}", e))?
        .ok_or("无法获取主显示器")?;
    let screen_w = monitor.size().width as f64 / monitor.scale_factor();
    Ok(screen_w - WINDOW_WIDTH - VISIBLE_GAP_PX)
}

/// 计算隐藏位置（屏幕外，露边缘）
fn get_hidden_x(app: &AppHandle) -> Result<f64, String> {
    let monitor = app.primary_monitor()
        .map_err(|e| format!("获取显示器失败: {}", e))?
        .ok_or("无法获取主显示器")?;
    let screen_w = monitor.size().width as f64 / monitor.scale_factor();
    Ok(screen_w - HIDDEN_EDGE_PX)
}

/// 核心：切换悬浮窗显隐（直接操作窗口位置，不依赖事件）  
/// 
/// 关键修复：用窗口 x 坐标判断显隐状态，而非 window.is_visible()。
/// 因为窗口移出屏幕外后仍为 "可见"，只是位置不同。
/// 供 IPC 命令 `toggle_floating_visibility` 和全局热键 handler 共用
pub fn do_toggle_floating(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(FLOATING_LABEL) {
        let current_pos = window.outer_position()
            .map_err(|e| format!("获取位置失败: {}", e))?;
        let y = current_pos.y as f64;
        
        // 用位置判断状态：x 坐标接近 hidden_x 即为隐藏状态
        let monitor = app.primary_monitor()
            .map_err(|e| format!("获取显示器失败: {}", e))?
            .ok_or("无法获取主显示器")?;
        let screen_w = monitor.size().width as f64 / monitor.scale_factor();
        let hidden_x = screen_w - HIDDEN_EDGE_PX;
        let shown_x = screen_w - WINDOW_WIDTH - VISIBLE_GAP_PX;
        let current_x = current_pos.x as f64;
        
        // 判断：如果窗口 x 坐标 >= (hidden_x - 10)，认为处于隐藏状态
        let is_effectively_hidden = current_x >= hidden_x - 10.0;
        
        if is_effectively_hidden {
            // 当前隐藏 → 显示
            let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(shown_x, y)));
            let _ = window.show();
            let _ = window.set_focus();
        } else {
            // 当前可见 → 隐藏
            let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(hidden_x, y)));
        }
        Ok(())
    } else {
        Err("悬浮窗不存在".to_string())
    }
}

/// IPC 命令：切换悬浮窗显隐
#[tauri::command]
pub fn toggle_floating_visibility(app: AppHandle) -> Result<(), String> {
    do_toggle_floating(&app)
}

/// 强制显示悬浮窗（如果当前处于隐藏状态）
/// 
/// 供主窗口焦点事件、系统托盘等场景使用。
/// 与 do_toggle_floating 的区别：只在隐藏时显示，不会在可见时隐藏。
pub fn force_show_floating(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(FLOATING_LABEL) {
        let current_pos = window.outer_position()
            .map_err(|e| format!("获取位置失败: {}", e))?;
        let y = current_pos.y as f64;
        
        let monitor = app.primary_monitor()
            .map_err(|e| format!("获取显示器失败: {}", e))?
            .ok_or("无法获取主显示器")?;
        let screen_w = monitor.size().width as f64 / monitor.scale_factor();
        let hidden_x = screen_w - HIDDEN_EDGE_PX;
        let shown_x = screen_w - WINDOW_WIDTH - VISIBLE_GAP_PX;
        let current_x = current_pos.x as f64;
        
        // 只在隐藏状态下才显示
        if current_x >= hidden_x - 10.0 {
            let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(shown_x, y)));
            let _ = window.show();
            let _ = window.set_focus();
        }
        // 同步前端状态
        app.emit_to(FLOATING_LABEL, "floating:show", ()).ok();
    }
    Ok(())
}

/// IPC 命令：设置悬浮窗位置（显示=true / 隐藏=false）
#[tauri::command]
pub fn set_floating_position(app: AppHandle, visible: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(FLOATING_LABEL) {
        let x = if visible {
            get_shown_x(&app)?
        } else {
            get_hidden_x(&app)?
        };
        // 获取当前 y 位置以保持垂直位置不变
        let current_pos = window.outer_position()
            .map_err(|e| format!("获取位置失败: {}", e))?;
        let y = current_pos.y as f64;
        let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition::new(x, y)));
        if visible {
            let _ = window.show();
            let _ = window.set_focus();
        }
        Ok(())
    } else {
        Err("悬浮窗不存在".to_string())
    }
}

/// IPC 命令：获取当前光标位置（供前端边缘检测轮询）
//
// 注意：Tauri v2 中光标位置需通过具体 Window 对象的 `cursor_position()` 方法获取。
// 此处提供占位实现，实际边缘检测由前端通过 `currentWindow.cursor_position()` 完成。
#[tauri::command]
pub fn get_cursor_position() -> Result<serde_json::Value, String> {
    // 占位实现：前端应通过 Window::cursor_position() 自行获取
    Ok(serde_json::json!({"x": 0, "y": 0}))
}

/// IPC 命令：检查悬浮窗是否可见
#[tauri::command]
pub fn is_floating_visible(app: AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window(FLOATING_LABEL) {
        window.is_visible()
            .map_err(|e| format!("获取可见性失败: {}", e))
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    /// 测试 get_cursor_position 命令返回正确格式的 JSON
    #[test]
    fn test_get_cursor_position_returns_json() {
        let result = get_cursor_position().unwrap();
        assert!(result.is_object());
        assert!(result.get("x").is_some());
        assert!(result.get("y").is_some());
        // 当前为占位实现，返回值应为 0
        assert_eq!(result.get("x").unwrap(), &Value::Number(0.into()));
        assert_eq!(result.get("y").unwrap(), &Value::Number(0.into()));
    }

    /// 测试：常量定义正确
    #[test]
    fn test_constants() {
        assert_eq!(FLOATING_LABEL, "floating");
        assert_eq!(WINDOW_WIDTH, 340.0);
        assert_eq!(WINDOW_HEIGHT, 560.0);
        assert_eq!(HIDDEN_EDGE_PX, 8.0);
        assert_eq!(VISIBLE_GAP_PX, 4.0);
    }

    /// 测试：窗口标签格式正确（用于 tauri.conf.json 或代码中引用）
    #[test]
    fn test_floating_label() {
        // 窗口标签应与 Rust 侧 create_floating_window 中使用的 label 一致
        let label = "floating";
        assert_eq!(FLOATING_LABEL, label);
    }
}

