// 悬浮窗主组件
//
// 职责：
// 1. 渲染悬浮窗 UI（340×560，无边框透明窗口内）
// 2. 监听 Tauri 事件（floating:toggle 等）
// 3. 管理悬浮窗显示/隐藏状态
// 4. 用 Framer Motion 驱动滑入/滑出动画（D-14）
// 5. 渲染内容区域（FloatingContent）
//
// 设计文档：D-13 悬浮窗窗口创建与停靠、D-14 悬浮窗交互体验
// 测试计划：10-floating-interaction.md

import { useEffect, useState, useCallback, useRef } from 'react'
import { motion } from 'framer-motion'
import { listen } from '@tauri-apps/api/event'
import FloatingContent from './FloatingContent'
import FloatingMiniSettings from './FloatingMiniSettings'

const SLIDE_DISTANCE = 316 // 窗口宽 340px，滑出时留 24px 边缘可见（配合 Rust HIDDEN_EDGE_PX）
const HIDE_DELAY = 3000   // 离开后自动隐藏延迟 (ms)

/**
 * 悬浮窗主容器
 *
 * 渲染在 /floating 路由下（Tauri 多窗口的独立 WebView）。
 * 窗口本身的配置（无边框、置顶、透明）由 Rust 侧 floating_window.rs 完成。
 * 滑入/滑出动画由 Framer Motion 驱动（前端 CSS 动画，非 Rust 窗口位移动画）。
 */
export default function FloatingWindow() {
  const [visible, setVisible] = useState(true) // 初始可见，让用户看到悬浮窗
  const [showSettings, setShowSettings] = useState(false)
  const [opacity, setOpacity] = useState(0.85)

  // 用 ref 跟踪 visible 状态，供 mouse 事件 handler 读取最新值
  const visibleRef = useRef(visible)
  useEffect(() => { visibleRef.current = visible }, [visible])

  // 监听 Rust 侧发来的事件（系统托盘、全局热键触发）
  useEffect(() => {
    const p1 = listen('floating:toggle', () => {
      console.log('[Floating] 收到 floating:toggle 事件')
      setVisible((v) => !v)
    })
    const p2 = listen('floating:show', () => {
      console.log('[Floating] 收到 floating:show 事件（强制显示）')
      setVisible(true)
    })
    const p3 = listen('floating:cycle_transparency', () => {
      console.log('[Floating] 收到 floating:cycle_transparency 事件')
      setOpacity((o) => {
        if (o >= 0.85) return 0.6
        if (o >= 0.6) return 0.4
        return 0.85
      })
    })

    return () => {
      p1.then((u) => u())
      p2.then((u) => u())
      p3.then((u) => u())
    }
  }, [])

  // 显隐状态同步到窗口位置（通过 Tauri IPC 调用 Rust 命令）
  useEffect(() => {
    console.log('[Floating] visible 变化: %s, 同步窗口位置...', visible)
    import('@tauri-apps/api/core')
      .then(({ invoke }) => invoke('set_floating_position', { visible }))
      .catch((err) => console.error('[Floating] 切换位置失败:', err))
  }, [visible])

  // 透明度变化 → 更新 CSS 变量
  useEffect(() => {
    document.documentElement.style.setProperty('--floating-opacity', String(opacity))
  }, [opacity])

  // 自动隐藏计时器（D-14）
  // 关键修复：用 ref 追踪指针状态，避免 useCallback 闭包过期值
  const hideTimerRef = useRef<number | null>(null)
  const pointerInsideRef = useRef(false)

  const resetHideTimer = useCallback(() => {
    if (hideTimerRef.current) {
      clearTimeout(hideTimerRef.current)
      hideTimerRef.current = null
    }
    // 每个计时器创建时检查当前 ref 值
    hideTimerRef.current = window.setTimeout(() => {
      // 延迟后再次检查 ref 值（保证拿到最新状态）
      if (!pointerInsideRef.current) {
        console.log('[Floating] 自动隐藏超时，隐藏悬浮窗')
        setVisible(false)
      }
    }, HIDE_DELAY)
  }, []) // 无依赖，通过 ref 读取最新值

  // 窗口显示时启动自动隐藏计时器
  useEffect(() => {
    if (visible) {
      resetHideTimer()
    }
    return () => {
      if (hideTimerRef.current) {
        clearTimeout(hideTimerRef.current)
      }
    }
  }, [visible, resetHideTimer])

  // 指针进入窗口 → 取消隐藏计时器 + 隐藏状态时自动弹出
  const handlePointerEnter = useCallback(() => {
    pointerInsideRef.current = true
    if (hideTimerRef.current) {
      clearTimeout(hideTimerRef.current)
      hideTimerRef.current = null
    }
    // 如果悬浮窗处于隐藏状态，鼠标滑入边缘时自动弹出
    if (!visibleRef.current) {
      console.log('[Floating] 边缘触发，显示悬浮窗')
      setVisible(true)
    }
  }, [])

  // 指针离开窗口 → 重启隐藏计时器
  const handlePointerLeave = useCallback(() => {
    pointerInsideRef.current = false
    resetHideTimer()
  }, [resetHideTimer])

  return (
    <div
      className="floating-window relative"
      style={{
        opacity,
        width: '100%',
        height: '100%',
        overflow: 'hidden',
        // 关键：透明窗口需要非透明背景才能接收鼠标事件（hit-testing）
        background: 'rgba(0, 0, 0, 0.01)',
      }}
      onMouseEnter={handlePointerEnter}
      onMouseLeave={handlePointerLeave}
    >
      {/* Framer Motion 动画容器：x 轴滑动 */}
      <motion.div
        className="h-full flex flex-col"
        initial={{ x: SLIDE_DISTANCE }}
        animate={{ x: visible ? 0 : SLIDE_DISTANCE }}
        transition={{
          type: 'spring',
          stiffness: 300,
          damping: 30,
        }}
      >
        {/* 顶部操作栏 */}
        <div className="flex items-center justify-between px-3 py-1.5 border-b border-white/10 flex-shrink-0">
          <span className="text-xs text-muted-foreground select-none">
            DateCalendar
          </span>
          <button
            className="text-xs text-muted-foreground hover:text-foreground p-0.5 rounded"
            onClick={() => setShowSettings((v) => !v)}
            title="设置"
          >
            ⚙
          </button>
        </div>

        {/* 迷你设置面板（条件显示） */}
        {showSettings && (
          <FloatingMiniSettings
            opacity={opacity}
            onOpacityChange={setOpacity}
            onClose={() => setShowSettings(false)}
          />
        )}

        {/* 主内容区域（可滚动） */}
        <div className="flex-1 overflow-y-auto">
          <FloatingContent />
        </div>

        {/* 底部关闭按钮 */}
        <div className="flex-shrink-0 p-2 flex justify-end border-t border-white/10">
          <button
            className="text-xs text-muted-foreground hover:text-foreground"
            onClick={() => setVisible(false)}
          >
            □ 隐藏
          </button>
        </div>
      </motion.div>
    </div>
  )
}
