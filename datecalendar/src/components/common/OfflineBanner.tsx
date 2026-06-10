/**
 * OfflineBanner — 离线模式提示组件
 * 当 Tauri 后端未启动时，显示黄色警告条提醒用户
 */
import { useState } from 'react';

interface OfflineBannerProps {
  /** 关闭回调 */
  onDismiss?: () => void;
}

export function OfflineBanner({ onDismiss }: OfflineBannerProps) {
  const [visible, setVisible] = useState(true);

  if (!visible) return null;

  const handleDismiss = () => {
    setVisible(false);
    onDismiss?.();
  };

  return (
    <div className="fixed bottom-0 left-0 right-0 z-50 animate-slide-up">
      <div className="flex items-center justify-between bg-amber-600/95 backdrop-blur-sm text-white px-4 py-2.5 text-sm shadow-lg">
        <div className="flex items-center gap-2">
          <svg className="w-4 h-4 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
          </svg>
          <span>
            <strong>离线模式</strong> — Tauri 后端未连接，数据仅保存在浏览器内存中，刷新页面将丢失。
          </span>
        </div>
        <button
          onClick={handleDismiss}
          className="ml-3 p-1 hover:bg-amber-500/50 rounded transition-colors shrink-0"
          aria-label="关闭提示"
        >
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
            <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    </div>
  );
}
