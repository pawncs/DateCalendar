import { useState } from 'react'
import {
  ChevronRight, ChevronDown, Star,
  Plus, Trash2, Flag
} from 'lucide-react'
import { cn } from '@/lib/utils'
import { Checkbox } from '@/components/ui/checkbox'
import { useTaskStore, getTaskDepth } from '@/stores/taskStore'
import type { Task, Priority } from '@/types/task'

interface TaskNodeProps {
  task: Task
  depth?: number
}

const priorityColors: Record<Priority, string> = {
  0: '',                    // 无优先级
  1: 'text-blue-400',       // 低
  2: 'text-yellow-400',     // 中
  3: 'text-red-400',        // 高
}

const statusColors: Record<string, string> = {
  pending: 'border-muted-foreground/30',
  in_progress: 'border-blue-400',
  completed: 'border-green-400',
  cancelled: 'border-red-400/50',
}

export function TaskNode({ task, depth = 0 }: TaskNodeProps) {
  const { selectedTaskId, selectTask, toggleExpand, expandedIds,
    updateTask, deleteTask, createTask, tasks } = useTaskStore()
  const [isHovered, setIsHovered] = useState(false)
  const [isEditing, setIsEditing] = useState(false)
  const [editTitle, setEditTitle] = useState(task.title)

  const isExpanded = expandedIds.has(task.id)
  const isSelected = selectedTaskId === task.id
  const hasChildren = task.children && task.children.length > 0
  const actualDepth = depth || getTaskDepth(tasks, task.id)

  const handleToggleStatus = async () => {
    const newStatus = task.status === 'completed' ? 'pending' : 'completed'
    await updateTask(task.id, { status: newStatus })
  }

  const handleTitleSubmit = async () => {
    if (editTitle.trim() && editTitle !== task.title) {
      await updateTask(task.id, { title: editTitle.trim() })
    }
    setIsEditing(false)
  }

  const handleAddSubtask = async () => {
    const newTask = await createTask({
      parent_id: task.id,
      title: '新子任务',
    })
    // 展开父节点以显示新子任务
    toggleExpand(task.id)
    selectTask(newTask.id)
  }

  return (
    <div>
      <div
        className={cn(
          'group flex items-center gap-1 px-2 py-1.5 rounded-md cursor-pointer transition-colors border-l-[3px]',
          isSelected
            ? 'bg-accent text-accent-foreground border-l-primary'
            : 'hover:bg-accent/50 border-l-transparent',
          task.status === 'completed' && 'opacity-60',
          statusColors[task.status] || 'border-l-transparent'
        )}
        style={{ paddingLeft: `${actualDepth * 20 + 8}px` }}
        onClick={() => selectTask(task.id)}
        onMouseEnter={() => setIsHovered(true)}
        onMouseLeave={() => setIsHovered(false)}
      >
        {/* 展开/折叠按钮 */}
        <button
          className="p-0.5 hover:bg-muted rounded-sm shrink-0"
          onClick={(e) => {
            e.stopPropagation()
            toggleExpand(task.id)
          }}
        >
          {hasChildren ? (
            isExpanded
              ? <ChevronDown className="size-3.5 text-muted-foreground" />
              : <ChevronRight className="size-3.5 text-muted-foreground" />
          ) : (
            <span className="w-3.5" />
          )}
        </button>

        {/* 完成复选框 */}
        <Checkbox
          checked={task.status === 'completed'}
          onChange={handleToggleStatus}
          className="shrink-0"
        />

        {/* 里程碑星标 */}
        {task.is_milestone && (
          <Star className="size-3.5 text-yellow-400 fill-yellow-400 shrink-0" />
        )}

        {/* 标题 */}
        {isEditing ? (
          <input
            className="flex-1 bg-transparent border-b border-primary outline-none text-sm px-1"
            value={editTitle}
            onChange={(e) => setEditTitle(e.target.value)}
            onBlur={handleTitleSubmit}
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleTitleSubmit()
              if (e.key === 'Escape') {
                setEditTitle(task.title)
                setIsEditing(false)
              }
            }}
            autoFocus
            onClick={(e) => e.stopPropagation()}
          />
        ) : (
          <span
            className={cn(
              'flex-1 text-sm truncate',
              task.status === 'completed' && 'line-through text-muted-foreground'
            )}
            onDoubleClick={() => setIsEditing(true)}
          >
            {task.title}
          </span>
        )}

        {/* 优先级标记 */}
        {task.priority > 0 && (
          <Flag className={cn('size-3 shrink-0', priorityColors[task.priority as Priority])} />
        )}

        {/* 操作按钮 (hover 显示) */}
        {isHovered && (
          <div className="flex items-center gap-0.5 shrink-0">
            <button
              className="p-1 hover:bg-muted rounded-sm"
              onClick={(e) => {
                e.stopPropagation()
                handleAddSubtask()
              }}
              title="添加子任务"
            >
              <Plus className="size-3.5 text-muted-foreground" />
            </button>
            <button
              className="p-1 hover:bg-red-500/20 rounded-sm"
              onClick={async (e) => {
                e.stopPropagation()
                if (confirm('确定删除此任务及其所有子任务？')) {
                  await deleteTask(task.id)
                }
              }}
              title="删除"
            >
              <Trash2 className="size-3.5 text-red-400" />
            </button>
          </div>
        )}

        {/* 颜色标记 */}
        {task.color && (
          <span
            className="size-2.5 rounded-full shrink-0"
            style={{ backgroundColor: task.color }}
          />
        )}
      </div>

      {/* 递归渲染子任务 */}
      {hasChildren && isExpanded && (
        <div>
          {task.children!.map((child) => (
            <TaskNode
              key={child.id}
              task={child}
              depth={actualDepth + 1}
            />
          ))}
        </div>
      )}
    </div>
  )
}
