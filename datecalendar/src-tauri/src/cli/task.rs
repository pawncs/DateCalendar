use clap::{Parser, Subcommand};
use std::io::Read;
use std::path::PathBuf;
use app_lib::db::models::NewTask;
use app_lib::services::task_service::TaskService;
use crate::output::output_result;

#[derive(Parser)]
pub struct TaskArgs {
    #[command(subcommand)]
    pub command: TaskCommands,
}

#[derive(Subcommand)]
pub enum TaskCommands {
    /// 列出所有任务
    List,
    /// 获取单个任务
    Get { id: String },
    /// 创建任务
    Create {
        /// 任务标题
        title: String,
        /// 父任务 ID（可选）
        #[arg(short = 'P', long)]
        parent_id: Option<String>,
        /// 任务描述
        #[arg(short, long)]
        description: Option<String>,
        /// 优先级 (0-3)
        #[arg(short, long)]
        priority: Option<i32>,
        /// 颜色
        #[arg(short, long)]
        color: Option<String>,
        /// 是否为里程碑
        #[arg(long)]
        is_milestone: bool,
        /// 从 stdin 读取 JSON
        #[arg(long)]
        stdin: bool,
    },
    /// 更新任务
    Update {
        id: String,
        /// 新标题
        #[arg(short, long)]
        title: Option<String>,
        /// 新描述
        #[arg(short, long)]
        description: Option<String>,
        /// 新状态 (pending|in_progress|completed|cancelled)
        #[arg(short, long)]
        status: Option<String>,
        /// 新优先级 (0-3)
        #[arg(short, long)]
        priority: Option<i32>,
        /// 新颜色
        #[arg(short, long)]
        color: Option<String>,
        /// 是否为里程碑
        #[arg(long)]
        is_milestone: Option<bool>,
        /// 父任务 ID（传 "null" 表示清除）
        #[arg(long)]
        parent_id: Option<String>,
        /// 排序位置
        #[arg(long)]
        sort_order: Option<i32>,
    },
    /// 删除任务
    Delete { id: String },
    /// 搜索任务
    Search { query: String },
    /// 标记任务完成
    Complete { id: String },
}

pub fn handle_task(args: TaskArgs, db_path: &Option<PathBuf>, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = crate::find_db_path(&db_path)?;
    let pool = crate::init_pool(&db_path)?;
    let service = TaskService::new(pool);

    match args.command {
        TaskCommands::List => {
            let tasks = service.get_all_tasks()?;
            output_result(&tasks, format)?;
        }
        TaskCommands::Get { id } => {
            let task = service.get_task(&id)?;
            match task {
                Some(t) => output_result(&t, format)?,
                None => {
                    eprintln!("任务不存在: {}", id);
                    std::process::exit(1);
                }
            }
        }
        TaskCommands::Create { title, parent_id, description, priority, color, is_milestone, stdin } => {
            let input = if stdin {
                let mut buffer = String::new();
                std::io::stdin().read_to_string(&mut buffer)?;
                serde_json::from_str::<NewTask>(&buffer)?
            } else {
                NewTask {
                    parent_id,
                    title,
                    description,
                    priority,
                    color,
                    is_milestone: if is_milestone { Some(true) } else { None },
                }
            };
            let task = service.create_task(input)?;
            output_result(&task, format)?;
        }
        TaskCommands::Update { id, title, description, status, priority, color, is_milestone, parent_id, sort_order } => {
            let parent_id_opt: Option<Option<&str>> = parent_id.as_deref().map(|s| if s == "null" { None } else { Some(s) });
            let task = service.update_task(
                &id,
                title.as_deref(),
                description.as_deref(),
                status.as_deref(),
                priority,
                color.as_deref(),
                is_milestone,
                parent_id_opt,
                sort_order,
            )?;
            output_result(&task, format)?;
        }
        TaskCommands::Delete { id } => {
            service.delete_task(&id)?;
            println!("任务已删除: {}", id);
        }
        TaskCommands::Search { query } => {
            let tasks = service.search_tasks(&query)?;
            output_result(&tasks, format)?;
        }
        TaskCommands::Complete { id } => {
            let task = service.update_task(
                &id,
                None,
                None,
                Some("completed"),
                None,
                None,
                None,
                None,
                None,
            )?;
            output_result(&task, format)?;
        }
    }

    Ok(())
}
