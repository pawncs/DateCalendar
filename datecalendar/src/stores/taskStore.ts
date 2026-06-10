import { create } from 'zustand'
import { adapter } from '@/adapters'
import type { Task, CreateTaskInput, UpdateTaskInput, MilestoneRisk, Note, TaskStatus, Priority } from '@/types/task'

// 筛选条件
export interface TaskFilter {
  statuses: TaskStatus[]
  minPriority: Priority
  searchQuery: string
}

// 默认筛选（全部显示）
const defaultFilter: TaskFilter = {
  statuses: [],
  minPriority: 0,
  searchQuery: '',
}

interface TaskState {
  // 数据
  tasks: Task[]
  selectedTaskId: string | null
  expandedIds: Set<string>

  // 加载状态
  loading: boolean
  error: string | null

  // 筛选
  filter: TaskFilter

  // 批量选择
  selectionMode: boolean
  selectedIds: Set<string>

  // 操作
  loadTasks: () => Promise<void>
  selectTask: (id: string | null) => void
  toggleExpand: (id: string) => void
  expandAll: () => void
  collapseAll: () => void

  createTask: (input: CreateTaskInput) => Promise<Task>
  updateTask: (id: string, input: UpdateTaskInput) => Promise<Task>
  deleteTask: (id: string) => Promise<void>

  // 排序
  reorderTask: (taskId: string, newParentId: string | null, newSortOrder: number) => Promise<void>

  // 筛选
  setFilter: (filter: Partial<TaskFilter>) => void
  clearFilter: () => void
  getFilteredTasks: () => Task[]

  // 批量操作
  enterSelectionMode: () => void
  exitSelectionMode: () => void
  toggleSelection: (id: string) => void
  batchComplete: () => Promise<void>
  batchDelete: () => Promise<void>
  batchMove: (newParentId: string | null) => Promise<void>

  // 风险
  loadRisks: (taskId: string) => Promise<MilestoneRisk[]>
  addRisk: (taskId: string, riskDesc: string, probability?: string, mitigation?: string) => Promise<MilestoneRisk>
  deleteRisk: (riskId: string) => Promise<void>

  // 笔记
  loadNotes: (taskId: string) => Promise<Note[]>
  saveNote: (taskId: string, noteId: string | null, title: string, content: string) => Promise<Note>
  deleteNote: (noteId: string) => Promise<void>

  // 搜索
  searchTasks: (query: string) => Promise<Task[]>
}

export const useTaskStore = create<TaskState>((set, get) => ({
  tasks: [],
  selectedTaskId: null,
  expandedIds: new Set<string>(),
  loading: false,
  error: null,
  filter: { ...defaultFilter },
  selectionMode: false,
  selectedIds: new Set<string>(),

  loadTasks: async () => {
    set({ loading: true, error: null })
    try {
      const tasks = await adapter.get_all_tasks()
      set({ tasks, loading: false })
    } catch (e) {
      set({ error: String(e), loading: false })
    }
  },

  selectTask: (id) => set({ selectedTaskId: id }),

  toggleExpand: (id) => {
    const expanded = new Set(get().expandedIds)
    if (expanded.has(id)) {
      expanded.delete(id)
    } else {
      expanded.add(id)
    }
    set({ expandedIds: expanded })
  },

  expandAll: () => {
    const ids = new Set(get().tasks.map((t) => t.id))
    set({ expandedIds: ids })
  },

  collapseAll: () => {
    set({ expandedIds: new Set() })
  },

  createTask: async (input) => {
    const task = await adapter.create_task(input)
    await get().loadTasks()
    return task
  },

  updateTask: async (id, input) => {
    const task = await adapter.update_task(
      id, input.title, input.description, input.status,
      input.priority, input.color, input.is_milestone,
      input.parent_id, input.sort_order,
    )
    await get().loadTasks()
    return task
  },

  deleteTask: async (id) => {
    await adapter.delete_task(id)
    if (get().selectedTaskId === id) {
      set({ selectedTaskId: null })
    }
    await get().loadTasks()
  },

  reorderTask: async (taskId, newParentId, newSortOrder) => {
    await adapter.reorder_task(taskId, newParentId, newSortOrder)
    await get().loadTasks()
  },

  // ===== 筛选 =====

  setFilter: (partial) => {
    set((s) => ({ filter: { ...s.filter, ...partial } }))
  },

  clearFilter: () => {
    set({ filter: { ...defaultFilter } })
  },

  getFilteredTasks: () => {
    const { tasks, filter } = get()
    let result = tasks

    // 按状态筛选
    if (filter.statuses.length > 0) {
      result = result.filter((t) => filter.statuses.includes(t.status))
    }

    // 按优先级筛选（>= minPriority）
    if (filter.minPriority > 0) {
      result = result.filter((t) => t.priority >= filter.minPriority)
    }

    // 按关键词搜索
    if (filter.searchQuery.trim()) {
      const q = filter.searchQuery.trim().toLowerCase()
      result = result.filter(
        (t) => t.title.toLowerCase().includes(q) || t.description.toLowerCase().includes(q)
      )
    }

    // 保留祖先节点以维持树形结构
    if (filter.statuses.length > 0 || filter.minPriority > 0 || filter.searchQuery.trim()) {
      const matchedIds = new Set(result.map((t) => t.id))
      // 向上查找所有祖先
      for (const task of tasks) {
        let current: Task | undefined = task
        while (current) {
          if (matchedIds.has(current.id)) {
            // 标记所有祖先
            let ancestor = tasks.find((t) => t.id === task.id)
            while (ancestor) {
              matchedIds.add(ancestor.id)
              ancestor = ancestor.parent_id ? tasks.find((t) => t.id === ancestor!.parent_id) : undefined
            }
            break
          }
          current = current.parent_id ? tasks.find((t) => t.id === current!.parent_id) : undefined
        }
      }
      result = tasks.filter((t) => matchedIds.has(t.id))
    }

    return result
  },

  // ===== 批量操作 =====

  enterSelectionMode: () => set({ selectionMode: true, selectedIds: new Set() }),
  exitSelectionMode: () => set({ selectionMode: false, selectedIds: new Set() }),

  toggleSelection: (id) => {
    const selected = new Set(get().selectedIds)
    if (selected.has(id)) {
      selected.delete(id)
    } else {
      selected.add(id)
    }
    set({ selectedIds: selected })
  },

  batchComplete: async () => {
    const ids = Array.from(get().selectedIds)
    if (ids.length === 0) return
    await adapter.batch_update_tasks(ids, 'completed')
    set({ selectionMode: false, selectedIds: new Set() })
    await get().loadTasks()
  },

  batchDelete: async () => {
    const ids = Array.from(get().selectedIds)
    if (ids.length === 0) return
    await adapter.batch_delete_tasks(ids)
    set({ selectionMode: false, selectedIds: new Set() })
    await get().loadTasks()
  },

  batchMove: async (newParentId) => {
    const ids = Array.from(get().selectedIds)
    if (ids.length === 0) return
    await adapter.batch_move_tasks(ids, newParentId)
    set({ selectionMode: false, selectedIds: new Set() })
    await get().loadTasks()
  },

  loadRisks: async (taskId) => {
    return adapter.get_risks(taskId)
  },

  addRisk: async (taskId, riskDesc, probability, mitigation) => {
    return adapter.add_risk(taskId, riskDesc, probability, mitigation)
  },

  deleteRisk: async (riskId) => {
    await adapter.delete_risk(riskId)
  },

  loadNotes: async (taskId) => {
    return adapter.get_notes(taskId)
  },

  saveNote: async (taskId, noteId, title, content) => {
    return adapter.save_note(taskId, noteId, title, content)
  },

  deleteNote: async (noteId) => {
    await adapter.delete_note(noteId)
  },

  searchTasks: async (query) => {
    return adapter.search_tasks(query)
  },
}))

/**
 * 工具函数：将平铺任务列表构建为树形结构
 *
 * 算法：单次遍历 O(n)
 * 1. 创建 id → Task 映射
 * 2. 遍历所有任务，根据 parent_id 挂到父节点的 children 数组
 * 3. 返回 parent_id 为 null 的根节点列表
 */
export function buildTaskTree(tasks: Task[]): Task[] {
  const map = new Map<string, Task>()
  const roots: Task[] = []

  // 第一遍：初始化所有节点，添加 children 数组
  for (const task of tasks) {
    map.set(task.id, { ...task, children: [] })
  }

  // 第二遍：根据 parent_id 挂载
  for (const task of map.values()) {
    if (task.parent_id && map.has(task.parent_id)) {
      map.get(task.parent_id)!.children!.push(task)
    } else {
      roots.push(task)
    }
  }

  return roots
}

/**
 * 工具函数：计算任务的缩进深度
 */
export function getTaskDepth(tasks: Task[], taskId: string): number {
  let depth = 0
  let current = tasks.find((t) => t.id === taskId)
  while (current?.parent_id) {
    depth++
    current = tasks.find((t) => t.id === current!.parent_id)
  }
  return depth
}
