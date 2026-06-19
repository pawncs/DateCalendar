# D-13: 悬浮窗窗口创建与屏幕停靠

## 1. 必要性 (Why)

### 问题
当前 DateCalendar 只有单一主窗口，用户必须 Alt+Tab 切换回应用才能查看日程。桌面悬浮窗作为「常驻桌面右侧的轻量面板」，让用户在工作流中一眼扫到今日待办，是产品核心差异化功能。

### 场景
- 用户正在写代码，瞄一眼桌面右侧 → 看到今天还剩 3 个待办
- 鼠标移到屏幕右边缘 → 悬浮窗从边缘滑出 → 打钩完成一个待办 → 鼠标移开 → 悬浮窗滑回隐藏
- 用户不需要时，悬浮窗完全隐藏在屏幕右边缘外，不占用工作空间

### 与主窗口的关系
- 悬浮窗是独立的 Tauri 窗口（独立 WebView），但共享同一个 Rust 后端 + 数据库
- 主窗口和悬浮窗通过 Tauri 事件系统通信：主窗口更新任务 → 悬浮窗刷新；悬浮窗点击任务 → 主窗口跳转
- 悬浮窗关闭不影响主窗口，主窗口关闭则悬浮窗也退出（由 Tauri 进程生命周期控制）

### 设计原则
- **非侵入**：悬浮窗平时隐藏于屏幕右边缘外，鼠标靠近右边缘时触发滑出
- **置顶显示**：`alwaysOnTop: true`，不被其他窗口遮挡
- **无边框毛玻璃**：`decorations: false` + `transparent: true`，融入桌面环境
- **跳过任务栏**：浮窗不应出现在任务栏中，避免视觉干扰

---

## 2. 实现方案 (How)

### 2.1 架构概览

```
Rust 侧 (src-tauri/src/floating_window.rs):
  创建悬浮窗窗口 (WebviewWindowBuilder)
  ├── 窗口配置：置顶、无边框、透明、跳过任务栏
  ├── 窗口定位：计算屏幕右侧坐标，贴边停靠
  ├── 鼠标边缘检测：轮询鼠标位置，触发显示/隐藏
  └── 导出命令：toggle_floating_visibility, update_floating_position

React 前端 (src/components/floating/):
  悬浮窗容器组件
  ├── 接收窗口大小、显示/隐藏状态
  └── 通过 IPC 事件与主窗口通信

主进程集成 (lib.rs):
  setup 阶段创建悬浮窗 + 注册新模块
```

### 2.2 窗口配置（tauri.conf.json）

在现有主窗口配置旁新增 `floating` 窗口：

```json
{
  "label": "floating",
  "title": "",
  "url": "/floating",           // 独立路由
  "width": 340,
  "height": 560,
  "resizable": false,
  "decorations": false,
  "transparent": true,
  "alwaysOnTop": true,
  "visible": true,              // 启动时可见（位置在屏幕外）
  "skipTaskbar": true,
  "shadow": false
}
```

> 注意：悬浮窗通过 Rust 代码动态创建，不直接在 tauri.conf.json 的 `windows` 数组中声明。原因：需要运行时获取屏幕尺寸来定位，且悬浮窗生命周期需由代码控制。

### 2.3 窗口创建（Rust 侧）

**新增文件**：`src-tauri/src/floating_window.rs`

```rust
use tauri::{WebviewWindowBuilder, WebviewUrl, Manager};
use tauri::window::Effect;

pub fn create_floating_window(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // 获取主显示器尺寸
    let monitor = app.primary_monitor()?.unwrap();
    let screen_size = monitor.size();
    let scale = monitor.scale_factor();

    let window_width: f64 = 340.0;
    let window_height: f64 = 560.0;

    // 首次创建：停在屏幕右边缘外（隐藏状态）
    let x = (screen_size.width as f64 / scale) + 10.0; // 屏幕外 10px
    let y = ((screen_size.height as f64 / scale) - window_height) / 2.0; // 垂直居中

    let floating = WebviewWindowBuilder::new(
        app,
        "floating",
        WebviewUrl::App("/floating".into()),
    )
    .title("")
    .inner_size(window_width, window_height)
    .position(x, y)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .shadow(false)
    .visible(true)
    .build()?;

    // 存储窗口位置信息到应用状态（供后续边缘检测使用）
    Ok(())
}
```

### 2.4 屏幕边缘停靠

**核心逻辑**：悬浮窗有"显示"和"隐藏"两个位置。

```
显示位置: x = screenWidth - windowWidth - 4px (贴右边缘，留 4px 间距)
隐藏位置: x = screenWidth - 8px (只露出 8px 边缘，其余在屏幕外)
```

窗口位置切换通过 `Window::set_position()` 实现。

**鼠标边缘检测**：
- 在前端 `FloatingWindow.tsx` 中监听全局 `mousemove`（通过 Tauri `Window::on_window_event` 或前端 `document` 级别监听）
- 当鼠标 `x >= screenWidth - triggerZone` 时 → 滑出（显示位置）
- 当鼠标离开窗口区域且 `x < screenWidth - windowWidth` 时 → 滑回（隐藏位置）
- 触发区域宽度：约 20px（可配置）

### 2.5 模块集成点

**lib.rs 新增**：
```rust
mod floating_window;

// setup 中：
floating_window::create_floating_window(app.handle())?;
```

**前端路由新增**：
在 React Router 中新增 `/floating` 路由 → 渲染 `<FloatingWindow />` 组件。

**适配层**：
悬浮窗内部同样通过适配层（`tauri` 模式）访问数据——与主窗口共用的 `adapter`。

---

## 3. 验证标准 (Verify)

### 功能验证

| 测试场景 | 预期结果 | 验证方法 |
|----------|----------|----------|
| 悬浮窗窗口存在 | 应用启动后，存在两个窗口（主窗口 + 悬浮窗） | 启动应用 → 检查任务管理器或 Alt+Tab |
| 悬浮窗初始位置 | 悬浮窗主体在屏幕外，仅露出右侧 8px 边缘 | 目视确认桌面右边缘有一条细边 |
| 窗口置顶 | 悬浮窗在所有其他窗口之上 | 打开其他全屏应用 → 悬浮窗仍可见 |
| 无边框透明 | 窗口无标题栏，背景透明 | 目视确认 |
| 不在任务栏 | 任务栏中无悬浮窗图标 | 检查 Windows 任务栏 |
| 鼠标靠近触发 | 鼠标移到屏幕右边缘 → 悬浮窗滑出到显示位置 | 手动操作 |
| 鼠标离开隐藏 | 鼠标离开悬浮窗区域 → 悬浮窗滑回隐藏位置 | 手动操作 |
| 窗口垂直居中 | 悬浮窗垂直方向居中于屏幕 | 目视确认 |

### 交互体验验证

| 场景 | 预期 |
|------|------|
| 鼠标快速划过 | 不触发显示（防止误触，需停留 200ms+） |
| 悬浮窗内操作 | 鼠标在窗口内时保持显示 |
| 多显示器 | 悬浮窗停靠在主显示器右边缘 |

### 技术验证

```bash
cargo check         # Rust 编译通过
npx tsc -b          # TypeScript 编译通过  
npx vite build      # 前端构建成功
npx tauri dev       # 手动 E2E：悬浮窗可见、可交互
```
