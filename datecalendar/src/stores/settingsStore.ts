import { create } from 'zustand'

export type Theme = 'dark' | 'light'

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
  theme: 'dark',
  sidebarWidth: 280,
  floatingOpacity: 85,
  floatingAutoHide: true,
  floatingAutoHideDelay: 3,
  hotkeyToggleFloating: 'Alt+Space',
  hotkeyToggleOpacity: 'Alt+O',

  setTheme: (theme) => {
    set({ theme })
    document.documentElement.classList.toggle('dark', theme === 'dark')
  },

  setSidebarWidth: (width) => set({ sidebarWidth: width }),
  setFloatingOpacity: (opacity) => set({ floatingOpacity: opacity }),
  setFloatingAutoHide: (autoHide) => set({ floatingAutoHide: autoHide }),

  setHotkey: (key, value) => {
    set({ [key]: value })
  },
}))
