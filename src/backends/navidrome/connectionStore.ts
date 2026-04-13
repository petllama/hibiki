/**
 * Navidrome connection store — manages connection state for the Subsonic backend.
 */

import { create } from "zustand"
import { NavidromeProvider } from "./provider"
import { useProviderStore } from "../../stores/providerStore"
import { useLibraryStore } from "../../stores/libraryStore"
import { usePlayerStore } from "../../stores/playerStore"
import {
  subsonicConnect,
  subsonicLoadSettings,
  subsonicSaveSettings,
} from "./api"

interface NavidromeConnectionState {
  baseUrl: string
  username: string
  isConnected: boolean
  isLoading: boolean
  error: string | null
  /** Backend-agnostic library identifier. Navidrome has a single library. */
  libraryKey: string | null

  loadAndConnect: () => Promise<void>
  connect: (baseUrl: string, username: string, password: string) => Promise<void>
  disconnect: () => void
  disconnectAndClear: () => Promise<void>
  clearError: () => void
}

export const useNavidromeConnectionStore = create<NavidromeConnectionState>((set, get) => ({
  baseUrl: "",
  username: "",
  isConnected: false,
  isLoading: false,
  error: null,
  libraryKey: null,

  loadAndConnect: async () => {
    set({ isLoading: true, error: null })
    try {
      const settings = await subsonicLoadSettings()
      if (!settings.base_url || !settings.username || !settings.password) {
        set({ isLoading: false })
        return
      }
      await get().connect(settings.base_url, settings.username, settings.password)
    } catch (err) {
      set({ isLoading: false, error: String(err) })
    }
  },

  connect: async (baseUrl: string, username: string, password: string) => {
    set({ isLoading: true, error: null })
    try {
      // Connect via Rust (creates client, pings server)
      await subsonicConnect(baseUrl, username, password)
      // Save credentials
      await subsonicSaveSettings(baseUrl, username, password)

      // Create and register the provider
      const provider = new NavidromeProvider()
      await provider.connect({ baseUrl, username })
      useProviderStore.getState().setProvider(provider)

      set({
        baseUrl,
        username,
        isConnected: true,
        isLoading: false,
        error: null,
        libraryKey: "navidrome", // Navidrome has a single library
      })
    } catch (err) {
      set({ isLoading: false, error: String(err), isConnected: false })
      throw err
    }
  },

  disconnect: () => {
    useProviderStore.getState().clearProvider()
    useLibraryStore.getState().clearAll()
    usePlayerStore.getState().stop()
    set({
      baseUrl: "",
      username: "",
      isConnected: false,
      libraryKey: null,
    })
  },

  disconnectAndClear: async () => {
    try {
      await subsonicSaveSettings("", "", "")
    } catch (err) {
      console.error("Failed to clear Navidrome settings:", err)
    }
    get().disconnect()
  },

  clearError: () => set({ error: null }),
}))
