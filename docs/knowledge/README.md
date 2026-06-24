# DateCalendar 领域知识手册

> 开发时的速查参考，记录项目相关的技术概念、架构约定、开发规范。
> 随开发进度持续更新。

---

## 目录

1. [双后端接入架构](#1-双后端接入架构)
2. [HTTP API 代理模式](#2-http-api-代理模式)
3. [SQL.js 离线降级](#3-sqljs-离线降级)
4. [适配器模式与三种运行环境](#4-适配器模式与三种运行环境)
5. [接口契约](#5-接口契约)
6. [数据库存储位置规范](#6-数据库存储位置规范)
7. [C 盘容量保护](#7-c-盘容量保护)
8. [测试分层与验证链路](#8-测试分层与验证链路)

---

## 1. 双后端接入架构

### 核心概念

本项目采用 **同一 Rust 后端，两种接入方式**：

- **IPC 接入**（Tauri 桌面应用）：前端通过 `invoke()` 直接调用 Rust 命令
- **HTTP 接入**（浏览器）：前端通过 `fetch()` 调用 `localhost:9876` 的 REST API

两种接入方式共享同一个 Rust 服务层和同一个 `datecalendar.db` 数据库。

```
                 ┌─ 共享前端 (React/TSX) ──────────────┐
                 │       同一套代码                      │
                 │       适配层选择接入方式               │
                 └──────────┬───────────────────────────┘
                            │
           ┌────────────────┼────────────────┐
           ▼                ▼                ▼
     ┌──────────┐   ┌──────────────┐   ┌──────────┐
     │ Tauri IPC │   │  HTTP API    │   │ SQL.js   │
     │ (桌面应用) │   │ :9876        │   │ (降级)   │
     └─────┬─────┘   └──────┬───────┘   └────┬─────┘
           │                │                │
           └────────┬───────┘                │
                    ▼                        ▼
           ┌─────────────────┐    ┌──────────────────┐
           │   Rust 服务层    │    │  浏览器内存 SQLite │
           │ 同一份业务逻辑    │    │  (离线降级模式)    │
           └────────┬────────┘    └──────────────────┘
                    ▼
           ┌─────────────────┐
           │    SQLite        │
           │ datecalendar.db  │
           │ (同一份文件)      │
           └─────────────────┘
```

### 关键原则

1. **同一后端**：Rust 服务层是唯一的业务逻辑实现
2. **两种接入**：IPC 和 HTTP 共享服务层，操作同一数据库
3. **离线降级**：Tauri 未启动时，浏览器使用 SQL.js 内存数据库作为降级方案
4. **接口契约统一**：三种接入方式的函数签名和返回格式完全一致

---

## 2. HTTP API 代理模式

### 工作原理

浏览器前端不直接操作数据库，而是通过 HTTP 请求代理到 Tauri 的 Actix-web 服务器：

```
浏览器前端
  → fetch('http://localhost:9876/api/tasks')
  → Actix-web (Rust 进程内 HTTP 服务)
  → TaskService (共享服务层)
  → SQLite (datecalendar.db)
  → 结果返回 JSON → 前端更新
```

### 优势
- 浏览器和桌面应用看到同一份数据
- 无需在浏览器端维护独立的数据库
- Rust 服务层的单元测试覆盖所有逻辑

### 前提
- Tauri 桌面应用必须已启动（HTTP API 嵌入在 Tauri 进程中）
- HTTP API 端口 `9876` 未被占用

---

## 3. SQL.js 离线降级

### 触发条件
Tauri 未启动时，浏览器无法连接 HTTP API，自动降级到 SQL.js 内存数据库。

### 特征
- 数据库完全在浏览器内存中（刷新丢失）
- Schema 与主数据库一致
- 接口签名与 HTTP API 一致
- 界面显示黄色 OfflineBanner 提示用户

### 用途
- 前端开发调试（无需启动 Tauri）
- 快速预览前端效果
- Playwright 自动化 UI 测试（无后端依赖）

---

## 4. 适配器模式与三种运行环境

```
🔍 知识点雷达: 适配器模式
   ├── 是什么: 在共享前端和多种后端接入方式之间插入适配层，根据运行环境自动选择
   ├── 为什么用: Store 层不感知接入方式，只需调用统一接口
   ├── 核心心智模型:
   │   Store → adapter.get_all_tasks()
   │              │
   │   ┌──────────┼──────────┐
   │   ▼          ▼          ▼
   │  IPC       HTTP       SQL.js
   │  (桌面)    (浏览器)    (离线)
   └── 关联概念: 策略模式、依赖注入、环境检测
```

### 三种模式

| 模式 | 检测条件 | 数据库 | 持久化 | 使用场景 |
|------|---------|--------|--------|----------|
| `tauri` | `__TAURI_INTERNALS__` 存在 | `datecalendar.db` | ✅ 磁盘 | 桌面应用 |
| `http` | `localhost:9876/health` 可达 | `datecalendar.db` | ✅ 磁盘 | 浏览器 + Tauri |
| `sqljs` | 以上皆不可达 | 浏览器内存 | ❌ 刷新丢失 | 前端开发/测试 |

### 环境检测方法

```typescript
// 1. 检测 Tauri
if ('__TAURI_INTERNALS__' in window) → 'tauri'

// 2. 检测 HTTP API
await fetch('http://localhost:9876/api/health') → OK → 'http'

// 3. 降级
else → 'sqljs'
```

---

## 5. 接口契约

### 契约内容

所有接入方式必须遵守的统一规范：

- **函数签名一致**：参数名、类型、可选性完全相同
- **返回值格式一致**：JSON 结构、字段名、类型完全相同
- **错误处理一致**：成功返回数据，失败抛出 `Error` 字符串
- **副作用一致**：相同输入 → 相同数据库状态

### 接口总览（27 个）

| 模块 | 函数数 | 主要函数 |
|------|--------|----------|
| 任务 CRUD | 5 | `get_all_tasks`, `get_task`, `create_task`, `update_task`, `delete_task` |
| 搜索 | 1 | `search_tasks` |
| 里程碑风险 | 3 | `get_risks`, `add_risk`, `delete_risk` |
| 笔记 | 3 | `get_notes`, `save_note`, `delete_note` |
| 排序批量 | 4 | `reorder_task`, `batch_update_tasks`, `batch_delete_tasks`, `batch_move_tasks` |
| 日程 CRUD | 8 | `get_all_schedules`, `get_schedule`, `get_schedules_in_range`, `get_day_schedules`, `get_week_schedules`, `get_schedules_by_task`, `create_schedule`, `update_schedule`, `delete_schedule` |
| 状态同步冲突 | 2 | `update_schedule_status`, `check_conflicts` |

完整 HTTP 路由映射详见 [D-11: 浏览器后端](../design/D-11-browser-backend.md)。

---

## 6. 数据库存储位置规范

| 场景 | 位置 | 说明 |
|------|------|------|
| Tauri 开发模式 | `datecalendar/src-tauri/target/datecalendar.db` | 项目 target 目录下 |
| Tauri 生产模式 | Tauri 应用数据目录 | 打包后自动管理 |
| HTTP 浏览器模式 | 同上（共享同一文件） | 通过 Tauri HTTP API 代理 |
| SQL.js 离线模式 | 内存（WASM） | 不落盘 |
| Rust 单元测试 | SQLite `:memory:` | 测试结束自动销毁 |
| 测试数据导出 | `target/test-data/` | 调试用 |

### .gitignore

```
target/
!target/*.md
target/test-data/
```

---

## 7. C 盘容量保护

### 需要询问的操作

- 下载安装新软件（npm 全局包、Rust 工具链组件、系统级工具）
- 在 C 盘创建大文件（>10MB）
- 任何写入 `C:\Users\` 下非项目目录的操作

### 默认允许

- npm/pip/cargo 包安装到项目本地（`node_modules/`、`target/`）
- 项目工作区内所有文件操作

---

## 8. 测试分层与验证链路

### 三层验证

| 层 | 目标 | 工具 | 对应测试 |
|----|------|------|----------|
| Rust 业务逻辑 | 验证服务层正确性 | `cargo test` | 34 个单元测试 |
| 前端 UI+交互 | 验证组件渲染和用户操作 | Playwright CLI | 黑盒用例 |

### 测试模式

| 测试场景 | 后端接入 | 数据库 |
|----------|---------|--------|
| Rust 单元测试 | 直接调用 Service | `:memory:` |
| 前端黑盒（离线） | SQL.js 降级 | 浏览器内存 |
| 前端黑盒（在线） | HTTP API | `datecalendar.db` |

### 每个功能同时覆盖两种接入方式

每个 `docs/test-plans/` 下的测试流程文档同时包含：
- Tauri 后端白盒测试命令（`cargo test`）
- 浏览器前端黑盒测试用例（两种模式均可执行）

### 关键测试命令

```bash
# Rust 后端白盒
cd datecalendar/src-tauri && cargo test --lib

# 前端黑盒（离线模式）
cd datecalendar && npx vite              # 终端1
playwright-cli open http://localhost:5173 # 终端2

# 前端黑盒（在线模式）
start start.bat                          # 启动 Tauri + 浏览器
playwright-cli open http://localhost:5173 # 通过 HTTP API 操作
```

---

## 9. 多窗口架构与悬浮窗

### 核心概念

DateCalendar 使用 Tauri 多窗口架构，共有两个窗口：
1. **主窗口**（`/main`）：常规应用窗口，1200×800，有标题栏
2. **悬浮窗**（`/floating`）：340×560，无边框、置顶、透明、跳过任务栏

两个窗口共享同一个 Rust 后端 + 同一个 SQLite 数据库。

### 窗口创建方式

悬浮窗不在 `tauri.conf.json` 中静态声明，而是在 `lib.rs` 的 `setup` 阶段通过 `WebviewWindowBuilder` 动态创建。

**原因**：
- 需要运行时获取屏幕尺寸来计算停靠位置
- 窗口生命周期需由代码控制（隐藏/显示切换）

### 窗口间通信

两个窗口通过 Tauri 事件系统通信：

```
主窗口更新任务 → emit('task:updated', data) → 悬浮窗监听 → 更新显示
悬浮窗点击任务 → emit('focus:task', id) → 主窗口监听 → 跳转对应任务
全局热键触发   → emit('floating:toggle', ()) → 悬浮窗切换显隐
```

### 系统托盘行为（D-17）

- 托盘图标常驻系统托盘区域
- 左键单击 → 显示/聚焦主窗口
- 右键菜单 → "显示主窗口"、"切换悬浮窗"、"设置..."、"退出"
- **关闭主窗口 ≠ 退出应用**：主窗口关闭时隐藏（`window.hide()`），托盘仍在，悬浮窗仍可用

### 全局热键（D-15）

通过 `tauri-plugin-global-shortcut` 插件在系统级注册热键：

| 热键 | 功能 |
|------|------|
| `Ctrl+Shift+D` | 切换悬浮窗显隐 |
| `Ctrl+Shift+T` | 循环透明度（85% → 60% → 40% → 85%） |

热键在任何应用中均生效（全局有效）。

### 前端路由配置

`src/main.tsx` 根据 `window.location.pathname` 决定渲染哪个组件：
- `/` → `<App />` → `<MainLayout />`
- `/floating` → `<FloatingWindow />`

### 悬浮窗动画（D-14）

使用 Framer Motion 驱动滑入/滑出动画：
- 隐藏态：`x: 310`（窗口宽 340px，留 30px 边缘可见）
- 显示态：`x: 0`
- 过渡：`spring`（stiffness: 300, damping: 30）

---

## 10. CLI 工具

### 核心概念

`datecalendar-cli` 是一个独立的 Rust 二进制程序，通过命令行参数接收指令，通过 stdout 输出 JSON 结果。它直接调用共享的 `TaskService` / `ScheduleService`，不经过 HTTP API。

### 命令结构

```
datecalendar-cli
├── task                # 任务管理
│   ├── list             # 列出所有任务
│   ├── create          # 创建任务
│   ├── get <ID>        # 获取单个任务
│   ├── update <ID>     # 更新任务
│   ├── delete <ID>     # 删除任务
│   ├── search <QUERY>  # 搜索任务
│   └── complete <ID>   # 标记完成
├── schedule           # 日程管理
│   ├── day <DATE>      # 查看某天日程
│   ├── week <START>    # 查看某周日程
│   ├── create          # 创建日程
│   ├── update <ID>     # 更新日程
│   ├── delete <ID>     # 删除日程
│   └── conflicts      # 检测时间冲突
└── health             # 检查数据库连接
```

### 退出码规范

| 退出码 | 含义 |
|--------|------|
| `0` | 成功 |
| `1` | 业务错误（任务不存在、日程冲突等） |
| `2` | 参数错误（缺少必填参数、格式错误） |
| `3` | 数据库错误（无法打开数据库、SQL 错误） |
| `4` | 数据库未找到（`--db-path` 指向不存在的文件） |

### 输出格式

- 默认：`json`（脚本友好，可被 `jq` 解析）
- `--format table`：人类友好的 ASCII 表格
- `--format csv`：CSV 格式

### 数据库路径发现

按以下顺序查找 `datecalendar.db`：
1. `--db-path` 命令行参数
2. 环境变量 `DATECALENDAR_DB`
3. 默认位置（按操作系统）
4. 当前目录 `./datecalendar.db`（开发模式）

---

## 11. API 认证（保留入口，暂不实现）

### 项目定位

DateCalendar 是个人使用的桌面应用，仅在本人电脑上运行：
- HTTP API 绑定 `127.0.0.1`（仅本机可访问）
- 无局域网/互联网暴露计划
- 无多用户场景
- **当前无需实现认证**

### 核心概念（预留）

HTTP API 使用 Bearer Token 认证，防止未授权访问。Token 是静态 UUID v4，存储在 `settings` 表中。

### 认证流程（预留）

```
请求 → Authorization: Bearer <token> → 中间件验证 → 通过/拒绝
```

### Token 管理（预留）

- **生成**：Tauri 应用首次启动时自动生成
- **获取**：
  - `GET /api/auth/token?secret=<setup_secret>`（首次，一次性 secret）
  - `datecalendar-cli auth token`（直接读数据库，最可靠）
  - 读配置文件 `%APPDATA%\DateCalendar\api_token.txt`
- **重置**：`datecalendar-cli auth reset-token`

### 白名单路径（预留）

以下路径无需认证：
- `GET /api/health`：健康检查
- `GET /api/auth/token?secret=<...>`：获取 token

### 当前状态

| 状态 | 说明 |
|------|------|
| ✅ 设计预留 | 认证方案、Token 管理、中间件实现均已设计完成 |
| ✅ OpenAPI 规范 | API 文档中包含 `BearerAuth` 安全方案定义 |
| ❌ 暂不实现 | 认证中间件、Token 生成、Token 验证均暂不实现 |
| ❌ 暂不测试 | 测试计划 `14-api-auth.md` 暂不执行 |

> 未来需要认证时（如局域网访问），再实现设计文档中的方案。

---

## 12. API 文档自动生成

### 核心概念

使用 `utoipa` 库从 Rust 代码中的宏自动生成 OpenAPI 3.0 规范，并通过 `utoipa-swagger-ui` 提供可交互的文档页面。

### 访问方式

| URL | 内容 |
|-----|------|
| `http://127.0.0.1:9876/docs` | Swagger UI 交互式文档 |
| `http://127.0.0.1:9876/api-docs/openapi.json` | OpenAPI 规范（JSON） |

### OpenAPI 规范内容

生成的规范包含：
- 所有 API 端点的路径、方法、参数、请求体、响应
- Schema 定义（TaskDto、NewTaskDto、ScheduleDto 等）
- 安全方案（Bearer Auth）
- 示例值

---

## 13. workbuddy Skill

### 核心概念

workbuddy Skill 是一个 Markdown 格式的文件（`skill.md`），描述如何调用 DateCalendar 的 API/CLI。它告诉 workbuddy：
- DateCalendar 的 API 契约（如何创建任务、如何安排日程）
- 调用方式（HTTP API 还是 CLI）
- 数据格式（请求体和响应体的字段名、类型）

### Skill 文件结构

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
        └── openapi.json      # OpenAPI 规范
```

### 典型场景

| 用户指令 | workbuddy 行为 |
|----------|----------------|
| "帮我把明天下午 3 点的会议加进去" | 解析时间 → 调用 `schedule create` → 返回确认 |
| "查看本周的所有待办" | 计算日期范围 → 调用 `schedule week` → 返回日程列表 |
| "把这个任务标记为完成" | 获取当前任务 ID → 调用 `task complete` → 返回成功 |

---

*文档版本: v2.2 | 更新日期: 2026-06-20 | 变更: 新增第10节(CLI工具)、第11节(API认证)、第12节(API文档)、第13节(workbuddy Skill)*
