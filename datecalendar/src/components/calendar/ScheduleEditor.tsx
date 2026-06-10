import { useState, useEffect, useCallback } from 'react'
import { X, Clock, CalendarDays, AlignLeft, Palette } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { useScheduleStore } from '@/stores/scheduleStore'
import { useTaskStore } from '@/stores/taskStore'
import type { Schedule, ScheduleType } from '@/types/schedule'

interface ScheduleEditorProps {
  schedule?: Schedule | null        // 编辑模式时传入
  preselectedDate?: string          // 预设日期
  preselectedTime?: string          // 预设时间 (HH:00)
  preselectedTaskId?: string        // 预设关联任务
  onClose: () => void
}

const scheduleTypeOptions: { value: ScheduleType; label: string }[] = [
  { value: 'fixed', label: '固定时间' },
  { value: 'todo_day', label: '今日待办' },
  { value: 'todo_week', label: '本周待办' },
]

const colorOptions = [
  '#3b82f6', '#10b981', '#f59e0b', '#ef4444',
  '#8b5cf6', '#ec4899', '#06b6d4', '#84cc16',
]

export function ScheduleEditor({
  schedule,
  preselectedDate,
  preselectedTime,
  preselectedTaskId,
  onClose,
}: ScheduleEditorProps) {
  const { createSchedule, updateSchedule, checkConflicts } = useScheduleStore()
  const { tasks } = useTaskStore()

  const isEditing = !!schedule

  const [title, setTitle] = useState(schedule?.title ?? '')
  const [scheduleType, setScheduleType] = useState<ScheduleType>(
    (schedule?.schedule_type as ScheduleType) ?? 'fixed'
  )
  const [date, setDate] = useState(
    schedule?.start_time?.split('T')[0] ?? preselectedDate ?? new Date().toISOString().split('T')[0]
  )
  const [startHour, setStartHour] = useState(
    schedule?.start_time?.split('T')[1]?.slice(0, 5) ?? preselectedTime ?? '09:00'
  )
  const [endHour, setEndHour] = useState(
    schedule?.end_time?.split('T')[1]?.slice(0, 5) ?? '10:00'
  )
  const [isAllDay, setIsAllDay] = useState(schedule?.is_all_day ?? false)
  const [color, setColor] = useState(schedule?.color ?? '#3b82f6')
  const [taskId, setTaskId] = useState(schedule?.task_id ?? preselectedTaskId ?? '')
  const [showTaskPicker, setShowTaskPicker] = useState(false)
  const [taskSearch, setTaskSearch] = useState('')
  const [conflicts, setConflicts] = useState<import('@/types/schedule').Schedule[]>([])
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState('')

  // 冲突检测 (debounce 300ms)
  const detectConflicts = useCallback(async () => {
    if (scheduleType !== 'fixed' || !title.trim()) {
      setConflicts([])
      return
    }
    try {
      const startTime = isAllDay
        ? `${date}T00:00:00`
        : `${date}T${startHour}:00`
      const endTime = isAllDay
        ? `${date}T23:59:59`
        : `${date}T${endHour}:00`
      const result = await checkConflicts(startTime, endTime, schedule?.id)
      setConflicts(result)
    } catch {
      setConflicts([])
    }
  }, [scheduleType, date, startHour, endHour, isAllDay, schedule?.id, checkConflicts, title])

  useEffect(() => {
    const timer = setTimeout(detectConflicts, 300)
    return () => clearTimeout(timer)
  }, [detectConflicts])

  const handleSave = async () => {
    if (!title.trim()) {
      setError('请输入日程标题')
      return
    }
    setSaving(true)
    setError('')

    try {
      const startTime = isAllDay
        ? `${date}T00:00:00`
        : scheduleType === 'fixed'
          ? `${date}T${startHour}:00`
          : `${date}T00:00:00`
      const endTime = isAllDay
        ? `${date}T23:59:59`
        : scheduleType === 'fixed'
          ? `${date}T${endHour}:00`
          : `${date}T23:59:59`

      if (isEditing && schedule) {
        await updateSchedule(schedule.id, {
          title: title.trim(),
          start_time: startTime,
          end_time: endTime,
          is_all_day: isAllDay,
          schedule_type: scheduleType,
          color,
          task_id: taskId || undefined,
        })
      } else {
        await createSchedule({
          task_id: taskId || '__unlinked__',
          title: title.trim(),
          start_time: startTime,
          end_time: endTime,
          is_all_day: isAllDay,
          schedule_type: scheduleType,
          color,
        })
      }
      onClose()
    } catch (e) {
      setError(String(e))
    } finally {
      setSaving(false)
    }
  }

  const filteredTasks = tasks.filter(
    (t) =>
      !taskSearch ||
      t.title.toLowerCase().includes(taskSearch.toLowerCase())
  )

  const selectedTask = tasks.find((t) => t.id === taskId)

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      onClick={(e) => { if (e.target === e.currentTarget) onClose() }}>
      <div className="bg-card border border-border rounded-lg shadow-xl w-full max-w-md mx-4"
        onKeyDown={(e) => {
          if (e.key === 'Escape') onClose()
          if (e.key === 'Enter') handleSave()
        }}>
        {/* 头部 */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-border">
          <h3 className="text-sm font-semibold">
            {isEditing ? '编辑日程' : '新建日程'}
          </h3>
          <Button variant="ghost" size="icon" className="size-7" onClick={onClose}>
            <X className="size-4" />
          </Button>
        </div>

        {/* 表单 */}
        <div className="px-4 py-3 space-y-3">
          {/* 标题 */}
          <Input
            placeholder="日程标题"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            className="h-9"
            autoFocus
          />

          {/* 类型选择 */}
          <div className="flex gap-1 flex-wrap">
            {scheduleTypeOptions.map((opt) => (
              <Badge
                key={opt.value}
                variant={scheduleType === opt.value ? 'default' : 'outline'}
                className="text-xs cursor-pointer"
                onClick={() => setScheduleType(opt.value)}
              >
                {opt.label}
              </Badge>
            ))}
          </div>

          {/* 时间设置 */}
          {scheduleType === 'fixed' && (
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <CalendarDays className="size-4 text-muted-foreground shrink-0" />
                <Input
                  type="date"
                  value={date}
                  onChange={(e) => setDate(e.target.value)}
                  className="h-8 text-sm flex-1"
                />
              </div>
              {!isAllDay && (
                <div className="flex items-center gap-2">
                  <Clock className="size-4 text-muted-foreground shrink-0" />
                  <Input
                    type="time"
                    value={startHour}
                    onChange={(e) => setStartHour(e.target.value)}
                    className="h-8 text-sm w-28"
                  />
                  <span className="text-muted-foreground text-sm">-</span>
                  <Input
                    type="time"
                    value={endHour}
                    onChange={(e) => setEndHour(e.target.value)}
                    className="h-8 text-sm w-28"
                  />
                </div>
              )}
              <label className="flex items-center gap-2 text-xs text-muted-foreground cursor-pointer">
                <input
                  type="checkbox"
                  checked={isAllDay}
                  onChange={(e) => setIsAllDay(e.target.checked)}
                  className="rounded"
                />
                全天
              </label>
            </div>
          )}

          {scheduleType === 'todo_day' && (
            <div className="flex items-center gap-2">
              <CalendarDays className="size-4 text-muted-foreground shrink-0" />
              <Input
                type="date"
                value={date}
                onChange={(e) => setDate(e.target.value)}
                className="h-8 text-sm flex-1"
              />
            </div>
          )}

          {/* 关联任务 */}
          <div className="relative">
            <button
              className="flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground"
              onClick={() => setShowTaskPicker(!showTaskPicker)}
            >
              <AlignLeft className="size-3.5" />
              {selectedTask ? `关联: ${selectedTask.title}` : '关联任务（可选）'}
            </button>
            {showTaskPicker && (
              <div className="absolute top-full left-0 mt-1 w-full bg-card border border-border rounded-md shadow-lg z-10">
                <Input
                  placeholder="搜索任务..."
                  value={taskSearch}
                  onChange={(e) => setTaskSearch(e.target.value)}
                  className="h-7 text-xs m-1"
                />
                <div className="max-h-32 overflow-auto">
                  <button
                    className="w-full text-left px-2 py-1 text-xs hover:bg-accent"
                    onClick={() => { setTaskId(''); setShowTaskPicker(false) }}
                  >
                    不关联
                  </button>
                  {filteredTasks.slice(0, 20).map((t) => (
                    <button
                      key={t.id}
                      className="w-full text-left px-2 py-1 text-xs hover:bg-accent"
                      onClick={() => { setTaskId(t.id); setShowTaskPicker(false) }}
                    >
                      {t.title}
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* 颜色选择 */}
          <div className="flex items-center gap-2">
            <Palette className="size-4 text-muted-foreground shrink-0" />
            <div className="flex gap-1">
              {colorOptions.map((c) => (
                <button
                  key={c}
                  className={`size-5 rounded-full border-2 transition-all ${
                    color === c ? 'border-foreground scale-110' : 'border-transparent'
                  }`}
                  style={{ backgroundColor: c }}
                  onClick={() => setColor(c)}
                />
              ))}
            </div>
          </div>

          {/* 冲突警告 */}
          {conflicts.length > 0 && (
            <div className="bg-yellow-500/10 border border-yellow-500/30 rounded-md px-3 py-2 text-xs text-yellow-400">
              该时段已有以下日程：
              {conflicts.map((c) => (
                <div key={c.id} className="mt-1 flex items-center gap-1">
                  <span className="size-2 rounded-full shrink-0" style={{ backgroundColor: c.color || '#f59e0b' }} />
                  {c.title} ({c.start_time.split('T')[1]?.slice(0, 5)}-{c.end_time.split('T')[1]?.slice(0, 5)})
                </div>
              ))}
            </div>
          )}

          {/* 错误提示 */}
          {error && (
            <p className="text-xs text-red-400">{error}</p>
          )}
        </div>

        {/* 底部按钮 */}
        <div className="flex justify-end gap-2 px-4 py-3 border-t border-border">
          <Button variant="outline" size="sm" onClick={onClose}>
            取消
          </Button>
          <Button size="sm" onClick={handleSave} disabled={saving}>
            {saving ? '保存中...' : isEditing ? '更新' : '创建'}
          </Button>
        </div>
      </div>
    </div>
  )
}
