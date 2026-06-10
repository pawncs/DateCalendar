use serde::{Deserialize, Serialize};

/// 任务数据模型 — 树形结构的核心实体
/// 通过 parent_id 自引用实现无限层级嵌套
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub description: String,
    pub status: String,        // pending | in_progress | completed | cancelled
    pub priority: i32,         // 0-3
    pub sort_order: i32,
    pub color: String,
    pub is_milestone: bool,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

/// 用于创建新任务的结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTask {
    pub parent_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub color: Option<String>,
    pub is_milestone: Option<bool>,
}

/// 里程碑风险
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneRisk {
    pub id: String,
    pub task_id: String,
    pub risk_desc: String,
    pub probability: String,   // low | medium | high
    pub mitigation: String,
    pub created_at: String,
    pub updated_at: String,
}

/// 笔记
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub task_id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

/// 日程安排
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: String,
    pub task_id: String,
    pub title: String,
    pub start_time: String,
    pub end_time: String,
    pub is_all_day: bool,
    pub schedule_type: String,  // fixed | todo_day | todo_week
    pub status: String,         // pending | completed | cancelled
    pub color: String,
    pub created_at: String,
    pub updated_at: String,
}
