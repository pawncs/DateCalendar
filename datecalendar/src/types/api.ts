// API 统一响应格式
export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

// HTTP API 端点的请求/响应类型
export interface TaskListResponse {
  tasks: import('./task').Task[]
}

export interface ScheduleQueryParams {
  from: string
  to: string
}
