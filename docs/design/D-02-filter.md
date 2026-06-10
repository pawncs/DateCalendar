# D-02: 状态/优先级筛选

## 1. 必要性 (Why)

### 问题
当前 TaskTree 的搜索栏只能按关键词搜索，无法按任务状态（pending/in_progress/completed/cancelled）或优先级（0-3）过滤。当任务树增长到 50+ 节点时，用户需要快速定位「所有进行中的任务」或「所有高优先级任务」。

### 场景
- 晨间规划：只看今天要做的 in_progress 任务
- 周回顾：过滤出所有 completed 任务，统计完成量
- 危机处理：筛选 priority=3 的高优先级任务

### 与 D-01 的关系
筛选和排序是独立的维度：排序改变顺序，筛选改变可见性。两者可组合使用。

---

## 2. 实现方案 (How)

### 2.1 整体流程

```
用户在筛选栏选择 status + priority
  → taskStore.setFilter({ status, priority })
  → getFilteredTasks() 前端过滤
  → TaskTree 重新渲染过滤后的树
```

🔍 知识点雷达: 前端过滤 vs 后端过滤
   ├── 是什么: 前端过滤在 Zustand store 中完成，不经过 IPC；后端过滤在 SQL 查询中完成
   ├── 为什么用前端过滤: Phase 1 已经一次性加载了所有任务到 store，数据量在百级别，前端过滤足够快且无 IPC 开销。当任务量超过 1000+ 时可迁移到后端分页+过滤
   ├── 核心心智模型: 全量数据在 store → 派生过滤状态 → 过滤函数返回子集 → 传给 buildTaskTree
   └── 关联概念: taskStore.tasks（全量）→ filterState（用户选择）→ filteredTasks（派生数据）

### 2.2 前端实现

**新建筛选栏组件**：`src/components/tasks/FilterBar.tsx`，包含状态多选标签和优先级单选标签。

**taskStore 扩展**：
- 新增 `TaskFilter` 接口：`statuses`（多选状态数组）、`priority`（最低优先级阈值）、`searchQuery`
- 新增 `setFilter`、`clearFilter`、`getFilteredTasks` 方法
- `getFilteredTasks` 按状态、优先级、搜索关键词三层过滤

**改造 TaskTree.tsx**：使用 `getFilteredTasks()` 代替 `tasks`，下方嵌入 `FilterBar`。

### 2.3 父子节点可见性

**规则**：如果子节点匹配筛选但父节点不匹配，父节点仍然显示（但可能变灰），以保持树形结构完整。实现 `filterWithAncestors` 函数：先标记所有匹配节点，再向上标记所有祖先，最后返回匹配节点 + 祖先节点的并集。

---

## 3. 验证标准 (Verify)

### 功能验证

| 测试场景 | 预期结果 | 验证方法 |
|----------|----------|----------|
| 单选状态 | 选「进行中」→ 只显示 in_progress 任务 | 手动操作 → 目视确认 |
| 多选状态 | 同时选「待办」+「进行中」→ 显示两种状态 | 手动操作 → 目视确认 |
| 优先级筛选 | 选 P2 → 显示 priority≥2 的任务 | 手动操作 → 检查节点 |
| 组合筛选 | status=pending + priority=3 → 同时满足两个条件 | 手动操作 |
| 搜索+筛选组合 | 输入关键词 + 选状态 → 交集结果 | 手动操作 |
| 清除筛选 | 点清除 → 恢复全部任务 | 手动操作 |
| 父子保持 | 子任务匹配但父任务不匹配 → 父任务仍显示 | 手动操作 |
| 空结果 | 筛选无匹配 → 显示「无匹配任务」提示 | 手动操作 |

### 交互体验验证

| 场景 | 预期 |
|------|------|
| 筛选标签视觉反馈 | 选中标签高亮，未选中标签灰色 |
| 筛选数量提示 | 筛选栏显示匹配数量：「找到 12 个任务」 |
| 状态保持 | 切换视图后筛选条件保持 |

### 技术验证

```bash
npx tsc -b      # 零错误
npx vite build  # 构建成功
```
