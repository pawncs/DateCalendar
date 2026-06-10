# 测试流程：日程状态同步与冲突检测

## 前置条件
- 在线模式：运行 `start.bat`（Tauri 桌面应用 + HTTP API :9876 + 浏览器 :5173）
- 离线模式：仅 `npx vite`（SQL.js 降级，数据不持久化）

## 白盒测试（Rust 后端）

```bash
cd datecalendar/src-tauri
cargo test --lib test_update_schedule_status_sync_task test_check_conflicts_no_conflict test_check_conflicts_with_conflict test_check_conflicts_exclude_id test_check_conflicts_ignores_cancelled -- --nocapture
```

### 覆盖用例

| # | 用例 | 验证点 |
|---|------|--------|
| 1 | `test_update_schedule_status_sync_task` | 日程完成→任务同步 |
| 2 | `test_check_conflicts_no_conflict` | 无冲突 |
| 3 | `test_check_conflicts_with_conflict` | 有冲突 |
| 4 | `test_check_conflicts_exclude_id` | 排除自身 |
| 5 | `test_check_conflicts_ignores_cancelled` | 排除已取消 |

## 前端黑盒测试（Playwright）

> 以下用例可在在线模式（HTTP API）或离线模式（SQL.js 降级）中执行。在线模式操作真实数据库，离线模式数据仅存内存。

### TC-01: 日程状态同步

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 创建任务"需求评审" | 任务状态为 pending |
| 2 | 创建关联该任务的日程 | 日程状态为 pending |
| 3 | 将日程标记为"已完成" | 日程状态变为 completed，关联任务也变为 completed |
| 4 | 将日程标记为"已取消" | 日程状态变为 cancelled，关联任务也变为 cancelled |

### TC-02: 冲突检测

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 创建日程A: 09:00-10:00 | 日程A 显示 |
| 2 | 尝试创建日程B: 09:30-10:30 | 系统提示时间冲突 |
| 3 | 修改日程B为 10:00-11:00 | 不再冲突，创建成功 |

### TC-03: 冲突检测排除已取消

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 创建日程A: 09:00-10:00 | 日程A 显示 |
| 2 | 取消日程A | 日程A 状态变为 cancelled |
| 3 | 创建日程B: 09:00-10:00 | 不报告冲突（已取消不参与检测） |
