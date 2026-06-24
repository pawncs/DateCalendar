use clap::{Parser, Subcommand};
use std::path::PathBuf;
use chrono::NaiveDate;
use app_lib::services::schedule_service::ScheduleService;
use crate::output::output_result;

#[derive(Parser)]
pub struct ScheduleArgs {
    #[command(subcommand)]
    pub command: ScheduleCommands,
}

#[derive(Subcommand)]
pub enum ScheduleCommands {
    /// 列出所有日程
    List,
    /// 查看某天日程
    Day { date: String },
    /// 查看某周日程
    Week { start: String },
    /// 创建日程
    Create {
        /// 关联的任务 ID
        task_id: String,
        /// 日程标题
        title: String,
        /// 开始时间 (ISO 8601: 2026-06-20T10:00:00)
        start_time: String,
        /// 结束时间
        end_time: String,
        /// 是否为全天事件
        #[arg(long)]
        all_day: bool,
        /// 日程类型 (fixed|todo_day|todo_week)
        #[arg(long, default_value = "fixed")]
        schedule_type: String,
        /// 颜色
        #[arg(short, long)]
        color: Option<String>,
    },
    /// 更新日程
    Update {
        id: String,
        /// 新标题
        #[arg(short, long)]
        title: Option<String>,
        /// 新开始时间
        #[arg(long)]
        start_time: Option<String>,
        /// 新结束时间
        #[arg(long)]
        end_time: Option<String>,
        /// 是否为全天事件
        #[arg(long)]
        all_day: Option<bool>,
        /// 日程类型
        #[arg(long)]
        schedule_type: Option<String>,
        /// 新状态 (pending|completed|cancelled)
        #[arg(short, long)]
        status: Option<String>,
        /// 新颜色
        #[arg(short, long)]
        color: Option<String>,
        /// 关联的新任务 ID
        #[arg(long)]
        task_id: Option<String>,
    },
    /// 删除日程
    Delete { id: String },
    /// 检测时间冲突
    Conflicts {
        /// 开始时间
        start_time: String,
        /// 结束时间
        end_time: String,
    },
}

pub fn handle_schedule(args: ScheduleArgs, db_path: &Option<PathBuf>, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = crate::find_db_path(&db_path)?;
    let pool = crate::init_pool(&db_path)?;
    let service = ScheduleService::new(pool);

    match args.command {
        ScheduleCommands::List => {
            let schedules = service.get_all_schedules()?;
            output_result(&schedules, format)?;
        }
        ScheduleCommands::Day { date } => {
            let schedules = service.get_day_schedules(&date)?;
            output_result(&schedules, format)?;
        }
        ScheduleCommands::Week { start } => {
            let start_date = NaiveDate::parse_from_str(&start, "%Y-%m-%d")
                .map_err(|e| format!("日期格式错误 (应为 YYYY-MM-DD): {}", e))?;
            let end_date = start_date + chrono::Duration::days(6);
            let schedules = service.get_schedules_in_range(&start, &end_date.format("%Y-%m-%d").to_string())?;
            output_result(&schedules, format)?;
        }
        ScheduleCommands::Create { task_id, title, start_time, end_time, all_day, schedule_type, color } => {
            let color = color.unwrap_or_default();
            let schedule = service.create_schedule(&task_id, &title, &start_time, &end_time, all_day, &schedule_type, &color)?;
            output_result(&schedule, format)?;
        }
        ScheduleCommands::Update { id, title, start_time, end_time, all_day, schedule_type, status, color, task_id } => {
            let schedule = service.update_schedule(
                &id,
                title.as_deref(),
                start_time.as_deref(),
                end_time.as_deref(),
                all_day,
                schedule_type.as_deref(),
                status.as_deref(),
                color.as_deref(),
                task_id.as_deref(),
            )?;
            output_result(&schedule, format)?;
        }
        ScheduleCommands::Delete { id } => {
            service.delete_schedule(&id)?;
            println!("日程已删除: {}", id);
        }
        ScheduleCommands::Conflicts { start_time, end_time } => {
            let conflicts = service.check_conflicts(&start_time, &end_time, None)?;
            if conflicts.is_empty() {
                println!("无冲突");
            } else {
                output_result(&conflicts, format)?;
            }
        }
    }

    Ok(())
}
