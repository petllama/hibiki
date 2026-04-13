/**
 * Backend registry — lists all available backend definitions.
 *
 * Components use this to enumerate backends (e.g. the Settings page)
 * without importing each backend directly.
 */

import type { BackendDefinition } from "./types"
import { plexBackend } from "./plex/definition"
import { navidromeBackend } from "./navidrome/definition"

export const backends: BackendDefinition[] = [plexBackend, navidromeBackend]

export function getBackend(id: string): BackendDefinition | undefined {
  return backends.find(b => b.id === id)
}
