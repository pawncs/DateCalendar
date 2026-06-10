import { useUIStore } from '@/stores/uiStore'
import { DayView } from './DayView'
import { WeekView } from './WeekView'
import { TodoListView } from './TodoListView'

export function CalendarView() {
  const { calendarView } = useUIStore()

  switch (calendarView) {
    case 'day':
      return <DayView />
    case 'week':
      return <WeekView />
    case 'todo_list':
      return <TodoListView />
    default:
      return <WeekView />
  }
}
