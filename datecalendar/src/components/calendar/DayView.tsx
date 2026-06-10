import { useEffect, useState, useRef } from 'react'
import { ChevronLeft, ChevronRight, Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useScheduleStore } from '@/stores/scheduleStore'
import { useUIStore } from '@/stores/uiStore'
import { ScheduleEditor } from './ScheduleEditor'
import type { Schedule } from '@/types/schedule'

const HOUR_HEIGHT = 60 // px per hour
const TOTAL_HOURS = 24

function formatDateCN(dateStr: string): string {
  const d = new Date(dateStr + 'T00:00:00')
  const weekDays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']
  return `${d.getMonth() + 1}月${d.getDate()}日 (${weekDays[d.getDay()]})`
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

function getDurationMinutes(start: string, end: string): number {
  return getMinutesFromMidnight(end) - getMinutesFromMidnight(start)
}

function addDays(dateStr: string, days: number): string {
  const d = new Date(dateStr + 'T00:00:00')
  d.setDate(d.getDate() + days)
  return d.toISOString().split('T')[0]
}

export function DayView() {
  const { daySchedules, currentDate, loadDaySchedules, loading } = useScheduleStore()
  const { setCalendarView } = useUIStore()
  const [showEditor, setShowEditor] = useState(false)
  const [editorPresetTime, setEditorPresetTime] = useState<string | undefined>()
  const [editingSchedule, setEditingSchedule] = useState<Schedule | null>(null)
  const scrollRef = useRef<HTMLDivElement>(null)
  const [currentTimePos, setCurrentTimePos] = useState(0)

  useEffect(() => {
    loadDaySchedules(currentDate)
  }, [currentDate])

  // 当前时间红线
  useEffect(() => {
    const updateTimeLine = () => {
      const now = new Date()
      const minutes = now.getHours() * 60 + now.getMinutes()
      setCurrentTimePos(minutes)
    }
    updateTimeLine()
    const timer = setInterval(updateTimeLine, 60000)
    return () => clearInterval(timer)
  }, [])

  // 打开时滚动到当前时间
  useEffect(() => {
    if (scrollRef.current && isToday(currentDate)) {
      const now = new Date()
      const minutes = now.getHours() * 60 + now.getMinutes()
      const scrollTo = Math.max(0, minutes - 120) // 当前时间上方 2 小时
      setTimeout(() => {
        scrollRef.current?.scrollTo({ top: scrollTo, behavior: 'smooth' })
      }, 100)
    }
  }, [currentDate])

  const goPrevDay = () => loadDaySchedules(addDays(currentDate, -1))
  const goNextDay = () => loadDaySchedules(addDays(currentDate, 1))
  const goToday = () => loadDaySchedules(new Date().toISOString().split('T')[0])

  const fixedSchedules = daySchedules.filter((s) => s.schedule_type === 'fixed')
  const todoSchedules = daySchedules.filter((s) => s.schedule_type === 'todo_day' || s.schedule_type === 'todo_week')
  const allDaySchedules = fixedSchedules.filter((s) => s.is_all_day)

  // 检测冲突
  const conflictMap = new Map<string, string[]>()
  for (const s1 of fixedSchedules) {
    if (s1.is_all_day) continue
    const conflicts: string[] = []
    for (const s2 of fixedSchedules) {
      if (s1.id === s2.id || s2.is_all_day) continue
      if (s1.start_time < s2.end_time && s2.start_time < s1.end_time) {
        conflicts.push(s2.id)
      }
    }
    if (conflicts.length > 0) conflictMap.set(s1.id, conflicts)
  }

  const handleGridClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const rect = e.currentTarget.getBoundingClientRect()
    const y = e.clientY - rect.top + (scrollRef.current?.scrollTop ?? 0)
    const minutes = Math.floor(y / (HOUR_HEIGHT / 60) / 30) * 30 // 对齐到 30 分钟
    const h = Math.floor(minutes / 60).toString().padStart(2, '0')
    const m = (minutes % 60).toString().padStart(2, '0')
    setEditorPresetTime(`${h}:${m}`)
    setEditingSchedule(null)
    setShowEditor(true)
  }

  const handleScheduleClick = (schedule: Schedule) => {
    setEditingSchedule(schedule)
    setShowEditor(true)
  }

  // 对同时段多个日程做并排计算
  const scheduleColumns = computeColumns(fixedSchedules.filter((s) => !s.is_all_day))

  return (
    <div className="flex flex-col h-full">
      {/* 顶部导航 */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-border shrink-0">
        <div className="flex items-center gap-2">
          <Button variant="ghost" size="icon" className="size-7" onClick={goPrevDay}>
            <ChevronLeft className="size-4" />
          </Button>
          <button
            className={`text-sm font-medium px-2 py-0.5 rounded ${
              isToday(currentDate) ? 'bg-primary text-primary-foreground' : 'hover:bg-accent'
            }`}
            onClick={goToday}
          >
            {formatDateCN(currentDate)}
          </button>
          <Button variant="ghost" size="icon" className="size-7" onClick={goNextDay}>
            <ChevronRight className="size-4" />
          </Button>
        </div>
        <div className="flex items-center gap-1">
          <Button variant="ghost" size="sm" className="h-7 text-xs" onClick={() => setCalendarView('week')}>
            周视图
          </Button>
          <Button variant="ghost" size="sm" className="h-7 text-xs" onClick={() => setCalendarView('todo_list')}>
            待办
          </Button>
          <Button
            size="sm"
            className="h-7 text-xs"
            onClick={() => { setEditingSchedule(null); setEditorPresetTime(undefined); setShowEditor(true) }}
          >
            <Plus className="size-3" />
            添加
          </Button>
        </div>
      </div>

      {/* 全天日程 */}
      {allDaySchedules.length > 0 && (
        <div className="px-14 py-1 border-b border-border shrink-0 flex flex-wrap gap-1">
          {allDaySchedules.map((s) => (
            <div
              key={s.id}
              className="text-xs px-2 py-0.5 rounded cursor-pointer hover:opacity-80"
              style={{ backgroundColor: s.color + '30', color: s.color, borderLeft: `3px solid ${s.color}` }}
              onClick={() => handleScheduleClick(s)}
            >
              {s.title}
            </div>
          ))}
        </div>
      )}

      {/* 时间网格 */}
      <div className="flex-1 relative overflow-hidden">
        <ScrollArea className="h-full" ref={scrollRef as any}>
          {loading ? (
            <div className="flex items-center justify-center h-32 text-sm text-muted-foreground">
              加载中...
            </div>
          ) : (
            <div className="flex" style={{ minHeight: TOTAL_HOURS * HOUR_HEIGHT }}>
              {/* 时间轴 */}
              <div className="w-12 shrink-0 border-r border-border">
                {Array.from({ length: TOTAL_HOURS }, (_, i) => (
                  <div key={i} className="relative" style={{ height: HOUR_HEIGHT }}>
                    <span className="absolute -top-2.5 right-2 text-xs text-muted-foreground">
                      {String(i).padStart(2, '0')}:00
                    </span>
                  </div>
                ))}
              </div>

              {/* 日程区域 */}
              <div className="flex-1 relative" onClick={handleGridClick}>
                {/* 网格线 */}
                {Array.from({ length: TOTAL_HOURS }, (_, i) => (
                  <div
                    key={i}
                    className="border-t border-border/50"
                    style={{ height: HOUR_HEIGHT }}
                  />
                ))}

                {/* 当前时间红线 */}
                {isToday(currentDate) && (
                  <div
                    className="absolute left-0 right-0 z-20 pointer-events-none"
                    style={{ top: currentTimePos }}
                  >
                    <div className="flex items-center">
                      <div className="size-2 rounded-full bg-red-500 -ml-1" />
                      <div className="flex-1 border-t border-red-500" />
                    </div>
                  </div>
                )}

                {/* 日程块 */}
                {fixedSchedules
                  .filter((s) => !s.is_all_day)
                  .map((s) => {
                    const top = getMinutesFromMidnight(s.start_time)
                    const duration = getDurationMinutes(s.start_time, s.end_time)
                    const colInfo = scheduleColumns.get(s.id)
                    const colIndex = colInfo?.index ?? 0
                    const colTotal = colInfo?.total ?? 1
                    const width = `${100 / colTotal}%`
                    const left = `${(100 / colTotal) * colIndex}%`
                    const hasConflict = conflictMap.has(s.id)

                    return (
                      <div
                        key={s.id}
                        className={`absolute z-10 mx-0.5 rounded px-1.5 py-0.5 text-xs cursor-pointer overflow-hidden transition-colors hover:opacity-90 ${
                          s.status === 'completed' ? 'opacity-50' : ''
                        }`}
                        style={{
                          top,
                          height: Math.max(duration, 20),
                          width,
                          left,
                          backgroundColor: s.color + '25',
                          borderLeft: `3px solid ${s.color}`,
                          border: hasConflict ? '1px solid #ef4444' : undefined,
                        }}
                        onClick={(e) => { e.stopPropagation(); handleScheduleClick(s) }}
                      >
                        <div className="font-medium truncate" style={{ color: s.color }}>
                          {s.title}
                        </div>
                        <div className="text-muted-foreground truncate">
                          {s.start_time.split('T')[1]?.slice(0, 5)}-{s.end_time.split('T')[1]?.slice(0, 5)}
                        </div>
                      </div>
                    )
                  })}
              </div>
            </div>
          )}
        </ScrollArea>
      </div>

      {/* 待办列表 */}
      {todoSchedules.length > 0 && (
        <div className="border-t border-border shrink-0 max-h-40 overflow-auto">
          <div className="px-4 py-2 text-xs font-medium text-muted-foreground border-b border-border/50">
            待办事项
          </div>
          {todoSchedules.map((s) => (
            <div
              key={s.id}
              className={`flex items-center gap-2 px-4 py-1.5 text-sm cursor-pointer hover:bg-accent/50 ${
                s.status === 'completed' ? 'opacity-50' : ''
              }`}
              onClick={() => handleScheduleClick(s)}
            >
              <input
                type="checkbox"
                checked={s.status === 'completed'}
                onChange={(e) => {
                  e.stopPropagation()
                  useScheduleStore.getState().toggleScheduleStatus(s.id, s.status)
                }}
                className="rounded shrink-0"
              />
              <span className="size-2 rounded-full shrink-0" style={{ backgroundColor: s.color }} />
              <span className={s.status === 'completed' ? 'line-through text-muted-foreground' : ''}>
                {s.title}
              </span>
              <span className="text-xs text-muted-foreground ml-auto">
                {s.schedule_type === 'todo_day' ? '今日' : '本周'}
              </span>
            </div>
          ))}
        </div>
      )}

      {/* 编辑器弹窗 */}
      {showEditor && (
        <ScheduleEditor
          schedule={editingSchedule}
          preselectedDate={currentDate}
          preselectedTime={editorPresetTime}
          onClose={() => { setShowEditor(false); setEditingSchedule(null) }}
        />
      )}
    </div>
  )
}

/**
 * 计算同时段多个日程的并排列
 */
function computeColumns(schedules: Schedule[]): Map<string, { index: number; total: number }> {
  const result = new Map<string, { index: number; total: number }>()
  if (schedules.length === 0) return result

  const sorted = [...schedules].sort((a, b) => {
    const aStart = getMinutesFromMidnight(a.start_time)
    const bStart = getMinutesFromMidnight(b.start_time)
    return aStart - bStart
  })

  // 贪心分组：找出所有重叠的日程组
  const groups: Schedule[][] = []
  const used = new Set<string>()

  for (const s of sorted) {
    if (used.has(s.id)) continue
    const group: Schedule[] = [s]
    used.add(s.id)
    for (const other of sorted) {
      if (used.has(other.id)) continue
      // 检查 other 是否与 group 中所有日程都重叠
      const overlaps = group.every(
        (g) => g.start_time < other.end_time && other.start_time < g.end_time
      )
      if (overlaps) {
        group.push(other)
        used.add(other.id)
      }
    }
    groups.push(group)
  }

  for (const group of groups) {
    group.forEach((s, i) => {
      result.set(s.id, { index: i, total: group.length })
    })
  }

  return result
}
