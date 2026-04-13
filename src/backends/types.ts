import type { ProviderCapabilities } from "../providers/types"
import type { ComponentType } from "react"

/** Minimal connection snapshot that the generic connection store syncs from. */
export interface BackendConnectionSnapshot {
  isConnected: boolean
  isLoading: boolean
  libraryKey: string | null
}

export interface BackendDefinition {
  id: string
  name: string
  description: string
  icon: ComponentType<{ size?: number }>
  capabilities: ProviderCapabilities
  SettingsComponent: ComponentType
  useIsConnected: () => boolean
  loadAndConnect: () => Promise<void>
  disconnectAndClear: () => Promise<void>
  /** Return current connection snapshot (non-reactive). */
  getConnectionSnapshot: () => BackendConnectionSnapshot
  /** Subscribe to connection state changes. Returns unsubscribe function. */
  subscribeConnection: (listener: (snap: BackendConnectionSnapshot) => void) => () => void
}
