# D-15: 全局热键系统

## 1. 必要性 (Why)

### 问题
悬浮窗需要通过鼠标触发，但如果用户在全屏 IDE 中工作，鼠标移到屏幕边缘会触发 IDE 侧边栏。全局热键是另一种触发方式——无论当前焦点在哪个应用，按下热键即可切换悬浮窗显隐。

### 场景
- 用户在全屏 VS Code 中写代码，按 `Ctrl+Shift+D` → 悬浮窗从右边缘滑出
- 看完了，再按 `Ctrl+Shift+D` → 悬浮窗滑回隐藏
- 用户按 `Ctrl+Shift+T` → 循环切换透明度档位（85% → 60% → 40% → 85%）

### 设计原则
- **全局有效**：热键在系统级注册，任何应用中均生效
- **可自定义**：用户可在设置页面修改热键组合
- **无冲突检测**：注册时检测是否被其他应用占用，提示用户
- **最少热键**：初期只提供 2-3 个热键，避免学习负担

---

## 2. 实现方案 (How)

### 2.1 技术选型

Tauri v2 提供 `tauri-plugin-global-shortcut` 插件，封装了系统级全局热键注册。

**Cargo.toml 新增**：
```toml
tauri-plugin-global-shortcut = "2"
```

**前端 npm 包**：
```bash
npm install @tauri-apps/plugin-global-shortcut
```

### 2.2 热键定义

| 热键 ID | 默认组合 | 功能 | 说明 |
|---------|----------|------|------|
| `toggle_floating` | `Ctrl+Shift+D` | 切换悬浮窗显隐 | D = DateCalendar |
| `toggle_transparency` | `Ctrl+Shift+T` | 循环透明度（85%→60%→40%→85%） | T = Transparency |
| `quick_capture` | `Ctrl+Shift+N` | 快速创建待办（暂不实现，预留） | N = New |

### 2.3 实现流程

```
应用启动 (lib.rs setup)
  │
  ├─1. 注册全局热键
  │   app.handle().plugin(
  │       tauri_plugin_global_shortcut::Builder::new()
  │           .with_handler(|app, shortcut, event| {
  │               match event.state() {
  │                   ShortcutState::Pressed => {
  │                       handle_shortcut_event(app, shortcut)
  │                   }
  │                   _ => {}
  │               }
  │           })
  │           .build()
  │   )
  │
  ├─2. 逐个注册热键
  │   plugin.register("Ctrl+Shift+D", move |app, shortcut, event| { ... })
  │   plugin.register("Ctrl+Shift+T", move |app, shortcut, event| { ... })
  │
  └─3. 热键处理函数
      handle_shortcut_event() → emit Tauri 事件 → 前端监听

事件流：
  Rust 热键回调 → app.emit("floating:toggle") → 前端 FloatingWindow 监听
```

**关键实现细节**：

1. **跨窗口通信**：热键处理器通过 `app.emit_to("floating", "floating:toggle")` 发送事件给悬浮窗，悬浮窗前端监听该事件执行显隐切换。

2. **主窗口内也可监听**：如果用户在主窗口中按热键，emit 到主窗口，由主窗口通过 Rust 侧命令切换悬浮窗显隐。

3. **循环透明度**：前端维护透明度档位 `[0.85, 0.6, 0.4]`，每次按 `Ctrl+Shift+T` 前进一档，通过 `window.document.documentElement.style.setProperty` 即时更新。

### 2.4 热键冲突检测

注册时使用 `tauri-plugin-global-shortcut` 的 `is_registered()` 检查：
- 如果已被其他应用占用 → 注册失败 → 前端显示警告通知
- 如果本项目未注册 → 正常注册

**前端设置页面**（后续 D-18 实现）中提供热键修改入口。

### 2.5 文件结构

```
src-tauri/src/
├── lib.rs                   # setup 中注册 global-shortcut 插件
└── floating_window.rs       # 新增：热键注册 + 事件发射逻辑

src/stores/
└── settingsStore.ts         # 新增 hotkey 相关状态

src/hooks/
└── useHotkey.ts             # 已存在，需完善热键事件监听
```

---

## 3. 验证标准 (Verify)

### 功能验证

| 测试场景 | 预期结果 | 验证方法 |
|----------|----------|----------|
| 注册热键 | 启动应用，热键成功注册，无错误日志 | 检查控制台/日志 |
| 悬浮窗显隐 | 按 `Ctrl+Shift+D` → 悬浮窗切换显隐 | 反复按热键 |
| 悬浮窗隐藏时热键 | 隐藏态按热键 → 悬浮窗滑出显示 | 按热键后观察 |
| 悬浮窗显示时热键 | 显示态按热键 → 悬浮窗滑回隐藏 | 按热键后观察 |
| 透明度循环 | 按 `Ctrl+Shift+T` → 85% → 60% → 40% → 85% | 反复按热键，目视透明度变化 |
| 其他应用焦点 | 焦点在 VS Code 时按热键 → 悬浮窗仍然响应 | 切换到 VS Code → 按热键 |
| 热键持久化 | 重启应用 → 热键仍生效 | 重启 → 按热键 |

### 交互体验验证

| 场景 | 预期 |
|------|------|
| 响应速度 | 按热键后 200ms 内悬浮窗开始动画 |
| 连续按键 | 快速连续按 5 次不崩溃、不重复触发 |
| 热键+鼠标联动 | 热键显示悬浮窗后，鼠标离开仍按自动隐藏逻辑（D-14） |

### 技术验证

```bash
# 确认插件可用
cargo check                         # 编译通过
grep "tauri-plugin-global-shortcut" Cargo.toml  # 依赖已添加
npx tauri dev -- --features global-shortcut    # 手动 E2E
```
