// 任务状态
export type TaskStatus = 'pending' | 'in_progress' | 'completed' | 'cancelled'

// 优先级 0-3
export type Priority = 0 | 1 | 2 | 3

// 任务数据模型
export interface Task {
  id: string
  parent_id: string | null
  title: string
  description: string
  status: TaskStatus
  priority: Priority
  sort_order: number
  color: string
  is_milestone: boolean
  created_at: string
  updated_at: string
  completed_at: string | null
  // 前端特有：树形展示用
  children?: Task[]
  depth?: number
}

// 创建任务的输入
export interface CreateTaskInput {
  parent_id: string | null
  title: string
  description?: string
  priority?: Priority
  color?: string
  is_milestone?: boolean
}

// 更新任务的输入
export interface UpdateTaskInput {
  title?: string
  description?: string
  status?: TaskStatus
  priority?: Priority
  color?: string
  is_milestone?: boolean
  parent_id?: string | null
  sort_order?: number
}

// 里程碑风险
export type RiskProbability = 'low' | 'medium' | 'high'

export interface MilestoneRisk {
  id: string
  task_id: string
  risk_desc: string
  probability: RiskProbability
  mitigation: string
  created_at: string
  updated_at: string
}

// 笔记
export interface Note {
  id: string
  task_id: string
  title: string
  content: string
  created_at: string
  updated_at: string
}
