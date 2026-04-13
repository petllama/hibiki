/**
 * Navidrome backend definition — registered in the backend registry.
 */

import type { BackendDefinition, BackendConnectionSnapshot } from "../types"
import { useNavidromeConnectionStore } from "./connectionStore"
import { NavidromeSettings } from "./settings"

function NavidromeIcon({ size = 18 }: { size?: number }) {
  return (
    <svg height={size} width={size} viewBox="0 0 24 24" fill="currentColor">
      <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm-1-13h2v6h-2zm0 8h2v2h-2z" />
      <circle cx="12" cy="12" r="3" />
      <path d="M12 7v4l3 3" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
    </svg>
  )
}

export const navidromeBackend: BackendDefinition = {
  id: "navidrome",
  name: "Navidrome",
  description: "Connect to a Navidrome server via the Subsonic API.",
  icon: NavidromeIcon,
  capabilities: {
    search: true,
    playlists: true,
    playlistEdit: true,
    ratings: true,
    radio: false,
    sonicSimilarity: false,
    djModes: false,
    playQueues: false,
    lyrics: false,
    streamLevels: false,
    hubs: false,
    stations: false,
    tags: true,
    scrobble: true,
    mixTracks: false,
    browseArtists: true,
    browseAlbums: true,
    browseTracks: true,
    syncArtists: false,
    syncAlbums: false,
    syncTracks: false,
  },
  SettingsComponent: NavidromeSettings,
  useIsConnected: () => useNavidromeConnectionStore(s => s.isConnected),
  loadAndConnect: () => useNavidromeConnectionStore.getState().loadAndConnect(),
  disconnectAndClear: () => useNavidromeConnectionStore.getState().disconnectAndClear(),
  getConnectionSnapshot: (): BackendConnectionSnapshot => {
    const s = useNavidromeConnectionStore.getState()
    return { isConnected: s.isConnected, isLoading: s.isLoading, libraryKey: s.libraryKey }
  },
  subscribeConnection: (listener) =>
    useNavidromeConnectionStore.subscribe((s) =>
      listener({ isConnected: s.isConnected, isLoading: s.isLoading, libraryKey: s.libraryKey })
    ),
}
