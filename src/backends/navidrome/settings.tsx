/**
 * Navidrome connection settings UI.
 *
 * Simple form: server URL, username, password + connect button.
 */

import { useState } from "react"
import { useShallow } from "zustand/react/shallow"
import { invoke } from "@tauri-apps/api/core"
import { useNavidromeConnectionStore } from "./connectionStore"
import { useLibraryStore } from "../../stores/libraryStore"

interface FullSyncResult {
  artists: { synced: number; deleted: number }
  albums: { synced: number; deleted: number }
  tracks: { synced: number; deleted: number }
  elapsed_ms: number
}

type SyncState = "idle" | "syncing" | "success" | "error"

export function NavidromeSettings() {
  const {
    baseUrl: connectedUrl,
    username: connectedUser,
    isConnected,
    isLoading,
    error,
    connect,
    disconnect,
    clearError,
  } = useNavidromeConnectionStore(
    useShallow(s => ({
      baseUrl: s.baseUrl,
      username: s.username,
      isConnected: s.isConnected,
      isLoading: s.isLoading,
      error: s.error,
      connect: s.connect,
      disconnect: s.disconnect,
      clearError: s.clearError,
    })),
  )

  const [url, setUrl] = useState(connectedUrl || "http://localhost:4533")
  const [user, setUser] = useState(connectedUser || "")
  const [pass, setPass] = useState("")
  const [connecting, setConnecting] = useState(false)

  async function handleConnect(e: React.FormEvent) {
    e.preventDefault()
    if (!url || !user || !pass) return
    setConnecting(true)
    clearError()
    try {
      await connect(url.replace(/\/$/, ""), user, pass)
      // After successful connect, trigger library fetch
      const lib = useLibraryStore.getState()
      void Promise.all([
        lib.fetchPlaylists(),
        lib.fetchRecentlyAdded(50),
        lib.fetchTags(),
      ])
    } catch {
      // Error is set in the store
    } finally {
      setConnecting(false)
    }
  }

  if (isConnected) {
    return (
      <div className="space-y-4">
        <div className="rounded-xl border border-white/10 bg-white/[0.03] p-4">
          <div className="flex items-center justify-between">
            <div>
              <div className="flex items-center gap-2">
                <span className="h-2 w-2 rounded-full bg-green-400" />
                <span className="text-sm font-medium text-white">Connected</span>
              </div>
              <p className="mt-1 text-xs text-white/40">{connectedUrl}</p>
              <p className="text-xs text-white/40">User: {connectedUser}</p>
            </div>
            <button
              onClick={disconnect}
              className="rounded-lg border border-white/10 px-3 py-1.5 text-xs text-white/70 hover:border-white/20 hover:text-white transition-colors"
            >
              Disconnect
            </button>
          </div>
        </div>

        <SyncCard />
      </div>
    )
  }

  return (
    <form onSubmit={e => void handleConnect(e)} className="space-y-4">
      <div className="rounded-xl border border-white/10 bg-white/[0.03] p-4 space-y-3">
        <div>
          <label className="block text-xs font-medium text-white/50 mb-1">Server URL</label>
          <input
            type="url"
            value={url}
            onChange={e => setUrl(e.target.value)}
            placeholder="http://localhost:4533"
            className="w-full rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-white placeholder:text-white/20 focus:border-accent focus:outline-none"
          />
        </div>
        <div>
          <label className="block text-xs font-medium text-white/50 mb-1">Username</label>
          <input
            type="text"
            value={user}
            onChange={e => setUser(e.target.value)}
            placeholder="admin"
            autoComplete="username"
            className="w-full rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-white placeholder:text-white/20 focus:border-accent focus:outline-none"
          />
        </div>
        <div>
          <label className="block text-xs font-medium text-white/50 mb-1">Password</label>
          <input
            type="password"
            value={pass}
            onChange={e => setPass(e.target.value)}
            placeholder="••••••••"
            autoComplete="current-password"
            className="w-full rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-white placeholder:text-white/20 focus:border-accent focus:outline-none"
          />
        </div>
      </div>

      {error && (
        <div className="rounded-lg border border-red-500/20 bg-red-500/10 px-3 py-2 text-xs text-red-400">
          {error}
        </div>
      )}

      <button
        type="submit"
        disabled={connecting || isLoading || !url || !user || !pass}
        className="rounded-lg bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent/90 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
      >
        {connecting ? "Connecting..." : "Connect"}
      </button>
    </form>
  )
}

// ---------------------------------------------------------------------------
// Sync card — shown only when connected
// ---------------------------------------------------------------------------

function SyncCard() {
  const [syncState, setSyncState] = useState<SyncState>("idle")
  const [result, setResult] = useState<FullSyncResult | null>(null)
  const [syncError, setSyncError] = useState<string | null>(null)

  async function handleSync() {
    setSyncState("syncing")
    setResult(null)
    setSyncError(null)
    try {
      const res = await invoke<FullSyncResult>("sync_navidrome_full")
      setResult(res)
      setSyncState("success")
    } catch (err) {
      setSyncError(err instanceof Error ? err.message : String(err))
      setSyncState("error")
    }
  }

  return (
    <div className="rounded-xl border border-white/10 bg-white/[0.03] p-4 space-y-3">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium text-white">Library Sync</p>
          <p className="text-xs text-white/40 mt-0.5">
            Sync your Navidrome library to the local database.
          </p>
        </div>
        <button
          onClick={() => void handleSync()}
          disabled={syncState === "syncing"}
          className="rounded-lg border border-white/10 px-3 py-1.5 text-xs text-white/70 hover:border-white/20 hover:text-white transition-colors disabled:opacity-40 disabled:cursor-not-allowed flex-shrink-0"
        >
          {syncState === "syncing" ? "Syncing..." : "Sync Library"}
        </button>
      </div>

      {syncState === "success" && result && (
        <div className="rounded-lg border border-green-500/20 bg-green-500/10 px-3 py-2 text-xs text-green-400 space-y-1">
          <p className="font-medium">Sync complete</p>
          <p>{result.artists.synced} artists, {result.albums.synced} albums, {result.tracks.synced} tracks</p>
          <p className="text-green-400/60">{(result.elapsed_ms / 1000).toFixed(1)}s</p>
        </div>
      )}

      {syncState === "error" && syncError && (
        <div className="rounded-lg border border-red-500/20 bg-red-500/10 px-3 py-2 text-xs text-red-400">
          Sync failed: {syncError}
        </div>
      )}
    </div>
  )
}
