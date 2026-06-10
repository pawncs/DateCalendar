# D-11: 浏览器后端

## 1. 必要性 (Why)

### 问题
当前项目后端只有 Rust/Tauri 实现，前端通过 `invoke()` 调用 IPC 命令。在纯浏览器环境（`npx vite`）中无法使用 Tauri IPC。但用户有同时通过桌面应用和浏览器操作的需求——两边应该共享同一份本地数据库。

### 场景
- 用户启动 Tauri 桌面应用后，同时用浏览器打开 `localhost:5173`，两边看到相同数据
- 用户在浏览器中创建任务 → 桌面应用实时看到新任务
- Tauri 未启动时，用户仍可在浏览器中快速预览前端（离线降级）

### 方案概述

**主方案**：浏览器通过 HTTP API（`localhost:9876`）代理到 Tauri Rust 后端，操作同一份 `datecalendar.db`。
**降级方案**：Tauri 未启动时，浏览器使用 SQL.js 内存数据库，提供基本的操作能力。

```
用户启动 start.bat
  → Tauri 桌面应用 + Actix-web HTTP API :9876 启动
  → 用户可用桌面应用操作，也可用浏览器打开 localhost:5173
  → 浏览器前端检测到 HTTP API 可达 → 通过 HTTP 操作数据库
  → 浏览器前端检测到 HTTP API 不可达 → 降级到 SQL.js 内存模式
  → 两种方式操作同一份 datecalendar.db（主方案）
```

### 与 Tauri 后端的关系
浏览器后端与 Tauri 后端通过 HTTP API 共享服务层，操作同一个数据库。这不是"两个后端"，而是**同一后端，两种接入方式**：IPC 和 HTTP。

---

## 2. 实现方案 (How)

### 2.1 架构图

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
        │ (桌面应用) │   │ localhost:9876│   │ (降级)   │
        └─────┬─────┘   └──────┬───────┘   └────┬─────┘
              │                │                │
              └────────┬───────┘                │
                       ▼                        ▼
              ┌─────────────────┐    ┌──────────────────┐
              │   Rust 服务层    │    │  浏览器内存 SQLite │
              │ TaskService     │    │  (离线降级模式)    │
              │ ScheduleService │    └──────────────────┘
              │ NoteService     │
              └────────┬────────┘
                       ▼
              ┌─────────────────┐
              │    SQLite        │
              │ datecalendar.db  │
              │ (同一份文件)      │
              └─────────────────┘
```

### 2.2 主方案：HTTP API 代理

浏览器环境下的适配层通过 `fetch()` 调用 Tauri 的 Actix-web HTTP API：

```typescript
// 适配层在浏览器 + HTTP API 可达时
async function get_all_tasks(): Promise<Task[]> {
  const res = await fetch('http://localhost:9876/api/tasks');
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

async function create_task(input: NewTask): Promise<Task> {
  const res = await fetch('http://localhost:9876/api/tasks', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(input),
  });
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}
```

**HTTP API 路由**（需与 Tauri commands 一一对应）：

| Method | Path | 对应 Tauri Command |
|--------|------|-------------------|
| GET | `/api/tasks` | `get_all_tasks` |
| GET | `/api/tasks/:id` | `get_task` |
| POST | `/api/tasks` | `create_task` |
| PUT | `/api/tasks/:id` | `update_task` |
| DELETE | `/api/tasks/:id` | `delete_task` |
| GET | `/api/tasks/search?q=` | `search_tasks` |
| GET | `/api/tasks/:id/risks` | `get_risks` |
| POST | `/api/tasks/:id/risks` | `add_risk` |
| DELETE | `/api/risks/:id` | `delete_risk` |
| GET | `/api/tasks/:id/notes` | `get_notes` |
| PUT | `/api/tasks/:id/notes` | `save_note` (upsert) |
| DELETE | `/api/notes/:id` | `delete_note` |
| PUT | `/api/tasks/reorder` | `reorder_task` |
| PUT | `/api/tasks/batch/status` | `batch_update_tasks` |
| POST | `/api/tasks/batch/delete` | `batch_delete_tasks` |
| PUT | `/api/tasks/batch/move` | `batch_move_tasks` |
| GET | `/api/schedules` | `get_all_schedules` |
| GET | `/api/schedules/:id` | `get_schedule` |
| GET | `/api/schedules/range` | `get_schedules_in_range` |
| GET | `/api/schedules/day/:date` | `get_day_schedules` |
| GET | `/api/schedules/week` | `get_week_schedules` |
| GET | `/api/schedules/task/:task_id` | `get_schedules_by_task` |
| POST | `/api/schedules` | `create_schedule` |
| PUT | `/api/schedules/:id` | `update_schedule` |
| DELETE | `/api/schedules/:id` | `delete_schedule` |
| PUT | `/api/schedules/:id/status` | `update_schedule_status` |
| GET | `/api/schedules/conflicts` | `check_conflicts` |

### 2.3 降级方案：SQL.js 离线模式

当 HTTP API 不可达（Tauri 未启动）时，降级到 SQL.js 内存数据库：

```typescript
// 检测 HTTP API 是否可达
async function checkHttpApi(): Promise<boolean> {
  try {
    const res = await fetch('http://localhost:9876/api/health');
    return res.ok;
  } catch {
    return false;
  }
}
```

**降级模式特征**：
- 使用 SQL.js 纯内存数据库（刷新后数据丢失）
- Schema 与主数据库完全一致（从 migrations.rs 复制）
- 接口签名与 HTTP API 完全一致
- 界面显示明显的降级提示

### 2.4 降级提示 UI

浏览器在降级模式下运行时，界面需要明确告知用户当前状态：

**状态栏提示**：页面底部显示黄色提示条：
> ⚠️ 离线模式 — Tauri 后端未连接，数据仅保存在浏览器内存中，刷新页面将丢失。请启动桌面应用以获得完整功能。

**组件**：`src/components/common/OfflineBanner.tsx`，固定在页面底部，带有关闭按钮。

**视觉设计**：
- 黄色/橙色背景（warning 色系）
- 左侧离线图标 + 文字说明
- 右侧「启动桌面应用」按钮（不可点击，仅提示） + 关闭按钮
- 带滑入动画

### 2.5 适配层三种模式

适配层根据环境自动选择，详见 D-12。

| 模式 | 检测条件 | 数据库 | 持久化 |
|------|---------|--------|--------|
| `tauri` | `__TAURI_INTERNALS__` 存在 | `datecalendar.db` | ✅ 磁盘 |
| `http` | HTTP API `localhost:9876` 可达 | `datecalendar.db` | ✅ 磁盘 |
| `sqljs` | 以上皆不可达 | 浏览器内存 | ❌ 刷新丢失 |

### 2.6 start.bat 改造

同时启动 Tauri 桌面应用和浏览器开发服务器：

```batch
@echo off
chcp 65001 >nul
echo ========================================
echo   DateCalendar 启动脚本
echo ========================================
echo.

cd /d "%~dp0datecalendar"

echo [1/3] 安装依赖（如需要）...
if not exist "node_modules" (
    echo   正在安装 npm 依赖...
    call npm install
) else (
    echo   node_modules 已存在，跳过
)

echo.
echo [2/3] 启动 Tauri 桌面应用（含 HTTP API :9876）...
start "DateCalendar-Tauri" npx tauri dev

echo [3/3] 启动浏览器开发服务器（:5173）...
echo   等待 Tauri 启动...
timeout /t 3 /nobreak >nul
start http://localhost:5173
npx vite --open

pause
```

---

## 3. 验证标准 (Verify)

### 功能验证

| 测试场景 | 预期结果 | 验证方法 |
|----------|----------|----------|
| 桌面应用创建任务 | 浏览器刷新后看到新任务 | 手动操作 → 浏览器验证 |
| 浏览器创建任务 | 桌面应用任务树自动更新 | 手动操作 → 桌面验证 |
| 浏览器编辑任务 | 桌面应用详情面板更新 | 手动操作 |
| 浏览器删除任务 | 桌面应用任务消失 | 手动操作 |
| Tauri 未启动 | 浏览器显示离线模式提示 | 关闭 Tauri → 浏览器刷新 |
| 离线模式创建任务 | 任务创建成功，但刷新后丢失 | 离线模式操作 → 刷新 |
| HTTP API 恢复 | 离线提示消失，数据恢复 | 启动 Tauri → 浏览器刷新 |

### 技术验证

```bash
cargo check                       # Rust 编译通过
npx tsc -b                        # TypeScript 编译通过
start start.bat                   # 双端同时启动正常
# 浏览器访问 localhost:5173       # 正常显示，可操作数据
# 桌面应用操作数据                 # 浏览器刷新后同步
```

---

*文档版本: v2.0 | 创建日期: 2026-06-10 | 变更: 主方案改为 HTTP API 代理，SQL.js 降为离线降级方案*
