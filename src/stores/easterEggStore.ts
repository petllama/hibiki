import { create } from "zustand"
import { persist } from "zustand/middleware"

interface EasterEggState {
  unlocked: boolean
  rainbow: boolean
  partyMode: boolean
  vinylSpin: boolean
  vaporwave: boolean

  unlock: () => void
  toggleRainbow: () => void
  togglePartyMode: () => void
  toggleVinylSpin: () => void
  toggleVaporwave: () => void
}

export const useEasterEggStore = create<EasterEggState>()(
  persist(
    (set) => ({
      unlocked: false,
      rainbow: false,
      partyMode: false,
      vinylSpin: false,
      vaporwave: false,

      unlock: () => set({ unlocked: true }),
      toggleRainbow: () => set((s) => ({ rainbow: !s.rainbow })),
      togglePartyMode: () => set((s) => ({ partyMode: !s.partyMode })),
      toggleVinylSpin: () => set((s) => ({ vinylSpin: !s.vinylSpin })),
      toggleVaporwave: () => set((s) => ({ vaporwave: !s.vaporwave })),
    }),
    { name: "easter-eggs-v1" },
  ),
)

// Konami code listener: ↑ ↑ ↓ ↓ ← → ← → B A
const KONAMI = ["ArrowUp","ArrowUp","ArrowDown","ArrowDown","ArrowLeft","ArrowRight","ArrowLeft","ArrowRight","b","a"]
let konamiPos = 0

if (typeof window !== "undefined") {
  window.addEventListener("keydown", (e) => {
    if (e.key === KONAMI[konamiPos]) {
      konamiPos++
      if (konamiPos === KONAMI.length) {
        konamiPos = 0
        const store = useEasterEggStore.getState()
        if (!store.unlocked) {
          store.unlock()
          import("@tauri-apps/plugin-notification").then(({ sendNotification }) => {
            sendNotification({ title: "Easter Eggs Unlocked", body: "Check Settings for hidden features!" })
          })
        }
      }
    } else {
      konamiPos = e.key === KONAMI[0] ? 1 : 0
    }
  })
}
