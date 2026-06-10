import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { initAdapter } from './adapters'

// 先渲染 App（不等待适配层），避免白屏
createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)

// 异步初始化适配层
initAdapter()
  .then((mode) => console.log(`[DateCalendar] Running in ${mode} mode`))
  .catch((err) => console.error('[DateCalendar] Adapter init failed:', err))

