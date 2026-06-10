import { useEffect, useState } from 'react'
import { Plus, Search, Expand, FoldVertical } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useTaskStore, buildTaskTree } from '@/stores/taskStore'
import { useUIStore } from '@/stores/uiStore'
import { TaskNode } from './TaskNode'

export function TaskTree() {
  const { tasks, loadTasks, loading, error, createTask, expandAll, collapseAll, searchTasks, selectTask } = useTaskStore()
  const { searchQuery, setSearchQuery, isSearchOpen, toggleSearch } = useUIStore()
  const [searchResults, setSearchResults] = useState<typeof tasks>([])

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

  const tree = buildTaskTree(tasks)
  const displayTasks = searchQuery.trim() ? searchResults : tree

  return (
    <div className="flex flex-col h-full">
      {/* 头部 */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-border">
        <h2 className="text-sm font-semibold text-foreground">任务</h2>
        <div className="flex items-center gap-1">
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

      {/* 任务列表 */}
      <ScrollArea className="flex-1">
        {loading ? (
          <div className="flex items-center justify-center py-8 text-sm text-muted-foreground">
            加载中...
          </div>
        ) : error ? (
          <div className="flex items-center justify-center py-8 text-sm text-red-400">
            加载失败: {error}
          </div>
        ) : displayTasks.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-sm text-muted-foreground gap-2">
            <p>还没有任务</p>
            <Button variant="outline" size="sm" onClick={handleAddRootTask}>
              <Plus className="size-3.5" />
              创建第一个任务
            </Button>
          </div>
        ) : (
          <div className="py-1">
            {displayTasks.map((task) => (
              <TaskNode key={task.id} task={task} />
            ))}
          </div>
        )}
      </ScrollArea>
    </div>
  )
}
