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
9. [概念关联图](#9-概念关联图)

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

### 6.1 悬浮窗窗口配置

```json
// tauri.conf.json 中配置多个窗口
{
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "DateCalendar",
        "width": 1200,
        "height": 800
      },
      {
        "label": "floating",
        "title": "DateCalendar Widget",
        "width": 320,
        "height": 600,
        "alwaysOnTop": true,
        "decorations": false,    // 无边框
        "transparent": true,     // 透明背景
        "visible": false,        // 初始隐藏
        "resizable": false
      }
    ]
  }
}
```

### 6.2 主窗口 ↔ 悬浮窗通信

两个窗口通过 Tauri 事件系统通信：

```
主窗口更新任务 → emit('task-updated', data) → 悬浮窗接收 → 更新显示
悬浮窗点击任务 → emit('focus-task', id) → 主窗口接收 → 跳转到对应任务
```

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
   ├── 是什么: 一个独立的命令行程序，通过参数接收指令，通过 stdout 输出 JSON 结果
   ├── 为什么用: 适合脚本化批量操作，workbuddy 可通过执行命令快速读写数据
   ├── 核心心智模型:
   │   $ datecalendar-cli task list --format json
   │   输入: 命令行参数
   │   输出: stdout 的 JSON
   │   退出码: 0=成功, 非0=错误
   └── 关联概念: clap 参数解析、stdin/stdout/stderr
```

### 8.1 使用示例

```bash
# 列出所有任务
datecalendar-cli task list

# 创建任务
datecalendar-cli task create --title "完成报告" --parent-id "xxx"

# 完成里程碑
datecalendar-cli milestone complete --id "xxx"

# 查看今日日程
datecalendar-cli schedule today
```

---

## 9. 概念关联图

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
              │  datecalendar.db │
              └─────────────────┘
```

---

*文档版本: v1.0 | 创建日期: 2026-06-10 | 涵盖范围: Phase 1.1 完成后的架构知识*
