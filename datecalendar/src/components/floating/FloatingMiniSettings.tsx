// 悬浮窗迷你设置面板
//
// 职责：
// 1. 透明度滑块（20% - 100%）
// 2. 自动隐藏延迟设置（秒）
// 3. 边缘触发区宽度设置（px）
//
// 设计文档：D-14 悬浮窗交互体验

import { useEffect, useState } from 'react'

interface Props {
  opacity: number
  onOpacityChange: (v: number) => void
  onClose: () => void
}

/**
 * 迷你设置面板
 *
 * 点击悬浮窗顶部的 ⚙ 图标弹出。
 * 设置项存入 settingsStore（后续实现持久化）。
 */
export default function FloatingMiniSettings({ opacity, onOpacityChange, onClose }: Props) {
  const [autoHideMs, setAutoHideMs] = useState(3000)
  const [triggerZone, setTriggerZone] = useState(20)

  return (
    <div className="floating-settings absolute top-7 right-2 w-44 bg-gray-900/95 border border-white/10 rounded-lg p-2.5 space-y-2.5 shadow-xl z-50 text-xs">
      <div className="flex items-center justify-between">
        <span className="text-muted-foreground/80 text-xs">设置</span>
        <button onClick={onClose} className="text-muted-foreground/60 hover:text-foreground text-xs">✕</button>
      </div>

      {/* 透明度 */}
      <label className="block space-y-1">
        <span className="text-muted-foreground/70 text-xs">
          透明度: {Math.round(opacity * 100)}%
        </span>
        <input
          type="range"
          min={0.2}
          max={1.0}
          step={0.05}
          value={opacity}
          onChange={e => onOpacityChange(parseFloat(e.target.value))}
          className="w-full accent-blue-400"
        />
      </label>

      {/* 自动隐藏延迟 */}
      <label className="block space-y-1">
        <span className="text-muted-foreground/70 text-xs">
          自动隐藏: {autoHideMs / 1000}秒
        </span>
        <input
          type="range"
          min={1000}
          max={30000}
          step={1000}
          value={autoHideMs}
          onChange={e => setAutoHideMs(parseInt(e.target.value))}
          className="w-full accent-blue-400"
        />
      </label>

      {/* 边缘触发区 */}
      <label className="block space-y-1">
        <span className="text-muted-foreground/70 text-xs">
          触发区: {triggerZone}px
        </span>
        <input
          type="range"
          min={5}
          max={100}
          step={5}
          value={triggerZone}
          onChange={e => setTriggerZone(parseInt(e.target.value))}
          className="w-full accent-blue-400"
        />
      </label>
    </div>
  )
}
