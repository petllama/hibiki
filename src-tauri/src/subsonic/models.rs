//! Serde models for Subsonic/OpenSubsonic JSON responses.
//!
//! Subsonic wraps all responses in `{"subsonic-response": {"status": "ok", ...}}`.
//! The inner payload varies by endpoint.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Response envelope
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SubsonicEnvelope<T> {
    #[serde(rename = "subsonic-response")]
    pub response: SubsonicResponse<T>,
}

#[derive(Debug, Deserialize)]
pub struct SubsonicResponse<T> {
    pub status: String,
    #[serde(default)]
    pub version: String,
    #[serde(flatten)]
    pub body: T,
    pub error: Option<SubsonicError>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubsonicError {
    pub code: i32,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Ping (empty body)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PingBody {}

// ---------------------------------------------------------------------------
// Artists index
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ArtistsBody {
    pub artists: Option<ArtistsContainer>,
}

#[derive(Debug, Deserialize)]
pub struct ArtistsContainer {
    #[serde(default)]
    pub index: Vec<ArtistIndex>,
}

#[derive(Debug, Deserialize)]
pub struct ArtistIndex {
    pub name: String,
    #[serde(default)]
    pub artist: Vec<SubsonicArtist>,
}

// ---------------------------------------------------------------------------
// Artist
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubsonicArtist {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub album_count: i32,
    pub cover_art: Option<String>,
    pub artist_image_url: Option<String>,
    pub starred: Option<String>,
    #[serde(default)]
    pub user_rating: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ArtistDetailBody {
    pub artist: Option<ArtistDetail>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArtistDetail {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub album_count: i32,
    pub cover_art: Option<String>,
    pub artist_image_url: Option<String>,
    pub starred: Option<String>,
    #[serde(default)]
    pub album: Vec<SubsonicAlbum>,
}

// ---------------------------------------------------------------------------
// Album
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubsonicAlbum {
    pub id: String,
    pub name: String,
    pub artist: Option<String>,
    pub artist_id: Option<String>,
    pub cover_art: Option<String>,
    #[serde(default)]
    pub song_count: i32,
    #[serde(default)]
    pub duration: i64,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub starred: Option<String>,
    #[serde(default)]
    pub play_count: Option<i64>,
    pub created: Option<String>,
    #[serde(default)]
    pub user_rating: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AlbumDetailBody {
    pub album: Option<AlbumDetail>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AlbumDetail {
    pub id: String,
    pub name: String,
    pub artist: Option<String>,
    pub artist_id: Option<String>,
    pub cover_art: Option<String>,
    #[serde(default)]
    pub song_count: i32,
    #[serde(default)]
    pub duration: i64,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub starred: Option<String>,
    #[serde(default)]
    pub play_count: Option<i64>,
    pub created: Option<String>,
    #[serde(default)]
    pub user_rating: Option<i32>,
    #[serde(default)]
    pub song: Vec<SubsonicSong>,
}

// ---------------------------------------------------------------------------
// Song (Child element)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubsonicSong {
    pub id: String,
    pub title: String,
    pub album: Option<String>,
    pub album_id: Option<String>,
    pub artist: Option<String>,
    pub artist_id: Option<String>,
    pub track: Option<i32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub cover_art: Option<String>,
    pub size: Option<i64>,
    pub content_type: Option<String>,
    pub suffix: Option<String>,
    #[serde(default)]
    pub duration: i64,
    pub bit_rate: Option<i32>,
    pub path: Option<String>,
    #[serde(default)]
    pub play_count: Option<i64>,
    pub created: Option<String>,
    pub starred: Option<String>,
    #[serde(default)]
    pub user_rating: Option<i32>,
    pub disc_number: Option<i32>,
    #[serde(default)]
    pub channels: Option<i32>,
    #[serde(default)]
    pub sampling_rate: Option<i64>,
    #[serde(default)]
    pub bit_depth: Option<i32>,
    pub replay_gain: Option<ReplayGain>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReplayGain {
    pub track_gain: Option<f64>,
    pub album_gain: Option<f64>,
    pub track_peak: Option<f64>,
    pub album_peak: Option<f64>,
}

// ---------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SearchResult3Body {
    #[serde(rename = "searchResult3")]
    pub search_result3: Option<SearchResult3>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchResult3 {
    #[serde(default)]
    pub artist: Vec<SubsonicArtist>,
    #[serde(default)]
    pub album: Vec<SubsonicAlbum>,
    #[serde(default)]
    pub song: Vec<SubsonicSong>,
}

// ---------------------------------------------------------------------------
// Album list
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AlbumList2Body {
    #[serde(rename = "albumList2")]
    pub album_list2: Option<AlbumList2>,
}

#[derive(Debug, Deserialize)]
pub struct AlbumList2 {
    #[serde(default)]
    pub album: Vec<SubsonicAlbum>,
}

// ---------------------------------------------------------------------------
// Top Songs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TopSongsBody {
    #[serde(rename = "topSongs")]
    pub top_songs: Option<TopSongs>,
}

#[derive(Debug, Deserialize)]
pub struct TopSongs {
    #[serde(default)]
    pub song: Vec<SubsonicSong>,
}

// ---------------------------------------------------------------------------
// Similar Songs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SimilarSongs2Body {
    #[serde(rename = "similarSongs2")]
    pub similar_songs2: Option<SimilarSongs2>,
}

#[derive(Debug, Deserialize)]
pub struct SimilarSongs2 {
    #[serde(default)]
    pub song: Vec<SubsonicSong>,
}

// ---------------------------------------------------------------------------
// Playlists
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PlaylistsBody {
    pub playlists: Option<PlaylistsContainer>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistsContainer {
    #[serde(default)]
    pub playlist: Vec<SubsonicPlaylist>,
}

#[derive(Debug, Deserialize)]
pub struct PlaylistDetailBody {
    pub playlist: Option<PlaylistDetail>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubsonicPlaylist {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub song_count: i32,
    #[serde(default)]
    pub duration: i64,
    pub owner: Option<String>,
    pub created: Option<String>,
    pub changed: Option<String>,
    pub cover_art: Option<String>,
    #[serde(rename = "public")]
    pub is_public: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistDetail {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub song_count: i32,
    #[serde(default)]
    pub duration: i64,
    pub owner: Option<String>,
    pub created: Option<String>,
    pub changed: Option<String>,
    pub cover_art: Option<String>,
    #[serde(default)]
    pub entry: Vec<SubsonicSong>,
}

// ---------------------------------------------------------------------------
// Starred
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct Starred2Body {
    pub starred2: Option<Starred2>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Starred2 {
    #[serde(default)]
    pub artist: Vec<SubsonicArtist>,
    #[serde(default)]
    pub album: Vec<SubsonicAlbum>,
    #[serde(default)]
    pub song: Vec<SubsonicSong>,
}

// ---------------------------------------------------------------------------
// Genres
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct GenresBody {
    pub genres: Option<GenresContainer>,
}

#[derive(Debug, Deserialize)]
pub struct GenresContainer {
    #[serde(default)]
    pub genre: Vec<SubsonicGenre>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubsonicGenre {
    pub value: String,
    #[serde(default)]
    pub song_count: i32,
    #[serde(default)]
    pub album_count: i32,
}

// ---------------------------------------------------------------------------
// Settings (persisted locally, not from server)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SubsonicSettings {
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub username: String,
    /// Stored as hex-encoded md5(password + salt)
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub salt: String,
    /// Keep the raw password so we can re-derive tokens with fresh salts.
    /// In a production app you'd use OS keychain; for now, same pattern as Plex token.
    #[serde(default)]
    pub password: String,
}
