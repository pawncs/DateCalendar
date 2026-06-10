import { useEffect, useState, useCallback } from 'react'
import { Plus, Search, Expand, FoldVertical, GripVertical } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useTaskStore, buildTaskTree } from '@/stores/taskStore'
import { useUIStore } from '@/stores/uiStore'
import { TaskNode } from './TaskNode'
import { FilterBar } from './FilterBar'
import { BatchActionBar } from './BatchActionBar'
import type { Task } from '@/types/task'
import {
  DndContext,
  PointerSensor,
  KeyboardSensor,
  useSensor,
  useSensors,
  closestCenter,
} from '@dnd-kit/core'
import type { DragStartEvent, DragEndEvent, DragOverEvent } from '@dnd-kit/core/dist/types'
import {
  SortableContext,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable'

/**
 * 根据拖拽事件计算新的 parent_id 和 sort_order
 */
function computeDropPosition(
  tasks: Task[],
  activeId: string,
  overId: string
): { newParentId: string | null; newSortOrder: number } | null {
  if (activeId === overId) return null

  const activeTask = tasks.find((t) => t.id === activeId)
  const overTask = tasks.find((t) => t.id === overId)
  if (!activeTask || !overTask) return null

  // 检查是否拖到自己的子孙上（防循环）
  const isDescendant = (ancestorId: string, childId: string): boolean => {
    const child = tasks.find((t) => t.id === childId)
    if (!child) return false
    if (child.parent_id === ancestorId) return true
    if (child.parent_id) return isDescendant(ancestorId, child.parent_id)
    return false
  }
  if (isDescendant(activeId, overId)) return null

  // 默认：放到 over 节点的同层级后面
  const newParentId = overTask.parent_id
  const siblings = tasks
    .filter((t) => t.parent_id === newParentId)
    .sort((a, b) => a.sort_order - b.sort_order)

  // 找到 over 任务在兄弟中的位置
  const overIndex = siblings.findIndex((t) => t.id === overId)
  const newSortOrder = overIndex + 1

  return { newParentId, newSortOrder }
}

/**
 * 收集任务的所有子任务 ID（递归）
 */
function collectDescendantIds(tasks: Task[], taskId: string): Set<string> {
  const ids = new Set<string>([taskId])
  for (const t of tasks) {
    if (t.parent_id === taskId) {
      for (const id of collectDescendantIds(tasks, t.id)) {
        ids.add(id)
      }
    }
  }
  return ids
}

export function TaskTree() {
  const {
    tasks, loadTasks, loading, error, createTask, expandAll, collapseAll,
    searchTasks, selectTask, reorderTask, selectionMode, selectedIds, getFilteredTasks,
  } = useTaskStore()
  const { searchQuery, setSearchQuery, isSearchOpen, toggleSearch } = useUIStore()
  const [searchResults, setSearchResults] = useState<Task[]>([])
  const [activeId, setActiveId] = useState<string | null>(null)

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
    useSensor(KeyboardSensor)
  )

  useEffect(() => {
    loadTasks()
  }, [])

  const handleSearch = async (query: string) => {
    setSearchQuery(query)
    if (query.trim()) {
      const results = await searchTasks(query.trim())
      setSearchResults(results)
    } else {
      setSearchResults([])
    }
  }

  const handleAddRootTask = async () => {
    const task = await createTask({
      parent_id: null,
      title: '新任务',
    })
    selectTask(task.id)
  }

  const handleDragStart = (event: DragStartEvent) => {
    setActiveId(event.active.id as string)
  }

  const handleDragEnd = useCallback(async (event: DragEndEvent) => {
    setActiveId(null)
    const { active, over } = event
    if (!over || active.id === over.id) return

    const result = computeDropPosition(tasks, active.id as string, over.id as string)
    if (!result) return

    await reorderTask(active.id as string, result.newParentId, result.newSortOrder)
  }, [tasks, reorderTask])

  const handleDragOver = (event: DragOverEvent) => {
    // 仅用于视觉反馈，实际 drop 在 handleDragEnd 中处理
  }

  const filteredTasks = getFilteredTasks()
  const tree = searchQuery.trim()
    ? searchResults
    : buildTaskTree(filteredTasks)

  const allDisplayIds = searchQuery.trim()
    ? searchResults.map((t) => t.id)
    : filteredTasks.map((t) => t.id)

  return (
    <div className="flex flex-col h-full">
      {/* 头部 */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-border">
        <h2 className="text-sm font-semibold text-foreground">任务</h2>
        <div className="flex items-center gap-1">
          <Button
            variant={selectionMode ? 'secondary' : 'ghost'}
            size="icon"
            className="size-7"
            onClick={() => {
              const store = useTaskStore.getState()
              if (store.selectionMode) {
                store.exitSelectionMode()
              } else {
                store.enterSelectionMode()
              }
            }}
            title={selectionMode ? '退出选择' : '批量选择'}
          >
            <GripVertical className="size-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="size-7"
            onClick={toggleSearch}
            title="搜索"
          >
            <Search className="size-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="size-7"
            onClick={expandAll}
            title="全部展开"
          >
            <Expand className="size-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="size-7"
            onClick={collapseAll}
            title="全部折叠"
          >
            <FoldVertical className="size-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="size-7"
            onClick={handleAddRootTask}
            title="添加任务"
          >
            <Plus className="size-3.5" />
          </Button>
        </div>
      </div>

      {/* 搜索框 */}
      {isSearchOpen && (
        <div className="px-3 py-2">
          <Input
            placeholder="搜索任务..."
            value={searchQuery}
            onChange={(e) => handleSearch(e.target.value)}
            className="h-8 text-sm"
            autoFocus
          />
        </div>
      )}

      {/* 筛选栏 */}
      <FilterBar />

      {/* 任务列表 */}
      <div className="flex-1 relative">
        <ScrollArea className="h-full">
          {loading ? (
            <div className="flex items-center justify-center py-8 text-sm text-muted-foreground">
              加载中...
            </div>
          ) : error ? (
            <div className="flex items-center justify-center py-8 text-sm text-red-400">
              加载失败: {error}
            </div>
          ) : tree.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-sm text-muted-foreground gap-2">
              <p>{filteredTasks.length === 0 && tasks.length > 0 ? '无匹配任务' : '还没有任务'}</p>
              <Button variant="outline" size="sm" onClick={handleAddRootTask}>
                <Plus className="size-3.5" />
                创建第一个任务
              </Button>
            </div>
          ) : (
            <DndContext
              sensors={sensors}
              collisionDetection={closestCenter}
              onDragStart={handleDragStart}
              onDragOver={handleDragOver}
              onDragEnd={handleDragEnd}
            >
              <SortableContext items={allDisplayIds} strategy={verticalListSortingStrategy}>
                <div className="py-1">
                  {tree.map((task) => (
                    <TaskNode
                      key={task.id}
                      task={task}
                      isDragActive={task.id === activeId}
                    />
                  ))}
                </div>
              </SortableContext>
            </DndContext>
          )}
        </ScrollArea>

        {/* 批量操作工具栏 */}
        {selectionMode && <BatchActionBar />}
      </div>
    </div>
  )
}
