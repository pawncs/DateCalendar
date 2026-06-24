# D-21: workbuddy Skill

> **状态**: ✅ 已完成 (2026-06-24)
> 
> **实现概要**: 
> - Skill 文件位置: `skills/datecalendar/`
> - 主文件: `skill.md` (workbuddy 读取)
> - 示例: `examples/create-task.sh`, `examples/create-task.ps1`, `examples/create-schedule.http`
> - 人类文档: `README.md`



## 1. 必要性 (Why)

### 问题

DateCalendar 的核心价值之一是通过 workbuddy（或任何 AI 助手）自动管理日程。但 workbuddy 需要知道：
- DateCalendar 的 API 契约（如何创建任务、如何安排日程）
- 调用方式（HTTP API 还是 CLI）
- 数据格式（请求体和响应体的字段名、类型）

没有 Skill，workbuddy 需要每次都「猜测」如何调用，效率低且容易出错。

### 场景

- 用户对 workbuddy 说：「帮我把明天下午 3 点的会议加到 DateCalendar」
  → workbuddy 读取 Skill → 调用 `datecalendar-cli schedule create` → 完成
- 用户对 workbuddy 说：「查看本周的所有待办」
  → workbuddy 读取 Skill → 调用 `datecalendar-cli schedule week $(date)` → 返回结果
- workbuddy 自动规划：AI 生成任务清单 → 自动调用 DateCalendar API 批量创建

### 设计原则

- **声明式**：Skill 描述「能做什么」，不描述「怎么做」（让 workbuddy 自己决定调用方式）
- **示例驱动**：每个功能都附带请求/响应示例
- **渐进式**：Skill 可以逐步扩展，先覆盖核心场景，再补充高级功能
- **与 API 文档一致**：Skill 引用 OpenAPI 规范，不重复定义

---

## 2. 实现方案 (How)

### 2.1 Skill 文件结构

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

### 2.2 Skill 主文件格式 (`skill.md`)

采用 Markdown 格式，结构清晰，便于 workbuddy 解析：

```markdown
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
datecalendar-cli schedule day $(date +%Y-%m-%d)
```

### 方式二：HTTP API

```bash
# 获取 token（首次）
token=$(curl -s "http://127.0.0.1:9876/api/auth/token?secret=<setup_secret>" | jq -r .token)

# 创建任务
curl -X POST http://127.0.0.1:9876/api/tasks \
  -H "Authorization: Bearer $token" \
  -H "Content-Type: application/json" \
  -d '{"title": "完成报告", "priority": 2}'
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
| 查看本周日程 | `datecalendar-cli schedule week <start>` | `GET /api/schedules/week?weekStart=&weekEnd=` |
| 创建日程 | `datecalendar-cli schedule create ...` | `POST /api/schedules` |
| 更新日程 | `datecalendar-cli schedule update <id> ...` | `PUT /api/schedules/:id` |
| 删除日程 | `datecalendar-cli schedule delete <id>` | `DELETE /api/schedules/:id` |
| 检测冲突 | `datecalendar-cli schedule conflicts ...` | `GET /api/schedules/conflicts?startTime=&endTime=` |

## 典型场景

### 场景 1：用户说「帮我把明天下午 3 点的会议加进去」

步骤：
1. 解析时间：「明天」= 当前日期 +1 天，「下午 3 点」= 15:00
2. 构造请求：
   ```bash
   datecalendar-cli schedule create \
     --title "会议" \
     --start "2026-06-21T15:00:00" \
     --end "2026-06-21T16:00:00" \
     --type fixed
   ```
3. 执行命令
4. 解析输出（JSON），确认创建成功

### 场景 2：用户说「查看本周的所有待办」

```bash
# 获取本周的开始和结束日期
week_start=$(date -d "monday this week" +%Y-%m-%d)
week_end=$(date -d "sunday this week" +%Y-%m-%d)

datecalendar-cli schedule week $week_start
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
| 未认证 | — | 401 | 引导用户获取 token |
| 日程冲突 | 1 | 200（返回冲突列表）| 提示用户选择如何处理 |

## 配置

### CLI 配置

配置文件位置（可选）：
- Windows: `%APPDATA%\DateCalendar\cli-config.json`
- macOS/Linux: `~/.config/datecalendar/cli-config.json`

配置内容：
```json
{
  "db_path": "C:\\Users\\...\\DateCalendar\\datecalendar.db",
  "token": "a1b2c3d4-...",
  "default_format": "json"
}
```

### HTTP API 配置

```bash
# token 可以通过环境变量传递
export DATECALENDAR_TOKEN="a1b2c3d4-..."

# CLI 会自动读取环境变量
datecalendar-cli task list
```

## 完整示例

详见 `examples/` 目录。
```

### 2.3 README.md（人类阅读）

```markdown
# DateCalendar workbuddy Skill

本 Skill 允许 workbuddy（或任何 AI 助手）通过 CLI 或 HTTP API 管理 DateCalendar 中的任务和日程。

## 安装

1. 确保 DateCalendar 已安装并运行
2. 确保 `datecalendar-cli` 在 PATH 中（或知道其路径）
3. 将本 Skill 目录添加到 workbuddy 的 Skill 搜索路径

## 使用

workbuddy 会自动读取 `skill.md`，无需手动操作。

示例对话：
- 用户：「帮我把明天下午 3 点的会议加到 DateCalendar」
- workbuddy：（读取 Skill → 调用 CLI → 返回确认）

## 开发

- `skill.md`：Skill 主文件，描述所有可用的操作
- `examples/`：各种场景的完整示例
- `schema/openapi.json`：OpenAPI 规范（从运行中的应用获取）

## 测试

```bash
# 确保 DateCalendar 正在运行
# 测试 CLI
datecalendar-cli task list

# 测试 HTTP API
curl http://127.0.0.1:9876/api/tasks \
  -H "Authorization: Bearer <token>"
```
```

### 2.4 示例文件

**`examples/create-task.sh`**：

```bash
#!/bin/bash
# 创建一个高优先级任务

datecalendar-cli task create \
  --title "完成 Q2 报告" \
  --priority 3 \
  --description "需要在 6 月 30 日前完成"
```

**`examples/create-schedule.http`**：

```http
POST /api/schedules HTTP/1.1
Host: 127.0.0.1:9876
Authorization: Bearer a1b2c3d4-e5f6-7890-abcd-ef1234567890
Content-Type: application/json

{
  "taskId": "existing-task-id",
  "title": "团队周会",
  "startTime": "2026-06-20T10:00:00",
  "endTime": "2026-06-20T11:00:00",
  "isAllDay": false,
  "scheduleType": "fixed",
  "color": "#3b82f6"
}
```

### 2.5 与 OpenAPI 规范的关联

Skill 不重复定义 API 契约，而是引用 OpenAPI 规范：

```markdown
## API 规范

完整的 OpenAPI 3.0 规范可从以下位置获取：
- 运行中的应用：`http://127.0.0.1:9876/api-docs/openapi.json`
- 本 Skill 目录：`schema/openapi.json`（建议符号链接）

Skill 中仅描述 workbuddy 需要的核心概念和高层操作，详细请求/响应格式请参考 OpenAPI 规范。
```

---

## 3. 验证标准 (Verify)

### 功能验证

| 测试场景 | 预期结果 | 验证方法 |
|----------|----------|----------|
| workbuddy 读取 Skill | 成功解析 `skill.md` | 在 workbuddy 中「加载 Skill」 |
| workbuddy 执行「创建任务」 | 调用 CLI 或 HTTP API 成功 | 观察 DateCalendar 中是否出现新任务 |
| workbuddy 执行「查看今日日程」 | 返回今日日程列表 | 对比 DateCalendar 悬浮窗内容 |
| Skill 中的示例可运行 | 所有 `examples/*.sh` 都能成功执行 | 手动执行每个示例 |

### workbuddy 集成验证

| 场景 | workbuddy 行为 | 预期 |
|------|----------------|------|
| 用户说「帮我把明天下午 3 点的会议加进去」 | 解析时间 → 调用 `schedule create` | 日程被创建 |
| 用户说「查看本周的所有待办」 | 计算日期范围 → 调用 `schedule week` | 返回本周日程 |
| 用户说「把这个任务标记为完成」 | 获取当前任务 ID → 调用 `task complete` | 任务状态更新 |

### 技术验证

```bash
# 1. 验证 Skill 文件格式
markdownlint skills/datecalendar/skill.md

# 2. 验证示例可运行
cd skills/datecalendar/examples
./create-task.sh  # 应成功创建任务

# 3. 验证 OpenAPI 规范可访问
curl http://127.0.0.1:9876/api-docs/openapi.json | jq .paths | jq 'keys | length'
# 应输出: 路由数量（如 15）

# 4. 端到端测试：模拟 workbuddy 调用
#    （用一个简单的 shell 脚本模拟 workbuddy）
./test-workbuddy-simulation.sh
```
