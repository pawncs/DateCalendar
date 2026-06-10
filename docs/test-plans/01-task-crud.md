# 测试流程：任务 CRUD

## 前置条件
- 在线模式：运行 `start.bat`（Tauri 桌面应用 + HTTP API :9876 + 浏览器 :5173）
- 离线模式：仅 `npx vite`（SQL.js 降级，数据不持久化）

## 白盒测试（Rust 后端）

```bash
cd datecalendar/src-tauri
cargo test --lib test_create_task_basic test_create_task_with_parent test_create_task_sort_order_increment test_get_all_tasks test_get_task_by_id test_update_task_fields test_update_task_status_completed_at test_delete_task_cascade -- --nocapture
```

### 覆盖用例

| # | 用例 | 验证点 |
|---|------|--------|
| 1 | `test_create_task_basic` | 创建任务含所有字段 |
| 2 | `test_create_task_with_parent` | 创建子任务 |
| 3 | `test_create_task_sort_order_increment` | sort_order 自动递增 |
| 4 | `test_get_all_tasks` | 获取全部任务列表 |
| 5 | `test_get_task_by_id` | 按 ID 查询 |
| 6 | `test_update_task_fields` | 更新所有字段 |
| 7 | `test_update_task_status_completed_at` | completed_at 自动填充 |
| 8 | `test_delete_task_cascade` | 递归 CTE 级联删除 |

## 前端黑盒测试（Playwright）

> 以下用例可在在线模式（HTTP API）或离线模式（SQL.js 降级）中执行。在线模式操作真实数据库，离线模式数据仅存内存。

### TC-01: 创建任务

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 点击"添加任务"按钮 | 弹出创建任务对话框 |
| 2 | 输入标题"测试任务" | 标题字段填充 |
| 3 | 选择优先级"高" | 优先级设为 3 |
| 4 | 选择颜色标记 | 颜色更新 |
| 5 | 点击"保存" | 任务创建成功，显示在任务树中 |

### TC-02: 编辑任务

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 选中一个任务 | 右侧显示任务详情面板 |
| 2 | 修改标题 | 标题字段可编辑 |
| 3 | 修改状态 | 状态下拉可切换 |
| 4 | 修改优先级 | 优先级下拉可切换 |
| 5 | 点击"保存" | 更新持久化，重新加载后保持 |

### TC-03: 删除任务

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 创建父子孙三级任务 | 三级树形结构 |
| 2 | 删除父任务 | 父子孙全部删除（级联） |
| 3 | 验证列表 | 无残留任务 |
