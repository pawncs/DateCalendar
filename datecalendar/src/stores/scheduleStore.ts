import { create } from 'zustand'
import { adapter } from '@/adapters'
import type { Schedule, CreateScheduleInput, ScheduleType } from '@/types/schedule'

interface ScheduleState {
  // 数据
  schedules: Schedule[]
  daySchedules: Schedule[]
  weekSchedules: Schedule[]

  // 当前日期上下文
  currentDate: string // ISO date string "2026-06-10"
  currentWeekStart: string
  currentWeekEnd: string

  // 加载状态
  loading: boolean
  error: string | null

  // 操作
  setCurrentDate: (date: string) => void
  setCurrentWeek: (weekStart: string, weekEnd: string) => void

  loadDaySchedules: (date: string) => Promise<void>
  loadWeekSchedules: (weekStart: string, weekEnd: string) => Promise<void>
  loadSchedulesByTask: (taskId: string) => Promise<Schedule[]>

  createSchedule: (input: CreateScheduleInput) => Promise<Schedule>
  updateSchedule: (id: string, updates: {
    title?: string
    start_time?: string
    end_time?: string
    is_all_day?: boolean
    schedule_type?: ScheduleType
    status?: string
    color?: string
    task_id?: string
  }) => Promise<Schedule>
  deleteSchedule: (id: string) => Promise<void>

  // 状态同步
  toggleScheduleStatus: (scheduleId: string, currentStatus: string) => Promise<void>

  // 冲突检测
  checkConflicts: (startTime: string, endTime: string, excludeId?: string) => Promise<Schedule[]>

  // 工具函数
  getConflictsForDay: () => Map<string, string[]> // 返回每段时间的冲突 ID 列表
}

export const useScheduleStore = create<ScheduleState>((set, get) => ({
  schedules: [],
  daySchedules: [],
  weekSchedules: [],
  currentDate: new Date().toISOString().split('T')[0],
  currentWeekStart: getWeekStart(new Date()),
  currentWeekEnd: getWeekEnd(new Date()),
  loading: false,
  error: null,

  setCurrentDate: (date) => set({ currentDate: date }),
  setCurrentWeek: (weekStart, weekEnd) => set({ currentWeekStart: weekStart, currentWeekEnd: weekEnd }),

  loadDaySchedules: async (date) => {
    set({ loading: true, error: null, currentDate: date })
    try {
      const schedules = await adapter.get_day_schedules(date)
      set({ daySchedules: schedules, loading: false })
    } catch (e) {
      set({ error: String(e), loading: false })
    }
  },

  loadWeekSchedules: async (weekStart, weekEnd) => {
    set({ loading: true, error: null, currentWeekStart: weekStart, currentWeekEnd: weekEnd })
    try {
      const schedules = await adapter.get_week_schedules(weekStart, weekEnd)
      set({ weekSchedules: schedules, loading: false })
    } catch (e) {
      set({ error: String(e), loading: false })
    }
  },

  loadSchedulesByTask: async (taskId) => {
    return adapter.get_schedules_by_task(taskId)
  },

  createSchedule: async (input) => {
    const schedule = await adapter.create_schedule(
      input.task_id, input.title, input.start_time, input.end_time,
      input.is_all_day ?? false, input.schedule_type ?? 'fixed', input.color ?? '',
    )
    const { currentDate, currentWeekStart, currentWeekEnd } = get()
    await get().loadDaySchedules(currentDate)
    await get().loadWeekSchedules(currentWeekStart, currentWeekEnd)
    return schedule
  },

  updateSchedule: async (id, updates) => {
    const schedule = await adapter.update_schedule(
      id, updates.title, updates.start_time, updates.end_time,
      updates.is_all_day, updates.schedule_type, updates.status,
      updates.color, updates.task_id,
    )
    const { currentDate, currentWeekStart, currentWeekEnd } = get()
    await get().loadDaySchedules(currentDate)
    await get().loadWeekSchedules(currentWeekStart, currentWeekEnd)
    return schedule
  },

  deleteSchedule: async (id) => {
    await adapter.delete_schedule(id)
    const { currentDate, currentWeekStart, currentWeekEnd } = get()
    await get().loadDaySchedules(currentDate)
    await get().loadWeekSchedules(currentWeekStart, currentWeekEnd)
  },

  toggleScheduleStatus: async (scheduleId, currentStatus) => {
    const newStatus = currentStatus === 'completed' ? 'pending' : 'completed'
    await adapter.update_schedule_status(scheduleId, newStatus)
    const { currentDate, currentWeekStart, currentWeekEnd } = get()
    await get().loadDaySchedules(currentDate)
    await get().loadWeekSchedules(currentWeekStart, currentWeekEnd)
  },

  checkConflicts: async (startTime, endTime, excludeId) => {
    return adapter.check_conflicts(startTime, endTime, excludeId ?? null)
  },

  getConflictsForDay: () => {
    const { daySchedules } = get()
    const fixed = daySchedules.filter((s) => s.schedule_type === 'fixed')
    const conflictMap = new Map<string, string[]>()

    for (const s1 of fixed) {
      const conflicts: string[] = []
      for (const s2 of fixed) {
        if (s1.id === s2.id) continue
        if (s1.start_time < s2.end_time && s2.start_time < s1.end_time) {
          conflicts.push(s2.id)
        }
      }
      if (conflicts.length > 0) {
        conflictMap.set(s1.id, conflicts)
      }
    }

    return conflictMap
  },
}))

// 工具函数
function getWeekStart(date: Date): string {
  const d = new Date(date)
  const day = d.getDay()
  const diff = d.getDate() - day + (day === 0 ? -6 : 1)
  d.setDate(diff)
  return d.toISOString().split('T')[0]
}

function getWeekEnd(date: Date): string {
  const d = new Date(getWeekStart(date))
  d.setDate(d.getDate() + 6)
  return d.toISOString().split('T')[0]
}

export { getWeekStart, getWeekEnd }
