import { create } from 'zustand'

export type SidebarView = 'tasks' | 'calendar' | 'settings'
export type CalendarViewType = 'day' | 'week' | 'todo_list'

interface UIState {
  sidebarView: SidebarView
  calendarView: CalendarViewType
  searchQuery: string
  isSearchOpen: boolean

  setSidebarView: (view: SidebarView) => void
  setCalendarView: (view: CalendarViewType) => void
  setSearchQuery: (query: string) => void
  toggleSearch: () => void
}

export const useUIStore = create<UIState>((set) => ({
  sidebarView: 'tasks',
  calendarView: 'week',
  searchQuery: '',
  isSearchOpen: false,

  setSidebarView: (view) => set({ sidebarView: view }),
  setCalendarView: (view) => set({ calendarView: view }),
  setSearchQuery: (query) => set({ searchQuery: query }),
  toggleSearch: () => set((s) => ({ isSearchOpen: !s.isSearchOpen })),
}))
