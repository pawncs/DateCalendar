# D-18: CLI 工具

> **状态**: ✅ 已完成 (2026-06-24)
> 
> **实现概要**: 
> - 二进制: `datecalendar-cli.exe` (独立二进制，不依赖 Tauri)
> - 位置: `datecalendar/src-tauri/src/cli/`
> - 测试: `datecalendar/src-tauri/tests/cli_tests.rs`
> - 数据库自动初始化: ✅ (自动创建数据库并运行迁移)



## 1. 必要性 (Why)

### 问题

模块五的前 4 个子任务已完成 HTTP API，但 workbuddy 和脚本化场景需要更轻量的调用方式：
- HTTP API 需要启动 Actix-web 服务器，有端口依赖
- 脚本中调用 CLI 比构造 HTTP 请求更自然
- 批量操作（如从文件导入任务）用 CLI 管道更方便

### 场景

- workbuddy 通过执行 `datecalendar-cli` 命令插入日程
- 用户写脚本批量导入任务（`cat tasks.csv | datecalendar-cli task import`）
- CI/CD 环境中快速查询/创建任务（无浏览器、无 HTTP 服务）
- 系统 cron 定时任务操作日程（如每日自动创建待办）

### 设计原则

- **单一二进制**：编译出一个独立的 `datecalendar-cli.exe`，不依赖 Tauri 运行时
- **共享服务层**：CLI 直接调用 `TaskService` / `ScheduleService`，不经过 HTTP
- **JSON stdin/stdout**：所有输出为 JSON，便于脚本解析；输入支持参数 + stdin JSON
- **退出码规范**：`0` = 成功，`1` = 业务错误，`2` = 参数错误，`3` = 数据库错误

---

## 2. 实现方案 (How)

### 2.1 技术选型

| 技术 | 用途 | 理由 |
|------|------|------|
| `clap` v4 + `derive` 模式 | 命令行参数解析 | 类型安全、自动生成 `--help`、Rust 生态标准 |
| `serde_json` | JSON 序列化 | 与 HTTP API 复用同一批 DTO |
| `r2d2` + `rusqlite` | 数据库连接 | 与 Tauri 后端共享同一份 `datecalendar.db` |
| `ctrlc` | Ctrl+C 信号处理 | 优雅退出（可选） |

### 2.2 命令结构设计

```
datecalendar-cli <子命令> [选项]
```

**顶层选项**：

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `--db-path <PATH>` | 数据库文件路径 | 自动发现（见 2.3） |
| `--format <FORMAT>` | 输出格式：`json` / `table` / `csv` | `json` |
| `--verbose` / `-v` | 详细日志 | `false` |
| `--version` | 版本号 | — |

**子命令树**：

```
datecalendar-cli
├── task                      # 任务管理
│   ├── list                 # 列出所有任务（树形）
│   ├── get <ID>            # 获取单个任务
│   ├── create              # 创建任务（参数或 stdin JSON）
│   ├── update <ID>         # 更新任务
│   ├── delete <ID>         # 删除任务
│   ├── search <QUERY>      # 搜索任务
│   ├── complete <ID>       # 标记完成
│   ├── batch-complete      # 批量完成（stdin 传 ID 列表）
│   └── import             # 从 stdin JSON 批量导入
├── schedule                 # 日程管理
│   ├── list                # 列出所有日程
│   ├── day <DATE>          # 查看某天日程
│   ├── week <START>        # 查看某周日程
│   ├── create              # 创建日程
│   ├── update <ID>         # 更新日程
│   ├── delete <ID>         # 删除日程
│   └── conflicts          # 检测时间冲突
├── health                   # 检查数据库连接
└── version                  # 显示版本信息
```

### 2.3 数据库路径自动发现

CLI 需要找到 `datecalendar.db`，按以下顺序查找：

```rust
fn discover_db_path() -> PathBuf {
    // 1. --db-path 命令行参数
    // 2. 环境变量 DATECALENDAR_DB
    // 3. 默认位置（按操作系统）
    //    Windows: %APPDATA%\DateCalendar\datecalendar.db
    //    macOS:   ~/Library/Application Support/DateCalendar/datecalendar.db
    //    Linux:   ~/.local/share/DateCalendar/datecalendar.db
    // 4. 当前目录 ./datecalendar.db（开发模式）
}
```

> Tauri 生产模式下数据库位置由 Tauri 的 `app_data_dir()` 决定，CLI 需与之对齐。

### 2.4 输入输出格式

**创建任务示例**：

```bash
# 方式一：命令行参数
datecalendar-cli task create \
  --title "完成报告" \
  --priority 2 \
  --parent-id "xxx"

# 输出
{"id":"abc-123","title":"完成报告","status":"pending",...}

# 方式二：stdin JSON（便于管道）
echo '{"title":"完成报告","priority":2}' | datecalendar-cli task create --stdin

# 方式三：简洁参数（workbuddy 友好）
datecalendar-cli task create "完成报告" --priority 2
```

**列出任务示例**：

```bash
# JSON 输出（默认，脚本友好）
datecalendar-cli task list
[{"id":"abc-123","title":"完成报告",...},...]

# Table 输出（人类友好）
datecalendar-cli task list --format table
ID        TITLE         STATUS    PRIORITY
abc-123   完成报告      pending   2
```

**workbuddy 典型调用**：

```bash
# workbuddy 插入一条日程
datecalendar-cli schedule create \
  --task-id "task-xxx" \
  --title "团队周会" \
  --start "2026-06-20T10:00:00" \
  --end "2026-06-20T11:00:00" \
  --type fixed

# 输出：JSON 格式日程对象
```

### 2.5 错误处理与退出码

| 退出码 | 含义 | 示例场景 |
|--------|------|----------|
| `0` | 成功 | 正常完成操作 |
| `1` | 业务错误 | 任务不存在、日程冲突 |
| `2` | 参数错误 | 缺少必填参数、格式错误 |
| `3` | 数据库错误 | 无法打开数据库、SQL 错误 |
| `4` | 数据库未找到 | `--db-path` 指向不存在的文件 |

错误时 stderr 输出错误信息，stdout 无输出（或输出 `{"error": "..."}` 当 `--format json` 时）。

### 2.6 实现流程

**项目结构**：

```
datecalendar-cli/
├── Cargo.toml                # 独立二进制 crate
├── src/
│   ├── main.rs              # clap 命令定义（derive 模式）
│   ├── db.rs                # 数据库路径发现 + 连接池
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── task_cmd.rs     # task 子命令实现
│   │   └── schedule_cmd.rs # schedule 子命令实现
│   └── output.rs           # 输出格式化（json/table/csv）
└── tests/
    └── cli_integration.rs  # CLI 集成测试
```

**关键代码片段**（main.rs 命令定义）：

```rust
/// DateCalendar CLI - 命令行任务与日程管理工具
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// 数据库文件路径
    #[arg(long)]
    db_path: Option<PathBuf>,

    /// 输出格式
    #[arg(long, default_value = "json", value_parser = ["json", "table", "csv"])]
    format: String,

    /// 详细日志
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 任务管理
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },
    /// 日程管理
    Schedule {
        #[command(subcommand)]
        command: ScheduleCommands,
    },
    /// 检查数据库连接
    Health,
}

#[derive(Subcommand)]
enum TaskCommands {
    /// 列出所有任务
    List,
    /// 创建任务
    Create {
        /// 任务标题
        title: String,
        /// 父任务 ID
        #[arg(long)]
        parent_id: Option<String>,
        /// 优先级 (0-3)
        #[arg(long, default_value = "0")]
        priority: i32,
        /// 从 stdin 读取 JSON
        #[arg(long)]
        stdin: bool,
    },
    /// 获取单个任务
    Get { id: String },
    /// 更新任务
    Update { id: String, ... },
    /// 删除任务
    Delete { id: String },
    /// 搜索任务
    Search { query: String },
    /// 标记任务完成
    Complete { id: String },
}

// ScheduleCommands 类似...
```

### 2.7 与 Tauri 后端的关系

```
┌─────────────────────────────────────────────────────────┐
│              datecalendar-cli (独立二进制)                 │
│                                                         │
│  main.rs (clap)                                        │
│    ↓                                                    │
│  commands/task_cmd.rs ──► TaskService (共享 lib)       │
│  commands/schedule_cmd.rs ──► ScheduleService (共享 lib)│
│    ↓                                                    │
│  db.rs ──► SQLite (datecalendar.db)                    │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│            Tauri 应用 (datecalendar.exe)                 │
│                                                         │
│  lib.rs (Tauri setup)                                  │
│    ├── IPC Commands ──► TaskService (共享 lib)          │
│    ├── HTTP API ──────► TaskService (共享 lib)          │
│    └── 共享同一个 datecalendar.db                       │
└─────────────────────────────────────────────────────────┘
```

**关键**：`TaskService` 和 `ScheduleService` 放在一个独立的 `datecalendar-core` crate 中，CLI 和 Tauri 后端都依赖它。

---

## 3. 验证标准 (Verify)

### 功能验证

| 测试场景 | 预期结果 | 验证方法 |
|----------|----------|----------|
| `datecalendar-cli --version` | 输出版本号 | 手动执行 |
| `datecalendar-cli health` | 输出 `{"status":"ok"}` + 退出码 0 | 手动执行 |
| `task list` 无参数 | 输出 JSON 数组 | `cargo run -- task list \| jq` |
| `task create` 最小参数 | 创建成功 + 输出 JSON | 手动执行 |
| `task create --stdin` | 从 stdin 读取 JSON 创建 | `echo '\{"title":"test"\}' \| cargo run -- task create --stdin` |
| `task get <exist-id>` | 输出任务 JSON | 手动执行 |
| `task get <fake-id>` | 退出码 1 + stderr 错误信息 | 手动执行 |
| `task delete <id>` | 退出码 0 | 手动执行 |
| `schedule day 2026-06-20` | 输出当天日程 JSON 数组 | 手动执行 |
| `--format table` | 输出对齐的 ASCII 表格 | 手动执行 |
| `--db-path <wrong>` | 退出码 4 + 错误信息 | 手动执行 |

### workbuddy 集成验证

| 场景 | workbuddy 调用 | 预期 |
|------|----------------|------|
| 插入单条日程 | `datecalendar-cli schedule create --title "X" --start "..." --end "..."` | 返回 JSON 格式日程 |
| 查询今日待办 | `datecalendar-cli schedule day $(date +%Y-%m-%d)` | 返回今日日程列表 |
| 批量导入任务 | `cat tasks.json \| datecalendar-cli task import` | 所有任务被创建 |

### 技术验证

```bash
# 编译
cd datecalendar-cli && cargo build --release

# 单元测试
cargo test --lib

# 集成测试（需要预先有 datecalendar.db）
cargo run -- task list
cargo run -- task create "测试任务"
cargo run -- task list | jq '.[-1].title'  # 应输出 "测试任务"
cargo run -- task delete <上一步的 ID>

# CLI 输出格式验证
cargo run -- task list --format json | jq .           # 合法 JSON
cargo run -- task list --format table                  # 表格格式
cargo run -- task list --format csv                    # CSV 格式

# 退出码验证
cargo run -- task get nonexistent; echo $LASTEXITCODE  # Windows: 应输出 1
```
