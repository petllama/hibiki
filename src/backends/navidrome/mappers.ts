/**
 * Map Subsonic types to generic MusicTrack / MusicAlbum / MusicArtist.
 *
 * The image resolver is an async function that fetches the cover art URL
 * from the Rust backend. Since cover art IDs need to be resolved to full
 * authenticated URLs, we pass a pre-bound resolver.
 */

import type {
  MusicTrack,
  MusicAlbum,
  MusicArtist,
  MusicPlaylist,
  MusicItem,
} from "../../types/music"
import type {
  SubsonicSong,
  SubsonicAlbum,
  SubsonicArtist,
  SubsonicPlaylist,
} from "./types"

/**
 * Synchronous image resolver — takes a cover art ID and returns
 * a ready-to-use image:// URL. The caller pre-resolves the URL
 * before calling the mapper.
 */
export type ImgResolver = (
  coverArtId: string | null,
  entityType: "artist" | "album" | "track" | "playlist",
  entityId: string,
  name?: string | null,
  artist?: string | null,
) => string | null

// ---------------------------------------------------------------------------
// Song → MusicTrack
// ---------------------------------------------------------------------------

export function subsonicSongToMusicTrack(
  song: SubsonicSong,
  img: ImgResolver,
): MusicTrack {
  const thumbUrl = img(song.coverArt, "track", song.id, song.title, song.artist)
  const albumThumbUrl = song.albumId
    ? img(song.coverArt, "album", song.albumId, song.album, song.artist)
    : thumbUrl

  // Map Subsonic 1-5 rating to 0-10 scale (matching Plex convention)
  const userRating = song.userRating ? song.userRating * 2 : null

  return {
    id: song.id,
    title: song.title,
    trackNumber: song.track ?? 0,
    duration: (song.duration ?? 0) * 1000, // Subsonic returns seconds, generic uses ms
    albumId: song.albumId ?? null,
    albumName: song.album ?? "",
    albumYear: song.year ?? null,
    artistId: song.artistId ?? null,
    artistName: song.artist ?? "",
    year: song.year ?? 0,
    playCount: song.playCount ?? 0,
    thumbUrl,
    albumThumbUrl,
    artistThumbUrl: null, // Subsonic songs don't carry artist thumb
    summary: null,
    userRating,
    addedAt: song.created ?? null,
    lastPlayedAt: null,
    guid: null,
    codec: song.suffix ?? null,
    bitrate: song.bitRate ?? null,
    channels: song.channels ?? null,
    bitDepth: song.bitDepth ?? null,
    samplingRate: song.samplingRate ?? null,
    streamUrl: null, // Resolved lazily via getStreamUrl
    gain: song.replayGain?.trackGain ?? null,
    albumGain: song.replayGain?.albumGain ?? null,
    peak: song.replayGain?.trackPeak ?? null,
    loudness: null,
    _providerData: song,
  }
}

// ---------------------------------------------------------------------------
// Album → MusicAlbum
// ---------------------------------------------------------------------------

export function subsonicAlbumToMusicAlbum(
  album: SubsonicAlbum,
  img: ImgResolver,
): MusicAlbum {
  const thumbUrl = img(album.coverArt, "album", album.id, album.name, album.artist)

  const userRating = album.userRating ? album.userRating * 2 : null

  return {
    id: album.id,
    title: album.name,
    artistId: album.artistId ?? null,
    artistName: album.artist ?? "",
    year: album.year ?? 0,
    trackCount: album.songCount ?? 0,
    studio: null,
    thumbUrl,
    artistThumbUrl: null,
    summary: null,
    userRating,
    addedAt: album.created ?? null,
    guid: null,
    genres: album.genre ? [album.genre] : [],
    styles: [],
    moods: [],
    labels: [],
    format: null,
  }
}

// ---------------------------------------------------------------------------
// Artist → MusicArtist
// ---------------------------------------------------------------------------

export function subsonicArtistToMusicArtist(
  artist: SubsonicArtist,
  img: ImgResolver,
): MusicArtist {
  const thumbUrl = img(artist.coverArt, "artist", artist.id, artist.name)

  return {
    id: artist.id,
    title: artist.name,
    thumbUrl,
    artUrl: artist.artistImageUrl ?? thumbUrl,
    summary: null,
    userRating: artist.userRating ? artist.userRating * 2 : null,
    addedAt: null,
    guid: null,
  }
}

// ---------------------------------------------------------------------------
// Playlist → MusicPlaylist
// ---------------------------------------------------------------------------

export function subsonicPlaylistToMusicPlaylist(
  playlist: SubsonicPlaylist,
  img: ImgResolver,
): MusicPlaylist {
  return {
    id: playlist.id,
    title: playlist.name,
    smart: false, // Subsonic doesn't distinguish smart playlists
    trackCount: playlist.songCount ?? 0,
    duration: (playlist.duration ?? 0) * 1000,
    thumbUrl: img(playlist.coverArt, "playlist", playlist.id),
    summary: null,
    addedAt: playlist.created ?? null,
  }
}

// ---------------------------------------------------------------------------
// Mixed search results → MusicItem[]
// ---------------------------------------------------------------------------

export function subsonicSearchToMusicItems(
  artists: SubsonicArtist[],
  albums: SubsonicAlbum[],
  songs: SubsonicSong[],
  img: ImgResolver,
): MusicItem[] {
  const items: MusicItem[] = []
  for (const a of artists) items.push({ ...subsonicArtistToMusicArtist(a, img), type: "artist" })
  for (const a of albums) items.push({ ...subsonicAlbumToMusicAlbum(a, img), type: "album" })
  for (const s of songs) items.push({ ...subsonicSongToMusicTrack(s, img), type: "track" })
  return items
}
