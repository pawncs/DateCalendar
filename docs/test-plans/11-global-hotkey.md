# 测试流程：全局热键系统

> 对应设计文档：[D-15 全局热键系统](../design/D-15-global-hotkey.md)

## 前置条件

- 仅桌面模式：运行 `start.bat`（Tauri 桌面应用）
- 需 `tauri-plugin-global-shortcut` 插件已集成：
  ```bash
  grep "tauri-plugin-global-shortcut" datecalendar/src-tauri/Cargo.toml
  npm list @tauri-apps/plugin-global-shortcut
  ```
- 测试前关闭可能占用 `Ctrl+Shift+D` / `Ctrl+Shift+T` / `Ctrl+Shift+N` 的其他应用（如某些截图工具、翻译软件等）
- 全局热键为系统级特性，**仅支持手动测试**，无法用 Playwright 自动化

## 白盒测试（Rust 后端）

```bash
cd datecalendar/src-tauri
cargo test --lib test_hotkey_registration test_hotkey_handler test_shortcut_event_emission -- --nocapture
```

### 覆盖用例

| # | 用例 | 验证点 |
|---|------|--------|
| 1 | `test_hotkey_registration` | `Ctrl+Shift+D` 和 `Ctrl+Shift+T` 成功注册 |
| 2 | `test_hotkey_is_registered` | `is_registered()` 返回 true 对已注册热键 |
| 3 | `test_hotkey_handler_toggle` | 热键处理器正确 emit `floating:toggle` 事件 |
| 4 | `test_hotkey_handler_transparency` | 热键处理器正确 emit `floating:cycle_transparency` 事件 |
| 5 | `test_hotkey_unregister` | 应用退出时热键正确注销 |
| 6 | `test_shortcut_event_to_floating` | 事件能正确路由到 "floating" 窗口 |
| 7 | `test_shortcut_event_to_main` | 主窗口也能接收热键事件并触发悬浮窗变化 |

## 手动黑盒测试

> 所有用例均需在桌面环境手动执行。

### TC-01: 热键注册验证

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 启动应用，查看控制台/日志 | 无热键注册错误 |
| 2 | 检查是否有冲突提示 | 如果快捷键被占用，显示警告通知 |

### TC-02: 悬浮窗显隐热键（Ctrl+Shift+D）

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 悬浮窗隐藏状态 → 按 `Ctrl+Shift+D` | 悬浮窗从右侧滑出显示 |
| 2 | 悬浮窗显示状态 → 按 `Ctrl+Shift+D` | 悬浮窗滑回隐藏 |
| 3 | 连续按 5 次 | 每次都能正确切换，不卡顿不崩溃 |
| 4 | 快速连续按 | 不重复触发（去抖），切换状态正确 |

### TC-03: 透明度循环热键（Ctrl+Shift+T）

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 悬浮窗显示 → 按 `Ctrl+Shift+T` | 透明度变为 60% |
| 2 | 再按 `Ctrl+Shift+T` | 透明度变为 40% |
| 3 | 再按 `Ctrl+Shift+T` | 透明度变为 85%（循环回到第一档） |
| 4 | 悬浮窗隐藏时按 | 悬浮窗不显示，但透明度档位已切换 |
| 5 | 连续按 6 次 | 循环 2 圈，状态正确，不卡死 |

### TC-04: 全局响应（其他应用中）

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 打开 VS Code → 全屏 | 焦点在 VS Code |
| 2 | 按 `Ctrl+Shift+D` | 悬浮窗切换显隐（VS Code 不响应） |
| 3 | 打开浏览器全屏 | 焦点在浏览器 |
| 4 | 按 `Ctrl+Shift+T` | 透明度切换 |
| 5 | 打开 Excel/Word/其他应用 | 热键均可响应 |

### TC-05: 热键 + 鼠标联动

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 用热键显示悬浮窗 | 悬浮窗显示 |
| 2 | 鼠标离开悬浮窗 | 3 秒后自动隐藏（D-14 逻辑不受影响） |
| 3 | 用热键隐藏悬浮窗 | 悬浮窗立即隐藏，不等 3 秒 |
| 4 | 鼠标在触发区 + 按热键隐藏 | 热键优先，窗口隐藏 |

### TC-06: 热键持久化

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 退出应用（托盘 → 退出） | 完全退出 |
| 2 | 重新启动应用 | 热键重新注册 |
| 3 | 按 `Ctrl+Shift+D` | 热键仍然生效 |

### TC-07: 热键冲突检测

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 先用其他应用注册 `Ctrl+Shift+D` | 其他应用占用 |
| 2 | 启动 DateCalendar | 控制台日志显示注册失败 |
| 3 | 前端显示通知 | "全局热键 Ctrl+Shift+D 已被其他应用占用" |

### TC-08: 边界情况

| 步骤 | 操作 | 预期结果 |
|------|------|---------|
| 1 | 按住 `Ctrl+Shift` 不放 → 反复按 D | 每按一次切换一次 |
| 2 | 主窗口全屏 + 悬浮窗隐藏 → 按热键 | 悬浮窗正常显示 |
| 3 | 悬浮窗显示 + 按热键隐藏 + 立即再按热键显示 | 响应迅速，无延迟问题 |
| 4 | 系统休眠唤醒后 | 热键仍生效 |

## 交互体验验证

| 场景 | 预期 |
|------|------|
| 响应速度 | 按下热键后 200ms 内悬浮窗开始动画 |
| 连续操作 | 快速连续按 5 次不崩溃、不重复触发 |
| 与其他热键共存 | 能与系统热键（Alt+Tab, Win+D 等）共存，不冲突 |
| 多应用切换 | 在多个应用间切换后，热键始终有效 |

## 技术验证

```bash
cargo check                                      # 编译通过
grep "tauri-plugin-global-shortcut" Cargo.toml   # 确认依赖
npx tauri dev                                    # 手动 E2E：
                                                 #   1. 在 VS Code 中按 Ctrl+Shift+D
                                                 #   2. 悬浮窗切换
                                                 #   3. 在其他应用中也测试
                                                 #   4. Ctrl+Shift+T 循环透明度
```
