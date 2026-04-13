/**
 * Generic connection store — routes to the active backend's connection state.
 *
 * Components import this instead of a specific backend's store.
 * The active backend is persisted to localStorage so reconnects work on restart.
 */

import { create } from "zustand"
import { backends, getBackend } from "../backends/registry"
import type { BackendDefinition } from "../backends/types"

const ACTIVE_BACKEND_KEY = "hibiki-active-backend"

function loadActiveBackendId(): string | null {
  try { return localStorage.getItem(ACTIVE_BACKEND_KEY) } catch { return null }
}

function saveActiveBackendId(id: string | null) {
  try {
    if (id) localStorage.setItem(ACTIVE_BACKEND_KEY, id)
    else localStorage.removeItem(ACTIVE_BACKEND_KEY)
  } catch { /* ignore */ }
}

interface ConnectionState {
  /** Which backend is active (null = none selected yet). */
  activeBackendId: string | null
  /** Resolved BackendDefinition for the active backend. */
  activeBackend: BackendDefinition | null

  // Delegated state — synced from the active backend's connection store.
  isConnected: boolean
  isLoading: boolean
  /** Generic library identifier (Plex section ID as string, etc.). */
  libraryKey: string | null

  /** Load saved backend and connect. */
  loadAndConnect: () => Promise<void>
  /** Switch the active backend (disconnects current, saves new). */
  setActiveBackend: (id: string) => Promise<void>
  /** Disconnect and clear saved credentials for the active backend. */
  disconnectAndClear: () => Promise<void>
}

/** Unsubscribe handle for the current backend subscription. */
let _unsub: (() => void) | null = null

function subscribeToBackend(
  set: (partial: Partial<ConnectionState>) => void,
  backend: BackendDefinition,
) {
  if (_unsub) { _unsub(); _unsub = null }

  // Initial sync
  const snap = backend.getConnectionSnapshot()
  set({ isConnected: snap.isConnected, isLoading: snap.isLoading, libraryKey: snap.libraryKey })

  // Reactive sync
  _unsub = backend.subscribeConnection((s) => {
    set({ isConnected: s.isConnected, isLoading: s.isLoading, libraryKey: s.libraryKey })
  })
}

export const useConnectionStore = create<ConnectionState>((set, get) => ({
  activeBackendId: null,
  activeBackend: null,
  isConnected: false,
  isLoading: true,
  libraryKey: null,

  loadAndConnect: async () => {
    // If there's a saved backend, use it. Otherwise default to the first registered backend.
    const savedId = loadActiveBackendId()
    const id = savedId ?? (backends.length > 0 ? backends[0].id : null)
    if (!id) { set({ isLoading: false }); return }

    const backend = getBackend(id)
    if (!backend) { set({ isLoading: false }); return }

    set({ activeBackendId: id, activeBackend: backend, isLoading: true })
    saveActiveBackendId(id)

    // Subscribe before connecting so we catch state changes during connect
    subscribeToBackend(set, backend)

    try {
      await backend.loadAndConnect()
    } catch {
      // Backend's own store handles its error state; subscription syncs it
    }
  },

  setActiveBackend: async (id: string) => {
    const { activeBackend } = get()
    const newBackend = getBackend(id)
    if (!newBackend) return

    // Disconnect current if different
    if (activeBackend && activeBackend.id !== id) {
      if (_unsub) { _unsub(); _unsub = null }
      await activeBackend.disconnectAndClear()
    }

    set({ activeBackendId: id, activeBackend: newBackend, isConnected: false, libraryKey: null })
    saveActiveBackendId(id)
    subscribeToBackend(set, newBackend)
  },

  disconnectAndClear: async () => {
    const { activeBackend } = get()
    if (activeBackend) {
      await activeBackend.disconnectAndClear()
    }
    // Subscription will sync isConnected = false from the backend
  },
}))
