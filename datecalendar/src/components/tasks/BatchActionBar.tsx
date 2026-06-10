import { useState } from 'react'
import { CheckCircle, Trash2, MoveRight, X, ChevronRight } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useTaskStore } from '@/stores/taskStore'
import { ScrollArea } from '@/components/ui/scroll-area'

export function BatchActionBar() {
  const { selectedIds, exitSelectionMode, batchComplete, batchDelete, batchMove, tasks } = useTaskStore()
  const [showMovePicker, setShowMovePicker] = useState(false)
  const count = selectedIds.size

  const handleBatchComplete = async () => {
    await batchComplete()
  }

  const handleBatchDelete = async () => {
    if (confirm(`确定删除选中的 ${count} 个任务及其子任务？此操作不可撤销。`)) {
      await batchDelete()
    }
  }

  const handleMoveTo = async (parentId: string | null) => {
    await batchMove(parentId)
    setShowMovePicker(false)
  }

  // 构建可选的目标父节点（排除已选中的）
  const availableTargets = tasks
    .filter((t) => !selectedIds.has(t.id))
    .sort((a, b) => a.sort_order - b.sort_order)

  return (
    <div className="absolute bottom-0 left-0 right-0 z-10">
      {/* 移动目标选择器 */}
      {showMovePicker && (
        <div className="mx-2 mb-1 border border-border bg-card rounded-md shadow-lg">
          <div className="flex items-center justify-between px-2 py-1.5 border-b border-border">
            <span className="text-xs font-medium">选择目标位置</span>
            <Button
              variant="ghost"
              size="icon"
              className="size-5"
              onClick={() => setShowMovePicker(false)}
            >
              <X className="size-3" />
            </Button>
          </div>
          <ScrollArea className="max-h-40">
            <button
              className="w-full text-left px-2 py-1.5 text-xs hover:bg-accent flex items-center gap-1.5"
              onClick={() => handleMoveTo(null)}
            >
              <ChevronRight className="size-3 text-muted-foreground" />
              <span className="font-medium">根层级</span>
            </button>
            {availableTargets.map((t) => (
              <button
                key={t.id}
                className="w-full text-left px-2 py-1.5 text-xs hover:bg-accent flex items-center gap-1.5"
                onClick={() => handleMoveTo(t.id)}
              >
                <ChevronRight className="size-3 text-muted-foreground" />
                <span>{t.title}</span>
              </button>
            ))}
          </ScrollArea>
        </div>
      )}

      {/* 底部工具栏 */}
      <div className="flex items-center gap-2 px-3 py-2 bg-sidebar border-t border-border">
        <span className="text-xs text-muted-foreground shrink-0">
          已选 {count} 项
        </span>
        <div className="flex-1" />
        <Button
          variant="outline"
          size="sm"
          className="h-7 text-xs"
          disabled={count === 0}
          onClick={handleBatchComplete}
        >
          <CheckCircle className="size-3" />
          完成
        </Button>
        <Button
          variant="outline"
          size="sm"
          className="h-7 text-xs"
          disabled={count === 0}
          onClick={() => setShowMovePicker(!showMovePicker)}
        >
          <MoveRight className="size-3" />
          移动
        </Button>
        <Button
          variant="outline"
          size="sm"
          className="h-7 text-xs text-red-400 hover:text-red-300"
          disabled={count === 0}
          onClick={handleBatchDelete}
        >
          <Trash2 className="size-3" />
          删除
        </Button>
        <Button
          variant="ghost"
          size="sm"
          className="h-7 text-xs"
          onClick={exitSelectionMode}
        >
          <X className="size-3" />
        </Button>
      </div>
    </div>
  )
}
