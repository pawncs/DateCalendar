# D-01: 拖拽排序

## 1. 必要性 (Why)

### 问题
当前任务树展示的是数据库 `sort_order` 决定的固定顺序。用户无法手动调整任务顺序或将任务移动到不同父节点下。对于一个日程管理工具，任务优先级和分组关系经常变化，手动排序是刚需。

### 场景
- 用户想把「紧急任务」拖到列表顶部
- 用户发现某个子任务应该属于另一个里程碑，想直接拖过去
- 用户想调整里程碑之间的先后顺序

### 技术收益
拖拽排序完成后，`sort_order` 和 `parent_id` 字段才真正发挥价值，之前它们只是数据库中的占位字段。

---

## 2. 实现方案 (How)

### 2.1 整体流程

```
用户拖拽 TaskNode
  → 前端计算新位置 (新 parent_id, 新 sort_order)
  → 调用 Rust 命令 update_task_order
  → 后端批量更新 sort_order (同父节点下的兄弟重排)
  → 前端刷新任务树
```

### 2.2 技术选型

🔍 知识点雷达: `@dnd-kit/core`
   ├── 是什么: 一个轻量级 React 拖拽库，支持可访问的键盘拖拽、排序、碰撞检测
   ├── 为什么用: 比 react-beautiful-dnd 更现代（维护活跃），比原生 HTML5 DnD API 更易用，支持树形结构
   ├── 核心心智模型: DndContext (拖拽上下文) → Sensors (输入源) → Collision Detection (碰撞算法) → SortableContext (排序容器)
   └── 关联概念: TaskTree → TaskNode 递归结构，每个节点是独立的 draggable

### 2.3 前端实现

**安装依赖**：
```bash
npm install @dnd-kit/core @dnd-kit/sortable @dnd-kit/utilities
```

**改造 TaskTree.tsx**：在 TaskTree 外层包裹 `DndContext`，处理 `onDragEnd` 事件。

**改造 TaskNode.tsx**：每个 TaskNode 使用 `useSortable` hook，获取 drag handle 和 transform。拖拽手柄是一个 GripVertical 图标，仅在 hover 时显示。拖拽时半透明效果 + 蓝色虚线边框表示目标位置。

**碰撞检测策略**：
- 同级节点间：使用 `verticalListSortingStrategy`，只允许在同级排序
- 跨父级移动：检测 drop 位置的父容器，更新 `parent_id`

**handleDragEnd 逻辑**：
```ts
async function handleDragEnd(event: DragEndEvent) {
  const { active, over } = event
  if (!over || active.id === over.id) return
  const draggedTaskId = active.id as string
  const targetTaskId = over.id as string
  const newParentId = computeNewParent(draggedTaskId, targetTaskId)
  const newSortOrder = computeSortOrder(draggedTaskId, targetTaskId, newParentId)
  await invoke('reorder_task', { taskId: draggedTaskId, newParentId, newSortOrder })
  await loadTasks()
}
```

### 2.4 后端实现

**Rust 命令**：`reorder_task`，在 `task_commands.rs` 中新增。

**TaskService 方法** `reorder_task`：
1. 更新该任务的 parent_id 和 sort_order
2. 重新编号同父节点的所有兄弟（消除间隙）
3. 整个操作在一个事务中完成

**防循环检测**：使用递归 CTE 检查目标是否是被拖拽任务的子孙节点，防止循环引用。

---

## 3. 验证标准 (Verify)

### 功能验证

| 测试场景 | 预期结果 | 验证方法 |
|----------|----------|----------|
| 同级拖拽排序 | 任务 A 从第 3 位拖到第 1 位，顺序更新 | 手动操作 → 刷新后确认顺序保持 |
| 跨父级拖拽 | 将子任务从「里程碑 A」拖到「里程碑 B」下，parent_id 更新 | 手动操作 → 确认树结构变化 |
| 防循环移动 | 尝试将父任务拖入其子任务下 → 操作被阻止，无反应 | 手动尝试 → 无变化 |
| 拖到空父节点 | 将一个任务拖到没有子节点的任务上，该任务变为父节点 | 手动操作 → 确认层级 |
| 数据库一致性 | 拖拽后 sort_order 连续无间隙 (0,1,2,3...) | SQL 查询验证 |

### 交互体验验证

| 场景 | 预期 |
|------|------|
| 拖拽时视觉反馈 | 被拖节点半透明，目标位置显示蓝色虚线 |
| 键盘可访问性 | 支持 Tab 聚焦 + Space 提起 + 方向键移动 |

### 技术验证

```bash
cargo check     # Rust 编译通过
npx tsc -b      # TypeScript 编译通过
npx tauri dev   # 手动 E2E
```
