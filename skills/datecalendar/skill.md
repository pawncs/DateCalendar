# DateCalendar Skill

## 概述

DateCalendar 是一个本地任务与日程管理应用。本 Skill 描述如何通过 CLI 或 HTTP API 与其交互。

## 快速开始

### 方式一：CLI（推荐，最简单）

```bash
# 创建任务
datecalendar-cli task create "完成报告" --priority 2

# 创建日程
datecalendar-cli schedule create \
  --title "团队周会" \
  --start "2026-06-20T10:00:00" \
  --end "2026-06-20T11:00:00" \
  --type fixed

# 查看今日日程
datecalendar-cli schedule day $(Get-Date -Format "yyyy-MM-dd")
```

### 方式二：HTTP API

```bash
# 创建任务
Invoke-RestMethod -Uri "http://127.0.0.1:9876/api/tasks" `
  -Method Post `
  -Body '{"title": "完成报告", "priority": 2}' `
  -ContentType "application/json"
```

## 核心概念

### 任务 (Task)

- `id`: UUID，全局唯一
- `title`: 任务标题
- `status`: `pending` | `in_progress` | `completed` | `cancelled`
- `priority`: 0（无）| 1（低）| 2（中）| 3（高）
- `parent_id`: 父任务 ID（用于树形结构），`null` 表示根任务
- `is_milestone`: 是否为里程碑

### 日程 (Schedule)

- `id`: UUID
- `task_id`: 关联的任务 ID
- `title`: 日程标题
- `start_time`: ISO 8601 格式，如 `2026-06-20T10:00:00`
- `end_time`: ISO 8601 格式
- `is_all_day`: 是否全天
- `schedule_type`: `fixed`（固定时间）| `todo_day`（本日待办）| `todo_week`（本周待办）
- `status`: `pending` | `completed` | `cancelled`

## API 端点清单

### 任务

| 操作 | CLI | HTTP API |
|------|-----|----------|
| 列出所有任务 | `datecalendar-cli task list` | `GET /api/tasks` |
| 创建任务 | `datecalendar-cli task create <title>` | `POST /api/tasks` |
| 获取任务 | `datecalendar-cli task get <id>` | `GET /api/tasks/:id` |
| 更新任务 | `datecalendar-cli task update <id> ...` | `PUT /api/tasks/:id` |
| 删除任务 | `datecalendar-cli task delete <id>` | `DELETE /api/tasks/:id` |
| 搜索任务 | `datecalendar-cli task search <query>` | `GET /api/tasks/search?q=<query>` |
| 标记完成 | `datecalendar-cli task complete <id>` | `PUT /api/tasks/:id` (status=completed) |

### 日程

| 操作 | CLI | HTTP API |
|------|-----|----------|
| 查看今日日程 | `datecalendar-cli schedule day <date>` | `GET /api/schedules/day/:date` |
| 查看本周日程 | `datecalendar-cli schedule week <start>` | `GET /api/schedules/week?weekStart=<start>&weekEnd=<end>` |
| 创建日程 | `datecalendar-cli schedule create ...` | `POST /api/schedules` |
| 更新日程 | `datecalendar-cli schedule update <id> ...` | `PUT /api/schedules/:id` |
| 删除日程 | `datecalendar-cli schedule delete <id>` | `DELETE /api/schedules/:id` |
| 检测冲突 | `datecalendar-cli schedule conflicts ...` | `GET /api/schedules/conflicts?startTime=<start>&endTime=<end>` |

## 典型场景

### 场景 1：用户说「帮我把明天下午 3 点的会议加进去」

步骤：
1. 解析时间：「明天」= 当前日期 +1 天，「下午 3 点」= 15:00
2. 构造请求：
   ```bash
   datecalendar-cli schedule create `
     --title "会议" `
     --start "2026-06-21T15:00:00" `
     --end "2026-06-21T16:00:00" `
     --type fixed
   ```
3. 执行命令
4. 解析输出（JSON），确认创建成功

### 场景 2：用户说「查看本周的所有待办」

```bash
# PowerShell 获取本周的开始和结束日期
$weekStart = (Get-Date).AddDays(-(Get-Date).DayOfWeek + 1).ToString("yyyy-MM-dd")
datecalendar-cli schedule week $weekStart
```

### 场景 3：用户说「把这个任务标记为完成」

```bash
datecalendar-cli task complete <task_id>
```

## 错误处理

| 错误 | CLI 退出码 | HTTP 状态码 | 处理方式 |
|------|------------|-------------|----------|
| 任务不存在 | 1 | 200（返回 null）| 提示用户任务不存在 |
| 参数错误 | 2 | 400 | 提示正确的参数格式 |
| 数据库错误 | 3 | 500 | 提示用户检查数据库 |
| 日程冲突 | 1 | 200（返回冲突列表）| 提示用户选择如何处理 |

## 配置

### CLI 配置

配置文件位置（可选）：
- Windows: `%APPDATA%\DateCalendar\cli-config.json`

配置内容：
```json
{
  "db_path": "C:\\Users\\...\\DateCalendar\\datecalendar.db",
  "default_format": "json"
}
```

### HTTP API 配置

```bash
# token 可以通过环境变量传递
$env:DATECALENDAR_TOKEN = "a1b2c3d4-..."

# CLI 会自动读取环境变量
datecalendar-cli task list
```

## 完整示例

详见 `examples/` 目录。
