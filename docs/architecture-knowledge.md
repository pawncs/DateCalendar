# DateCalendar 架构知识手册

> 本文档随开发进度持续更新，记录项目中涉及的架构概念和关键技术决策。
> 每个概念采用"知识点雷达"格式：是什么 → 为什么用 → 核心心智模型 → 关联概念。

---

## 目录

1. [整体架构概览](#1-整体架构概览)
2. [Tauri v2 桌面框架](#2-tauri-v2-桌面框架)
3. [SQLite 数据层设计](#3-sqlite-数据层设计)
4. [Rust 服务层设计](#4-rust-服务层设计)
5. [前端状态管理 (Zustand)](#5-前端状态管理-zustand)
6. [多窗口架构与悬浮窗](#6-多窗口架构与悬浮窗)
7. [HTTP API 嵌入模式](#7-http-api-嵌入模式)
8. [CLI 工具设计](#8-cli-工具设计)
9. [API 认证设计](#9-api-认证设计)
10. [API 文档自动生成](#10-api-文档自动生成)
11. [workbuddy Skill 设计](#11-workbuddy-skill-设计)
12. [多端接入架构](#12-多端接入架构)
13. [概念关联图](#13-概念关联图)

---

## 1. 整体架构概览

### 1.1 系统分层

```
┌────────────────────────────────────────────┐
│            表示层 (Presentation)             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ 主窗口    │  │ 悬浮窗    │  │ 系统托盘  │  │
│  │ React TS │  │ React TS │  │ 原生菜单  │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  │
├───────┼──────────────┼──────────────┼───────┤
│       │     Tauri IPC (进程间通信)   │        │
├───────┼──────────────┼──────────────┼───────┤
│            应用层 (Application)              │
│  ┌──────────────────────────────────────┐   │
│  │         Tauri Runtime (Rust)          │   │
│  │  ┌────────┐ ┌────────┐ ┌─────────┐   │   │
│  │  │Commands│ │HTTP API│ │CLI      │   │   │
│  │  │(IPC)   │ │(Actix) │ │(clap)   │   │   │
│  │  └───┬────┘ └───┬────┘ └────┬────┘   │   │
│  │      └──────────┼───────────┘         │   │
│  │           ┌─────┴─────┐               │   │
│  │           │ 服务层     │               │   │
│  │           │ Services  │               │   │
│  │           └─────┬─────┘               │   │
│  └─────────────────┼────────────────────┘   │
├────────────────────┼────────────────────────┤
│            数据层 (Data)                      │
│           ┌───────┴───────┐                  │
│           │    SQLite     │                  │
│           │  (单文件DB)    │                  │
│           └───────────────┘                  │
└────────────────────────────────────────────┘
```

### 1.2 三条通信路径

| 路径 | 触发方 | 协议 | 用途 |
|------|--------|------|------|
| **Tauri IPC** | React 前端 | Tauri invoke (基于 IPC) | 主窗口/悬浮窗 ↔ Rust 后端 |
| **HTTP REST API** | workbuddy / 外部工具 | HTTP (localhost:9876) | 外部进程读写任务数据 |
| **CLI** | workbuddy / 终端 | 命令行 + JSON stdout | 脚本化批量操作 |

**为什么三条路径并存？**
- IPC：前端内部通信，最直接、最低延迟
- HTTP API：workbuddy 作为独立进程，无法使用 IPC，需要标准协议
- CLI：适合一次性批量操作和脚本集成

---

## 2. Tauri v2 桌面框架

```
🔍 知识点雷达: Tauri
   ├── 是什么: 用 Rust 驱动、Web 技术渲染的桌面应用框架。类似 Electron，但后端是 Rust 而非 Node.js。
   ├── 为什么用: 内存占用~10MB（Electron ~100MB+），Rust 性能优异，原生系统API调用能力强
   ├── 核心心智模型:
   │   ┌─────────┐     IPC(进程间通信)     ┌─────────┐
   │   │ 前端进程 │ ◄──────────────────► │ Rust进程 │
   │   │ (WebView)│   invoke()/event     │ (后端)   │
   │   └─────────┘                       └─────────┘
   │   前端通过 invoke('command_name', args) 调用 Rust 函数
   │   Rust 通过 #[tauri::command] 宏暴露函数给前端
   └── 关联概念: IPC、WebView、Rust所有权系统
```

### 2.1 Tauri 进程模型

Tauri 应用运行两个进程：
- **Rust 后端进程**（主进程）：管理窗口、文件系统、数据库、系统API
- **WebView 前端进程**（渲染进程）：运行 React 应用，通过 IPC 与后端通信

前端无法直接访问文件系统或数据库，必须通过 `invoke()` 调用 Rust 函数。

### 2.2 IPC Command 模式

```rust
// Rust 侧：用 #[tauri::command] 标记函数
#[tauri::command]
fn get_tasks() -> Result<Vec<Task>, String> {
    // 业务逻辑
}

// 注册命令
app.setup(|_app| {
    // 命令在 builder 中注册
    Ok(())
});
```

```typescript
// 前端侧：用 invoke() 调用
import { invoke } from '@tauri-apps/api/core';
const tasks = await invoke<Task[]>('get_tasks');
```

### 2.3 Tauri v2 关键变化（相对 v1）

| v1 | v2 | 影响 |
|----|----|------|
| `tauri::Builder::default()` | 插件系统重构 | 更模块化 |
| `#[tauri::command]` | 基本一致 | 无影响 |
| 窗口管理 API 变化 | `WebviewWindow` 替代 `Window` | 悬浮窗创建方式不同 |
| 全局热键 | `tauri-plugin-global-shortcut` | 需单独引入插件 |

---

## 3. SQLite 数据层设计

```
🔍 知识点雷达: SQLite 嵌入式数据库
   ├── 是什么: 零配置、无服务器的自包含数据库引擎，数据存储在单个文件中
   ├── 为什么用: 无需安装数据库服务，单文件易于备份，支持标准SQL，足够处理个人级数据量
   ├── 核心心智模型:
   │   应用启动 → 打开/创建 .db 文件 → 连接池管理连接 → 执行SQL → 结果返回
   │   与 MySQL/PostgreSQL 最大的区别：没有"数据库服务器"概念，数据库即文件
   └── 关联概念: 连接池(r2d2)、递归CTE、事务、迁移
```

### 3.1 树形结构存储策略

本项目核心难点：如何在关系型数据库中高效存储和查询树形任务结构。

**方案：邻接表模型 (Adjacency List)**

```sql
CREATE TABLE tasks (
    id        TEXT PRIMARY KEY,
    parent_id TEXT REFERENCES tasks(id),  -- 自引用外键
    title     TEXT NOT NULL,
    -- ...
);
```

```
Task A (parent_id = NULL)
├── Task B (parent_id = A.id)
│   └── Task C (parent_id = B.id)
└── Task D (parent_id = A.id)
```

**优点**：插入/移动节点只需修改一行，结构简单
**缺点**：查询整棵树需要递归

### 3.2 递归 CTE 查询整棵树

```sql
WITH RECURSIVE task_tree AS (
    -- 锚点：找到根节点
    SELECT *, 0 AS depth FROM tasks WHERE parent_id IS NULL
    
    UNION ALL
    
    -- 递归：找到子节点，depth + 1
    SELECT t.*, tt.depth + 1
    FROM tasks t
    JOIN task_tree tt ON t.parent_id = tt.id
)
SELECT * FROM task_tree ORDER BY depth, sort_order;
```

**为什么不用嵌套集 (Nested Set) 或物化路径 (Materialized Path)？**
- 本项目是交互式任务管理，频繁增删改移动节点
- 邻接表模型在节点移动时成本最低（只改一行 parent_id）
- SQLite 的递归 CTE 足以应对个人级数据量的整树查询
- 如需性能优化，后续可引入"路径缓存列"（存储如 `/A/B/C` 的路径）

### 3.3 连接池 (r2d2 + rusqlite)

```rust
// 连接池模式
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

let manager = SqliteConnectionManager::file("datecalendar.db");
let pool = Pool::builder()
    .max_size(4)       // SQLite 写锁限制，不需要太多连接
    .build(manager)?;

// 使用时从池中获取连接
let conn = pool.get()?;
conn.execute("INSERT INTO ...", params![])?;
// conn 离开作用域自动归还池
```

**为什么连接数设 4？** SQLite 同时只能有一个写者，过多连接无意义且浪费资源。

---

## 4. Rust 服务层设计

```
🔍 知识点雷达: 服务层模式 (Service Layer Pattern)
   ├── 是什么: 在数据访问层之上封装业务逻辑的中间层，隔离"做什么"和"怎么做"
   ├── 为什么用: 让 IPC Commands / HTTP API / CLI 共享同一套业务逻辑，避免代码重复
   ├── 核心心智模型:
   │   Commands (IPC入口) ─┐
   │   HTTP Routes ────────┼──► TaskService ──► Database
   │   CLI Handlers ───────┘    (业务逻辑)     (数据存储)
   │   三个入口共享同一个服务层，各自只负责协议转换
   └── 关联概念: 分层架构、关注点分离、DAO模式
```

### 4.1 分层职责

| 层 | 职责 | 示例 |
|----|------|------|
| **入口层** (Commands/API/CLI) | 协议转换：把 IPC/HTTP/CLI 参数转成服务调用 | `fn get_tasks() -> Result<Vec<TaskDto>, String>` |
| **服务层** (Services) | 业务逻辑：校验、编排、事务管理 | `fn create_task(title: &str, parent_id: Option<&str>) -> Result<Task>` |
| **数据层** (DAO/DB) | 数据存取：SQL 执行、结果映射 | `fn insert_task(conn: &Connection, task: &NewTask) -> Result<String>` |

### 4.2 Rust 项目结构

```
src-tauri/src/
├── main.rs          # 程序入口，启动 Tauri
├── lib.rs           # 库入口，注册 commands + 初始化
├── db/
│   ├── mod.rs       # 数据库模块入口
│   ├── connection.rs # 连接池初始化
│   ├── migrations.rs # 建表/迁移 SQL
│   └── models.rs    # 数据模型 struct
├── services/
│   ├── mod.rs
│   ├── task_service.rs
│   ├── schedule_service.rs
│   └── note_service.rs
├── commands/        # Tauri IPC 命令
│   ├── mod.rs
│   ├── task_commands.rs
│   └── schedule_commands.rs
├── api/             # HTTP API
│   ├── mod.rs
│   ├── server.rs    # Actix-web 服务启动
│   └── routes/
├── cli/             # CLI 工具
│   └── main.rs
└── floating_window.rs # 悬浮窗管理
```

### 4.3 关键 Rust 概念速查

| 概念 | 一句话 | 本项目使用场景 |
|------|--------|---------------|
| `struct` + `impl` | 数据 + 方法 | 数据模型、服务对象 |
| `Result<T, E>` | 可能失败的操作 | 所有数据库操作、API 调用 |
| `Option<T>` | 可能有/没有的值 | parent_id 可空、查询结果可选 |
| `Mutex<T>` | 线程安全的可变共享 | 连接池、全局状态 |
| `#[derive(...)]` | 自动生成 trait 实现 | Serialize, Deserialize, Clone |
| `serde` | 序列化/反序列化 | JSON ↔ struct 转换 |

---

## 5. 前端状态管理 (Zustand)

```
🔍 知识点雷达: Zustand
   ├── 是什么: 一个极简的 React 状态管理库，比 Redux 轻量，比 Context 强大
   ├── 为什么用: 本项目状态复杂度中等，不需要 Redux 的样板代码，Zustand 的 API 简洁直观
   ├── 核心心智模型:
   │   创建一个 store = 一个包含状态 + 修改方法的 hook
   │   任何组件都可以直接使用 useXxxStore() 读写状态
   │   无需 Provider 包裹，store 是模块级单例
   └── 关联概念: 不可变更新、选择器模式、中间件
```

### 5.1 Store 设计示例

```typescript
// stores/taskStore.ts
import { create } from 'zustand';

interface TaskStore {
  tasks: Task[];
  selectedId: string | null;
  
  // 操作方法
  loadTasks: () => Promise<void>;
  addTask: (task: NewTask) => Promise<void>;
  selectTask: (id: string | null) => void;
  // ...
}

export const useTaskStore = create<TaskStore>((set, get) => ({
  tasks: [],
  selectedId: null,
  
  loadTasks: async () => {
    const tasks = await invoke<Task[]>('get_tasks');
    set({ tasks });
  },
  
  addTask: async (task) => {
    await invoke('create_task', { task });
    await get().loadTasks(); // 重新加载
  },
  
  selectTask: (id) => set({ selectedId: id }),
}));
```

### 5.2 数据流方向

```
用户操作 → React 组件 → Zustand Store.method() → invoke() → Rust Command → Service → DB
                                                                                    ↓
用户看到 ← React 重渲染 ← Zustand 状态更新 ← invoke() 返回值 ← Service ← DB 结果
```

**关键原则**：前端 Store 是后端数据的"缓存镜像"，所有持久化数据必须通过 IPC 存到 SQLite。

### 5.3 本项目 Store 划分

| Store | 管理内容 | 文件 |
|-------|---------|------|
| `useTaskStore` | 任务树、选中任务、展开/折叠状态 | `stores/taskStore.ts` |
| `useScheduleStore` | 日程列表、当前视图(日/周)、日期范围 | `stores/scheduleStore.ts` |
| `useSettingsStore` | 主题、热键、悬浮窗透明度等 | `stores/settingsStore.ts` |
| `useUIStore` | 面板宽度、侧栏状态等纯 UI 状态 | `stores/uiStore.ts` |

---

## 6. 多窗口架构与悬浮窗

```
🔍 知识点雷达: Tauri 多窗口
   ├── 是什么: Tauri 允许一个应用创建多个独立窗口，每个窗口有独立的 WebView 实例
   ├── 为什么用: 悬浮窗需要独立于主窗口（不同尺寸、置顶、无边框、透明），必须用多窗口实现
   ├── 核心心智模型:
   │   ┌────────────┐    ┌────────────┐
   │   │  主窗口     │    │  悬浮窗     │
   │   │  1200x800  │    │  320x屏幕高 │
   │   │  普通窗口  │    │  置顶+无边框│
   │   │  /main     │    │  /floating  │
   │   └─────┬──────┘    └──────┬─────┘
   │         │                  │
   │         └──────┬───────────┘
   │                │
   │         ┌──────┴──────┐
   │         │  Rust 后端   │ (共享同一个数据库连接)
   │         └─────────────┘
   └── 关联概念: WebView、窗口层级(Z-order)、全局热键
```

### 6.1 悬浮窗窗口创建（动态，Rust 侧）

悬浮窗不在 `tauri.conf.json` 中静态声明，而是在 `lib.rs` 的 `setup` 阶段通过 `WebviewWindowBuilder` 动态创建。原因：
- 需要运行时获取屏幕尺寸来计算停靠位置
- 窗口生命周期需由代码控制（隐藏/显示切换）

```rust
// floating_window.rs
pub fn create_floating_window(app: &AppHandle) -> Result<WebviewWindow, Box<dyn std::error::Error>> {
    let monitor = app.primary_monitor()?.ok_or("无法获取主显示器")?;
    let screen_w = monitor.size().width as f64 / monitor.scale_factor();

    // 隐藏位置：窗口在屏幕外，仅露 8px 边缘
    let hidden_x = screen_w - 8.0;
    let hidden_y = (screen_h - 560.0) / 2.0;

    let floating = WebviewWindowBuilder::new(app, "floating", WebviewUrl::App("/floating".into()))
        .title("")
        .inner_size(340.0, 560.0)
        .position(hidden_x, hidden_y)
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
```

前端通过 `/floating` 路由渲染悬浮窗组件（路径由 `WebviewUrl::App("/floating")` 指定）。

### 6.2 主窗口 ↔ 悬浮窗通信

两个窗口通过 Tauri 事件系统通信：

```
主窗口更新任务 → emit('task-updated', data) → 悬浮窗接收 → 更新显示
悬浮窗点击任务 → emit('focus-task', id) → 主窗口接收 → 跳转到对应任务
全局热键触发 → emit('floating:toggle', ()) → 悬浮窗切换显隐
```

### 6.3 系统托盘（D-17）

系统托盘让应用常驻后台，关闭主窗口 ≠ 退出应用。

```rust
// lib.rs setup 中
fn create_system_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let menu = MenuBuilder::new(app)
        .items(&[&show_main, &toggle_floating, &separator, &settings, &separator, &quit])
        .build()?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("DateCalendar")
        .on_menu_event(|app, event| { /* 处理菜单点击 */ })
        .on_tray_icon_event(|tray, event| { /* 左键单击显示主窗口 */ })
        .build(app)?;
    Ok(())
}
```

**关键行为**：
- 左键单击托盘图标 → 显示/聚焦主窗口
- 右键菜单 → "显示主窗口"、"切换悬浮窗"、"设置..."、"退出"
- 关闭主窗口 → 隐藏（不是退出），托盘仍在

### 6.4 全局热键（D-15）

全局热键在系统级注册，任何应用中按下均生效。

```rust
// global_hotkey.rs
use tauri_plugin_global_shortcut::GlobalShortcutExt;

pub fn register_global_hotkeys(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut_manager = app.global_shortcut();

    shortcut_manager.register("Ctrl+Shift+D", move |_app, shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            app.emit_to("floating", "floating:toggle", ()).ok();
        }
    })?;

    // Ctrl+Shift+T → 循环透明度
    shortcut_manager.register("Ctrl+Shift+T", move |_app, shortcut, event| { /* ... */ })?;

    Ok(())
}
```

**默认热键**：
| 热键 | 功能 |
|------|------|
| `Ctrl+Shift+D` | 切换悬浮窗显隐 |
| `Ctrl+Shift+T` | 循环透明度（85% → 60% → 40% → 85%） |

---

## 7. HTTP API 嵌入模式

```
🔍 知识点雷达: Actix-web 嵌入 Tauri
   ├── 是什么: 在 Tauri 应用的 Rust 进程中启动一个 HTTP 服务器，监听 localhost 端口
   ├── 为什么用: workbuddy 是独立进程，无法使用 Tauri IPC，需要标准 HTTP 协议通信
   ├── 核心心智模型:
   │   ┌──────────────┐     HTTP      ┌──────────────┐
   │   │  workbuddy    │ ──────────►  │  DateCalendar │
   │   │  (外部进程)    │ localhost:9876│  (Tauri应用)  │
   │   └──────────────┘               └──────┬───────┘
   │                                         │
   │                                  共享 TaskService
   │                                  共享数据库连接池
   └── 关联概念: RESTful API、JSON 序列化、端口管理
```

### 7.1 启动方式

```rust
// 在 Tauri setup 阶段启动 HTTP 服务
app.setup(|app| {
    let pool = app.state::<DbPool>().clone();
    
    // 在后台线程启动 Actix-web
    std::thread::spawn(move || {
        actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .app_data(pool.clone())
                .service(task_routes)
        })
        .bind("127.0.0.1:9876")?
        .run()
    });
    
    Ok(())
});
```

### 7.2 API 路由设计

| Method | Path | 功能 |
|--------|------|------|
| GET | `/api/tasks` | 获取所有任务（树形） |
| POST | `/api/tasks` | 创建任务 |
| PUT | `/api/tasks/:id` | 更新任务 |
| DELETE | `/api/tasks/:id` | 删除任务 |
| PATCH | `/api/tasks/:id/status` | 更新任务状态 |
| GET | `/api/schedules?from=&to=` | 获取日程 |
| POST | `/api/schedules` | 创建日程 |

---

## 8. CLI 工具设计

```
🔍 知识点雷达: CLI 工具 (clap)
   ├── 是什么: 一个独立的命令行程序（独立二进制），通过参数接收指令，通过 stdout 输出 JSON 结果
   ├── 为什么用: 适合脚本化批量操作，workbuddy 可通过执行命令快速读写数据；无需启动 HTTP 服务器
   ├── 核心心智模型:
   │   $ datecalendar-cli task list --format json
   │   输入: 命令行参数 (+ 可选 stdin JSON)
   │   输出: stdout 的 JSON（或 table/csv）
   │   退出码: 0=成功, 1=业务错误, 2=参数错误, 3=数据库错误, 4=数据库未找到
   └── 关联概念: clap 参数解析、stdin/stdout/stderr、退出码规范
```

### 8.1 命令结构

```
datecalendar-cli
├── task                      # 任务管理
│   ├── list                 # 列出所有任务（树形）
│   ├── get <ID>            # 获取单个任务
│   ├── create              # 创建任务（参数或 stdin JSON）
│   ├── update <ID>         # 更新任务
│   ├── delete <ID>         # 删除任务
│   ├── search <QUERY>      # 搜索任务
│   ├── complete <ID>       # 标记完成
│   └── import             # 从 stdin JSON 批量导入
├── schedule                 # 日程管理
│   ├── list                # 列出所有日程
│   ├── day <DATE>          # 查看某天日程
│   ├── week <START>        # 查看某周日程
│   ├── create              # 创建日程
│   ├── update <ID>         # 更新日程
│   ├── delete <ID>         # 删除日程
│   └── conflicts          # 检测时间冲突
├── health                   # 检查数据库连接
└── version                  # 显示版本信息
```

### 8.2 数据库路径自动发现

CLI 需要找到 `datecalendar.db`，按以下顺序查找：

1. `--db-path` 命令行参数
2. 环境变量 `DATECALENDAR_DB`
3. 默认位置（按操作系统）
   - Windows: `%APPDATA%\DateCalendar\datecalendar.db`
   - macOS: `~/Library/Application Support/DateCalendar/datecalendar.db`
   - Linux: `~/.local/share/DateCalendar/datecalendar.db`
4. 当前目录 `./datecalendar.db`（开发模式）

### 8.3 共享服务层

CLI 不直接访问数据库，而是通过共享的 `TaskService` / `ScheduleService`（放在独立的 `datecalendar-core` crate 中）。这确保：
- CLI 和 Tauri 后端的行为一致
- 业务逻辑只需维护一份
- 测试可以共享

### 8.4 使用示例

```bash
# 列出所有任务（JSON 输出，脚本友好）
datecalendar-cli task list

# 创建任务（命令行参数）
datecalendar-cli task create "完成报告" --priority 2

# 创建任务（stdin JSON，便于管道）
echo '{"title":"完成报告","priority":2}' | datecalendar-cli task create --stdin

# 查看今日日程
datecalendar-cli schedule day $(date +%Y-%m-%d)

# Table 格式输出（人类友好）
datecalendar-cli task list --format table
```

---

## 9. API 认证设计（保留入口，暂不实现）

```
🔍 知识点雷达: API 认证 (Bearer Token)
   ├── 是什么: 在 HTTP API 层添加 Bearer Token 认证，防止未授权访问
   ├── 为什么用: DateCalendar 是个人使用的桌面应用，仅在本人电脑上运行，当前无需认证
   │              但保留认证设计入口，以便未来扩展（如局域网访问）
   ├── 核心心智模型:
   │   请求 → Authorization: Bearer <token> → 中间件验证 → 通过/拒绝
   │   Token 存储在 settings 表，启动时自动生成
   │   白名单路径（/api/health, /api/auth/token）无需认证
   └── 关联概念: Actix-web 中间件、Bearer Token、OpenAPI Security Scheme
```

### 项目定位

DateCalendar 是个人使用的桌面应用：
- HTTP API 绑定 `127.0.0.1`（仅本机可访问）
- 无局域网/互联网暴露计划
- 无多用户场景
- **当前无需实现认证**

### 保留认证入口的原因

1. **未来扩展**：如果后续需要局域网访问（如手机访问桌面端），认证设计已就绪
2. **workbuddy 集成**：workbuddy 可以假设 API 有认证，提前做好 token 管理逻辑
3. **OpenAPI 规范完整**：API 文档中包含安全方案，符合业界标准

### 9.1 认证方案（预留）

| 方案 | 优点 | 缺点 | 最终选择 |
|------|------|------|------------|
| HTTP Basic Auth | 简单 | 每次传密码 | ❌ |
| Bearer Token (静态) | 简单、标准、workbuddy 友好 | Token 泄露 = 全权限 | ✅ 预留 |
| API Key (Query) | 简单 | URL 中可见 | ❌ |
| mTLS | 最安全 | 配置复杂 | ❌ 过度设计 |

### 9.2 Token 管理（预留）

- **生成**：Tauri 应用首次启动时自动生成 UUID v4，存入 `settings` 表
- **获取**：
  - 方式一：`GET /api/auth/token?secret=<setup_secret>`（首次，一次性 secret）
  - 方式二：`datecalendar-cli auth token`（直接读数据库）
  - 方式三：读配置文件 `%APPDATA%\DateCalendar\api_token.txt`
- **重置**：`datecalendar-cli auth reset-token`

### 9.3 白名单路径（预留）

| 路径 | 方法 | 说明 |
|------|------|------|
| `/api/health` | GET | 健康检查，监控系统用 |
| `/api/auth/token` | GET | 获取 token（需要 setup_secret） |

### 9.4 当前状态

| 状态 | 说明 |
|------|------|
| ✅ 设计预留 | 认证方案、Token 管理、中间件实现均已设计完成 |
| ✅ OpenAPI 规范 | API 文档中包含 `BearerAuth` 安全方案定义 |
| ❌ 暂不实现 | 认证中间件、Token 生成、Token 验证均暂不实现 |
| ❌ 暂不测试 | 测试计划 `14-api-auth.md` 暂不执行 |

> 未来需要认证时（如局域网访问），再实现第 2 节的设计方案。

---

## 10. API 文档自动生成

```
🔍 知识点雷达: OpenAPI/Swagger 文档自动生成
   ├── 是什么: 通过 utoipa 宏从 Rust 代码自动生成 OpenAPI 3.0 规范，并提供 Swagger UI 可交互文档页面
   ├── 为什么用: 手写文档易过时，OpenAPI 是业界标准，工具链丰富（Postman、Swagger UI、代码生成）
   ├── 核心心智模型:
   │   代码 + #[utoipa::path(...)] 宏 → 编译时生成 OpenAPI 规范 → /api-docs/openapi.json
   │   Swagger UI 读取规范 → 提供可交互文档页面 → /docs
   └── 关联概念: OpenAPI 3.0、Swagger UI、utoipa、actix-web 集成
```

### 10.1 访问方式

启动 Tauri 应用后，API 文档可通过以下 URL 访问：

| URL | 内容 |
|-----|------|
| `http://127.0.0.1:9876/docs` | Swagger UI 交互式文档 |
| `http://127.0.0.1:9876/api-docs/openapi.json` | OpenAPI 规范（JSON） |

### 10.2 规范内容

生成的 OpenAPI 规范包含：
- 所有 API 端点的路径、方法、参数、请求体、响应
- Schema 定义（TaskDto、NewTaskDto、ScheduleDto 等）
- 安全方案（Bearer Auth）
- 示例值

### 10.3 workbuddy 如何使用

workbuddy 可以：
1. 读取 `http://127.0.0.1:9876/api-docs/openapi.json` 获取完整的 API 规范
2. 根据规范自动生成正确的 API 调用代码
3. 在 Swagger UI 中手动测试 API

---

## 11. workbuddy Skill 设计

```
🔍 知识点雷达: workbuddy Skill
   ├── 是什么: 一个 Markdown 格式的技能描述文件，告诉 workbuddy 如何调用 DateCalendar 的 API/CLI
   ├── 为什么用: workbuddy 需要知道 DateCalendar 的 API 契约、调用方式、数据格式，才能正确操作
   ├── 核心心智模型:
   │   workbuddy 读取 skill.md → 理解可用操作 → 根据用户指令选择操作 → 调用 CLI/HTTP API → 返回结果
   │   Skill 是 workbuddy 的"使用手册"
   └── 关联概念: CodeBuddy Skill、OpenAPI 规范、CLI、HTTP API
```

### 11.1 Skill 文件结构

```
skills/
└── datecalendar/
    ├── skill.md              # Skill 主文件（workbuddy 读取）
    ├── README.md             # 人类阅读的文档
    ├── examples/
    │   ├── create-task.http  # HTTP 请求示例
    │   ├── create-task.sh    # CLI 调用示例
    │   └── response.json     # 响应示例
    └── schema/
        └── openapi.json      # OpenAPI 规范（符号链接或副本）
```

### 11.2 Skill 核心内容

`skill.md` 包含：
- **快速开始**：CLI 和 HTTP API 的调用示例
- **核心概念**：Task 和 Schedule 的字段说明
- **API 端点清单**：所有操作的 CLI 命令和 HTTP 端点对照表
- **典型场景**：workbuddy 可能遇到的用户指令及处理方式
- **错误处理**：各种错误的退出码和 HTTP 状态码

### 11.3 典型场景示例

| 用户指令 | workbuddy 行为 |
|----------|----------------|
| "帮我把明天下午 3 点的会议加进去" | 解析时间 → 调用 `schedule create` → 返回确认 |
| "查看本周的所有待办" | 计算日期范围 → 调用 `schedule week` → 返回日程列表 |
| "把这个任务标记为完成" | 获取当前任务 ID → 调用 `task complete` → 返回成功 |

---

## 12. 多端接入架构

```
🔍 知识点雷达: 多端接入架构
   ├── 是什么: 同一 Rust 后端提供两种接入方式——IPC（桌面应用）和 HTTP API（浏览器），共享同一个数据库。Tauri 未启动时浏览器降级到 SQL.js 内存模式
   ├── 为什么用: 用户既可用桌面应用也可用浏览器操作同一份数据；前端开发调试无需启动 Tauri（离线降级）
   ├── 核心心智模型:
   │   ┌─────────────────────────────────────────┐
   │   │         共享前端 (React/TSX)              │
   │   │         适配层选择接入方式                 │
   │   ├──────────┬──────────────┬────────────────┤
   │   │ Tauri IPC│  HTTP API    │  SQL.js 降级   │
   │   │ (桌面)   │  :9876       │  (离线)        │
   │   └────┬─────┴──────┬───────┴───────┬────────┘
   │        │            │               │
   │        └─────┬──────┘               │
   │              ▼                      ▼
   │     Rust 服务层 (唯一)    浏览器内存 SQLite
   │              │
   │     datecalendar.db (同一份)
   └── 关联概念: 适配器模式、HTTP API 代理、SQL.js 降级、环境检测
```

### 12.1 三种运行模式

| 模式 | 条件 | 数据库 | 使用场景 |
|------|------|--------|----------|
| `tauri` | `__TAURI_INTERNALS__` 存在 | `datecalendar.db` | 桌面应用 |
| `http` | `localhost:9876` 可达 | `datecalendar.db`（通过 HTTP 代理） | 浏览器 + Tauri |
| `sqljs` | 以上皆不可达 | 浏览器内存 | 离线降级 |

### 12.2 为什么用 HTTP API 代理而非独立后端

**方案演进**：最初考虑浏览器后端独立实现（SQL.js 作为平等后端），但用户需要在浏览器和桌面应用间共享数据。独立后端意味着两个数据库，无法共享。

**最终方案**：浏览器通过 HTTP API 代理到同一个 Rust 后端，操作同一份 `datecalendar.db`。SQL.js 仅作为 Tauri 未启动时的降级方案。

### 12.3 降级提示 UI

离线模式下，页面底部显示黄色 OfflineBanner：
> ⚠️ 离线模式 — Tauri 后端未连接，数据仅保存在浏览器内存中，刷新页面将丢失。

用户明确知道当前处于降级状态。

### 12.4 测试分层

| 层 | 目标 | 工具 |
|----|------|------|
| Rust 业务逻辑 | 验证服务层正确性 | `cargo test` (34 个) |
| 前端 UI（离线） | 验证组件渲染 + SQL.js 交互 | Playwright |
| 前端 UI（在线） | 验证完整数据流（HTTP API） | Playwright + Tauri |

---

## 13. 概念关联图

```
                    ┌──────────────┐
                    │  用户/外部工具 │
                    └──┬───┬───┬──┘
                       │   │   │
              ┌────────┘   │   └────────┐
              ▼            ▼            ▼
        ┌──────────┐ ┌──────────┐ ┌──────────┐
        │ Tauri IPC │ │HTTP API  │ │   CLI    │
        │ (invoke)  │ │(Actix)   │ │ (clap)   │
        └─────┬─────┘ └────┬─────┘ └────┬─────┘
              │             │             │
              └─────────┬───┴─────────────┘
                        ▼
              ┌─────────────────┐
              │   服务层 (Rust)   │
              │  TaskService     │
              │  ScheduleService │
              │  NoteService     │
              └────────┬────────┘
                       ▼
              ┌─────────────────┐
              │   数据层 (Rust)   │
              │  r2d2 连接池     │
              │  rusqlite        │
              └────────┬────────┘
                       ▼
              ┌─────────────────┐
              │    SQLite        │
              │  target/xxx.db   │
              └─────────────────┘

        ═══════════ 多端接入架构 ═══════════

              ┌─────────────────────┐
              │   共享前端 (React)   │
              └──────────┬──────────┘
                         │
              ┌──────────┴──────────┐
              │     适配层           │
              │  3 种模式自动切换     │
              └──────────┬──────────┘
                         │
     ┌──────────┬────────┼────────┬──────────┐
     ▼          ▼        ▼        ▼          ▼
  ┌──────┐ ┌──────┐ ┌──────┐
  │ IPC  │ │HTTP  │ │SQL.js│
  │(桌面)│ │:9876 │ │(降级)│
  └──┬───┘ └──┬───┘ └──┬───┘
     │        │         │
     └───┬────┘         │
         ▼              ▼
  ┌────────────┐ ┌────────────┐
  │ Rust 服务层 │ │ 内存 SQLite │
  │ (唯一)     │ │ (离线降级)  │
  └─────┬──────┘ └────────────┘
        ▼
  ┌────────────┐
  │  SQLite    │
  │  target/   │
  │  xxx.db    │
  └────────────┘
```

---

*文档版本: v2.2 | 更新日期: 2026-06-20 | 变更: 新增第9节(API认证)、第10节(API文档)、第11节(workbuddy Skill)*
