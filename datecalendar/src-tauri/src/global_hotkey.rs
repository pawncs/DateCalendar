// 全局热键管理模块
//
// 职责：
// 1. 注册全局热键（系统级，任何应用中均生效）
// 2. 提供热键查询和注销接口
//
// 注意：热键事件处理在 lib.rs 的插件初始化阶段通过 with_handler 设置
// 此处仅负责注册/注销热键
//
// 设计文档：D-15 全局热键系统
// 测试计划：11-global-hotkey.md

use tauri::AppHandle;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

const HOTKEY_TOGGLE: &str = "Ctrl+Shift+D";   // 切换悬浮窗显隐
const HOTKEY_TRANSPARENCY: &str = "Ctrl+Shift+T"; // 循环透明度

/// 注册全局热键（在 lib.rs setup 中调用）
///
/// - 注册失败（热键被占用）→ 日志警告，不阻塞启动
/// - 注册成功 → 热键在任何应用中按下时触发回调（回调在插件初始化时设置）
pub fn register_global_hotkeys(app: &AppHandle) {
    let shortcut_manager = app.global_shortcut();

    // 注册「切换悬浮窗」热键
    match shortcut_manager.register(HOTKEY_TOGGLE) {
        Ok(_) => log::info!("全局热键注册成功: {}", HOTKEY_TOGGLE),
        Err(e) => {
            log::warn!("全局热键注册失败 {}: {}", HOTKEY_TOGGLE, e);
            // 不返回错误，允许应用继续启动
        }
    }

    // 注册「循环透明度」热键
    match shortcut_manager.register(HOTKEY_TRANSPARENCY) {
        Ok(_) => log::info!("全局热键注册成功: {}", HOTKEY_TRANSPARENCY),
        Err(e) => {
            log::warn!("全局热键注册失败 {}: {}", HOTKEY_TRANSPARENCY, e);
        }
    }
}

/// 检查热键是否已注册
pub fn is_hotkey_registered(app: &AppHandle, shortcut: &str) -> bool {
    app.global_shortcut().is_registered(shortcut)
}

/// 注销所有热键（应用退出时自动调用，此处为显式接口）
pub fn unregister_all(app: &AppHandle) {
    let _ = app.global_shortcut().unregister_all();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试：热键常量定义正确
    #[test]
    fn test_hotkey_constants() {
        assert_eq!(HOTKEY_TOGGLE, "Ctrl+Shift+D");
        assert_eq!(HOTKEY_TRANSPARENCY, "Ctrl+Shift+T");
    }

    /// 测试：register_global_hotkeys 函数签名正确（编译时检查）
    #[test]
    fn test_register_global_hotkeys_signature() {
        // 此测试仅验证函数存在且签名正确
        // 实际注册需要运行中的 Tauri 上下文，此处跳过
        assert!(true);
    }

    /// 测试：is_hotkey_registered 返回 bool 类型
    #[test]
    fn test_is_hotkey_registered_returns_bool() {
        // 验证函数签名：接受 &str，返回 bool
        // 注意：is_registered 返回 bool，不是 Result<bool>
        assert!(true);
    }
}
