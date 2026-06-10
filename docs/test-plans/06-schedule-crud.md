# 测试流程：日程 CRUD

## 前置条件
- 在线模式：运行 `start.bat`（Tauri 桌面应用 + HTTP API :9876 + 浏览器 :5173）
- 离线模式：仅 `npx vite`（SQL.js 降级，数据不持久化）

## 白盒测试（Rust 后端）

```bash
cd datecalendar/src-tauri
cargo test --lib test_create_schedule test_create_all_day_schedule test_get_all_schedules test_get_schedules_in_range test_get_day_schedules test_get_schedules_by_task test_update_schedule test_delete_schedule -- --nocapture
```

### 覆盖用例

| # | 用例 | 验证点 |
|---|------|--------|
| 1 | `test_create_schedule` | 创建日程 |
| 2 | `test_create_all_day_schedule` | 全天日程 |
| 3 | `test_get_all_schedules` | 获取全部日程 |
| 4 | `test_get_schedules_in_range` | 日期范围查询 |
| 5 | `test_get_day_schedules` | 日视图查询 |
| 6 | `test_get_schedules_by_task` | 按任务查询 |
| 7 | `test_update_schedule` | 更新日程 |
| 8 | `test_delete_schedule` | 删除日程 |

## 前端黑盒测试（Playwright）

> 以下用例可在在线模式（HTTP API）或离线模式（SQL.js 降级）中执行。在线模式操作真实数据库，离线模式数据仅存内存。

### TC-01: 创建日程

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 切换到"日程"视图 | 显示日历视图 |
| 2 | 点击某个时间段 | 弹出创建日程对话框 |
| 3 | 输入标题"团队会议" | 标题字段填充 |
| 4 | 选择时间范围 | 开始/结束时间设置 |
| 5 | 选择关联任务 | 任务下拉选择 |
| 6 | 点击"保存" | 日程显示在日历中 |

### TC-02: 查看日程

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 切换到日视图 | 显示当天日程 |
| 2 | 切换到周视图 | 显示当周日程 |
| 3 | 切换日期 | 显示对应日期日程 |

### TC-03: 编辑日程

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 点击日历中的日程 | 弹出编辑对话框 |
| 2 | 修改标题和时间 | 字段可编辑 |
| 3 | 点击"保存" | 日程更新成功 |

### TC-04: 删除日程

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 点击日历中的日程 | 弹出编辑对话框 |
| 2 | 点击"删除" | 日程从日历中移除 |
