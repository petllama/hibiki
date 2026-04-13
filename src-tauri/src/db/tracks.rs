//! Track CRUD operations — backend-agnostic.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::artists::{get_tags, delete_tags, insert_tag, TagRow};

/// Track row — generic across all backends.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrackRow {
    pub backend: String,
    pub id: String,
    pub title: String,
    pub track_number: i64,
    pub duration: i64,
    pub album_id: Option<String>,
    pub album_name: String,
    pub artist_id: Option<String>,
    pub artist_name: String,
    pub year: i64,
    pub play_count: i64,
    pub thumb: Option<String>,
    pub codec: Option<String>,
    pub bitrate: Option<i64>,
    pub channels: Option<i64>,
    pub bit_depth: Option<i64>,
    pub sampling_rate: Option<i64>,
    pub user_rating: Option<f64>,
    pub added_at: Option<String>,
    pub updated_at: Option<String>,
    pub guid: Option<String>,
    /// Backend-specific data (JSON). Opaque to the DB layer.
    pub json_extra: String,
    pub tags: Vec<TagRow>,
}

/// Upsert a single track.
pub fn upsert(conn: &Connection, row: &TrackRow) -> Result<(), String> {
    conn.execute(
        "INSERT INTO tracks (backend, id, title, track_number, duration, album_id,
            album_name, artist_id, artist_name, year, play_count, thumb, codec,
            bitrate, channels, bit_depth, sampling_rate, user_rating, added_at,
            updated_at, guid, json_extra)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22)
         ON CONFLICT(backend, id) DO UPDATE SET
            title=?3, track_number=?4, duration=?5, album_id=?6, album_name=?7,
            artist_id=?8, artist_name=?9, year=?10, play_count=?11, thumb=?12,
            codec=?13, bitrate=?14, channels=?15, bit_depth=?16, sampling_rate=?17,
            user_rating=?18, added_at=?19, updated_at=?20, guid=?21, json_extra=?22",
        rusqlite::params![
            row.backend, row.id, row.title, row.track_number, row.duration,
            row.album_id, row.album_name, row.artist_id, row.artist_name,
            row.year, row.play_count, row.thumb, row.codec, row.bitrate,
            row.channels, row.bit_depth, row.sampling_rate, row.user_rating,
            row.added_at, row.updated_at, row.guid, row.json_extra,
        ],
    )
    .map_err(|e| format!("upsert track error: {e}"))?;

    // Replace tags
    delete_tags(conn, &row.backend, "track", &row.id)?;
    for tag in &row.tags {
        insert_tag(conn, &row.backend, "track", &row.id, &tag.tag_type, &tag.tag)?;
    }

    Ok(())
}

/// Upsert many tracks in a single transaction.
pub fn upsert_bulk(conn: &Connection, rows: &[TrackRow]) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("begin tx error: {e}"))?;
    for row in rows {
        upsert(&tx, row)?;
    }
    tx.commit().map_err(|e| format!("commit error: {e}"))?;
    Ok(())
}

/// Get a single track by backend + ID.
pub fn get(conn: &Connection, backend: &str, id: &str) -> Result<Option<TrackRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT backend, id, title, track_number, duration, album_id, album_name,
                    artist_id, artist_name, year, play_count, thumb, codec, bitrate,
                    channels, bit_depth, sampling_rate, user_rating, added_at,
                    updated_at, guid, json_extra
             FROM tracks WHERE backend = ?1 AND id = ?2",
        )
        .map_err(|e| format!("prepare error: {e}"))?;

    let row = stmt
        .query_row(rusqlite::params![backend, id], map_track_row)
        .optional()
        .map_err(|e| format!("query error: {e}"))?;

    match row {
        None => Ok(None),
        Some(mut t) => {
            t.tags = get_tags(conn, &t.backend, "track", &t.id)?;
            Ok(Some(t))
        }
    }
}

/// Search tracks by title within a backend.
pub fn search(conn: &Connection, backend: &str, query: &str, limit: i64) -> Result<Vec<TrackRow>, String> {
    let pattern = format!("%{query}%");
    let mut stmt = conn
        .prepare(
            "SELECT backend, id, title, track_number, duration, album_id, album_name,
                    artist_id, artist_name, year, play_count, thumb, codec, bitrate,
                    channels, bit_depth, sampling_rate, user_rating, added_at,
                    updated_at, guid, json_extra
             FROM tracks
             WHERE backend = ?1 AND title LIKE ?2 COLLATE NOCASE
             ORDER BY title COLLATE NOCASE
             LIMIT ?3",
        )
        .map_err(|e| format!("prepare error: {e}"))?;

    let rows: Vec<TrackRow> = stmt
        .query_map(rusqlite::params![backend, pattern, limit], map_track_row)
        .map_err(|e| format!("query error: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

/// Get all tracks for an album within a backend.
pub fn get_by_album(conn: &Connection, backend: &str, album_id: &str) -> Result<Vec<TrackRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT backend, id, title, track_number, duration, album_id, album_name,
                    artist_id, artist_name, year, play_count, thumb, codec, bitrate,
                    channels, bit_depth, sampling_rate, user_rating, added_at,
                    updated_at, guid, json_extra
             FROM tracks
             WHERE backend = ?1 AND album_id = ?2
             ORDER BY track_number",
        )
        .map_err(|e| format!("prepare error: {e}"))?;

    let rows: Vec<TrackRow> = stmt
        .query_map(rusqlite::params![backend, album_id], map_track_row)
        .map_err(|e| format!("query error: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

/// Count tracks for a backend.
pub fn count(conn: &Connection, backend: &str) -> Result<i64, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM tracks WHERE backend = ?1",
        [backend],
        |r| r.get(0),
    )
    .map_err(|e| format!("count error: {e}"))
}

/// Delete all tracks for a backend.
pub fn delete_all(conn: &Connection, backend: &str) -> Result<i64, String> {
    conn.execute(
        "DELETE FROM tags WHERE backend = ?1 AND entity_type = 'track'",
        [backend],
    )
    .map_err(|e| format!("delete track tags error: {e}"))?;

    let deleted = conn.execute(
        "DELETE FROM tracks WHERE backend = ?1",
        [backend],
    )
    .map_err(|e| format!("delete tracks error: {e}"))?;

    Ok(deleted as i64)
}

fn map_track_row(row: &rusqlite::Row) -> rusqlite::Result<TrackRow> {
    Ok(TrackRow {
        backend: row.get(0)?,
        id: row.get(1)?,
        title: row.get(2)?,
        track_number: row.get(3)?,
        duration: row.get(4)?,
        album_id: row.get(5)?,
        album_name: row.get(6)?,
        artist_id: row.get(7)?,
        artist_name: row.get(8)?,
        year: row.get(9)?,
        play_count: row.get(10)?,
        thumb: row.get(11)?,
        codec: row.get(12)?,
        bitrate: row.get(13)?,
        channels: row.get(14)?,
        bit_depth: row.get(15)?,
        sampling_rate: row.get(16)?,
        user_rating: row.get(17)?,
        added_at: row.get(18)?,
        updated_at: row.get(19)?,
        guid: row.get(20)?,
        json_extra: row.get(21)?,
        tags: Vec::new(),
    })
}

trait Optional<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> Optional<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbState;

    #[test]
    fn test_track_round_trip() {
        let db = DbState::open_in_memory().unwrap();
        let conn = db.0.lock().unwrap();

        let track = TrackRow {
            backend: "navidrome".into(), id: "tr-abc123".into(),
            title: "Test Track".into(), track_number: 3, duration: 240000,
            album_name: "Test Album".into(), artist_name: "Test Artist".into(),
            codec: Some("flac".into()), bitrate: Some(1411),
            json_extra: "{}".into(), ..Default::default()
        };
        upsert(&conn, &track).unwrap();

        let loaded = get(&conn, "navidrome", "tr-abc123").unwrap().expect("should exist");
        assert_eq!(loaded.title, "Test Track");
        assert_eq!(loaded.codec, Some("flac".into()));
    }

    #[test]
    fn test_tracks_by_album() {
        let db = DbState::open_in_memory().unwrap();
        let conn = db.0.lock().unwrap();

        // Create parent album (FK constraint)
        crate::db::albums::upsert(&conn, &crate::db::albums::AlbumRow {
            backend: "plex".into(), id: "100".into(), title: "Album".into(),
            json_extra: "{}".into(), ..Default::default()
        }).unwrap();

        for i in 1..=5 {
            upsert(&conn, &TrackRow {
                backend: "plex".into(), id: format!("{i}"),
                title: format!("Track {i}"), track_number: i,
                album_id: Some("100".into()), album_name: "Album".into(),
                json_extra: "{}".into(), ..Default::default()
            }).unwrap();
        }

        let tracks = get_by_album(&conn, "plex", "100").unwrap();
        assert_eq!(tracks.len(), 5);
        assert_eq!(tracks[0].track_number, 1);
        assert_eq!(tracks[4].track_number, 5);
    }
}
