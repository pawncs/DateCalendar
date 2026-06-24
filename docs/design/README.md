# DateCalendar 功能设计文档索引

> 生成日期: 2026-06-10 | 更新日期: 2026-06-19
> 对应开发计划: 模块零~模块四

## 文档列表

### 模块零：多端接入（桌面 + 浏览器共享数据）✅ 已实现

| 编号 | 文档 | 功能 | 依赖 |
|------|------|------|------|
| D-11 | [浏览器后端](D-11-browser-backend.md) | HTTP API 代理（主方案）+ SQL.js 离线降级、API 路由表、降级 UI 提示、start.bat 改造 | D-04, D-09, D-10 |
| D-12 | [前端适配层](D-12-frontend-adapter.md) | 三种模式环境检测（tauri/http/sqljs）、适配器实现、OfflineBanner 组件 | D-11 |

### 模块二：任务管理核心 ✅ 已实现

| 编号 | 文档 | 功能 | 依赖 |
|------|------|------|------|
| D-01 | [拖拽排序](D-01-drag-sort.md) | @dnd-kit 拖拽调整任务顺序和层级 | 现有 TaskTree/TaskNode |
| D-02 | [状态/优先级筛选](D-02-filter.md) | 按状态/优先级/关键词过滤任务树 | 现有 taskStore |
| D-03 | [批量操作](D-03-batch-operations.md) | 多选任务 → 批量完成/删除/移动 | D-01 建议先完成 |

### 模块三：日程管理 ✅ 已实现

| 编号 | 文档 | 功能 | 依赖 |
|------|------|------|------|
| D-04 | [Schedule 后端 CRUD](D-04-schedule-backend.md) | Rust service + IPC 命令 + scheduleStore | 现有数据库 schedules 表 |
| D-05 | [日视图](D-05-day-view.md) | 24h 时间轴 + 任务块 + 当前时间红线 | D-04 |
| D-06 | [周视图](D-06-week-view.md) | 7 列网格 + 跨天任务块 | D-04, D-05 |
| D-07 | [日程创建/编辑](D-07-schedule-editor.md) | 三种创建入口 + ScheduleEditor 弹窗 | D-04 |
| D-08 | [待办列表视图](D-08-todo-list-view.md) | 今日/本周待办 + 进度条 + 打钩动画 | D-04, D-07 |
| D-09 | [日程状态同步](D-09-status-sync.md) | 日程 ↔ 任务双向状态联动 | D-04 |
| D-10 | [时间冲突检测](D-10-time-conflict.md) | 创建日程时检测时段冲突 + 视图红色标记 | D-04, D-05 |

### 模块四：桌面悬浮窗 🔨 设计中（本次实现目标）

| 编号 | 设计文档 | 测试计划 | 功能 | 依赖 | 可独立验证 |
|------|---------|---------|------|------|-----------|
| D-13 | [悬浮窗窗口创建与停靠](D-13-floating-window.md) | [09-浮动窗口](../test-plans/09-floating-window.md) | Tauri 多窗口、无边框、置顶、透明、屏幕右侧贴边 | D-04（数据查询） | ✅ 启动即见悬浮窗，目视验证位置/样式 |
| D-14 | [悬浮窗交互体验](D-14-floating-interaction.md) | [10-交互内容](../test-plans/10-floating-interaction.md) | Framer Motion 滑入/滑出动画、透明度滑块、定时自动隐藏 | D-13 | ✅ 鼠标触发展示/隐藏，调节透明度 |
| D-15 | [全局热键系统](D-15-global-hotkey.md) | [11-全局热键](../test-plans/11-global-hotkey.md) | 全局热键注册（显隐/透明度循环）、插件集成 | D-13 | ✅ 任意应用中按热键，悬浮窗响应 |
| D-16 | [悬浮窗内容视图](D-16-floating-content.md) | [10-交互内容](../test-plans/10-floating-interaction.md) | 今日待办+接下来日程+本周概览、打钩完成、跳转主窗口 | D-13, D-04 | ✅ 在主窗口创建日程后悬浮窗同步显示 |
| D-17 | [系统托盘菜单](D-17-system-tray.md) | [12-系统托盘](../test-plans/12-system-tray.md) | 托盘图标、右键菜单（显隐主窗口/悬浮窗/退出）、关闭≠退出 | 无硬依赖 | ✅ 托盘图标出现，菜单功能可用 |

### 模块五：API 服务 & CLI ✅ CLI 已完成

| 编号 | 设计文档 | 测试计划 | 功能 | 依赖 | 可独立验证 |
|------|---------|---------|------|------|-----------|
| D-18 | [CLI 工具](D-18-cli-tool.md) ✅ | [13-CLI工具](../test-plans/13-cli-tool.md) | clap 命令行工具、任务/日程 CRUD、JSON 输出、退出码规范 | 共享 TaskService/ScheduleService | ✅ 编译后直接运行 CLI 测试 |
| D-19 | [API 认证（保留入口，暂不实现）](D-19-api-auth.md) | [14-API认证](../test-plans/14-api-auth.md) | Bearer Token 认证设计（预留）、中间件（未实现）、白名单路径（预留） | D-11（HTTP API） | ⏳ 仅验证设计文档，不测试认证 |
| D-20 | [API 文档](D-20-api-docs.md) | [15-API文档](../test-plans/15-api-docs.md) | utoipa 自动生成 OpenAPI 规范、Swagger UI 交互文档 | D-19（认证集成到文档） | ✅ 浏览器打开 /docs 查看文档 |

### 模块六：workbuddy Skill ✅ 已完成

| 编号 | 设计文档 | 测试计划 | 功能 | 依赖 | 可独立验证 |
|------|---------|---------|------|------|-----------|
| D-21 | [workbuddy Skill](D-21-workbuddy-skill.md) ✅ | [16-workbuddy-Skill](../test-plans/16-workbuddy-skill.md) | Skill 定义文件、CLI/HTTP API 调用示例、典型场景描述 | D-18, D-19, D-20 | ✅ 模拟 workbuddy 调用 CLI/API |

## 建议执行顺序

```
Phase 0: 多端接入（已完成 ✅）
  D-11 → D-12 → 实现 + 测试

Phase 2a: 模块二（已完成 ✅）
  D-01 → D-02 → D-03

Phase 2b: 模块三（已完成 ✅）
  D-04 → D-07 → D-05 → D-06 → D-08 → D-09 → D-10

Phase 3: 模块四 — 桌面悬浮窗（已完成 ✅）
  第一步: D-13 (窗口创建停靠) → D-17 (系统托盘)
          ├── 这两个偏 Rust/系统层，可并行或先后进行
          ├── 验证：启动应用 → 悬浮窗贴边停靠 + 托盘图标出现
          └── 测试：09-floating-window.md + 12-system-tray.md
  第二步: D-14 (交互动画) → D-16 (内容视图)
          ├── 这两个偏前端，依赖 D-13 的窗口就绪
          ├── 验证：悬浮窗滑入滑出 + 展示今日待办
          └── 测试：10-floating-interaction.md
  第三步: D-15 (全局热键)
          ├── 最后集成，因为需要前几步都就绪才能验证完整链路
          └── 测试：11-global-hotkey.md

Phase 4: 模块五 — API 服务 & CLI（待实现）
  第一步: D-18 (CLI 工具)
          ├── 独立 crate，不依赖 HTTP API
          ├── 验证：编译后运行 CLI 命令
          └── 测试：13-cli-tool.md
  第二步: D-19 (API 认证) ⏳ 保留入口，暂不实现
          ├── 个人使用场景，无需认证
          ├── 设计文档已完成（预留认证方案）
          ├── 未来需要时可快速实现
          └── 跳过测试：14-api-auth.md（暂不执行）
  第三步: D-20 (API 文档)
          ├── 依赖 D-19（认证集成到文档）
          ├── 验证：浏览器打开 /docs 查看 Swagger UI
          └── 测试：15-api-docs.md

Phase 5: 模块六 — workbuddy Skill（待实现）
  第一步: D-21 (workbuddy Skill)
          ├── 依赖 D-18, D-19, D-20（CLI、API 认证、API 文档）
          ├── 验证：模拟 workbuddy 调用 CLI/API
          └── 测试：16-workbuddy-skill.md
```

## 领域知识

开发时需了解的架构概念、技术约定和开发规范，参见 `docs/knowledge/README.md`。

## 验证方法总览

每个文档的「验证标准」部分包含：
- **功能验证**：具体的测试场景和预期结果
- **交互体验验证**：视觉反馈、动画、无障碍
- **技术验证**：编译通过 + 构建成功 + 手动 E2E

每个功能的测试覆盖三种接入方式：IPC（桌面）、HTTP API（浏览器在线）、SQL.js（浏览器离线）。详见 `docs/test-plans/`。

---

*文档版本: v3.1 | 更新日期: 2026-06-24 | 变更: 修改 D-19 API 认证为保留入口但不实现（个人使用场景）*

