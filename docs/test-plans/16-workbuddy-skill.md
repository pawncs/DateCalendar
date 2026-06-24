# 测试流程：workbuddy Skill

> 对应设计文档：[D-21 workbuddy Skill](../design/D-21-workbuddy-skill.md)

## 前置条件

- DateCalendar Tauri 应用已启动
- `datecalendar-cli` 已编译且在 PATH 中（或知道其路径）
- Skill 文件已放置在 workbuddy 可读取的位置
- （可选）workbuddy 或类似 AI 助手正在运行

---

## 白盒测试（Skill 文件格式）

```bash
# 验证 Skill 文件格式
markdownlint skills/datecalendar/skill.md

# 验证示例文件可运行
cd skills/datecalendar/examples
./create-task.sh  # 应成功创建任务
```

### 覆盖用例

| # | 用例 | 验证点 |
|---|------|--------|
| 1 | `test_skill_md_exists` | `skill.md` 文件存在且非空 |
| 2 | `test_skill_md_format` | Markdown 格式合法，可被广泛解析 |
| 3 | `test_examples_exist` | 所有示例文件都存在 |
| 4 | `test_examples_runnable` | 所有示例都能成功执行 |
| 5 | `test_openapi_accessible` | OpenAPI 规范可访问（`/api-docs/openapi.json`） |
| 6 | `test_readme_exists` | `README.md` 存在，内容完整 |

---

## 手动黑盒测试（模拟 workbuddy 调用）

### TC-01: 模拟 workbuddy 读取 Skill

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 打开 `skills/datecalendar/skill.md` | 文件可读，结构清晰 |
| 2 | 检查文件是否包含「快速开始」部分 | 包含，且有 CLI 和 HTTP API 示例 |
| 3 | 检查文件是否包含「核心概念」部分 | 包含，且定义了 Task 和 Schedule |
| 4 | 检查文件是否包含「API 端点清单」部分 | 包含，且有 CLI 和 HTTP API 对照 |

### TC-02: 模拟 workbuddy 执行「创建任务」

根据 `skill.md` 中的说明，模拟 workbuddy 的行为：

```bash
# 场景：用户说「帮我创建一个任务，标题是 测试任务」
# workbuddy 应执行：

datecalendar-cli task create "测试任务"

# 验证：任务是否被创建
datecalendar-cli task list --format json | jq '.[] | select(.title=="测试任务")'
# 应有输出
```

### TC-03: 模拟 workbuddy 执行「创建日程」

```bash
# 场景：用户说「帮我把明天下午 3 点的会议加进去」
# workbuddy 需要：
#   1. 解析时间：「明天」= 当前日期 +1 天，「下午 3 点」= 15:00
#   2. 构造命令

# 计算时间（Windows PowerShell）
$tomorrow = (Get-Date).AddDays(1).ToString("yyyy-MM-dd")
$start = "$tomorrow T15:00:00"
$end = "$tomorrow T16:00:00"

# 执行命令
datecalendar-cli schedule create `
  --title "会议" `
  --start "$start" `
  --end "$end" `
  --type fixed

# 验证
datecalendar-cli schedule day $tomorrow
```

### TC-04: 模拟 workbuddy 执行「查看今日待办」

```bash
# 场景：用户说「查看今天的待办」
# workbuddy 执行：

$today = (Get-Date).ToString("yyyy-MM-dd")
datecalendar-cli schedule day $today

# 验证：输出为 JSON 数组
```

---

## workbuddy 集成测试（如果 workbuddy 可用）

### TC-05: workbuddy 加载 Skill

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 在 workbuddy 中「加载 Skill」 | 成功加载 `skills/datecalendar/skill.md` |
| 2 | workbuddy 读取 Skill 内容 | 能理解 DateCalendar 的 API |
| 3 | 用户对 workbuddy 说：「你能帮助我管理 DateCalendar 日程吗？」 | workbuddy 回答「能」，并说明可以做什么 |

### TC-06: workbuddy 执行简单任务

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 用户：「帮我创建一个任务，标题是 workbuddy测试」 | workbuddy 调用 CLI 或 HTTP API |
| 2 | 检查 DateCalendar | 出现名为「workbuddy测试」的任务 |
| 3 | 用户：「把这个任务标记为完成」 | workbuddy 获取任务 ID，调用 `task complete` |
| 4 | 检查 DateCalendar | 该任务状态变为 `completed` |

### TC-07: workbuddy 执行复杂任务

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 用户：「帮我规划下周的日程：周一到周五每天上午 9 点到 10 点深度学习 Rust」 | workbuddy 解析：<br>- 下周一 = 日期<br>- 每天创建日程 |
| 2 | workbuddy 执行（可能调用批量创建） | 5 条日程被创建 |
| 3 | 检查 DateCalendar | 下周一到周五 9:00-10:00 都有日程 |

---

## 示例文件验证

### TC-08: CLI 示例验证

```bash
# 运行 skills/datecalendar/examples/create-task.sh
cd skills/datecalendar/examples
./create-task.sh

# 验证
datecalendar-cli task list --format json | jq '.[] | select(.priority==3) | .title'
# 应输出 "完成 Q2 报告"
```

### TC-09: HTTP 示例验证

使用 Postman 或 curl 导入 `examples/create-schedule.http`：

```bash
# 手动执行 HTTP 请求（需要先获取 token）
token=$(datecalendar-cli auth token)

curl -X POST http://127.0.0.1:9876/api/schedules \
  -H "Authorization: Bearer $token" \
  -H "Content-Type: application/json" \
  -d '{
    "taskId": "<existing-task-id>",
    "title": "团队周会",
    "startTime": "2026-06-20T10:00:00",
    "endTime": "2026-06-20T11:00:00",
    "isAllDay": false,
    "scheduleType": "fixed",
    "color": "#3b82f6"
  }'
```

---

## Skill 文档完整性检查

### TC-10: Skill 覆盖核心场景

检查 `skill.md` 是否包含以下场景：

| 场景 | 是否在 Skill 中 | 是否有示例 |
|------|----------------|------------|
| 创建任务 | ✅ / ❌ | ✅ / ❌ |
| 创建日程 | ✅ / ❌ | ✅ / ❌ |
| 查看今日/本周日程 | ✅ / ❌ | ✅ / ❌ |
| 标记任务完成 | ✅ / ❌ | ✅ / ❌ |
| 搜索任务 | ✅ / ❌ | ✅ / ❌ |
| 错误处理 | ✅ / ❌ | ✅ / ❌ |

---

## 技术验证

```bash
# 1. 验证 Skill 文件存在
ls -la skills/datecalendar/
# 应输出：skill.md, README.md, examples/, schema/

# 2. 验证示例可运行
cd skills/datecalendar/examples
./create-task.sh
./create-schedule.sh  # 如果存在

# 3. 验证 OpenAPI 规范可访问（Skill 中引用）
curl http://127.0.0.1:9876/api-docs/openapi.json | jq .info.title
# 应输出："DateCalendar API"

# 4. 端到端测试：模拟 workbuddy
#   （用一个简单的 shell 脚本模拟 workbuddy 的调用）
./test-workbuddy-simulation.sh
```

### `test-workbuddy-simulation.sh` 示例

```bash
#!/bin/bash
# 模拟 workbuddy 的行为

echo "=== workbuddy 模拟测试 ==="

# 场景 1：创建任务
echo "[场景 1] 创建任务"
datecalendar-cli task create "workbuddy模拟任务"
if [ $? -eq 0 ]; then
  echo "✅ 场景 1 通过"
else
  echo "❌ 场景 1 失败"
fi

# 场景 2：查看今日日程
echo "[场景 2] 查看今日日程"
today=$(date +%Y-%m-%d)
datecalendar-cli schedule day $today
if [ $? -eq 0 ]; then
  echo "✅ 场景 2 通过"
else
  echo "❌ 场景 2 失败"
fi

echo "=== 测试完成 ==="
```
