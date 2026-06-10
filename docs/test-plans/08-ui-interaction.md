# 测试流程：UI 交互

## 前置条件
- 在线模式：运行 `start.bat`（Tauri 桌面应用 + HTTP API :9876 + 浏览器 :5173）
- 离线模式：仅 `npx vite`（SQL.js 降级，数据不持久化）

## 白盒测试

UI 交互无白盒测试，通过 Playwright CLI 进行自动化前端黑盒验证。

## 前端黑盒测试（Playwright）

> 以下用例可在在线模式（HTTP API）或离线模式（SQL.js 降级）中执行。

### TC-01: 主题切换

```bash
# Playwright CLI 自动测试
playwright-cli open http://localhost:5173
playwright-cli click e11  # 点击主题切换按钮
playwright-cli screenshot --filename=theme-toggle.png
playwright-cli eval "document.documentElement.classList.contains('dark')"
```

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 点击"切换到亮色模式"按钮 | 界面切换为亮色主题 |
| 2 | 点击"切换到暗色模式"按钮 | 界面切换为暗色主题 |
| 3 | 刷新页面 | 主题保持（localStorage 持久化） |

### TC-02: 视图导航

```bash
# Playwright CLI 自动测试
playwright-cli click e9   # 点击"日程"
playwright-cli screenshot --filename=schedule-view.png
playwright-cli click e8   # 点击"任务"
playwright-cli screenshot --filename=task-view.png
```

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 点击左侧"任务"按钮 | 显示任务树视图 |
| 2 | 点击左侧"日程"按钮 | 显示日程日历视图 |
| 3 | 再次点击"任务" | 切换回任务视图 |

### TC-03: 展开/折叠

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 点击"全部展开" | 所有任务节点展开 |
| 2 | 点击"全部折叠" | 所有任务节点折叠 |
| 3 | 点击单个任务的展开箭头 | 该任务子节点展开/折叠 |

### TC-04: 错误处理

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 在纯浏览器环境打开 | 显示"加载失败"提示（无 Tauri 后端时） |
| 2 | 在 Tauri 环境打开 | 正常加载任务数据 |
| 3 | 关闭 Tauri 后端 | 后续操作失败，显示错误提示（不会白屏） |

### TC-05: 响应式布局

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 调整窗口大小 | 布局自适应 |
| 2 | 缩小到窄屏 | 侧边栏可能折叠或隐藏 |
