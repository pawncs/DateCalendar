# D-12: 前端适配层

## 1. 必要性 (Why)

### 问题
前端需要在三种运行环境中无缝切换：Tauri 桌面（IPC）、浏览器连接 Tauri（HTTP API）、浏览器离线（SQL.js）。不能每处调用都写 `if/else` 判断环境，需要一个统一的适配层。

### 场景
- 用户双击桌面应用 → 适配层检测 Tauri 环境 → 使用 IPC
- 用户在 `start.bat` 启动后用浏览器打开 `localhost:5173` → 适配层检测 HTTP API → 使用 HTTP
- 用户只用 `npx vite` 启动前端 → 适配层检测无 Tauri → 降级到 SQL.js，显示离线提示

### 设计原则
- **对上层透明**：Store 层只调用 `adapter.xxx()`，不感知环境
- **自动检测**：无需手动配置，启动时自动判断
- **不修改现有组件**：现有 React 组件代码不变
- **降级可感知**：用户能明确知道当前是否在离线模式

---

## 2. 实现方案 (How)

### 2.1 环境检测流程

```
App 启动
  → detectEnv()
  → 检测 __TAURI_INTERNALS__ ?
      YES → 'tauri' 模式（IPC）
      NO  → 检测 localhost:9876/health ?
              YES → 'http' 模式（HTTP API）
              NO  → 'sqljs' 模式（离线降级）
  → 初始化对应后端
  → 如果是 sqljs → 显示 OfflineBanner
  → 渲染 App
```

### 2.2 文件结构

```
src/adapters/
├── index.ts              # 统一导出 + 环境检测 + 后端路由
├── types.ts              # 接口类型定义
├── tauriBackend.ts       # Tauri IPC 封装
├── httpBackend.ts        # HTTP API 客户端（fetch）
└── sqljsBackend.ts       # SQL.js 离线降级
```

### 2.3 适配器实现

#### index.ts — 统一路由

```typescript
type BackendMode = 'tauri' | 'http' | 'sqljs';

let mode: BackendMode;
let backend: BackendInterface;

export async function initAdapter(): Promise<BackendMode> {
  // 1. 检测 Tauri 环境
  if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
    mode = 'tauri';
    backend = tauriBackend;
    return mode;
  }

  // 2. 检测 HTTP API 是否可达
  try {
    const res = await fetch('http://localhost:9876/api/health');
    if (res.ok) {
      mode = 'http';
      backend = httpBackend;
      return mode;
    }
  } catch {}

  // 3. 降级到 SQL.js
  mode = 'sqljs';
  const { initDatabase } = await import('../backend/db');
  await initDatabase();
  backend = sqljsBackend;
  return mode;
}

export function getMode(): BackendMode {
  return mode;
}

export function isOffline(): boolean {
  return mode === 'sqljs';
}

export const adapter = new Proxy({} as BackendInterface, {
  get(_, prop: string) {
    return (...args: any[]) => (backend as any)[prop](...args);
  }
});
```

#### tauriBackend.ts — Tauri IPC

```typescript
import { invoke } from '@tauri-apps/api/core';

export async function get_all_tasks(): Promise<Task[]> {
  return invoke<Task[]>('get_all_tasks');
}

export async function create_task(input: NewTask): Promise<Task> {
  return invoke<Task>('create_task', { input });
}
// ... 其余 25 个函数直接透传 invoke
```

#### httpBackend.ts — HTTP API 客户端

```typescript
const BASE = 'http://localhost:9876';

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
  });
  if (!res.ok) throw new Error(await res.text());
  return res.json();
}

export async function get_all_tasks(): Promise<Task[]> {
  return request<Task[]>('/api/tasks');
}

export async function create_task(input: NewTask): Promise<Task> {
  return request<Task>('/api/tasks', {
    method: 'POST',
    body: JSON.stringify(input),
  });
}
// ... 其余函数映射到 HTTP 路由
```

#### sqljsBackend.ts — 离线降级

与上一版相同，调用 `src/backend/` 中的 SQL.js 实现。详见 D-11 中的降级方案。

### 2.4 Store 层修改

```typescript
// 修改前
import { invoke } from '@tauri-apps/api/core';
const tasks = await invoke<Task[]>('get_all_tasks');

// 修改后
import { adapter } from '../adapters';
const tasks = await adapter.get_all_tasks();
```

### 2.5 离线模式 UI

#### OfflineBanner 组件

```typescript
// src/components/common/OfflineBanner.tsx
// 仅在 isOffline() === true 时渲染
// 固定在页面底部，黄色背景
// 文案：「离线模式 — Tauri 后端未连接，数据仅保存在浏览器内存中」
// 右侧关闭按钮
```

#### 使用方式

```typescript
// src/App.tsx
import { isOffline } from '../adapters';
import { OfflineBanner } from '../components/common/OfflineBanner';

function App() {
  const [offline, setOffline] = useState(false);

  useEffect(() => {
    setOffline(isOffline());
  }, []);

  return (
    <>
      {/* 正常页面内容 */}
      {offline && <OfflineBanner onDismiss={() => setOffline(false)} />}
    </>
  );
}
```

### 2.6 初始化流程

```typescript
// src/main.tsx
import { initAdapter } from './adapters';

async function bootstrap() {
  const mode = await initAdapter();
  console.log(`[Adapter] Running in ${mode} mode`);

  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
}

bootstrap();
```

---

## 3. 验证标准 (Verify)

### 功能验证

| 测试场景 | 预期结果 | 验证方法 |
|----------|----------|----------|
| Tauri 环境自动检测 | 适配层选择 `tauri` 模式 | 启动 `npx tauri dev` → 控制台日志 |
| HTTP API 可达 | 适配层选择 `http` 模式 | `start.bat` 后浏览器访问 → 控制台日志 |
| HTTP API 不可达 | 适配层降级到 `sqljs` 模式 | 仅 `npx vite` → 控制台日志 |
| 离线模式提示 | 页面底部显示黄色 OfflineBanner | 目视确认 |
| 关闭离线提示 | 点击关闭按钮 → Banner 消失 | 手动操作 |
| Store 透明调用 | Store 代码中无 `invoke` 导入 | 代码审查 |
| 三种模式数据操作一致 | 相同操作在三模式下返回相同结果 | 对比测试 |

### 边界条件

| 场景 | 预期 |
|------|------|
| HTTP API 启动后恢复连接 | 需刷新页面重新检测（或定时重试） |
| Tauri 未启动 + 浏览器访问 | 自动降级，显示离线提示 |
| 网络慢导致 HTTP 超时 | 设置合理超时（如 3s），超时后降级 |

### 技术验证

```bash
npx tsc -b                        # TypeScript 零错误
npx vite build                    # 构建成功
npx vite                          # 浏览器启动 → sqljs 模式 + 离线提示
start start.bat                   # 双端启动 → 浏览器 http 模式 + 无离线提示
npx tauri dev                     # 桌面启动 → tauri 模式
```

---

*文档版本: v2.0 | 创建日期: 2026-06-10 | 依赖: D-11*
