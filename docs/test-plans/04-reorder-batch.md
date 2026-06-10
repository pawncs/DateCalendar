# 测试流程：排序与批量操作

## 前置条件
- 在线模式：运行 `start.bat`（Tauri 桌面应用 + HTTP API :9876 + 浏览器 :5173）
- 离线模式：仅 `npx vite`（SQL.js 降级，数据不持久化）

## 白盒测试（Rust 后端）

```bash
cd datecalendar/src-tauri
cargo test --lib test_reorder_task_same_level test_reorder_task_move_to_parent test_reorder_task_cycle_detection test_batch_update_status test_batch_delete test_batch_move -- --nocapture
```

### 覆盖用例

| # | 用例 | 验证点 |
|---|------|--------|
| 1 | `test_reorder_task_same_level` | 同级重排 |
| 2 | `test_reorder_task_move_to_parent` | 跨级移动 |
| 3 | `test_reorder_task_cycle_detection` | 循环引用检测 |
| 4 | `test_batch_update_status` | 批量更新状态 |
| 5 | `test_batch_delete` | 批量删除 |
| 6 | `test_batch_move` | 批量移动 |

## 前端黑盒测试（Playwright）

> 以下用例可在在线模式（HTTP API）或离线模式（SQL.js 降级）中执行。在线模式操作真实数据库，离线模式数据仅存内存。

### TC-01: 拖拽排序

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 创建多个同级任务 | 任务按 sort_order 排列 |
| 2 | 拖拽第三个任务到第一位 | 任务移动到第一位，sort_order 更新 |
| 3 | 刷新验证 | 排序持久化 |

### TC-02: 跨级拖拽

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 创建父任务和独立子任务 | 两个任务无父子关系 |
| 2 | 拖拽独立任务到父任务下 | 独立任务的 parent_id 更新 |
| 3 | 刷新验证 | 父子关系持久化 |

### TC-03: 批量选择

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 点击"批量选择"按钮 | 进入批量选择模式 |
| 2 | 勾选多个任务 | 批量操作栏出现 |
| 3 | 点击"完成" | 所选任务状态变为 completed |
| 4 | 退出选择模式 | 恢复常规视图 |

### TC-04: 批量删除

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 进入批量选择模式 | 选择模式激活 |
| 2 | 勾选多个任务 | 批量操作栏显示 |
| 3 | 点击"删除" | 所选任务及其子任务被删除 |

### TC-05: 批量移动

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 进入批量选择模式 | 选择模式激活 |
| 2 | 勾选多个任务 | 批量操作栏显示 |
| 3 | 选择目标父节点 | 任务移动到新父节点下 |
