import { useEffect, useState } from 'react'
import { Check, Trash2, ChevronRight } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useScheduleStore } from '@/stores/scheduleStore'
import { useUIStore } from '@/stores/uiStore'
import type { Schedule } from '@/types/schedule'

type ViewMode = 'today' | 'week'

export function TodoListView() {
  const { daySchedules, weekSchedules, currentDate, currentWeekStart, currentWeekEnd,
    loadDaySchedules, loadWeekSchedules, toggleScheduleStatus, deleteSchedule } = useScheduleStore()
  const { setCalendarView } = useUIStore()
  const [mode, setMode] = useState<ViewMode>('today')
  const [hoveredId, setHoveredId] = useState<string | null>(null)

  useEffect(() => {
    if (mode === 'today') {
      loadDaySchedules(currentDate)
    } else {
      loadWeekSchedules(currentWeekStart, currentWeekEnd)
    }
  }, [mode, currentDate, currentWeekStart, currentWeekEnd])

  const schedules = mode === 'today' ? daySchedules : weekSchedules
  const todoSchedules = schedules.filter(
    (s) => s.schedule_type === 'todo_day' || s.schedule_type === 'todo_week'
  )

  const completedCount = todoSchedules.filter((s) => s.status === 'completed').length
  const totalCount = todoSchedules.length
  const progressPercent = totalCount > 0 ? Math.round((completedCount / totalCount) * 100) : 0

  const handleToggle = async (s: Schedule) => {
    await toggleScheduleStatus(s.id, s.status)
  }

  const handleDelete = async (s: Schedule) => {
    if (confirm(`删除待办「${s.title}」？`)) {
      await deleteSchedule(s.id)
    }
  }

  return (
    <div className="flex flex-col h-full">
      {/* 头部 */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-border shrink-0">
        <h2 className="text-sm font-semibold">待办列表</h2>
        <div className="flex items-center gap-1">
          <div className="flex bg-muted rounded-md p-0.5">
            <button
              className={`px-2 py-0.5 text-xs rounded transition-colors ${
                mode === 'today' ? 'bg-background shadow-sm' : 'hover:bg-background/50'
              }`}
              onClick={() => setMode('today')}
            >
              今日
            </button>
            <button
              className={`px-2 py-0.5 text-xs rounded transition-colors ${
                mode === 'week' ? 'bg-background shadow-sm' : 'hover:bg-background/50'
              }`}
              onClick={() => setMode('week')}
            >
              本周
            </button>
          </div>
          <Button variant="ghost" size="sm" className="h-7 text-xs" onClick={() => setCalendarView('day')}>
            日视图
          </Button>
          <Button variant="ghost" size="sm" className="h-7 text-xs" onClick={() => setCalendarView('week')}>
            周视图
          </Button>
        </div>
      </div>

      {/* 进度条 */}
      {totalCount > 0 && (
        <div className="px-4 py-3 border-b border-border shrink-0">
          <div className="flex items-center justify-between mb-1.5">
            <span className="text-xs text-muted-foreground">
              {mode === 'today' ? '今日' : '本周'}进度
            </span>
            <span className="text-xs font-medium">
              {completedCount}/{totalCount} · {progressPercent}%
            </span>
          </div>
          <div className="h-1.5 bg-muted rounded-full overflow-hidden">
            <div
              className="h-full bg-primary rounded-full transition-all duration-500"
              style={{ width: `${progressPercent}%` }}
            />
          </div>
        </div>
      )}

      {/* 待办列表 */}
      <ScrollArea className="flex-1">
        {todoSchedules.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-sm text-muted-foreground gap-2">
            <p>{mode === 'today' ? '今天没有待办事项' : '本周没有待办事项'}</p>
            <Button variant="outline" size="sm" onClick={() => setCalendarView('day')}>
              创建日程
            </Button>
          </div>
        ) : (
          <div className="py-1">
            {todoSchedules.map((s) => (
              <div
                key={s.id}
                className={`flex items-center gap-3 px-4 py-2 hover:bg-accent/50 transition-colors group ${
                  s.status === 'completed' ? 'opacity-60' : ''
                }`}
                onMouseEnter={() => setHoveredId(s.id)}
                onMouseLeave={() => setHoveredId(null)}
              >
                {/* 打钩 */}
                <button
                  className={`size-5 rounded border-2 flex items-center justify-center shrink-0 transition-all ${
                    s.status === 'completed'
                      ? 'bg-primary border-primary scale-100'
                      : 'border-muted-foreground/40 hover:border-primary/50'
                  }`}
                  onClick={() => handleToggle(s)}
                >
                  {s.status === 'completed' && <Check className="size-3 text-primary-foreground" />}
                </button>

                {/* 颜色标记 */}
                <span className="size-2.5 rounded-full shrink-0" style={{ backgroundColor: s.color }} />

                {/* 标题 */}
                <span
                  className={`flex-1 text-sm ${
                    s.status === 'completed' ? 'line-through text-muted-foreground' : ''
                  }`}
                >
                  {s.title}
                </span>

                {/* 类型标签 */}
                <span className="text-xs text-muted-foreground shrink-0">
                  {s.schedule_type === 'todo_day' ? '今日' : '本周'}
                </span>

                {/* 删除按钮 */}
                {hoveredId === s.id && (
                  <button
                    className="p-1 hover:bg-red-500/20 rounded shrink-0 opacity-0 group-hover:opacity-100 transition-opacity"
                    onClick={() => handleDelete(s)}
                  >
                    <Trash2 className="size-3.5 text-red-400" />
                  </button>
                )}

                {/* 查看关联任务 */}
                <ChevronRight className="size-4 text-muted-foreground/30 shrink-0" />
              </div>
            ))}
          </div>
        )}
      </ScrollArea>
    </div>
  )
}
