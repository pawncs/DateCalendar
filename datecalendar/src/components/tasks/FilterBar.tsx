import { X, Filter } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { useTaskStore } from '@/stores/taskStore'
import type { TaskStatus, Priority } from '@/types/task'

const statusOptions: { value: TaskStatus; label: string }[] = [
  { value: 'pending', label: '待办' },
  { value: 'in_progress', label: '进行中' },
  { value: 'completed', label: '已完成' },
  { value: 'cancelled', label: '已取消' },
]

const statusColorMap: Record<TaskStatus, string> = {
  pending: 'bg-muted text-muted-foreground hover:bg-muted/80',
  in_progress: 'bg-blue-500/10 text-blue-400 hover:bg-blue-500/20',
  completed: 'bg-green-500/10 text-green-400 hover:bg-green-500/20',
  cancelled: 'bg-red-500/10 text-red-400 hover:bg-red-500/20',
}

const priorityOptions: { value: Priority; label: string }[] = [
  { value: 0, label: '全部' },
  { value: 1, label: 'P1+' },
  { value: 2, label: 'P2+' },
  { value: 3, label: 'P3+' },
]

const priorityColorMap: Record<Priority, string> = {
  0: 'bg-muted text-muted-foreground hover:bg-muted/80',
  1: 'bg-blue-500/10 text-blue-400 hover:bg-blue-500/20',
  2: 'bg-yellow-500/10 text-yellow-400 hover:bg-yellow-500/20',
  3: 'bg-red-500/10 text-red-400 hover:bg-red-500/20',
}

export function FilterBar() {
  const { filter, setFilter, clearFilter, tasks } = useTaskStore()
  const hasActiveFilter = filter.statuses.length > 0 || filter.minPriority > 0

  const toggleStatus = (s: TaskStatus) => {
    const current = filter.statuses
    const next = current.includes(s) ? current.filter((v) => v !== s) : [...current, s]
    setFilter({ statuses: next })
  }

  const setMinPriority = (p: Priority) => {
    setFilter({ minPriority: p })
  }

  const matchedCount = (() => {
    let result = tasks
    if (filter.statuses.length > 0) {
      result = result.filter((t) => filter.statuses.includes(t.status))
    }
    if (filter.minPriority > 0) {
      result = result.filter((t) => t.priority >= filter.minPriority)
    }
    return result.length
  })()

  return (
    <div className="px-3 py-1.5 border-b border-border space-y-1.5">
      <div className="flex items-center gap-1 flex-wrap">
        <Filter className="size-3 text-muted-foreground shrink-0" />
        {statusOptions.map((opt) => (
          <Badge
            key={opt.value}
            variant={filter.statuses.includes(opt.value) ? 'default' : 'outline'}
            className={`text-xs cursor-pointer transition-colors ${
              filter.statuses.includes(opt.value) ? statusColorMap[opt.value] : ''
            }`}
            onClick={() => toggleStatus(opt.value)}
          >
            {opt.label}
          </Badge>
        ))}
        <span className="text-muted-foreground/50 mx-1">|</span>
        {priorityOptions.map((opt) => (
          <Badge
            key={opt.value}
            variant={filter.minPriority === opt.value ? 'default' : 'outline'}
            className={`text-xs cursor-pointer transition-colors ${
              filter.minPriority === opt.value ? priorityColorMap[opt.value] : ''
            }`}
            onClick={() => setMinPriority(opt.value)}
          >
            {opt.label}
          </Badge>
        ))}
        {hasActiveFilter && (
          <Button
            variant="ghost"
            size="icon"
            className="size-5 ml-auto"
            onClick={clearFilter}
            title="清除筛选"
          >
            <X className="size-3" />
          </Button>
        )}
      </div>
      {hasActiveFilter && (
        <p className="text-xs text-muted-foreground">
          找到 {matchedCount} 个任务
        </p>
      )}
    </div>
  )
}
