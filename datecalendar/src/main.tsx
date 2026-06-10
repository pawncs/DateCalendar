import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { initAdapter } from './adapters'

async function bootstrap() {
  // 初始化适配层：检测环境 → 选择后端（tauri/http/sqljs）
  const mode = await initAdapter()
  console.log(`[DateCalendar] Running in ${mode} mode`)

  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <App />
    </StrictMode>,
  )
}

bootstrap()

