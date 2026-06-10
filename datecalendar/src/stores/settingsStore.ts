import { create } from 'zustand'

export type Theme = 'dark' | 'light'

// 初始化时同步 HTML class
const getInitialTheme = (): Theme => {
  const stored = localStorage.getItem('theme')
  if (stored === 'light' || stored === 'dark') {
    document.documentElement.classList.toggle('dark', stored === 'dark')
    return stored
  }
  // 默认跟随系统或默认暗色
  const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches
  document.documentElement.classList.toggle('dark', isDark)
  return isDark ? 'dark' : 'light'
}

interface SettingsState {
  theme: Theme
  sidebarWidth: number
  floatingOpacity: number
  floatingAutoHide: boolean
  floatingAutoHideDelay: number
  hotkeyToggleFloating: string
  hotkeyToggleOpacity: string

  setTheme: (theme: Theme) => void
  setSidebarWidth: (width: number) => void
  setFloatingOpacity: (opacity: number) => void
  setFloatingAutoHide: (autoHide: boolean) => void
  setHotkey: (key: 'hotkeyToggleFloating' | 'hotkeyToggleOpacity', value: string) => void
}

export const useSettingsStore = create<SettingsState>((set) => ({
  theme: getInitialTheme(),
  sidebarWidth: 280,
  floatingOpacity: 85,
  floatingAutoHide: true,
  floatingAutoHideDelay: 3,
  hotkeyToggleFloating: 'Alt+Space',
  hotkeyToggleOpacity: 'Alt+O',

  setTheme: (theme) => {
    set({ theme })
    document.documentElement.classList.toggle('dark', theme === 'dark')
    localStorage.setItem('theme', theme)
  },

  setSidebarWidth: (width) => set({ sidebarWidth: width }),
  setFloatingOpacity: (opacity) => set({ floatingOpacity: opacity }),
  setFloatingAutoHide: (autoHide) => set({ floatingAutoHide: autoHide }),

  setHotkey: (key, value) => {
    set({ [key]: value })
  },
}))
