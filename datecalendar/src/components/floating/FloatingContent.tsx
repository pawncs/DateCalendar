// 悬浮窗内容区域组件
//
// 职责：
// 1. 显示今日待办（todo_day 类型日程）
// 2. 显示接下来日程（fixed 类型，start_time > now）
// 3. 显示本周概览（todo_week 完成进度）
// 4. 支持打钩完成待办
// 5. 支持点击跳转到主窗口
//
// 设计文档：D-16 悬浮窗内容视图

import { useEffect, useState, useCallback } from 'react'
import { useScheduleStore } from '@/stores/scheduleStore'
import { useTaskStore } from '@/stores/taskStore'
import { getTodayStr, getWeekRange } from '@/lib/date'

/**
 * 悬浮窗内容区
 *
 * 加载今日日程 + 本周日程，展示紧凑视图。
 */
export default function FloatingContent() {
  const [today] = useState(getTodayStr())
  const { weekStart, weekEnd } = getWeekRange(today)

  const {
    daySchedules,
    weekSchedules,
    loadDaySchedules,
    loadWeekSchedules,
    toggleScheduleStatus,
  } = useScheduleStore()

  const { tasks, loadTasks } = useTaskStore()

  // 初始化加载
  useEffect(() => {
    loadDaySchedules(today)
    loadWeekSchedules(weekStart, weekEnd)
    loadTasks()
  }, [today, weekStart, weekEnd])

  // 「接下来」：fixed 类型，且开始时间 > 现在
  const now = new Date()
  const upcoming = (daySchedules || [])
    .filter(s => s.schedule_type === 'fixed' && new Date(s.start_time) > now)
    .sort((a, b) => new Date(a.start_time).getTime() - new Date(b.start_time).getTime())
    .slice(0, 3)

  // 「今日待办」：todo_day 类型
  const todayTodos = (daySchedules || [])
    .filter(s => s.schedule_type === 'todo_day')

  // 「本周概览」：todo_week 类型
  const weekTodos = (weekSchedules || [])
    .filter(s => s.schedule_type === 'todo_week')
  const weekCompleted = weekTodos.filter(s => s.status === 'completed').length
  const weekTotal = weekTodos.length
  const weekProgress = weekTotal > 0 ? Math.round((weekCompleted / weekTotal) * 100) : 0

  return (
    <div className="floating-content flex-1 overflow-y-auto p-2 space-y-3">
      {/* 日期头部 */}
      <div className="text-center text-xs text-muted-foreground select-none">
        {new Date().toLocaleDateString('zh-CN', { month: 'long', day: 'numeric', weekday: 'short' })}
      </div>

      {/* 「接下来」区域 */}
      <section>
        <div className="text-xs font-medium text-muted-foreground mb-1 select-none">
          ⏰ 接下来
        </div>
        {upcoming.length === 0 ? (
          <div className="text-xs text-muted-foreground/60 italic p-1">
            暂无即将到来的日程
          </div>
        ) : (
          <div className="space-y-1">
            {upcoming.map(s => (
              <div
                key={s.id}
                className="text-xs p-1.5 rounded bg-white/5 border-l-2 border-blue-400 cursor-pointer hover:bg-white/10"
              >
                <div className="font-medium truncate">{s.title}</div>
                <div className="text-muted-foreground/70">
                  {s.start_time?.slice(11, 16)} - {s.end_time?.slice(11, 16)}
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* 「今日待办」区域 */}
      <section>
        <div className="text-xs font-medium text-muted-foreground mb-1 select-none flex items-center justify-between">
          <span>📋 今日待办 ({todayTodos.filter(t => t.status === 'completed').length}/{todayTodos.length})</span>
        </div>
        {todayTodos.length === 0 ? (
          <div className="text-xs text-muted-foreground/60 italic p-1">
            今天没有待办
          </div>
        ) : (
          <div className="space-y-0.5">
            {todayTodos.map(s => (
              <TodoItem
                key={s.id}
                title={s.title}
                completed={s.status === 'completed'}
                onToggle={async () => {
                  await toggleScheduleStatus(s.id, s.status)
                  await loadDaySchedules(today)
                }}
              />
            ))}
          </div>
        )}
      </section>

      {/* 「本周概览」区域 */}
      <section>
        <div className="text-xs font-medium text-muted-foreground mb-1 select-none">
          📊 本周概览
        </div>
        <div className="text-xs text-muted-foreground/70 mb-1">
          待办: {weekTotal} 项 已完成: {weekCompleted} 项
        </div>
        <div className="w-full h-1.5 bg-white/10 rounded-full overflow-hidden">
          <div
            className={`h-full rounded-full transition-all duration-500 ${
              weekProgress > 50 ? 'bg-green-400' : weekProgress > 30 ? 'bg-orange-400' : 'bg-gray-400'
            }`}
            style={{ width: `${weekProgress}%` }}
          />
        </div>
      </section>
    </div>
  )
}

/** 待办项（带打钩框） */
function TodoItem({ title, completed, onToggle }: {
  title: string
  completed: boolean
  onToggle: () => void
}) {
  return (
    <div
      className={`flex items-center gap-1.5 text-xs p-0.5 rounded cursor-pointer hover:bg-white/5 ${
        completed ? 'line-through text-muted-foreground/50' : ''
      }`}
      onClick={onToggle}
    >
      <span className={`w-3.5 h-3.5 rounded border flex items-center justify-center flex-shrink-0 ${
        completed ? 'bg-green-500/80 border-green-400' : 'border-white/30'
      }`}>
        {completed && <span className="text-[8px] text-white">✓</span>}
      </span>
      <span className="truncate">{title}</span>
    </div>
  )
}
