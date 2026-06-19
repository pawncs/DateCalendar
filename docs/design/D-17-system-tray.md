# D-17: 系统托盘菜单

## 1. 必要性 (Why)

### 问题
当前 DateCalendar 关闭主窗口后应用退出，用户无法在后台保持运行。系统托盘让应用常驻后台，悬浮窗和热键持续生效。托盘菜单提供快捷操作入口。

### 场景
- 用户关闭主窗口 → 应用缩小到托盘，悬浮窗仍正常工作
- 右键托盘图标 → 快速显示/隐藏主窗口
- 右键托盘图标 → 快速显示/隐藏悬浮窗
- 右键托盘图标 → 退出应用

### 设计原则
- **常驻后台**：关闭主窗口 = 最小化到托盘，不退出进程
- **简洁菜单**：只提供核心操作，不超过 5 个菜单项
- **托盘图标**：复用应用图标，清晰可辨

---

## 2. 实现方案 (How)

### 2.1 技术选型

Tauri v2 原生支持系统托盘，通过 `tauri::tray::TrayIconBuilder` API 实现，无需额外插件。

### 2.2 菜单设计

```
┌─────────────────────┐
│  显示主窗口          │  ← show_main_window
│  切换悬浮窗          │  ← toggle_floating
├─────────────────────┤
│  设置...             │  ← 打开主窗口设置页（预留）
├─────────────────────┤
│  退出 DateCalendar   │  ← quit
└─────────────────────┘
```

### 2.3 实现流程

**lib.rs setup 中**：

```rust
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItemBuilder};

// 1. 构建托盘菜单
let show_main = MenuItemBuilder::with_id("show_main", "显示主窗口").build(app)?;
let toggle_floating = MenuItemBuilder::with_id("toggle_floating", "切换悬浮窗").build(app)?;
let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
let settings = MenuItemBuilder::with_id("settings", "设置...").build(app)?;
let quit = MenuItemBuilder::with_id("quit", "退出 DateCalendar").build(app)?;

let menu = MenuBuilder::new(app)
    .items(&[&show_main, &toggle_floating, &separator, &settings, &separator, &quit])
    .build()?;

// 2. 创建托盘图标
let tray = TrayIconBuilder::new()
    .icon(app.default_window_icon().unwrap().clone())
    .menu(&menu)
    .tooltip("DateCalendar")
    .on_menu_event(|app, event| {
        match event.id().as_ref() {
            "show_main" => {
                // 显示/聚焦主窗口
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "toggle_floating" => {
                // 切换悬浮窗显隐
                app.emit("floating:toggle", ()).ok();
            }
            "settings" => {
                // 打开主窗口 + 跳转到设置页
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    app.emit("navigate:settings", ()).ok();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        }
    })
    .on_tray_icon_event(|tray, event| {
        // 左键单击托盘图标 → 显示主窗口
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event {
            if let Some(window) = tray.app_handle().get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    })
    .build(app)?;
```

### 2.4 关闭主窗口行为

修改主窗口关闭事件：关闭 → 隐藏而非退出。

**在 lib.rs setup 或 main window 创建时**：

```rust
// 在主窗口创建时设置关闭行为
let main_window = app.get_webview_window("main").unwrap();
main_window.on_window_event(|event| {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        // 阻止默认关闭行为 → 隐藏窗口
        api.prevent_close();
        if let Some(window) = api.window() {
            let _ = window.hide();
        }
    }
});
```

这确保：
- 点击主窗口 X 按钮 → 窗口隐藏，应用继续在托盘运行
- 通过托盘「退出」或任务管理器结束进程 → 真正退出

### 2.5 文件结构

```
src-tauri/src/lib.rs    # setup 中新增系统托盘初始化代码
                        # 约 40 行（菜单构建 + 事件处理 + 关闭行为）
```

> 托盘功能代码量少且与 `lib.rs` 的 `setup` 高度耦合，不单独拆文件。

---

## 3. 验证标准 (Verify)

### 功能验证

| 测试场景 | 预期结果 | 验证方法 |
|----------|----------|----------|
| 托盘图标出现 | 应用启动后系统托盘出现 DateCalendar 图标 | 目视检查系统托盘区域 |
| 托盘提示文字 | 鼠标悬停显示 "DateCalendar" | 悬停托盘图标 |
| 左键单击托盘 | 主窗口显示/聚焦 | 关闭主窗口 → 左键点托盘 → 主窗口弹出 |
| 显示主窗口 | 右键 → "显示主窗口" → 主窗口显示 | 关闭主窗口 → 右键菜单操作 |
| 切换悬浮窗 | 右键 → "切换悬浮窗" → 悬浮窗显隐切换 | 悬浮窗隐藏时点 → 显示；再点 → 隐藏 |
| 退出应用 | 右键 → "退出" → 应用完全退出，托盘图标消失 | 点退出 → 检查任务管理器 |
| 关闭窗口不退出 | 按主窗口 X 按钮 → 窗口隐藏，托盘仍在 | 关闭主窗口 → 检查托盘 |
| 悬浮窗仍在 | 关闭主窗口后 → 悬浮窗和热键仍正常工作 | 关闭主窗口 → 用热键切换悬浮窗 |

### 交互体验验证

| 场景 | 预期 |
|------|------|
| 右键菜单样式 | 使用系统原生右键菜单（与系统其他托盘图标一致） |
| 图标清晰度 | 在不同缩放比例下（100%/125%/150%）图标清晰 |
| 退出确认 | 不需要二次确认弹窗（菜单操作已足够明确） |

### 技术验证

```bash
cargo check         # 编译通过
npx tauri dev       # 手动 E2E：
                    #   1. 启动 → 检查托盘图标
                    #   2. 关闭主窗口 → 托盘仍在
                    #   3. 右键菜单操作
                    #   4. 左键点托盘 → 主窗口恢复
                    #   5. 悬浮窗热键仍可用
                    #   6. 退出 → 进程终止
```
