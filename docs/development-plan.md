# DateCalendar 桌面日程管理应用 — 开发计划

## 一、项目概述

**DateCalendar** 是一个本地桌面日程管理应用，核心功能：
- 树形任务管理（任务 → 里程碑 → 子任务，支持无限层级）
- 日程安排（任务关联时间段，日/周视图）
- 桌面悬浮小部件（屏幕右侧，丝滑交互）
- 本地 HTTP REST API + CLI 工具，供 workbuddy 调用
- 纯本地存储，无需登录/云端

---

## 二、技术选型

| 层面 | 选型 | 理由 |
|------|------|------|
| 桌面框架 | **Tauri v2** | 内存低(~10MB)，Rust后端性能好，原生系统API，跨平台 |
| 前端框架 | **React 18 + TypeScript** | 生态成熟，状态管理方便，组件化开发 |
| UI 组件库 | **Ant Design / shadcn/ui** | 开箱即用，支持暗色模式 |
| 状态管理 | **Zustand** | 轻量，适合中等复杂度应用 |
| 数据存储 | **SQLite (via rusqlite)** | 结构化查询，支持树形关系，单文件存储 |
| 悬浮窗 | **Tauri 多窗口 + 原生置顶** | 独立窗口，always-on-top，无边框，透明背景 |
| 全局热键 | **Tauri global-shortcut** | 系统级热键注册 |
| API 服务 | **Actix-web (嵌入 Rust)** | 轻量 HTTP 服务，localhost 运行 |
| CLI 工具 | **clap (Rust)** | 命令行参数解析，JSON stdout 输出 |
| 日程渲染 | **FullCalendar / 自研轻量组件** | 日/周视图 |
| 动画 | **Framer Motion** | 悬浮窗滑入/滑出动画 |

---

## 三、系统架构

```
┌─────────────────────────────────────────────────────────────────┐
│                       DateCalendar                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌───────────────┐  │
│  │ 主窗口    │  │ 悬浮窗    │  │ 浏览器    │  │  系统托盘      │  │
│  │ (React)  │  │ (React)  │  │ (React)  │  │  (Tray Icon)  │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └───────┬───────┘  │
│       │             │             │                  │          │
│       │    ┌────────┴────────┐    │                  │          │
│       │    │  适配层 (选择接入) │◄───┘                  │          │
│       │    │  tauri/http/sqljs │                       │          │
│       │    └────────┬────────┘                          │          │
│       │             │                                   │          │
│  ┌────┴─────────────┴───────────────────────────────────┴──────┐  │
│  │                    Tauri Runtime (Rust)                      │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────────┐                 │  │
│  │  │ Commands │ │ HTTP API │ │ CLI Handler   │                 │  │
│  │  │ (IPC)    │ │ (Actix)  │ │ (clap)        │                 │  │
│  │  │ 桌面调用  │ │ 浏览器调用 │ │ 外部脚本调用   │                 │  │
│  │  └────┬─────┘ └────┬─────┘ └──────┬───────┘                 │  │
│  │       │             │              │                          │  │
│  │  ┌────┴─────────────┴──────────────┴──────────────────────┐  │  │
│  │  │                 Core Service Layer                     │  │  │
│  │  │  ┌──────────┐ ┌──────────┐ ┌───────────┐               │  │  │
│  │  │  │ TaskSvc  │ │SchedSvc  │ │ NoteSvc   │               │  │  │
│  │  │  └────┬─────┘ └────┬─────┘ └─────┬─────┘               │  │  │
│  │  │       │             │             │                      │  │  │
│  │  │  ┌────┴─────────────┴─────────────┴──────────────────┐  │  │  │
│  │  │  │              SQLite Database                      │  │  │  │
│  │  │  │            datecalendar.db (同一份)                │  │  │  │
│  │  │  └───────────────────────────────────────────────────┘  │  │  │
│  │  └─────────────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  浏览器离线降级: SQL.js 内存数据库（Tauri 未启动时）                  │
└─────────────────────────────────────────────────────────────────────┘

四条通信路径:
  桌面应用   → Tauri IPC (invoke)       → Rust 服务层 → SQLite
  浏览器     → HTTP REST API (:9876)     → Rust 服务层 → SQLite
  浏览器离线 → SQL.js WASM (降级模式)    → 内存 SQLite
  外部工具   → CLI (datecalendar-cli)    → Rust 服务层 → SQLite
```

---

## 四、数据模型设计

### 4.1 核心表结构

```sql
-- 任务表 (树形结构，通过 parent_id 自引用)
CREATE TABLE tasks (
    id            TEXT PRIMARY KEY,        -- UUID
    parent_id     TEXT REFERENCES tasks(id), -- 父任务ID，NULL=根任务
    title         TEXT NOT NULL,           -- 任务标题
    description   TEXT DEFAULT '',         -- 任务描述
    status        TEXT DEFAULT 'pending',  -- pending | in_progress | completed | cancelled
    priority      INTEGER DEFAULT 0,       -- 优先级 0-3
    sort_order    INTEGER DEFAULT 0,       -- 同级排序
    color         TEXT DEFAULT '',         -- 标记颜色
    is_milestone  INTEGER DEFAULT 0,       -- 是否为里程碑节点
    created_at    TEXT NOT NULL,           -- ISO 8601
    updated_at    TEXT NOT NULL,           -- ISO 8601
    completed_at  TEXT                     -- ISO 8601，完成时间
);

-- 里程碑/节点风险备注
CREATE TABLE milestone_risks (
    id            TEXT PRIMARY KEY,
    task_id       TEXT NOT NULL REFERENCES tasks(id),
    risk_desc     TEXT NOT NULL,           -- 风险描述
    probability   TEXT DEFAULT 'medium',   -- low | medium | high
    mitigation    TEXT DEFAULT '',         -- 应对措施
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

-- 笔记表 (与任务关联，但不体现在日程中)
CREATE TABLE notes (
    id            TEXT PRIMARY KEY,
    task_id       TEXT NOT NULL REFERENCES tasks(id),
    title         TEXT NOT NULL,
    content       TEXT DEFAULT '',
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

-- 日程安排表 (将任务关联到时间段)
CREATE TABLE schedules (
    id            TEXT PRIMARY KEY,
    task_id       TEXT NOT NULL REFERENCES tasks(id),
    title         TEXT NOT NULL,           -- 日程标题（可与任务标题不同）
    start_time    TEXT NOT NULL,           -- ISO 8601
    end_time      TEXT NOT NULL,           -- ISO 8601
    is_all_day    INTEGER DEFAULT 0,       -- 是否全天
    schedule_type TEXT DEFAULT 'fixed',    -- fixed(固定时间) | todo_day(本日待办) | todo_week(本周待办)
    status        TEXT DEFAULT 'pending',  -- pending | completed | cancelled
    color         TEXT DEFAULT '',
    created_at    TEXT NOT NULL,
    updated_at    TEXT NOT NULL
);

-- 应用设置
CREATE TABLE settings (
    key           TEXT PRIMARY KEY,
    value         TEXT NOT NULL
);
```

### 4.2 数据关系示意

```
Task A (根任务)
├── 里程碑 1 (is_milestone=1)
│   ├── risk: "技术方案可能变更"
│   ├── 子任务 1.1
│   │   └── 笔记: "调研笔记..."
│   └── 子任务 1.2
├── 里程碑 2 (is_milestone=1)
│   └── 子任务 2.1
└── 子任务 A.1
    └── notes: "随手记..."

Schedule:
  里程碑 1 → 6月10日-6月12日 (todo_week)
  子任务 1.1 → 6月10日 14:00-16:00 (fixed)
  子任务 2.1 → 6月11日 (todo_day)
```

---

## 五、代码量估算

| 层级 | 技术栈 | 预估代码量 | 说明 |
|------|--------|-----------|------|
| Rust 后端 (src-tauri) | Rust | 8,000-12,000 行 | 数据库层、服务层、IPC Commands、HTTP API 路由、CLI、悬浮窗管理、热键 |
| React 前端 (src) | TSX + CSS | 12,000-18,000 行 | 主窗口、悬浮窗、所有页面组件、状态管理 |
| 适配层 + 浏览器后端 | TypeScript | 1,500-2,500 行 | 环境检测、三种接入适配器、HTTP 客户端、SQL.js 降级、OfflineBanner |
| CLI 工具 | Rust (clap) | 1,500-2,500 行 | 独立 CLI 二进制，命令行参数解析 |
| workbuddy Skill | Markdown + JSON | 400-600 行 | Skill 定义、格式规范、示例 |
| 配置文件 | TOML / JSON / YAML | 500-800 行 | Cargo.toml, tauri.conf, vite, tsconfig 等 |
| 测试代码 | Rust + TypeScript | 3,000-5,000 行 | Rust 单元测试、前端 Playwright 黑盒测试 |
| **合计** | | **约 27,000-42,000 行** | |

---

## 六、UI/UX 设计阶段

> 由于本项目由 AI 全栈开发，UI 设计将在编码过程中同步产出。设计采用 **shadcn/ui + Tailwind CSS** 组件体系，利用其内置的设计语言（间距、颜色、圆角、阴影），减少从零设计的工作量。整体风格定位：**现代简约、暗色优先、毛玻璃质感**。

### 6.1 设计原则

| 原则 | 说明 |
|------|------|
| 暗色优先 | 默认暗色主题，降低长时间使用的视觉疲劳 |
| 毛玻璃质感 | 悬浮窗、弹窗采用 backdrop-blur，营造层次感 |
| 低饱和度配色 | 避免刺眼，任务颜色标记使用柔和色调 |
| 微交互 | 打钩动画、拖拽反馈、展开折叠过渡，提升品质感 |
| 信息密度适中 | 主窗口充分展示信息，悬浮窗精简紧凑 |

### 6.2 关键页面设计清单

| 页面/组件 | 设计重点 | 预估工时 |
|-----------|----------|----------|
| 主窗口布局 | 左侧任务树 + 右侧日程视图双栏，可拖拽调整分栏宽度，底部状态栏 | 0.5天 |
| 任务树面板 | 树形缩进、展开/折叠图标、打钩复选框、拖拽手柄、里程碑星标、优先级色条 | 0.5天 |
| 任务详情面板 | 标题编辑、描述区、优先级选择器、里程碑风险备注区、笔记列表入口 | 0.5天 |
| 日视图 | 24小时时间轴左侧、任务块右侧、当前时间红线、固定时间/待办区分样式 | 0.5天 |
| 周视图 | 7列网格、任务块跨天展示、今日列高亮、顶部日期导航 | 0.5天 |
| 待办列表视图 | 紧凑卡片式、打钩动画 + 划线、进度条、按优先级排序 | 0.25天 |
| 桌面悬浮窗 | 紧凑今日待办卡片、毛玻璃背景、透明度滑块、滑入/滑出动效 | 0.5天 |
| 设置页面 | 分组设置(通用/热键/悬浮窗/API)、表单布局 | 0.25天 |
| 系统托盘菜单 | 右键菜单、快捷操作入口 | 0.25天 |
| 空状态引导 | 无任务/无日程时的插画+引导文案 | 0.25天 |
| 暗色/亮色主题 | 两套配色方案，CSS变量驱动 | 0.25天 |
| 交互动效 | 悬浮窗滑入滑出、打钩划线动画、拖拽反馈、展开折叠过渡 | 0.5天 |
| **合计** | | **约 5 天** |

### 6.3 设计 Token（参考）

```
颜色系统 (HSL):
  Background: 220 15% 8% (dark) / 0 0% 100% (light)
  Surface:    220 15% 12% (dark) / 220 15% 96% (light)
  Primary:    210 100% 55% (蓝色强调)
  Success:    140 60% 50% (完成状态绿)
  Warning:    40 100% 55% (风险/优先级橙)
  Text:       220 15% 90% (dark) / 220 15% 15% (light)

间距系统: 4px 基准 (4, 8, 12, 16, 20, 24, 32, 48, 64)
圆角: sm=4px, md=8px, lg=12px, xl=16px
阴影: 多层阴影系统，悬浮窗使用大范围柔和阴影
字体: 系统默认字体栈 (-apple-system, BlinkMacSystemFont, "Segoe UI", ...)
```

### 6.4 设计交付方式

- 不产出独立 Figma 文件
- 设计直接在代码中通过 shadcn/ui + Tailwind 实现
- 关键页面先做静态原型，确认后再接入数据
- 利用 shadcn/ui 的组件体系保证视觉一致性

---

## 七、功能模块划分

### 模块零：多端接入 (预计 3-4 天)

| 子任务 | 说明 | 预估工时 |
|--------|------|----------|
| 0.1 适配层 | 环境检测、三种模式路由（tauri/http/sqljs）、Proxy 代理 | 1天 |
| 0.2 HTTP 客户端 | fetch 封装、27 个 API 接口映射、错误处理 | 1天 |
| 0.3 SQL.js 降级 | Schema 同步、任务/日程 CRUD、离线数据库初始化 | 1.5天 |
| 0.4 离线提示 UI | OfflineBanner 组件、状态栏集成、自动检测 | 0.5天 |

### 模块一：基础设施 (预计 5-7 天)

| 子任务 | 说明 | 预估工时 |
|--------|------|----------|
| 1.1 Tauri 项目初始化 | 创建 Tauri v2 + React + TS 项目骨架 | 0.5天 |
| 1.2 项目工程化配置 | ESLint, Prettier, Husky, 路径别名, 构建配置 | 0.5天 |
| 1.3 SQLite 数据库层 | 建表、迁移、连接池 (r2d2 + rusqlite) | 1天 |
| 1.4 数据访问层 (DAO) | CRUD 封装，树形查询（递归CTE） | 1.5天 |
| 1.5 全局状态管理 | Zustand store 设计，前端数据流 | 1天 |
| 1.6 主题系统 | 亮色/暗色主题，CSS变量 | 0.5天 |
| 1.7 图标与字体 | 图标库集成，字体配置 | 0.5天 |

### 模块二：任务管理核心 (预计 8-10 天)

| 子任务 | 说明 | 预估工时 |
|--------|------|----------|
| 2.1 任务 CRUD | 创建、编辑、删除任务 | 1.5天 |
| 2.2 树形结构组件 | 无限层级树，拖拽排序，展开/折叠 | 2天 |
| 2.3 里程碑管理 | 里程碑标记、风险备注输入、风险列表 | 1.5天 |
| 2.4 任务状态流转 | pending → in_progress → completed，打钩交互 | 1天 |
| 2.5 笔记系统 | 任务关联笔记，富文本/Markdown编辑 | 1.5天 |
| 2.6 搜索与筛选 | 任务搜索，状态/优先级筛选 | 1天 |
| 2.7 批量操作 | 批量完成、批量删除、批量移动 | 0.5天 |

### 模块三：日程管理 (预计 5-7 天)

| 子任务 | 说明 | 预估工时 |
|--------|------|----------|
| 3.1 日视图 | 当日时间轴，任务安排展示 | 1.5天 |
| 3.2 周视图 | 一周概览，拖拽安排任务 | 1.5天 |
| 3.3 日程创建/编辑 | 将任务拖入时间段，设置固定/待办类型 | 1.5天 |
| 3.4 待办列表视图 | 本日待办 / 本周待办列表 | 1天 |
| 3.5 日程状态管理 | 完成打钩，自动同步任务状态 | 1天 |
| 3.6 时间冲突检测 | 同一时段多任务提醒 | 0.5天 |

### 模块四：桌面悬浮窗 (预计 6-8 天)

| 子任务 | 说明 | 预估工时 |
|--------|------|----------|
| 4.1 悬浮窗窗口创建 | Tauri多窗口，无边框，置顶，透明背景 | 1天 |
| 4.2 屏幕右侧停靠 | 窗口吸附右侧，屏幕边缘检测 | 1.5天 |
| 4.3 滑入/滑出动画 | Framer Motion 丝滑动画，鼠标靠近触发 | 1.5天 |
| 4.4 透明度控制 | 滑块调节透明度，快捷键切换 | 0.5天 |
| 4.5 热键系统 | 全局热键注册 (显示/隐藏、切换透明度) | 1天 |
| 4.6 定时自动隐藏 | 无操作N秒自动隐藏，可配置 | 0.5天 |
| 4.7 悬浮窗内容 | 今日待办/本周概览紧凑视图 | 1天 |
| 4.8 系统托盘 | 托盘图标，右键菜单 (显示/隐藏/退出) | 0.5天 |

### 模块五：API 服务 & CLI (预计 5-7 天)

| 子任务 | 说明 | 预估工时 |
|--------|------|----------|
| 5.1 HTTP API 框架 | Actix-web 嵌入 Tauri，localhost 端口 | 1天 |
| 5.2 任务 API | 查询/创建/修改/删除任务，结构化JSON | 1天 |
| 5.3 日程 API | 日/周日程查询，日程安排/修改 | 1天 |
| 5.4 状态更新 API | 完成任务/里程碑打钩 | 0.5天 |
| 5.5 CLI 工具 | clap 命令行，任务/日程 CRUD，JSON输出 | 1.5天 |
| 5.6 API 认证 | 本地 token 简单认证，防止其他进程调用 | 0.5天 |
| 5.7 API 文档 | OpenAPI/Swagger 文档自动生成 | 0.5天 |

### 模块六：workbuddy Skill (预计 2-3 天)

| 子任务 | 说明 | 预估工时 |
|--------|------|----------|
| 6.1 Skill 规范设计 | 定义 Skill 的输入输出格式规范 | 0.5天 |
| 6.2 Skill 实现 | 创建 CodeBuddy skill，封装 API 调用 | 1天 |
| 6.3 Skill 文档 | 使用说明、示例、格式规范 | 0.5天 |
| 6.4 集成测试 | workbuddy 调用场景覆盖测试 | 0.5天 |

### 模块七：测试与优化 (预计 4-5 天)

| 子任务 | 说明 | 预估工时 |
|--------|------|----------|
| 7.1 单元测试 | Rust 后端测试，关键逻辑覆盖 | 1天 |
| 7.2 集成测试 | 前后端联调，API 测试 | 1天 |
| 7.3 性能优化 | 大数据量树形渲染优化，虚拟列表 | 1天 |
| 7.4 内存优化 | Tauri 打包体积优化，运行时内存监控 | 0.5天 |
| 7.5 异常处理 | 全局错误处理，优雅降级 | 0.5天 |
| 7.6 安装包制作 | Windows MSI/NSIS 安装包，自动更新 | 1天 |

---

## 八、总体时间估算

| 模块 | 名称 | 预估时间 |
|------|------|----------|
| 设计 | UI/UX 设计（随编码同步） | 5 天 |
| 零 | 多端接入（适配层 + HTTP + SQL.js 降级） | 3-4 天 |
| 一 | 基础设施 | 5-7 天 |
| 二 | 任务管理核心 | 8-10 天 |
| 三 | 日程管理 | 5-7 天 |
| 四 | 桌面悬浮窗 | 6-8 天 |
| 五 | API 服务 & CLI | 5-7 天 |
| 六 | workbuddy Skill | 2-3 天 |
| 七 | 测试与优化 | 4-5 天 |
| **合计** | | **43-56 天** |

> **预计总工期：约 9-12 周**（按单人全职开发估算）
> 代码量预估：**27,000-42,000 行**

---

## 九、开发阶段规划

```
Phase 0 (第1周): 多端接入 + UI/UX 设计
  ├── 适配层（tauri/http/sqljs 三种模式）
  ├── HTTP 客户端 + SQL.js 降级
  ├── OfflineBanner 离线提示
  └── 各页面设计随对应功能模块同步产出

Phase 1 (第1-2周): 基础设施 + 任务管理核心
  └── 完成数据库、树形任务CRUD、里程碑、笔记

Phase 2 (第3-4周): 日程管理 + 桌面悬浮窗
  └── 完成日/周视图、待办列表、悬浮窗交互

Phase 3 (第5-6周): HTTP API 完善 + CLI + workbuddy Skill
  └── 完善 REST API 路由、CLI工具、Skill包

Phase 4 (第7-8周): 测试优化 + 打包发布
  └── 测试、性能优化、安装包制作、文档
```

---

## 十、关键风险点

| 风险 | 影响 | 应对措施 |
|------|------|----------|
| Tauri v2 API 不稳定 | 开发阻塞 | 锁定版本，关注更新日志，准备 Electron 降级方案 |
| 悬浮窗跨平台兼容性 | Windows/Mac 行为不一致 | 优先保证 Windows，Mac 适配留到后期 |
| 树形结构大数据性能 | 界面卡顿 | 使用虚拟列表 (react-virtuoso)，懒加载子节点 |
| HTTP API 端口冲突 | localhost 端口被占用 | 端口自动检测 + 可配置 |
| 全局热键冲突 | 与其他应用热键冲突 | 可自定义热键组合，冲突检测提示 |

---

## 十一、后续可扩展方向（本期不实现）

- [ ] 数据导出/导入 (JSON/CSV)
- [ ] 定期提醒/通知
- [ ] 番茄钟集成
- [ ] 统计面板 (完成任务数、时间分布)
- [ ] 标签系统
- [ ] 附件支持
- [ ] 多语言 (i18n)
- [ ] 插件系统
- [ ] 云同步 (可选)

---

## 十二、文件结构概览

```
DateCalendar/
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs               # 入口
│   │   ├── lib.rs                # 库入口
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── connection.rs     # SQLite 连接管理
│   │   │   ├── migrations.rs     # 数据库迁移
│   │   │   └── models.rs         # 数据模型
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── task_service.rs   # 任务服务
│   │   │   ├── schedule_service.rs
│   │   │   ├── note_service.rs
│   │   │   └── settings_service.rs
│   │   ├── api/
│   │   │   ├── mod.rs
│   │   │   ├── server.rs         # Actix-web 服务
│   │   │   ├── task_routes.rs
│   │   │   ├── schedule_routes.rs
│   │   │   └── auth.rs
│   │   ├── commands/
│   │   │   ├── mod.rs            # Tauri IPC Commands
│   │   │   ├── task_commands.rs
│   │   │   └── schedule_commands.rs
│   │   ├── cli/
│   │   │   ├── mod.rs
│   │   │   └── main.rs           # CLI 入口
│   │   ├── floating_window.rs    # 悬浮窗管理
│   │   └── hotkey.rs             # 全局热键
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                          # React 前端（桌面 + 浏览器共享）
│   ├── main.tsx
│   ├── App.tsx
│   ├── adapters/                 # 适配层（新增）
│   │   ├── index.ts              #   环境检测 + 三种模式路由
│   │   ├── types.ts              #   接口类型定义
│   │   ├── tauriBackend.ts       #   Tauri IPC 封装
│   │   ├── httpBackend.ts        #   HTTP API 客户端
│   │   └── sqljsBackend.ts       #   SQL.js 离线降级
│   ├── backend/                  # 浏览器降级后端（新增）
│   │   ├── db.ts                 #   SQL.js 初始化
│   │   ├── schema.ts             #   建表 SQL（从 Rust 复制）
│   │   ├── utils.ts              #   UUID、时间戳等工具
│   │   ├── taskBackend.ts        #   任务 CRUD
│   │   └── scheduleBackend.ts    #   日程 CRUD
│   ├── components/
│   │   ├── layout/
│   │   │   ├── MainLayout.tsx
│   │   │   └── Sidebar.tsx
│   │   ├── tasks/
│   │   │   ├── TaskTree.tsx       # 树形任务组件
│   │   │   ├── TaskNode.tsx       # 单个任务节点
│   │   │   ├── TaskEditor.tsx     # 任务编辑面板
│   │   │   ├── MilestonePanel.tsx # 里程碑+风险面板
│   │   │   └── NoteEditor.tsx     # 笔记编辑器
│   │   ├── calendar/
│   │   │   ├── DayView.tsx
│   │   │   ├── WeekView.tsx
│   │   │   ├── TodoListView.tsx
│   │   │   └── ScheduleEditor.tsx
│   │   ├── floating/
│   │   │   ├── FloatingWindow.tsx  # 悬浮窗主体
│   │   │   ├── TodayTodos.tsx
│   │   │   └── WeekOverview.tsx
│   │   └── common/
│   │       ├── Checkbox.tsx
│   │       ├── SearchBar.tsx
│   │       ├── PriorityBadge.tsx
│   │       └── OfflineBanner.tsx  # 离线模式提示（新增）
│   ├── stores/
│   │   ├── taskStore.ts
│   │   ├── scheduleStore.ts
│   │   └── settingsStore.ts
│   ├── hooks/
│   │   ├── useTasks.ts
│   │   ├── useSchedule.ts
│   │   └── useHotkey.ts
│   ├── styles/
│   │   ├── global.css
│   │   ├── themes/
│   │   │   ├── light.css
│   │   │   └── dark.css
│   │   └── floating.css
│   └── types/
│       ├── task.ts
│       ├── schedule.ts
│       └── api.ts
├── skills/                       # workbuddy Skill
│   └── datecalendar/
│       ├── skill.md              # Skill 定义
│       └── README.md
├── start.bat                     # 一键启动（Tauri + 浏览器）
├── package.json
├── tsconfig.json
├── vite.config.ts
└── README.md
```

---

*文档版本: v2.0 | 更新日期: 2026-06-10 | 变更: 新增模块零多端接入、更新架构图（四条通信路径）、更新代码量/工时估算、更新文件结构*
