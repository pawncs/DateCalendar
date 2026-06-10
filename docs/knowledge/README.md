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

*文档版本: v2.0 | 创建日期: 2026-06-10 | 变更: 主方案改为 HTTP API 代理，SQL.js 降为离线降级*
