# D-14: 悬浮窗交互体验 — 动画 / 透明度 / 自动隐藏

## 1. 必要性 (Why)

### 问题
D-13 解决了"有悬浮窗"，但还需要解决"悬浮窗好用"。裸窗口的显隐切换是生硬的瞬移，需要丝滑动画来营造品质感。同时用户应该能自主调节透明度以适配不同桌面背景，以及设置多久不操作自动隐藏。

### 场景
- 鼠标靠近右边缘 → 悬浮窗以弹性动画滑出（ease-out），而非瞬移
- 用户将透明度设为 60%，透过悬浮窗能看到背后的 IDE 代码
- 用户查看完待办后继续工作，5 秒无操作 → 悬浮窗自动滑回隐藏

### 设计原则
- **丝滑动画**：使用 Framer Motion 的 `spring` 或 `ease-out`，滑出约 250ms，滑回约 300ms
- **透明度可控**：滑块 20%-100%，存储到 `settingsStore` + `settings` 表，持久化
- **自动隐藏**：鼠标离开后 N 秒无操作自动隐藏，N 可配置（默认 3 秒）

---

## 2. 实现方案 (How)

### 2.1 滑入/滑出动画

使用 **Framer Motion** 为悬浮窗容器添加 x 轴位移动画。

**方案选择**：动画层面由**前端 CSS/Framer Motion** 控制，而非 Rust 侧逐帧设置窗口位置。原因如下：
1. Framer Motion 提供 `spring` 和 `ease` 预设，无需手动插值
2. 前端动画运行在 GPU 加速的 WebView 内，性能足够
3. Rust 侧只负责告知前端"显示"或"隐藏"状态

**实现方式**：
- `FloatingWindow.tsx` 用一个 `isVisible` 状态驱动
- `animate={{ x: isVisible ? 0 : 310 }}`（窗口宽 340px，留 30px 边缘可见）
- 过渡：`spring`（stiffness: 300, damping: 30）→ 轻微弹性效果
- 使用 `motion.div` 包裹整个悬浮窗内容

```
屏幕右边缘
    │
    │  ┌──────────┐
    │  │          │  隐藏态: transformX = windowWidth - 8px (留8px边缘)
    │  │ 悬浮窗    │
    │  │          │  显示态: transformX = 0
    │  └──────────┘
    │◄── 340px ──►
```

### 2.2 边缘检测与触发

前端 `FloatingWindow.tsx` 中：

```typescript
// 使用 Tauri 窗口事件监听鼠标位置
// 由于透明/无边框窗口不能直接捕获外部鼠标事件，
// 方案：在前端用间隔轮询 + Tauri Window API 获取光标位置

const TRIGGER_ZONE = 20;    // 触发区宽度 (px)
const HIDE_DELAY = 3000;    // 离开后自动隐藏延迟 (ms)
const STAY_DELAY = 200;     // 停留触发延迟 (ms)，防误触

// 光标进入触发区 → 等待 STAY_DELAY → 显示
// 光标离开窗口区域 → 等待 HIDE_DELAY → 隐藏
// 光标在窗口内 → 取消隐藏计时器
```

**Rust 侧辅助**：提供 `get_cursor_position` 命令，返回 `{x, y}` 屏幕坐标，供前端轮询使用。

### 2.3 透明度控制

**数据存储**：
- 前端 `settingsStore` 中新增 `floatingOpacity: number`（0.2 ~ 1.0，默认 0.85）
- 同步保存到数据库 `settings` 表：`key='floating_opacity', value='0.85'`
- 启动时从数据库加载

**UI 控件**：
- 悬浮窗顶部或底部添加一个小齿轮图标 → 点击弹出迷你设置面板
- 面板中：透明度滑块（`<input type="range">`）
- 实时预览：拖动滑块时窗口透明度即时更新

**技术实现**：
- 前端通过 CSS 变量 `--floating-opacity: 0.85` 控制根容器 `opacity`
- Framer Motion 的 `animate` 也可以控制 `opacity`，实现平滑过渡

> 注意：window-level transparency（`transparent: true`）已在 D-13 的窗口配置中启用。此处所述 opacity 是内容层的 CSS 透明度。

### 2.4 定时自动隐藏

**逻辑**：
1. 悬浮窗显示状态下，启动一个定时器（`setTimeout`）
2. 用户每次在悬浮窗内交互（mousemove/click/scroll）→ 重置定时器
3. 定时器到期 → 触发隐藏动画

**可配置项**（存入 settingsStore + 数据库）：
| 配置项 | key | 默认值 | 范围 |
|--------|-----|--------|------|
| 自动隐藏延迟 | `floating_auto_hide_ms` | 3000 | 1000-30000 |
| 鼠标停留触发延迟 | `floating_stay_ms` | 200 | 0-1000 |
| 边缘触发区宽度 | `floating_trigger_zone` | 20 | 5-100 |

### 2.5 文件结构

```
src/components/floating/
├── FloatingWindow.tsx       # 悬浮窗主容器（动画、边缘检测、自动隐藏）
├── FloatingMiniSettings.tsx # 迷你设置面板（透明度滑块、自动隐藏设置）
└── FloatingContent.tsx      # 内容视图（见 D-16）

src-tauri/src/
└── floating_window.rs      # 新增 get_cursor_position 命令
```

---

## 3. 验证标准 (Verify)

### 功能验证

| 测试场景 | 预期结果 | 验证方法 |
|----------|----------|----------|
| 滑出动画 | 鼠标靠近右边缘 200ms 后，悬浮窗以 spring 动画滑出 | 手动操作，观察动画 |
| 滑回动画 | 鼠标离开 3 秒后（默认），悬浮窗滑回 | 手动操作，计时 |
| 防误触 | 鼠标快速划过（<200ms）不触发显示 | 快速划过屏幕右边缘 |
| 透明度调节 | 拖动滑块 → 悬浮窗透明度实时变化 | 拖动滑块 → 目视 |
| 透明度持久化 | 设置 60% → 重启应用 → 仍是 60% | 重启 → 检查 |
| 自动隐藏 | 显示悬浮窗 → 不动鼠标 3 秒 → 自动隐藏 | 计时验证 |
| 交互重置计时 | 显示悬浮窗 → 移动鼠标 → 计时器重置 | 持续交互 → 不自动隐藏 |
| 配置持久化 | 修改隐藏延迟为 5 秒 → 重启 → 仍是 5 秒 | 重启 → 验证 |

### 交互体验验证

| 场景 | 预期 |
|------|------|
| 动画帧率 | 滑出/滑回动画稳定 60fps |
| 弹性效果 | 滑出到位时轻微回弹（spring 效果） |
| 透明度过渡 | 拖动滑块时透明度平滑过渡，无跳变 |
| 迷你设置面板 | 点击齿轮 → 面板弹出 → 设置后自动收起 |

### 技术验证

```bash
npm list framer-motion    # 确认 framer-motion 已安装
npx tsc -b                 # 零错误
npx vite build             # 构建成功
npx tauri dev              # 手动 E2E
```
