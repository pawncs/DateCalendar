import { Moon, Sun, Calendar, ListTodo, Settings } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useSettingsStore } from '@/stores/settingsStore'
import { useUIStore } from '@/stores/uiStore'
import { TaskTree } from '@/components/tasks/TaskTree'
import { TaskEditor } from '@/components/tasks/TaskEditor'
import { CalendarView } from '@/components/calendar/CalendarView'

export function MainLayout() {
  const { theme, setTheme } = useSettingsStore()
  const { sidebarView, setSidebarView } = useUIStore()

  const toggleTheme = () => {
    setTheme(theme === 'dark' ? 'light' : 'dark')
  }

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
      {/* 左侧导航栏 */}
      <aside className="w-14 border-r border-border bg-sidebar flex flex-col items-center py-3 gap-2 shrink-0">
        <div className="mb-2">
          <Calendar className="size-6 text-primary" />
        </div>

        <Button
          variant={sidebarView === 'tasks' ? 'secondary' : 'ghost'}
          size="icon"
          className="size-9"
          onClick={() => setSidebarView('tasks')}
          title="任务"
        >
          <ListTodo className="size-4" />
        </Button>

        <Button
          variant={sidebarView === 'calendar' ? 'secondary' : 'ghost'}
          size="icon"
          className="size-9"
          onClick={() => setSidebarView('calendar')}
          title="日程"
        >
          <Calendar className="size-4" />
        </Button>

        <div className="flex-1" />

        <Button
          variant={sidebarView === 'settings' ? 'secondary' : 'ghost'}
          size="icon"
          className="size-9"
          onClick={() => setSidebarView('settings')}
          title="设置"
        >
          <Settings className="size-4" />
        </Button>

        <Button
          variant="ghost"
          size="icon"
          className="size-9"
          onClick={toggleTheme}
          title={theme === 'dark' ? '切换到亮色模式' : '切换到暗色模式'}
        >
          {theme === 'dark' ? <Sun className="size-4" /> : <Moon className="size-4" />}
        </Button>
      </aside>

      {/* 左侧面板：任务树 */}
      {sidebarView === 'tasks' && (
        <div className="w-72 border-r border-border bg-sidebar flex flex-col shrink-0">
          <TaskTree />
        </div>
      )}

      {/* 右侧：任务详情 / 日历视图 / 设置 */}
      <main className="flex-1 flex flex-col min-w-0">
        {sidebarView === 'tasks' && <TaskEditor />}
        {sidebarView === 'calendar' && <CalendarView />}
        {sidebarView === 'settings' && (
          <div className="flex items-center justify-center h-full text-muted-foreground text-sm">
            设置 — 即将实现
          </div>
        )}
      </main>
    </div>
  )
}
