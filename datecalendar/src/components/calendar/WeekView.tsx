import { useEffect, useState } from 'react'
import { ChevronLeft, ChevronRight, Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useScheduleStore, getWeekStart, getWeekEnd } from '@/stores/scheduleStore'
import { useUIStore } from '@/stores/uiStore'
import { ScheduleEditor } from './ScheduleEditor'
import type { Schedule } from '@/types/schedule'

const HOUR_HEIGHT = 48

function addDays(dateStr: string, days: number): string {
  const d = new Date(dateStr + 'T00:00:00')
  d.setDate(d.getDate() + days)
  return d.toISOString().split('T')[0]
}

function addWeeks(dateStr: string, weeks: number): string {
  const d = new Date(dateStr + 'T00:00:00')
  d.setDate(d.getDate() + weeks * 7)
  return d.toISOString().split('T')[0]
}

function isToday(dateStr: string): boolean {
  return dateStr === new Date().toISOString().split('T')[0]
}

function getMinutesFromMidnight(timeStr: string): number {
  const parts = timeStr.split('T')
  if (parts.length < 2) return 0
  const [h, m] = parts[1].split(':').map(Number)
  return h * 60 + (m || 0)
}

function isScheduleOnDay(s: Schedule, dayStr: string): boolean {
  const dayStart = dayStr + 'T00:00:00'
  const dayEnd = dayStr + 'T23:59:59'
  return s.start_time <= dayEnd && s.end_time >= dayStart
}

export function WeekView() {
  const { weekSchedules, currentWeekStart, currentWeekEnd, loadWeekSchedules, loading } = useScheduleStore()
  const { setCalendarView } = useUIStore()
  const [showEditor, setShowEditor] = useState(false)
  const [editorPresetDate, setEditorPresetDate] = useState<string | undefined>()
  const [editingSchedule, setEditingSchedule] = useState<Schedule | null>(null)

  const today = new Date().toISOString().split('T')[0]

  useEffect(() => {
    loadWeekSchedules(currentWeekStart, currentWeekEnd)
  }, [currentWeekStart, currentWeekEnd])

  const goPrevWeek = () => {
    const newStart = addWeeks(currentWeekStart, -1)
    const newEnd = addWeeks(currentWeekEnd, -1)
    loadWeekSchedules(newStart, newEnd)
  }

  const goNextWeek = () => {
    const newStart = addWeeks(currentWeekStart, 1)
    const newEnd = addWeeks(currentWeekEnd, 1)
    loadWeekSchedules(newStart, newEnd)
  }

  const goThisWeek = () => {
    const now = new Date()
    const start = getWeekStart(now)
    const end = getWeekEnd(now)
    loadWeekSchedules(start, end)
  }

  // 生成 7 天
  const days: string[] = []
  for (let i = 0; i < 7; i++) {
    days.push(addDays(currentWeekStart, i))
  }

  const weekDayLabels = ['一', '二', '三', '四', '五', '六', '日']

  // 收集全天日程
  const allDaySchedulesByDay = new Map<string, Schedule[]>()
  days.forEach((d) => allDaySchedulesByDay.set(d, []))

  weekSchedules
    .filter((s) => s.is_all_day)
    .forEach((s) => {
      days.forEach((d) => {
        if (isScheduleOnDay(s, d)) {
          allDaySchedulesByDay.get(d)?.push(s)
        }
      })
    })

  // 每天的非全天 fixed 日程
  const fixedByDay = new Map<string, Schedule[]>()
  const todoByDay = new Map<string, number>() // 待办数量

  days.forEach((d) => {
    fixedByDay.set(d, [])
    todoByDay.set(d, 0)
  })

  weekSchedules.forEach((s) => {
    days.forEach((d) => {
      if (s.schedule_type === 'fixed' && !s.is_all_day && isScheduleOnDay(s, d)) {
        fixedByDay.get(d)?.push(s)
      }
      if ((s.schedule_type === 'todo_day' || s.schedule_type === 'todo_week') && isScheduleOnDay(s, d)) {
        todoByDay.set(d, (todoByDay.get(d) ?? 0) + 1)
      }
    })
  })

  const handleDayClick = (day: string) => {
    setCalendarView('day')
    useScheduleStore.getState().setCurrentDate(day)
  }

  const handleScheduleClick = (schedule: Schedule) => {
    setEditingSchedule(schedule)
    setShowEditor(true)
  }

  return (
    <div className="flex flex-col h-full">
      {/* 顶部导航 */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-border shrink-0">
        <div className="flex items-center gap-2">
          <Button variant="ghost" size="icon" className="size-7" onClick={goPrevWeek}>
            <ChevronLeft className="size-4" />
          </Button>
          <button
            className="text-sm font-medium px-2 py-0.5 rounded hover:bg-accent"
            onClick={goThisWeek}
          >
            {currentWeekStart} ~ {currentWeekEnd}
          </button>
          <Button variant="ghost" size="icon" className="size-7" onClick={goNextWeek}>
            <ChevronRight className="size-4" />
          </Button>
        </div>
        <div className="flex items-center gap-1">
          <Button variant="ghost" size="sm" className="h-7 text-xs" onClick={() => setCalendarView('day')}>
            日视图
          </Button>
          <Button variant="ghost" size="sm" className="h-7 text-xs" onClick={() => setCalendarView('todo_list')}>
            待办
          </Button>
          <Button
            size="sm"
            className="h-7 text-xs"
            onClick={() => { setEditingSchedule(null); setEditorPresetDate(undefined); setShowEditor(true) }}
          >
            <Plus className="size-3" />
            添加
          </Button>
        </div>
      </div>

      {/* 星期头 */}
      <div className="flex border-b border-border shrink-0">
        <div className="w-12 shrink-0" />
        {days.map((d, i) => (
          <button
            key={d}
            className={`flex-1 text-center py-2 cursor-pointer hover:bg-accent/50 transition-colors ${
              isToday(d) ? 'bg-primary/10' : ''
            }`}
            onClick={() => handleDayClick(d)}
          >
            <div className="text-xs text-muted-foreground">{weekDayLabels[i]}</div>
            <div className={`text-sm font-medium ${isToday(d) ? 'text-primary' : ''}`}>
              {parseInt(d.split('-')[2])}
            </div>
          </button>
        ))}
      </div>

      {/* 全天日程行 */}
      {Array.from(allDaySchedulesByDay.values()).some((arr) => arr.length > 0) && (
        <div className="flex border-b border-border shrink-0">
          <div className="w-12 shrink-0 text-xs text-muted-foreground text-right pr-2 py-1">
            全天
          </div>
          {days.map((d) => (
            <div key={d} className="flex-1 px-1 py-0.5 min-h-0">
              {(allDaySchedulesByDay.get(d) ?? []).map((s) => (
                <div
                  key={s.id}
                  className="text-xs px-1 py-0.5 mb-0.5 rounded cursor-pointer truncate"
                  style={{ backgroundColor: s.color + '30', color: s.color }}
                  onClick={() => handleScheduleClick(s)}
                >
                  {s.title}
                </div>
              ))}
            </div>
          ))}
        </div>
      )}

      {/* 时间网格 */}
      <div className="flex-1 overflow-hidden">
        <ScrollArea className="h-full">
          {loading ? (
            <div className="flex items-center justify-center h-32 text-sm text-muted-foreground">
              加载中...
            </div>
          ) : (
            <div className="flex" style={{ minHeight: 24 * HOUR_HEIGHT }}>
              {/* 时间轴 */}
              <div className="w-12 shrink-0 border-r border-border">
                {Array.from({ length: 24 }, (_, i) => (
                  <div key={i} className="relative" style={{ height: HOUR_HEIGHT }}>
                    <span className="absolute -top-2 right-2 text-xs text-muted-foreground">
                      {String(i).padStart(2, '0')}
                    </span>
                  </div>
                ))}
              </div>

              {/* 7 列 */}
              {days.map((d) => (
                <div
                  key={d}
                  className={`flex-1 border-r border-border/50 relative ${
                    isToday(d) ? 'bg-primary/5' : ''
                  }`}
                >
                  {/* 网格线 */}
                  {Array.from({ length: 24 }, (_, i) => (
                    <div
                      key={i}
                      className="border-t border-border/30"
                      style={{ height: HOUR_HEIGHT }}
                    />
                  ))}

                  {/* 日程块 */}
                  {(fixedByDay.get(d) ?? []).map((s) => {
                    const startMinutes = getMinutesFromMidnight(s.start_time)
                    const endMinutes = Math.max(getMinutesFromMidnight(s.end_time), startMinutes + 15)
                    const top = startMinutes
                    const height = endMinutes - startMinutes

                    return (
                      <div
                        key={s.id}
                        className="absolute left-0.5 right-0.5 rounded px-1 py-0.5 text-xs cursor-pointer overflow-hidden hover:opacity-90 z-10"
                        style={{
                          top,
                          height: Math.max(height, 18),
                          backgroundColor: s.color + '20',
                          borderLeft: `2px solid ${s.color}`,
                        }}
                        onClick={() => handleScheduleClick(s)}
                      >
                        <div className="font-medium truncate" style={{ color: s.color }}>
                          {s.title}
                        </div>
                      </div>
                    )
                  })}

                  {/* 待办计数 */}
                  {(todoByDay.get(d) ?? 0) > 0 && (
                    <div className="absolute bottom-0 left-0 right-0 px-1 py-0.5 text-xs text-muted-foreground bg-background/80">
                      {(todoByDay.get(d) ?? 0)} 项待办
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </ScrollArea>
      </div>

      {/* 编辑器弹窗 */}
      {showEditor && (
        <ScheduleEditor
          schedule={editingSchedule}
          preselectedDate={editorPresetDate}
          onClose={() => { setShowEditor(false); setEditingSchedule(null) }}
        />
      )}
    </div>
  )
}
