# CLAUDE.md

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

## 5. Teaching-Oriented Coding Rules (教学级编码规则)

This project is both a real application AND a teaching environment. The AI acts as **Chief Architect + Technical Tutor + Programmer**. Follow these rules:

### 5.1 Macro Before Micro (先宏观再微观)

Before writing any implementation code:
1. **Explain the big picture first**: What module are we building? Where does it sit in the overall architecture? What problem does it solve?
2. **Then zoom into the implementation**: With context established, write the actual code.
3. **Summarize after**: What did we just build? How does it connect to the next piece?

Pattern:
```
[架构讲解] → 这部分在系统中的位置是…，它解决的问题是…
[实现代码] → 具体代码
[小结] → 我们完成了X，它与Y通过Z连接，下一步是…
```

### 5.2 Knowledge Radar (知识点雷达)

When encountering a concept/technology that the user may not be familiar with:
1. **Flag it**: "这里涉及一个你可能不熟悉的概念：[概念名]"
2. **Explain concisely**: What it is, why we use it, the key mental model (1-3 sentences).
3. **Connect**: How it relates to other concepts already covered.
4. **Record it**: If it's a significant architectural concept, it should be documented in `docs/architecture-knowledge.md`.

Radar format:
```
🔍 知识点雷达: [概念名]
   ├── 是什么: [一句话定义]
   ├── 为什么用: [在这个项目中的理由]
   ├── 核心心智模型: [关键理解]
   └── 关联概念: [与已学概念的关联]
```

### 5.3 Architectural Gatekeeping (宏观把控引导)

The AI must actively guide the user's architectural understanding:
- **Before starting each Phase**: Summarize which architecture concepts are needed, point to relevant docs.
- **During implementation**: When a design decision is made, explain the tradeoff.
- **When the user seems lost**: Pause and offer to explain the relevant architecture before continuing.
- **Keep `docs/architecture-knowledge.md` updated**: Add new concepts as they appear in development.

### 5.4 Code as Teaching Material

- Comments in code should explain **why**, not **what**.
- For key architectural patterns (e.g., Tauri command registration, SQLite connection pooling, Zustand store design), include a brief doc comment explaining the pattern.
- Non-obvious Rust idioms should be explained inline.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

---

## 6. Environment Constraints (环境约束)

### 6.0 C 盘容量限制

**C 盘容量有限，以下操作需先询问用户：**
- 下载安装新软件（如 npm 全局包、Rust 工具链组件、系统级工具）
- 在 C 盘创建大文件（>10MB）
- 任何写入 `C:\Users\` 下非项目目录的操作

**默认允许的操作：**
- npm/pip/cargo 包安装到项目本地（`node_modules/`、`target/`）
- 项目工作区内所有文件操作

### 6.0.1 数据库存储位置

本项目数据库文件统一存放在工程的 `target/` 目录下：
- Tauri 桌面应用数据库：`datecalendar/src-tauri/target/datecalendar.db`
- 浏览器端数据库（SQL.js）：内存中运行，不落盘；测试数据存 `target/test-data/`
- 测试用数据库：SQLite `:memory:` 模式或 `target/test-data/` 下临时文件

---

## 7. Testing Standards (测试规范)

### 7.1 多端接入测试架构

本项目采用 **同一 Rust 后端，两种接入方式** 的架构：

```
                 ┌─ 共享前端 (React/TSX) ──────────────┐
                 │       适配层选择接入方式               │
                 └──────────┬───────────────────────────┘
                            │
           ┌────────────────┼────────────────┐
           ▼                ▼                ▼
     ┌──────────┐   ┌──────────────┐   ┌──────────┐
     │ Tauri IPC │   │  HTTP API    │   │ SQL.js   │
     │ (桌面应用) │   │ :9876        │   │ (降级)   │
     └─────┬─────┘   └──────┬───────┘   └────┬─────┘
           │                │                │
           └────────┬───────┘                │
                    ▼                        ▼
           ┌──────────────┐       ┌──────────────┐
           │  Rust 服务层  │       │ 内存 SQLite   │
           │  (唯一)      │       │ (离线降级)    │
           └──────┬───────┘       └──────────────┘
                  ▼
           ┌──────────────┐
           │   SQLite      │
           │ datecalendar  │
           │   .db (同一份) │
           └──────────────┘
```

**三种运行模式：**
1. **`tauri` 模式**：桌面应用，IPC 直接调用 Rust 服务层
2. **`http` 模式**：浏览器通过 HTTP API (`:9876`) 代理到 Rust 服务层，操作同一数据库
3. **`sqljs` 模式**：Tauri 未启动时的离线降级，使用 SQL.js 内存数据库

**验证链路：**
```
Rust 服务层  ──cargo test──→  34 个单元测试 ✅ → 业务逻辑正确性
前端 UI     ──Playwright──→  黑盒测试 ✅ → UI+交互正确性
  ├── 在线模式 (HTTP API) → 操作真实数据库
  └── 离线模式 (SQL.js)   → 操作内存数据库
```

### 7.2 测试文档结构

```
docs/
  knowledge/                  # 领域知识手册（开发时速查）
    README.md                 #   多端接入架构、HTTP API、SQL.js降级、适配器、存储规范等
  design/                     # 需求设计文档（指导开发）
    README.md                 #   设计文档索引
    D-01~D-12                 #   各功能设计文档
  test-plans/                 # 测试流程文档（按功能大类拆分）
    01-task-crud.md           # 任务 CRUD
    02-milestone-risk.md      # 里程碑与风险
    03-notes.md               # 笔记
    04-reorder-batch.md       # 排序与批量操作
    05-search-filter.md       # 搜索与筛选
    06-schedule-crud.md       # 日程 CRUD
    07-schedule-sync-conflict.md # 日程状态同步与冲突检测
    08-ui-interaction.md      # UI 交互

target/                       # 测试报告 + 测试数据（不入 docs）
    test-report-YYYY-MM-DD.md
    test-data/                # 测试数据库文件
```

### 7.3 测试流程文档规范

- 每个功能大类一个独立文件，放在 `docs/test-plans/`
- 文件命名：`{序号}-{功能英文名}.md`
- 内容结构：
  - 前置条件（在线模式 `start.bat` + 离线模式 `npx vite`）
  - Rust 后端白盒测试命令（`cargo test`）
  - 覆盖用例表格（用例名 + 验证点，不含测试状态）
  - 前端黑盒测试用例表（步骤 + 操作 + 预期结果，标注在线/离线模式）
- **测试流程文档是"怎么测"的指导，不含任何测试结果状态**
- 拆分原则：防止单个文件过长，一个文件不超过约 100 行

### 7.4 测试报告规范

- 存放位置：`target/` 目录（不在 docs 中）
- 文件命名：`test-report-YYYY-MM-DD.md`（带日期）
- 内容结构：
  - **开头必须声明**：本次报告执行了哪些测试流程文档（链接到 `docs/test-plans/`）
  - Tauri 后端白盒测试结果（按流程文档分组）
  - 浏览器后端白盒测试结果（按流程文档分组）
  - 前端黑盒测试结果（标注哪些已执行、哪些待验证）
  - 发现的问题与修复
  - 测试覆盖度摘要

### 7.5 测试执行流程

当用户要求测试时：
1. 先确认要测试哪些功能模块（对应哪些 test-plan 文档）
2. 运行 Rust 后端白盒测试：`cd datecalendar/src-tauri && cargo test --lib`
3. 运行前端黑盒测试（在线模式）：`start.bat` + Playwright CLI
4. 运行前端黑盒测试（离线模式）：`npx vite` + Playwright CLI
5. 生成测试报告到 `target/test-report-YYYY-MM-DD.md`
6. 报告开头列出执行的流程文档清单

### 7.6 关键测试命令

```bash
# === Rust 后端白盒测试 ===
cd datecalendar/src-tauri && cargo test --lib                    # 全部
cargo test --lib task_service                                    # 按模块
cargo test --lib schedule_service
cargo test --lib test_update_task_milestone_save -- --nocapture  # 单个

# === 前端黑盒测试（在线模式） ===
start start.bat                                                  # 启动 Tauri + 浏览器
playwright-cli open http://localhost:5173                        # Playwright 操作
playwright-cli screenshot --filename=target/screenshots/online-01.png

# === 前端黑盒测试（离线模式） ===
cd datecalendar && npx vite                                      # 仅 Vite
playwright-cli open http://localhost:5173
playwright-cli screenshot --filename=target/screenshots/offline-01.png
```
