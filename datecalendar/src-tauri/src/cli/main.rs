#![allow(unused_mut, unused_variables, dead_code)]

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod task;
mod schedule;
mod output;

#[derive(Parser)]
#[command(version, about = "DateCalendar CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// 数据库路径（可选，自动发现）
    #[arg(long, global = true)]
    db_path: Option<PathBuf>,

    /// 输出格式: json, table, csv
    #[arg(short, long, global = true, default_value = "json")]
    format: String,
}

#[derive(Subcommand)]
enum Commands {
    /// 任务管理
    Task(task::TaskArgs),
    /// 日程管理
    Schedule(schedule::ScheduleArgs),
    /// 检查数据库连接
    Health,
    /// 显示版本信息
    Version,
}

fn main() {
    let cli = Cli::parse();
    let result = execute_command(cli);
    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("错误: {}", e);
            std::process::exit(1);
        }
    }
}

fn execute_command(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Task(args) => task::handle_task(args, &cli.db_path, &cli.format),
        Commands::Schedule(args) => schedule::handle_schedule(args, &cli.db_path, &cli.format),
        Commands::Health => {
            let db_path = find_db_path(&cli.db_path)?;
            let pool = init_pool(&db_path)?;
            let conn = pool.get()?;
            // 使用 PRAGMA 或简单查询验证连接
            conn.query_row("SELECT 1", [], |_| Ok(()))?;
            output::output_result(&serde_json::json!({"status": "ok", "db_path": db_path.display().to_string()}), &cli.format)?;
            Ok(())
        }
        Commands::Version => {
            println!("datecalendar-cli {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}

/// 自动发现数据库路径
fn find_db_path(db_path: &Option<PathBuf>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // 1. 命令行参数
    if let Some(path) = db_path {
        // 如果文件不存在，尝试创建父目录（让 init_pool 去创建文件）
        if !path.exists() {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
        }
        return Ok(path.clone());
    }

    // 2. 环境变量
    if let Ok(env_path) = std::env::var("DATECALENDAR_DB") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Ok(p);
        }
    }

    // 3. 默认位置（按操作系统）
    if let Some(default_path) = get_default_db_path() {
        if default_path.exists() {
            return Ok(default_path);
        }
        // 数据库不存在：自动创建目录，让 init_pool 去创建文件并运行迁移
        if let Some(parent) = default_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        return Ok(default_path);
    }

    // 4. 当前目录
    let current = std::env::current_dir()?.join("datecalendar.db");
    if current.exists() {
        return Ok(current);
    }

    Err("找不到数据库文件。请使用 --db-path 指定，或设置 DATECALENDAR_DB 环境变量".into())
}

fn get_default_db_path() -> Option<PathBuf> {
    if let Some(project_dirs) = directories::ProjectDirs::from("com", "DateCalendar", "DateCalendar") {
        let mut path = project_dirs.data_dir().to_path_buf();
        path.push("datecalendar.db");
        Some(path)
    } else {
        None
    }
}

fn init_pool(db_path: &PathBuf) -> Result<r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>, Box<dyn std::error::Error>> {
    let manager = r2d2_sqlite::SqliteConnectionManager::file(db_path);
    let pool = r2d2::Pool::builder()
        .max_size(2)
        .build(manager)?;
    
    // 运行数据库迁移（自动创建表结构）
    // SQLite 会在首次连接时自动创建数据库文件
    let conn = pool.get()?;
    app_lib::db::migrations::run_migrations(&conn)?;
    
    Ok(pool)
}
