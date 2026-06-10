import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'
import type { Task, CreateTaskInput, UpdateTaskInput, MilestoneRisk, Note } from '@/types/task'

interface TaskState {
  // 数据
  tasks: Task[]
  selectedTaskId: string | null
  expandedIds: Set<string>

  // 加载状态
  loading: boolean
  error: string | null

  // 操作
  loadTasks: () => Promise<void>
  selectTask: (id: string | null) => void
  toggleExpand: (id: string) => void
  expandAll: () => void
  collapseAll: () => void

  createTask: (input: CreateTaskInput) => Promise<Task>
  updateTask: (id: string, input: UpdateTaskInput) => Promise<Task>
  deleteTask: (id: string) => Promise<void>

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

  loadTasks: async () => {
    set({ loading: true, error: null })
    try {
      const tasks = await invoke<Task[]>('get_all_tasks')
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
    const task = await invoke<Task>('create_task', { input })
    await get().loadTasks()
    return task
  },

  updateTask: async (id, input) => {
    const task = await invoke<Task>('update_task', { id, ...input })
    await get().loadTasks()
    return task
  },

  deleteTask: async (id) => {
    await invoke('delete_task', { id })
    // 如果删除的是当前选中的任务，清除选中状态
    if (get().selectedTaskId === id) {
      set({ selectedTaskId: null })
    }
    await get().loadTasks()
  },

  loadRisks: async (taskId) => {
    return invoke<MilestoneRisk[]>('get_risks', { taskId })
  },

  addRisk: async (taskId, riskDesc, probability, mitigation) => {
    return invoke<MilestoneRisk>('add_risk', { taskId, riskDesc, probability, mitigation })
  },

  deleteRisk: async (riskId) => {
    await invoke('delete_risk', { riskId })
  },

  loadNotes: async (taskId) => {
    return invoke<Note[]>('get_notes', { taskId })
  },

  saveNote: async (taskId, noteId, title, content) => {
    return invoke<Note>('save_note', { taskId, noteId, title, content })
  },

  deleteNote: async (noteId) => {
    await invoke('delete_note', { noteId })
  },

  searchTasks: async (query) => {
    return invoke<Task[]>('search_tasks', { query })
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
