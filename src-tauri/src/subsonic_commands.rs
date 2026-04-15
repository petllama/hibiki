//! Tauri command bridge for the Subsonic/Navidrome backend.
//!
//! All commands are prefixed with `subsonic_` to avoid collisions with Plex commands.

use tauri::State;
use tokio::sync::Mutex;

use crate::subsonic::{
    SubsonicClient, SubsonicSettings,
    SubsonicArtist, SubsonicAlbum, SubsonicSong, SubsonicPlaylist,
    SubsonicGenre, ArtistDetail, AlbumDetail, PlaylistDetail,
    SearchResult3, Starred2,
};

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

pub struct SubsonicState(pub Mutex<Option<SubsonicClient>>);

impl SubsonicState {
    pub fn new() -> Self {
        SubsonicState(Mutex::new(None))
    }
}

macro_rules! client {
    ($state:expr) => {{
        let guard = $state.0.lock().await;
        match guard.as_ref() {
            Some(c) => c.clone(),
            None => return Err("Subsonic client not connected. Call subsonic_connect first.".to_string()),
        }
    }};
}

// ---------------------------------------------------------------------------
// Connection
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn subsonic_connect(
    base_url: String,
    username: String,
    password: String,
    state: State<'_, SubsonicState>,
) -> Result<(), String> {
    let client = SubsonicClient::new(base_url, username, password)
        .map_err(|e| format!("{:#}", e))?;
    // Test the connection
    client.ping().await.map_err(|e| format!("{:#}", e))?;
    let mut guard = state.0.lock().await;
    *guard = Some(client);
    Ok(())
}

#[tauri::command]
pub async fn subsonic_ping(
    state: State<'_, SubsonicState>,
) -> Result<(), String> {
    let c = client!(state);
    c.ping().await.map_err(|e| format!("{:#}", e))
}

// ---------------------------------------------------------------------------
// Library browsing
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn subsonic_get_artists(
    state: State<'_, SubsonicState>,
) -> Result<Vec<SubsonicArtist>, String> {
    let c = client!(state);
    c.get_artists().await.map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn subsonic_get_artist(
    id: String,
    state: State<'_, SubsonicState>,
) -> Result<ArtistDetail, String> {
    let c = client!(state);
    c.get_artist(&id).await.map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn subsonic_get_album(
    id: String,
    state: State<'_, SubsonicState>,
) -> Result<AlbumDetail, String> {
    let c = client!(state);
    c.get_album(&id).await.map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn subsonic_get_song(
    id: String,
    state: State<'_, SubsonicState>,
) -> Result<SubsonicSong, String> {
    let c = client!(state);
    c.get_song(&id).await.map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn subsonic_search(
    query: String,
    artist_count: Option<i32>,
    album_count: Option<i32>,
    song_count: Option<i32>,
    state: State<'_, SubsonicState>,
) -> Result<SearchResult3, String> {
    let c = client!(state);
    c.search3(&query, artist_count, album_count, song_count)
        .await.map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn subsonic_get_album_list(
    list_type: String,
    size: Option<i32>,
    offset: Option<i32>,
    state: State<'_, SubsonicState>,
) -> Result<Vec<SubsonicAlbum>, String> {
    let c = client!(state);
    c.get_album_list2(&list_type, size, offset)
        .await.map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn subsonic_get_top_songs(
    artist_name: String,
    count: Option<i32>,
    state: State<'_, SubsonicState>,
) -> Result<Vec<SubsonicSong>, String> {
    let c = client!(state);
    c.get_top_songs(&artist_name, count)
        .await.map_err(|e| format!("{:#}", e))
}

// ---------------------------------------------------------------------------
// Streaming URLs
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn subsonic_get_stream_url(
    id: String,
    state: State<'_, SubsonicState>,
) -> Result<String, String> {
    let c = client!(state);
    Ok(c.stream_url(&id))
}

#[tauri::command]
pub async fn subsonic_get_cover_art_url(
    id: String,
    size: Option<i32>,
    state: State<'_, SubsonicState>,
) -> Result<String, String> {
    let c = client!(state);
    Ok(c.cover_art_url(&id, size))
}

// ---------------------------------------------------------------------------
// Playlists
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn subsonic_get_playlists(
    state: State<'_, SubsonicState>,
) -> Result<Vec<SubsonicPlaylist>, String> {
    let c = client!(state);
    c.get_playlists().await.map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn subsonic_get_playlist(
    id: String,
    state: State<'_, SubsonicState>,
) -> Result<PlaylistDetail, String> {
    let c = client!(state);
    c.get_playlist(&id).await.map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn subsonic_create_playlist(
    name: String,
    song_ids: Vec<String>,
    state: State<'_, SubsonicState>,
) -> Result<SubsonicPlaylist, String> {
    let c = client!(state);
    c.create_playlist(&name, &song_ids).await.map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn subsonic_update_playlist(
    id: String,
    name: Option<String>,
    song_ids_to_add: Option<Vec<String>>,
    song_indexes_to_remove: Option<Vec<i32>>,
    state: State<'_, SubsonicState>,
) -> Result<(), String> {
    let c = client!(state);
    c.update_playlist(
        &id,
        name.as_deref(),
        &song_ids_to_add.unwrap_or_default(),
        &song_indexes_to_remove.unwrap_or_default(),
    ).await.map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn subsonic_delete_playlist(
    id: String,
    state: State<'_, SubsonicState>,
) -> Result<(), String> {
    let c = client!(state);
    c.delete_playlist(&id).await.map_err(|e| format!("{:#}", e))
}

// ---------------------------------------------------------------------------
// Ratings & scrobble
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn subsonic_star(
    id: String,
    state: State<'_, SubsonicState>,
) -> Result<(), String> {
    let c = client!(state);
    c.star(&id).await.map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn subsonic_unstar(
    id: String,
    state: State<'_, SubsonicState>,
) -> Result<(), String> {
    let c = client!(state);
    c.unstar(&id).await.map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn subsonic_set_rating(
    id: String,
    rating: i32,
    state: State<'_, SubsonicState>,
) -> Result<(), String> {
    let c = client!(state);
    c.set_rating(&id, rating).await.map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn subsonic_scrobble(
    id: String,
    submission: Option<bool>,
    state: State<'_, SubsonicState>,
) -> Result<(), String> {
    let c = client!(state);
    c.scrobble(&id, submission.unwrap_or(true)).await.map_err(|e| format!("{:#}", e))
}

// ---------------------------------------------------------------------------
// Starred / Genres
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn subsonic_get_starred(
    state: State<'_, SubsonicState>,
) -> Result<Starred2, String> {
    let c = client!(state);
    c.get_starred2().await.map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn subsonic_get_genres(
    state: State<'_, SubsonicState>,
) -> Result<Vec<SubsonicGenre>, String> {
    let c = client!(state);
    c.get_genres().await.map_err(|e| format!("{:#}", e))
}

// ---------------------------------------------------------------------------
// Settings persistence
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn subsonic_load_settings(
    app: tauri::AppHandle,
) -> Result<SubsonicSettings, String> {
    use tauri::Manager;
    let config_dir = app.path().app_config_dir().map_err(|e| format!("{:#}", e))?;
    crate::subsonic::auth::load(&config_dir).map_err(|e| format!("{:#}", e))
}

#[tauri::command]
pub async fn subsonic_save_settings(
    base_url: String,
    username: String,
    password: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use tauri::Manager;
    let config_dir = app.path().app_config_dir().map_err(|e| format!("{:#}", e))?;
    let settings = SubsonicSettings {
        base_url,
        username,
        password,
        token: String::new(),
        salt: String::new(),
    };
    crate::subsonic::auth::save(&config_dir, &settings).map_err(|e| format!("{:#}", e))
}

// ---------------------------------------------------------------------------
// Sync — full library sync into local DB
// ---------------------------------------------------------------------------

use crate::db::{self, DbState};
use crate::db::artists::ArtistRow;
use crate::db::albums::AlbumRow;
use crate::db::tracks::TrackRow;
use tracing::info;
use std::time::Instant;

const BACKEND: &str = "navidrome";

/// Sync result for a single entity type.
#[derive(serde::Serialize, Clone)]
pub struct SyncResult {
    pub synced: i64,
    pub deleted: i64,
}

/// Combined result from a full sync.
#[derive(serde::Serialize)]
pub struct FullSyncResult {
    pub artists: SyncResult,
    pub albums: SyncResult,
    pub tracks: SyncResult,
    pub elapsed_ms: u64,
}

// ---------------------------------------------------------------------------
// Fetch helpers (async, no DB access)
// ---------------------------------------------------------------------------

async fn fetch_artist_rows(client: &crate::subsonic::SubsonicClient) -> Result<Vec<ArtistRow>, String> {
    info!("[sync:artists] fetching from server...");
    let artists = client.get_artists().await.map_err(|e| format!("{:#}", e))?;
    info!("[sync:artists] fetched {} artists", artists.len());

    Ok(artists.iter().map(|a| ArtistRow {
        backend: BACKEND.into(),
        id: a.id.clone(),
        title: a.name.clone(),
        thumb: a.cover_art.clone(),
        art: a.artist_image_url.clone(),
        user_rating: a.user_rating.map(|r| r as f64),
        json_extra: serde_json::to_string(a).unwrap_or_else(|_| "{}".into()),
        ..Default::default()
    }).collect())
}

async fn fetch_album_rows(client: &crate::subsonic::SubsonicClient) -> Result<Vec<AlbumRow>, String> {
    info!("[sync:albums] fetching from server...");
    let mut all_albums: Vec<SubsonicAlbum> = Vec::new();
    let page_size = 500;
    let mut offset = 0;
    loop {
        let page = client.get_album_list2("alphabeticalByName", Some(page_size), Some(offset))
            .await.map_err(|e| format!("{:#}", e))?;
        let count = page.len();
        all_albums.extend(page);
        if count < page_size as usize { break; }
        offset += page_size;
    }
    info!("[sync:albums] fetched {} albums ({} pages)", all_albums.len(), (offset / page_size) + 1);

    Ok(all_albums.iter().map(|a| AlbumRow {
        backend: BACKEND.into(),
        id: a.id.clone(),
        title: a.name.clone(),
        artist_id: a.artist_id.clone(),
        artist_name: a.artist.clone().unwrap_or_default(),
        year: a.year.unwrap_or(0) as i64,
        track_count: a.song_count as i64,
        thumb: a.cover_art.clone(),
        user_rating: a.user_rating.map(|r| r as f64),
        added_at: a.created.clone(),
        genre: a.genre.clone(),
        json_extra: serde_json::to_string(a).unwrap_or_else(|_| "{}".into()),
        ..Default::default()
    }).collect())
}

async fn fetch_track_rows(client: &crate::subsonic::SubsonicClient) -> Result<Vec<TrackRow>, String> {
    info!("[sync:tracks] fetching album list...");
    let mut album_ids: Vec<String> = Vec::new();
    let page_size = 500;
    let mut offset = 0;
    loop {
        let page = client.get_album_list2("alphabeticalByName", Some(page_size), Some(offset))
            .await.map_err(|e| format!("{:#}", e))?;
        let count = page.len();
        album_ids.extend(page.into_iter().map(|a| a.id));
        if count < page_size as usize { break; }
        offset += page_size;
    }
    info!("[sync:tracks] found {} albums, fetching songs...", album_ids.len());

    let mut all_rows: Vec<TrackRow> = Vec::new();
    for (i, album_id) in album_ids.iter().enumerate() {
        let detail = client.get_album(album_id).await.map_err(|e| format!("{:#}", e))?;
        for s in &detail.song {
            all_rows.push(TrackRow {
                backend: BACKEND.into(),
                id: s.id.clone(),
                title: s.title.clone(),
                track_number: s.track.unwrap_or(0) as i64,
                duration: s.duration * 1000,
                album_id: s.album_id.clone(),
                album_name: s.album.clone().unwrap_or_default(),
                artist_id: s.artist_id.clone(),
                artist_name: s.artist.clone().unwrap_or_default(),
                year: s.year.unwrap_or(0) as i64,
                play_count: s.play_count.unwrap_or(0),
                thumb: s.cover_art.clone(),
                codec: s.suffix.clone(),
                bitrate: s.bit_rate.map(|b| b as i64),
                channels: s.channels.map(|c| c as i64),
                bit_depth: s.bit_depth.map(|b| b as i64),
                sampling_rate: s.sampling_rate,
                user_rating: s.user_rating.map(|r| r as f64),
                added_at: s.created.clone(),
                json_extra: serde_json::to_string(s).unwrap_or_else(|_| "{}".into()),
                ..Default::default()
            });
        }
        if (i + 1) % 100 == 0 {
            info!("[sync:tracks] fetched {}/{} albums ({} tracks so far)", i + 1, album_ids.len(), all_rows.len());
        }
    }
    info!("[sync:tracks] fetched {} tracks from {} albums", all_rows.len(), album_ids.len());
    Ok(all_rows)
}

// ---------------------------------------------------------------------------
// Write helpers (sync, DB only — no async, no MutexGuard-across-await)
// ---------------------------------------------------------------------------

fn write_artists(conn: &rusqlite::Connection, rows: &[ArtistRow]) -> Result<SyncResult, String> {
    let tx = conn.unchecked_transaction().map_err(|e| format!("begin tx: {e}"))?;
    let deleted = db::artists::delete_all(&tx, BACKEND)?;
    for row in rows {
        db::artists::upsert(&tx, row)?;
    }
    tx.commit().map_err(|e| format!("commit: {e}"))?;
    let synced = rows.len() as i64;
    info!("[sync:artists] written — synced {synced}, deleted {deleted}");
    Ok(SyncResult { synced, deleted })
}

fn write_albums(conn: &rusqlite::Connection, rows: &[AlbumRow]) -> Result<SyncResult, String> {
    let tx = conn.unchecked_transaction().map_err(|e| format!("begin tx: {e}"))?;
    let deleted = db::albums::delete_all(&tx, BACKEND)?;
    for row in rows {
        db::albums::upsert(&tx, row)?;
    }
    tx.commit().map_err(|e| format!("commit: {e}"))?;
    let synced = rows.len() as i64;
    info!("[sync:albums] written — synced {synced}, deleted {deleted}");
    Ok(SyncResult { synced, deleted })
}

fn write_tracks(conn: &rusqlite::Connection, rows: &[TrackRow]) -> Result<SyncResult, String> {
    let tx = conn.unchecked_transaction().map_err(|e| format!("begin tx: {e}"))?;
    let deleted = db::tracks::delete_all(&tx, BACKEND)?;
    for row in rows {
        db::tracks::upsert(&tx, row)?;
    }
    tx.commit().map_err(|e| format!("commit: {e}"))?;
    let synced = rows.len() as i64;
    info!("[sync:tracks] written — synced {synced}, deleted {deleted}");
    Ok(SyncResult { synced, deleted })
}

// ---------------------------------------------------------------------------
// Tauri commands — individual sync
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn sync_navidrome_artists(
    state: State<'_, SubsonicState>,
    db: State<'_, DbState>,
) -> Result<SyncResult, String> {
    let c = client!(state);
    let rows = fetch_artist_rows(&c).await?;
    let conn = db.0.lock().map_err(|e| format!("db lock error: {e}"))?;
    write_artists(&conn, &rows)
}

#[tauri::command]
pub async fn sync_navidrome_albums(
    state: State<'_, SubsonicState>,
    db: State<'_, DbState>,
) -> Result<SyncResult, String> {
    let c = client!(state);
    let rows = fetch_album_rows(&c).await?;
    let conn = db.0.lock().map_err(|e| format!("db lock error: {e}"))?;
    write_albums(&conn, &rows)
}

#[tauri::command]
pub async fn sync_navidrome_tracks(
    state: State<'_, SubsonicState>,
    db: State<'_, DbState>,
) -> Result<SyncResult, String> {
    let c = client!(state);
    let rows = fetch_track_rows(&c).await?;
    let conn = db.0.lock().map_err(|e| format!("db lock error: {e}"))?;
    write_tracks(&conn, &rows)
}

// ---------------------------------------------------------------------------
// Full sync orchestrator
// ---------------------------------------------------------------------------

/// Run a full Navidrome library sync: artists → albums → tracks, in order.
/// Each phase completes (fetch + write) before the next starts.
#[tauri::command]
pub async fn sync_navidrome_full(
    state: State<'_, SubsonicState>,
    db: State<'_, DbState>,
) -> Result<FullSyncResult, String> {
    let start = Instant::now();
    info!("[sync] ── starting full Navidrome library sync ──");

    let c = client!(state);

    // Disable FK constraints for the duration of the sync — delete-all + re-insert
    // across related tables triggers cascading SET NULL on NOT NULL columns.
    {
        let conn = db.0.lock().map_err(|e| format!("db lock error: {e}"))?;
        conn.execute_batch("PRAGMA foreign_keys = OFF;").map_err(|e| format!("pragma: {e}"))?;
    }

    // Phase 1: Artists
    let artist_rows = fetch_artist_rows(&c).await?;
    let artists = {
        let conn = db.0.lock().map_err(|e| format!("db lock error: {e}"))?;
        write_artists(&conn, &artist_rows)?
    };

    // Phase 2: Albums
    let album_rows = fetch_album_rows(&c).await?;
    let albums = {
        let conn = db.0.lock().map_err(|e| format!("db lock error: {e}"))?;
        write_albums(&conn, &album_rows)?
    };

    // Phase 3: Tracks
    let track_rows = fetch_track_rows(&c).await?;
    let tracks = {
        let conn = db.0.lock().map_err(|e| format!("db lock error: {e}"))?;
        write_tracks(&conn, &track_rows)?
    };

    // Re-enable FK constraints
    {
        let conn = db.0.lock().map_err(|e| format!("db lock error: {e}"))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;").map_err(|e| format!("pragma: {e}"))?;
    }

    let elapsed_ms = start.elapsed().as_millis() as u64;
    info!(
        "[sync] ── full sync complete in {elapsed_ms}ms — {} artists, {} albums, {} tracks ──",
        artists.synced, albums.synced, tracks.synced
    );

    Ok(FullSyncResult { artists, albums, tracks, elapsed_ms })
}
