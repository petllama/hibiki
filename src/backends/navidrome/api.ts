/**
 * Tauri invoke wrappers for the Subsonic backend.
 *
 * Each function maps 1:1 to a `subsonic_*` Tauri command.
 */

import { invoke } from "@tauri-apps/api/core"
import type {
  SubsonicArtist,
  ArtistDetail,
  AlbumDetail,
  SubsonicAlbum,
  SubsonicSong,
  SearchResult3,
  SubsonicPlaylist,
  PlaylistDetail,
  Starred2,
  SubsonicGenre,
  SubsonicSettings,
} from "./types"

// Connection
export function subsonicConnect(baseUrl: string, username: string, password: string): Promise<void> {
  return invoke("subsonic_connect", { baseUrl, username, password })
}

export function subsonicPing(): Promise<void> {
  return invoke("subsonic_ping")
}

// Library
export function subsonicGetArtists(): Promise<SubsonicArtist[]> {
  return invoke("subsonic_get_artists")
}

export function subsonicGetArtist(id: string): Promise<ArtistDetail> {
  return invoke("subsonic_get_artist", { id })
}

export function subsonicGetAlbum(id: string): Promise<AlbumDetail> {
  return invoke("subsonic_get_album", { id })
}

export function subsonicGetSong(id: string): Promise<SubsonicSong> {
  return invoke("subsonic_get_song", { id })
}

export function subsonicSearch(
  query: string,
  artistCount?: number | null,
  albumCount?: number | null,
  songCount?: number | null,
): Promise<SearchResult3> {
  return invoke("subsonic_search", {
    query,
    artistCount: artistCount ?? null,
    albumCount: albumCount ?? null,
    songCount: songCount ?? null,
  })
}

export function subsonicGetAlbumList(
  listType: string,
  size?: number | null,
  offset?: number | null,
): Promise<SubsonicAlbum[]> {
  return invoke("subsonic_get_album_list", {
    listType,
    size: size ?? null,
    offset: offset ?? null,
  })
}

export function subsonicGetTopSongs(artistName: string, count?: number | null): Promise<SubsonicSong[]> {
  return invoke("subsonic_get_top_songs", { artistName, count: count ?? null })
}

// Streaming
export function subsonicGetStreamUrl(id: string): Promise<string> {
  return invoke("subsonic_get_stream_url", { id })
}

export function subsonicGetCoverArtUrl(id: string, size?: number | null): Promise<string> {
  return invoke("subsonic_get_cover_art_url", { id, size: size ?? null })
}

// Playlists
export function subsonicGetPlaylists(): Promise<SubsonicPlaylist[]> {
  return invoke("subsonic_get_playlists")
}

export function subsonicGetPlaylist(id: string): Promise<PlaylistDetail> {
  return invoke("subsonic_get_playlist", { id })
}

export function subsonicCreatePlaylist(name: string, songIds: string[]): Promise<SubsonicPlaylist> {
  return invoke("subsonic_create_playlist", { name, songIds })
}

export function subsonicUpdatePlaylist(
  id: string,
  name?: string | null,
  songIdsToAdd?: string[] | null,
  songIndexesToRemove?: number[] | null,
): Promise<void> {
  return invoke("subsonic_update_playlist", {
    id,
    name: name ?? null,
    songIdsToAdd: songIdsToAdd ?? null,
    songIndexesToRemove: songIndexesToRemove ?? null,
  })
}

export function subsonicDeletePlaylist(id: string): Promise<void> {
  return invoke("subsonic_delete_playlist", { id })
}

// Ratings & scrobble
export function subsonicStar(id: string): Promise<void> {
  return invoke("subsonic_star", { id })
}

export function subsonicUnstar(id: string): Promise<void> {
  return invoke("subsonic_unstar", { id })
}

export function subsonicSetRating(id: string, rating: number): Promise<void> {
  return invoke("subsonic_set_rating", { id, rating })
}

export function subsonicScrobble(id: string, submission?: boolean): Promise<void> {
  return invoke("subsonic_scrobble", { id, submission: submission ?? null })
}

// Starred / Genres
export function subsonicGetStarred(): Promise<Starred2> {
  return invoke("subsonic_get_starred")
}

export function subsonicGetGenres(): Promise<SubsonicGenre[]> {
  return invoke("subsonic_get_genres")
}

// Settings
export function subsonicLoadSettings(): Promise<SubsonicSettings> {
  return invoke("subsonic_load_settings")
}

export function subsonicSaveSettings(baseUrl: string, username: string, password: string): Promise<void> {
  return invoke("subsonic_save_settings", { baseUrl, username, password })
}
