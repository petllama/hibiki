/**
 * TypeScript types mirroring the Rust Subsonic models.
 * Field names match the Rust serde output (camelCase).
 */

export interface SubsonicArtist {
  id: string
  name: string
  albumCount: number
  coverArt: string | null
  artistImageUrl: string | null
  starred: string | null
  userRating: number | null
}

export interface ArtistDetail extends SubsonicArtist {
  album: SubsonicAlbum[]
}

export interface SubsonicAlbum {
  id: string
  name: string
  artist: string | null
  artistId: string | null
  coverArt: string | null
  songCount: number
  duration: number
  year: number | null
  genre: string | null
  starred: string | null
  playCount: number | null
  created: string | null
  userRating: number | null
}

export interface AlbumDetail extends SubsonicAlbum {
  song: SubsonicSong[]
}

export interface SubsonicSong {
  id: string
  title: string
  album: string | null
  albumId: string | null
  artist: string | null
  artistId: string | null
  track: number | null
  year: number | null
  genre: string | null
  coverArt: string | null
  size: number | null
  contentType: string | null
  suffix: string | null
  duration: number
  bitRate: number | null
  path: string | null
  playCount: number | null
  created: string | null
  starred: string | null
  userRating: number | null
  discNumber: number | null
  channels: number | null
  samplingRate: number | null
  bitDepth: number | null
  replayGain: ReplayGain | null
}

export interface ReplayGain {
  trackGain: number | null
  albumGain: number | null
  trackPeak: number | null
  albumPeak: number | null
}

export interface SearchResult3 {
  artist: SubsonicArtist[]
  album: SubsonicAlbum[]
  song: SubsonicSong[]
}

export interface SubsonicPlaylist {
  id: string
  name: string
  songCount: number
  duration: number
  owner: string | null
  created: string | null
  changed: string | null
  coverArt: string | null
}

export interface PlaylistDetail extends SubsonicPlaylist {
  entry: SubsonicSong[]
}

export interface Starred2 {
  artist: SubsonicArtist[]
  album: SubsonicAlbum[]
  song: SubsonicSong[]
}

export interface SubsonicGenre {
  value: string
  songCount: number
  albumCount: number
}

export interface SubsonicSettings {
  base_url: string
  username: string
  password: string
  token: string
  salt: string
}
