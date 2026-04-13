/**
 * NavidromeProvider — wraps Subsonic API calls behind the MusicProvider interface.
 *
 * Conservative capabilities: only features that are fully wired are enabled.
 */

import type {
  MusicProvider,
  ProviderCapabilities,
  TrackPlaybackInfo,
} from "../../providers/types"
import type {
  MusicTrack,
  MusicAlbum,
  MusicArtist,
  MusicPlaylist,
  MusicItem,
  MusicHub,
  PagedResult,
} from "../../types/music"

import * as api from "./api"
import {
  subsonicSongToMusicTrack,
  subsonicAlbumToMusicAlbum,
  subsonicArtistToMusicArtist,
  subsonicPlaylistToMusicPlaylist,
  subsonicSearchToMusicItems,
  type ImgResolver,
} from "./mappers"
import { buildNavidromeImageUrl } from "./imageUrl"

export class NavidromeProvider implements MusicProvider {
  readonly name = "Navidrome"
  readonly capabilities: ProviderCapabilities = {
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
  }

  private _connected = false

  /**
   * Cached cover art URL map: coverArtId → full URL.
   * We resolve these lazily and cache to avoid repeated Tauri invokes.
   */
  private _coverArtCache = new Map<string, string>()

  /** Image resolver bound to current connection. */
  private img: ImgResolver = () => null

  // ---------------------------------------------------------------------------
  // Connection
  // ---------------------------------------------------------------------------

  async connect(config: Record<string, unknown>): Promise<void> {
    // Connection is already established by the connectionStore calling
    // subsonicConnect. We just need to build the image resolver.
    this._coverArtCache.clear()
    this.img = this._buildImgResolver()
    this._connected = true
  }

  async disconnect(): Promise<void> {
    this._connected = false
    this._coverArtCache.clear()
  }

  isConnected(): boolean {
    return this._connected
  }

  // ---------------------------------------------------------------------------
  // Image resolver
  // ---------------------------------------------------------------------------

  private _buildImgResolver(): ImgResolver {
    const cache = this._coverArtCache
    return (coverArtId, entityType, entityId, name, artist) => {
      if (!coverArtId) return null

      // Check cache first
      const cached = cache.get(coverArtId)
      if (cached) {
        return buildNavidromeImageUrl(cached, entityType, entityId, name, artist)
      }

      // Resolve async and cache — first call will miss, but the URL
      // is deterministic so we can construct it synchronously.
      // We eagerly populate the cache in the background.
      void api.subsonicGetCoverArtUrl(coverArtId).then(url => {
        cache.set(coverArtId, url)
      })

      // For the initial render, construct a placeholder that will be resolved
      // by the image protocol handler. We use the cover art endpoint directly.
      // The Rust image handler will fetch this URL.
      const placeholderUrl = cache.get(coverArtId)
      if (placeholderUrl) {
        return buildNavidromeImageUrl(placeholderUrl, entityType, entityId, name, artist)
      }

      // We need a synchronous URL. Eagerly resolve it.
      // For now, return null — the cache will be populated on next access.
      return null
    }
  }

  /**
   * Pre-resolve a cover art URL and return it.
   * Used when we need the URL synchronously in mappers.
   */
  private async resolveCoverArt(coverArtId: string | null): Promise<string | null> {
    if (!coverArtId) return null
    const cached = this._coverArtCache.get(coverArtId)
    if (cached) return cached
    const url = await api.subsonicGetCoverArtUrl(coverArtId)
    this._coverArtCache.set(coverArtId, url)
    return url
  }

  /**
   * Build an ImgResolver that has pre-resolved cover art URLs.
   * Call this after pre-fetching cover art URLs for a batch of items.
   */
  private imgWithCache(): ImgResolver {
    const cache = this._coverArtCache
    return (coverArtId, entityType, entityId, name, artist) => {
      if (!coverArtId) return null
      const url = cache.get(coverArtId)
      if (!url) return null
      return buildNavidromeImageUrl(url, entityType, entityId, name, artist)
    }
  }

  /**
   * Pre-resolve cover art URLs for a batch of cover art IDs.
   */
  private async preResolveCoverArts(ids: (string | null | undefined)[]): Promise<void> {
    const unique = [...new Set(ids.filter((id): id is string => !!id && !this._coverArtCache.has(id)))]
    await Promise.all(unique.map(id => this.resolveCoverArt(id)))
  }

  // ---------------------------------------------------------------------------
  // Browse
  // ---------------------------------------------------------------------------

  async search(query: string, type?: "track" | "album" | "artist"): Promise<MusicItem[]> {
    const ac = type === "artist" || !type ? 20 : 0
    const alc = type === "album" || !type ? 20 : 0
    const sc = type === "track" || !type ? 20 : 0
    const result = await api.subsonicSearch(query, ac, alc, sc)

    // Pre-resolve cover arts
    const coverIds = [
      ...result.artist.map(a => a.coverArt),
      ...result.album.map(a => a.coverArt),
      ...result.song.map(s => s.coverArt),
    ]
    await this.preResolveCoverArts(coverIds)

    return subsonicSearchToMusicItems(result.artist, result.album, result.song, this.imgWithCache())
  }

  async getRecentlyAdded(_type?: string, limit?: number): Promise<MusicItem[]> {
    const albums = await api.subsonicGetAlbumList("newest", limit ?? 50)
    await this.preResolveCoverArts(albums.map(a => a.coverArt))
    const img = this.imgWithCache()
    return albums.map(a => ({ ...subsonicAlbumToMusicAlbum(a, img), type: "album" as const }))
  }

  async getHubs(): Promise<MusicHub[]> {
    // Navidrome doesn't have hubs — return empty.
    // Capability is set to false, so this shouldn't be called.
    return []
  }

  // ---------------------------------------------------------------------------
  // Library
  // ---------------------------------------------------------------------------

  async getTrack(id: string): Promise<MusicTrack> {
    const song = await api.subsonicGetSong(id)
    await this.preResolveCoverArts([song.coverArt])
    return subsonicSongToMusicTrack(song, this.imgWithCache())
  }

  async getAlbum(id: string): Promise<MusicAlbum> {
    const detail = await api.subsonicGetAlbum(id)
    await this.preResolveCoverArts([detail.coverArt])
    return subsonicAlbumToMusicAlbum(detail, this.imgWithCache())
  }

  async getArtist(id: string): Promise<MusicArtist> {
    const detail = await api.subsonicGetArtist(id)
    await this.preResolveCoverArts([detail.coverArt])
    return subsonicArtistToMusicArtist(detail, this.imgWithCache())
  }

  async getAlbumTracks(albumId: string): Promise<MusicTrack[]> {
    const detail = await api.subsonicGetAlbum(albumId)
    await this.preResolveCoverArts(detail.song.map(s => s.coverArt))
    const img = this.imgWithCache()
    return detail.song.map(s => subsonicSongToMusicTrack(s, img))
  }

  async getArtistAlbums(artistId: string, _formatFilter?: string): Promise<MusicAlbum[]> {
    const detail = await api.subsonicGetArtist(artistId)
    await this.preResolveCoverArts(detail.album.map(a => a.coverArt))
    const img = this.imgWithCache()
    return detail.album.map(a => subsonicAlbumToMusicAlbum(a, img))
  }

  async getArtistPopularTracks(artistId: string, limit?: number): Promise<MusicTrack[]> {
    // getTopSongs requires the artist name, not ID. Fetch artist first.
    const artist = await api.subsonicGetArtist(artistId)
    const songs = await api.subsonicGetTopSongs(artist.name, limit ?? 10)
    await this.preResolveCoverArts(songs.map(s => s.coverArt))
    const img = this.imgWithCache()
    return songs.map(s => subsonicSongToMusicTrack(s, img))
  }

  async getArtistSimilar(_artistId: string): Promise<MusicArtist[]> {
    // Navidrome's getSimilarSongs2 returns songs, not artists.
    // Without a dedicated similar-artists endpoint, return empty.
    return []
  }

  async getRelatedHubs(_itemId: string): Promise<MusicHub[]> {
    return []
  }

  // ---------------------------------------------------------------------------
  // Playlists
  // ---------------------------------------------------------------------------

  async getPlaylists(): Promise<MusicPlaylist[]> {
    const playlists = await api.subsonicGetPlaylists()
    await this.preResolveCoverArts(playlists.map(p => p.coverArt))
    const img = this.imgWithCache()
    return playlists.map(p => subsonicPlaylistToMusicPlaylist(p, img))
  }

  async getPlaylistItems(playlistId: string, offset?: number, limit?: number): Promise<PagedResult<MusicTrack>> {
    const detail = await api.subsonicGetPlaylist(playlistId)
    await this.preResolveCoverArts(detail.entry.map(s => s.coverArt))
    const img = this.imgWithCache()
    let tracks = detail.entry.map((s, i) => {
      const t = subsonicSongToMusicTrack(s, img)
      // Use index as playlist item ID for removal (Subsonic uses index-based removal)
      t.playlistItemId = String(i)
      return t
    })

    // Client-side pagination
    const total = tracks.length
    if (offset) tracks = tracks.slice(offset)
    if (limit) tracks = tracks.slice(0, limit)

    return { items: tracks, total }
  }

  async createPlaylist(title: string, itemIds: string[]): Promise<MusicPlaylist> {
    const p = await api.subsonicCreatePlaylist(title, itemIds)
    await this.preResolveCoverArts([p.coverArt])
    return subsonicPlaylistToMusicPlaylist(p, this.imgWithCache())
  }

  async addToPlaylist(playlistId: string, itemIds: string[]): Promise<void> {
    await api.subsonicUpdatePlaylist(playlistId, null, itemIds)
  }

  async removeFromPlaylist(playlistId: string, playlistItemIds: string[]): Promise<void> {
    // Subsonic uses index-based removal. playlistItemIds are the stringified indexes.
    const indexes = playlistItemIds.map(Number).sort((a, b) => b - a) // Remove from end first
    await api.subsonicUpdatePlaylist(playlistId, null, null, indexes)
  }

  async deletePlaylist(playlistId: string): Promise<void> {
    await api.subsonicDeletePlaylist(playlistId)
  }

  async editPlaylist(playlistId: string, title?: string, _summary?: string): Promise<void> {
    if (title) {
      await api.subsonicUpdatePlaylist(playlistId, title)
    }
  }

  async movePlaylistItem(_playlistId: string, _itemId: string, _afterItemId: string): Promise<void> {
    // Subsonic doesn't support reordering individual items.
    // Would need to rebuild the entire playlist. Skip for now.
    throw new Error("Playlist reordering is not supported by Navidrome")
  }

  // ---------------------------------------------------------------------------
  // Liked
  // ---------------------------------------------------------------------------

  async getLikedTracks(_limit?: number): Promise<MusicTrack[]> {
    const starred = await api.subsonicGetStarred()
    await this.preResolveCoverArts(starred.song.map(s => s.coverArt))
    const img = this.imgWithCache()
    return starred.song.map(s => subsonicSongToMusicTrack(s, img))
  }

  async getLikedAlbums(_limit?: number): Promise<MusicAlbum[]> {
    const starred = await api.subsonicGetStarred()
    await this.preResolveCoverArts(starred.album.map(a => a.coverArt))
    const img = this.imgWithCache()
    return starred.album.map(a => subsonicAlbumToMusicAlbum(a, img))
  }

  async getLikedArtists(_limit?: number): Promise<MusicArtist[]> {
    const starred = await api.subsonicGetStarred()
    await this.preResolveCoverArts(starred.artist.map(a => a.coverArt))
    const img = this.imgWithCache()
    return starred.artist.map(a => subsonicArtistToMusicArtist(a, img))
  }

  // ---------------------------------------------------------------------------
  // Playback
  // ---------------------------------------------------------------------------

  async getStreamUrl(track: MusicTrack): Promise<string> {
    return api.subsonicGetStreamUrl(track.id)
  }

  async getPlaybackInfo(track: MusicTrack): Promise<TrackPlaybackInfo> {
    const url = await api.subsonicGetStreamUrl(track.id)
    return {
      url,
      trackKey: hashStringToNumber(track.id),
      partId: 0,
      parentKey: track.albumId ?? "",
      startRamp: null,
      endRamp: null,
    }
  }

  async rate(itemId: string, rating: number | null): Promise<void> {
    if (rating === null || rating === 0) {
      await api.subsonicUnstar(itemId)
      await api.subsonicSetRating(itemId, 0)
    } else {
      // Convert 0-10 scale to 1-5 for Subsonic
      const subsonicRating = Math.max(1, Math.min(5, Math.round(rating / 2)))
      await api.subsonicSetRating(itemId, subsonicRating)
      if (rating >= 6) {
        await api.subsonicStar(itemId)
      }
    }
  }

  async markPlayed(trackId: string): Promise<void> {
    await api.subsonicScrobble(trackId, true)
  }

  async reportProgress(_trackId: string, _positionMs: number, _state: string, _duration: number): Promise<void> {
    // Subsonic doesn't have timeline reporting. Scrobble on track end handles it.
  }

  // ---------------------------------------------------------------------------
  // Tags (genres)
  // ---------------------------------------------------------------------------

  async getTags(_tagType: string): Promise<{ tag: string; count: number | null }[]> {
    const genres = await api.subsonicGetGenres()
    return genres.map(g => ({ tag: g.value, count: g.albumCount }))
  }

  // ---------------------------------------------------------------------------
  // Browsing (all artists/albums/tracks)
  // ---------------------------------------------------------------------------

  async getAllArtists(_offset?: number, _limit?: number): Promise<PagedResult<MusicArtist>> {
    const artists = await api.subsonicGetArtists()
    await this.preResolveCoverArts(artists.map(a => a.coverArt))
    const img = this.imgWithCache()
    const mapped = artists.map(a => subsonicArtistToMusicArtist(a, img))
    return { items: mapped, total: mapped.length }
  }

  async getAllAlbums(offset?: number, limit?: number): Promise<PagedResult<MusicAlbum>> {
    const albums = await api.subsonicGetAlbumList("alphabeticalByName", limit ?? 500, offset ?? 0)
    await this.preResolveCoverArts(albums.map(a => a.coverArt))
    const img = this.imgWithCache()
    const mapped = albums.map(a => subsonicAlbumToMusicAlbum(a, img))
    return { items: mapped, total: mapped.length }
  }

  // ---------------------------------------------------------------------------
  // Now Playing — delegates to Tauri media session (same as Plex)
  // ---------------------------------------------------------------------------

  async updateNowPlaying(title: string, artist: string, album: string, thumbPath: string | null, durationMs: number): Promise<void> {
    const { invoke } = await import("@tauri-apps/api/core")
    await invoke("update_now_playing", { title, artist, album, thumbPath, durationMs })
  }

  async setNowPlayingState(state: "playing" | "paused" | "stopped", positionMs?: number): Promise<void> {
    const { invoke } = await import("@tauri-apps/api/core")
    await invoke("set_now_playing_state", { state, positionMs: positionMs ?? null })
  }

  // ---------------------------------------------------------------------------
  // Track lifecycle hooks
  // ---------------------------------------------------------------------------

  onTrackStart(track: MusicTrack): void {
    // Scrobble "now playing" (submission=false)
    void api.subsonicScrobble(track.id, false)
  }

  onTrackEnd(track: MusicTrack, _startedAtUnix: number, _listenedMs: number): void {
    // Scrobble completed play (submission=true)
    void api.subsonicScrobble(track.id, true)
  }
}

/** Simple string hash to produce a numeric key for the audio engine. */
function hashStringToNumber(s: string): number {
  let hash = 0
  for (let i = 0; i < s.length; i++) {
    const char = s.charCodeAt(i)
    hash = ((hash << 5) - hash) + char
    hash |= 0 // Convert to 32-bit int
  }
  return Math.abs(hash)
}
