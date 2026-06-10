# D-04: Schedule 数据模型与后端 CRUD

## 1. 必要性 (Why)

### 问题
当前项目只有任务（Task）相关的数据层，但 `schedules` 表已在数据库迁移中创建，却没有任何 Rust 服务层和 IPC 命令。日程管理（模块三）的所有功能（日视图、周视图、待办列表）都依赖这个基础。

### 场景
- 用户将一个任务安排到「6月10日 14:00-16:00」
- 用户查看今天的所有日程安排
- 用户将一个待办标记为「本周待办」(todo_week)

### 这是模块三的基石
D-05（日视图）、D-06（周视图）、D-07（日程创建）、D-08（待办列表）全部依赖此模块。

---

## 2. 实现方案 (How)

### 2.1 数据模型回顾

```sql
CREATE TABLE schedules (
    id            TEXT PRIMARY KEY,        -- UUID
    task_id       TEXT NOT NULL REFERENCES tasks(id),
    title         TEXT NOT NULL,
    start_time    TEXT NOT NULL,           -- ISO 8601
    end_time      TEXT NOT NULL,           -- ISO 8601
    is_all_day    INTEGER DEFAULT 0,
    schedule_type TEXT DEFAULT 'fixed',    -- fixed | todo_day | todo_week
    status        TEXT DEFAULT 'pending',
    color         TEXT DEFAULT '',
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);
```

🔍 知识点雷达: schedule_type 三种模式
   ├── 是什么: fixed = 固定时间段（14:00-16:00），todo_day = 今日待办（不指定具体时间），todo_week = 本周待办
   ├── 为什么用: 灵活的时间管理方式——有些任务需要精确安排，有些只需要知道今天要做，有些本周内做就行
   ├── 核心心智模型: 时间精度递减 → fixed（精确到小时）→ todo_day（精确到天）→ todo_week（精确到周）
   └── 关联概念: 日视图显示 fixed + todo_day，周视图显示全部三种

### 2.2 Rust 后端实现

**新增文件**：
- `src-tauri/src/services/schedule_service.rs` — 日程服务层
- `src-tauri/src/commands/schedule_commands.rs` — 日程 IPC 命令

**models.rs 新增**：`Schedule`、`CreateScheduleInput`、`UpdateScheduleInput` 结构体。

**schedule_service 核心方法**：
- `create_schedule` — 创建日程（UUID 生成 + 默认值）
- `get_schedules_in_range` — 按日期范围查询（日/周视图核心查询）
- `get_day_schedules` — 查询某天的日程（fixed + todo_day）
- `get_week_schedules` — 查询某周的日程（全部三种类型）
- `update_schedule` — 动态更新（与 task_service 模式一致）
- `delete_schedule` — 删除日程

**Tauri 命令**（8 个）：`create_schedule`、`get_schedule`、`get_schedules_in_range`、`get_schedules_by_task`、`get_day_schedules`、`get_week_schedules`、`update_schedule`、`delete_schedule`。

**lib.rs 注册**：在 `invoke_handler` 中注册全部 8 个命令。

### 2.3 前端类型定义

**新建 scheduleStore.ts**：与 taskStore 模式一致，包含 `loadDaySchedules`、`loadWeekSchedules`、`createSchedule`、`updateSchedule`、`deleteSchedule`。

---

## 3. 验证标准 (Verify)

### 功能验证

| 测试场景 | 预期结果 | 验证方法 |
|----------|----------|----------|
| 创建 fixed 日程 | 调用 create_schedule，返回完整 Schedule 对象 | IPC 调用 → 检查返回值 |
| 创建 todo_day 日程 | schedule_type=todo_day | IPC 调用 |
| 查询日期范围 | 传入 6月10日-6月12日，返回范围内所有日程 | IPC 调用 → 检查数量 |
| 查询某天 | get_day_schedules("2026-06-10") 返回 fixed + todo_day | 手动验证 |
| 更新日程 | 修改 title → 数据库更新 | 查询验证 |
| 删除日程 | 删除 → 数据库无此记录 | 查询验证 |

### 技术验证

```bash
cargo check     # Rust 编译通过
npx tsc -b      # TypeScript 编译通过
```

### HTTP API 路由

本模块的服务方法同时通过 HTTP API 暴露（`localhost:9876`），供浏览器前端调用。路由映射详见 [D-11: 浏览器后端](D-11-browser-backend.md) 中的 API 路由表。
