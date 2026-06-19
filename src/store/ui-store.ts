import { create } from 'zustand'
import { devtools } from 'zustand/middleware'

interface UIState {
  commandPaletteOpen: boolean
  preferencesOpen: boolean

  toggleCommandPalette: () => void
  setCommandPaletteOpen: (open: boolean) => void
  togglePreferences: () => void
  setPreferencesOpen: (open: boolean) => void
}

export const useUIStore = create<UIState>()(
  devtools(
    set => ({
      commandPaletteOpen: false,
      preferencesOpen: false,

      toggleCommandPalette: () =>
        set(
          state => ({ commandPaletteOpen: !state.commandPaletteOpen }),
          undefined,
          'toggleCommandPalette'
        ),

      setCommandPaletteOpen: open =>
        set({ commandPaletteOpen: open }, undefined, 'setCommandPaletteOpen'),

      togglePreferences: () =>
        set(
          state => ({ preferencesOpen: !state.preferencesOpen }),
          undefined,
          'togglePreferences'
        ),

      setPreferencesOpen: open =>
        set({ preferencesOpen: open }, undefined, 'setPreferencesOpen'),
    }),
    {
      name: 'ui-store',
    }
  )
)
