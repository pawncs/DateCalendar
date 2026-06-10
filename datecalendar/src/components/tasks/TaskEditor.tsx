import { useState, useEffect } from 'react'
import { Save, Star, AlertTriangle, FileText } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Select } from '@/components/ui/select'
import { useTaskStore } from '@/stores/taskStore'
import { MilestonePanel } from './MilestonePanel'
import { NoteEditor } from './NoteEditor'
import type { TaskStatus, Priority } from '@/types/task'

export function TaskEditor() {
  const { selectedTaskId, tasks, updateTask } = useTaskStore()
  const task = tasks.find((t) => t.id === selectedTaskId)

  const [title, setTitle] = useState('')
  const [description, setDescription] = useState('')
  const [status, setStatus] = useState<TaskStatus>('pending')
  const [priority, setPriority] = useState<Priority>(0)
  const [isMilestone, setIsMilestone] = useState(false)
  const [color, setColor] = useState('')
  const [activeTab, setActiveTab] = useState<'details' | 'risks' | 'notes'>('details')

  useEffect(() => {
    if (task) {
      setTitle(task.title)
      setDescription(task.description)
      setStatus(task.status as TaskStatus)
      setPriority(task.priority as Priority)
      setIsMilestone(task.is_milestone)
      setColor(task.color)
    }
  }, [task?.id])

  if (!task) {
    return (
      <div className="flex items-center justify-center h-full text-sm text-muted-foreground">
        选择一个任务查看详情
      </div>
    )
  }

  const handleSave = async () => {
    await updateTask(task.id, {
      title,
      description,
      status,
      priority,
      is_milestone: isMilestone,
      color,
    })
  }

  const presetColors = [
    '', '#ef4444', '#f97316', '#eab308', '#22c55e',
    '#06b6d4', '#3b82f6', '#8b5cf6', '#ec4899',
  ]

  return (
    <div className="flex flex-col h-full">
      {/* 头部 */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-border">
        <div className="flex items-center gap-2">
          {isMilestone && <Star className="size-4 text-yellow-400 fill-yellow-400" />}
          <h2 className="text-sm font-semibold text-foreground">任务详情</h2>
        </div>
        <Button size="sm" onClick={handleSave}>
          <Save className="size-3.5" />
          保存
        </Button>
      </div>

      {/* 标签切换 */}
      <div className="flex border-b border-border">
        {(['details', 'risks', 'notes'] as const).map((tab) => (
          <button
            key={tab}
            className={`px-4 py-2 text-sm border-b-2 transition-colors ${
              activeTab === tab
                ? 'border-primary text-foreground'
                : 'border-transparent text-muted-foreground hover:text-foreground'
            }`}
            onClick={() => setActiveTab(tab)}
          >
            {tab === 'details' && '详情'}
            {tab === 'risks' && (
              <span className="flex items-center gap-1">
                <AlertTriangle className="size-3" />
                风险
              </span>
            )}
            {tab === 'notes' && (
              <span className="flex items-center gap-1">
                <FileText className="size-3" />
                笔记
              </span>
            )}
          </button>
        ))}
      </div>

      {/* 内容区域 */}
      <div className="flex-1 overflow-auto">
        {activeTab === 'details' && (
          <div className="p-4 space-y-4">
            {/* 标题 */}
            <div>
              <label className="text-xs text-muted-foreground mb-1 block">标题</label>
              <Input
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                className="text-sm"
              />
            </div>

            {/* 描述 */}
            <div>
              <label className="text-xs text-muted-foreground mb-1 block">描述</label>
              <Textarea
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                className="text-sm min-h-[80px]"
                placeholder="添加任务描述..."
              />
            </div>

            {/* 状态和优先级 */}
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="text-xs text-muted-foreground mb-1 block">状态</label>
                <Select
                  value={status}
                  onChange={(v) => setStatus(v as TaskStatus)}
                  className="w-full"
                >
                  <option value="pending">待开始</option>
                  <option value="in_progress">进行中</option>
                  <option value="completed">已完成</option>
                  <option value="cancelled">已取消</option>
                </Select>
              </div>
              <div>
                <label className="text-xs text-muted-foreground mb-1 block">优先级</label>
                <Select
                  value={String(priority)}
                  onChange={(v) => setPriority(Number(v) as Priority)}
                  className="w-full"
                >
                  <option value="0">无</option>
                  <option value="1">低</option>
                  <option value="2">中</option>
                  <option value="3">高</option>
                </Select>
              </div>
            </div>

            {/* 里程碑开关 */}
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                id="isMilestone"
                checked={isMilestone}
                onChange={(e) => setIsMilestone(e.target.checked)}
                className="rounded"
              />
              <label htmlFor="isMilestone" className="text-sm cursor-pointer">
                标记为里程碑
              </label>
            </div>

            {/* 颜色选择 */}
            <div>
              <label className="text-xs text-muted-foreground mb-1 block">颜色标记</label>
              <div className="flex gap-2">
                {presetColors.map((c) => (
                  <button
                    key={c || 'none'}
                    className={`size-6 rounded-full border-2 transition-all ${
                      color === c ? 'border-foreground scale-110' : 'border-transparent'
                    }`}
                    style={{ backgroundColor: c || '#666' }}
                    onClick={() => setColor(c)}
                    title={c || '无颜色'}
                  />
                ))}
              </div>
            </div>

            {/* 时间信息 */}
            <div className="text-xs text-muted-foreground space-y-1 pt-2 border-t border-border">
              <p>创建: {new Date(task.created_at).toLocaleString()}</p>
              <p>更新: {new Date(task.updated_at).toLocaleString()}</p>
              {task.completed_at && (
                <p className="text-green-400">完成: {new Date(task.completed_at).toLocaleString()}</p>
              )}
            </div>
          </div>
        )}

        {activeTab === 'risks' && <MilestonePanel taskId={task.id} />}
        {activeTab === 'notes' && <NoteEditor taskId={task.id} />}
      </div>
    </div>
  )
}
