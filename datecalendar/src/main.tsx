import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import FloatingWindow from './components/floating/FloatingWindow'
import { initAdapter } from './adapters'

// 根据路径决定渲染主窗口还是悬浮窗
// Tauri 多窗口：主窗口加载 "/" ，悬浮窗加载 "/floating"
const isFloating = window.location.pathname === '/floating'

// 先渲染对应组件（不等待适配层），避免白屏
createRoot(document.getElementById('root')!).render(
  <StrictMode>
    {isFloating ? <FloatingWindow /> : <App />}
  </StrictMode>,
)

// 异步初始化适配层
initAdapter()
  .then((mode) => console.log(`[DateCalendar] Running in ${mode} mode`))
  .catch((err) => console.error('[DateCalendar] Adapter init failed:', err))

