// 日程类型
export type ScheduleType = 'fixed' | 'todo_day' | 'todo_week'
export type ScheduleStatus = 'pending' | 'completed' | 'cancelled'

// 日程数据模型
export interface Schedule {
  id: string
  task_id: string
  title: string
  start_time: string
  end_time: string
  is_all_day: boolean
  schedule_type: ScheduleType
  status: ScheduleStatus
  color: string
  created_at: string
  updated_at: string
}

// 创建日程的输入
export interface CreateScheduleInput {
  task_id: string
  title: string
  start_time: string
  end_time: string
  is_all_day?: boolean
  schedule_type?: ScheduleType
  color?: string
}

// 视图类型
export type CalendarView = 'day' | 'week' | 'todo_list'
