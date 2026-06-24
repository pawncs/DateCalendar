# 测试流程：CLI 工具

> 对应设计文档：[D-18 CLI 工具](../design/D-18-cli-tool.md)
> 
> **状态**: ✅ 已实现并验证 (2026-06-24)
> 
> **已验证功能**:
> - ✅ `health` - 数据库连接检查
> - ✅ `task create` - 创建任务（命令行参数）
> - ✅ `task list` - 列出任务（json/table 格式）
> - ✅ `task update` - 更新任务
> - ✅ `task delete` - 删除任务
> - ✅ `task complete` - 标记完成
> - ✅ `task search` - 搜索任务
> - ✅ `schedule create` - 创建日程
> - ✅ `schedule day` - 查看某天日程
> - ✅ `schedule conflicts` - 检测时间冲突
> - ✅ 自动初始化数据库（自动创建 + 迁移）
> - ✅ `--db-path` 参数支持
> 
> **测试环境**:
> - ✅ PowerShell
> - ✅ cmd (Windows Command Prompt)
> - ✅ Rust 单元测试
> 
> **已知限制**:
> - ⚠️ **cmd中引号处理**: 在cmd中使用双引号包裹含空格的参数时，参数值会被加上额外的转义引号。建议使用PowerShell或避免引号（参数不含空格时）。



## 前置条件

- 仅桌面模式：确保 `datecalendar-cli` 已编译（`cargo build --release`）
- 数据库文件 `datecalendar.db` 存在且有测试数据
- 终端/PowerShell 可用

---

## 白盒测试（Rust 后端）

```bash
cd datecalendar-cli
cargo test --lib
```

### 覆盖用例

| # | 用例 | 验证点 |
|---|------|--------|
| 1 | `test_db_discovery_default` | 无 `--db-path` 时自动发现数据库 |
| 2 | `test_db_discovery_env` | 环境变量 `DATECALENDAR_DB` 生效 |
| 3 | `test_db_discovery_arg` | `--db-path` 参数优先级最高 |
| 4 | `test_task_create_minimal` | 最小参数创建任务成功 |
| 5 | `test_task_create_stdin` | stdin JSON 创建任务 |
| 6 | `test_task_list_json` | `--format json` 输出合法 JSON |
| 7 | `test_task_list_table` | `--format table` 输出对齐表格 |
| 8 | `test_task_get_exists` | 获取存在的任务返回 0 退出码 |
| 9 | `test_task_get_not_exists` | 获取不存在的任务返回退出码 1 |
| 10 | `test_schedule_day` | 查看指定日期的日程 |
| 11 | `test_exit_code_db_error` | 数据库错误返回退出码 3 |
| 12 | `test_exit_code_db_not_found` | 数据库不存在返回退出码 4 |

---

## 手动黑盒测试

### TC-01: 版本与帮助信息

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | `datecalendar-cli --version` | 输出版本号（如 `1.0.0`） |
| 2 | `datecalendar-cli --help` | 显示所有子命令和顶层选项 |
| 3 | `datecalendar-cli task --help` | 显示 task 子命令 |
| 4 | `datecalendar-cli task create --help` | 显示 create 的参数说明 |

### TC-02: 数据库连接检查

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | `datecalendar-cli health` | 输出 `{"status":"ok"}`，退出码 0 |
| 2 | `datecalendar-cli --db-path /nonexistent/db.sqlite health` | 输出错误信息，退出码 4 |

### TC-03: 任务创建（命令行参数）

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | `datecalendar-cli task create "CLI测试任务" --priority 2` | 输出 JSON，包含 `id` 和 `title` |
| 2 | 检查输出 JSON 的 `title` 字段 | 值为 `"CLI测试任务"` |
| 3 | 检查输出 JSON 的 `priority` 字段 | 值为 `2` |
| 4 | 检查输出 JSON 的 `status` 字段 | 值为 `"pending"` |

### TC-04: 任务创建（stdin JSON）

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | `echo '{"title":"stdin任务","priority":1}' \| datecalendar-cli task create --stdin` | 输出 JSON，`title` 为 `"stdin任务"` |
| 2 | 无效 JSON | 退出码 2，stderr 输出错误信息 |

### TC-05: 任务列表查询

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | `datecalendar-cli task list` | 输出 JSON 数组 |
| 2 | `datecalendar-cli task list --format table` | 输出 ASCII 表格，包含 ID、TITLE、STATUS、PRIORITY 列 |
| 3 | `datecalendar-cli task list --format csv` | 输出 CSV，逗号分隔 |

### TC-06: 任务获取与删除

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 先创建任务，记录 ID | 创建成功 |
| 2 | `datecalendar-cli task get <ID>` | 输出该任务的 JSON |
| 3 | `datecalendar-cli task delete <ID>` | 退出码 0 |
| 4 | `datecalendar-cli task get <ID>` | 退出码 1（任务已删除） |

### TC-07: 日程创建与查询

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | `datecalendar-cli schedule create --task-id <TASK_ID> --title "CLI日程" --start "2026-06-20T10:00:00" --end "2026-06-20T11:00:00"` | 输出日程 JSON |
| 2 | `datecalendar-cli schedule day 2026-06-20` | 输出包含刚创建的日程 |
| 3 | `datecalendar-cli schedule week 2026-06-16` | 输出本周所有日程 |

### TC-08: 错误场景

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | `datecalendar-cli task get nonexistent-id` | 退出码 1，stderr 有错误信息 |
| 2 | `datecalendar-cli --db-path /fake/path task list` | 退出码 4 |
| 3 | `datecalendar-cli task create` （缺少 title） | 退出码 2，提示缺少参数 |

---

## workbuddy 集成模拟测试

### TC-09: workbuddy 插入日程

模拟 workbuddy 调用 CLI：

```bash
# 模拟 workbuddy 插入一条日程
datecalendar-cli schedule create \
  --title "团队周会" \
  --start "2026-06-20T10:00:00" \
  --end "2026-06-20T11:00:00" \
  --type fixed

# 验证：查询该日日程
datecalendar-cli schedule day 2026-06-20 | jq '.[] | select(.title=="团队周会")'
# 应有输出（找到该日程）
```

### TC-10: workbuddy 查询今日待办

```bash
# 模拟 workbuddy 查询今日待办
today=$(powershell -Command "Get-Date -Format 'yyyy-MM-dd'")
datecalendar-cli schedule day $today

# 验证：输出为 JSON 数组
```

---

## 交互体验验证

| 场景 | 预期 |
|------|------|
| CLI 响应时间 | 单次调用 < 500ms（不含数据库首次连接） |
| 输出格式 | JSON 格式合法，可被 `jq` 解析 |
| 错误信息 | stderr 输出，人类可读 |
| 帮助信息 | 中英文均可（默认英文） |

---

## 技术验证

```bash
# 编译
cd datecalendar-cli
cargo build --release

# 单元测试
cargo test --lib

# 端到端测试
cd datecalendar-cli/target/release
./datecalendar-cli --version
./datecalendar-cli health
./datecalendar-cli task list --format json | jq .
./datecalendar-cli task create "测试任务" --priority 1
./datecalendar-cli task list --format table

# Windows PowerShell 测试
$ErrorActionPreference = "Stop"
datecalendar-cli.exe task list
if ($LASTEXITCODE -ne 0) { Write-Error "CLI failed" }
```
