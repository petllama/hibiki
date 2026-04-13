//! Playlist CRUD operations + playlist_tracks membership — backend-agnostic.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// Playlist row — generic across all backends.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaylistRow {
    pub backend: String,
    pub id: String,
    pub title: String,
    pub smart: bool,
    pub track_count: i64,
    pub duration: Option<i64>,
    pub summary: Option<String>,
    pub thumb: Option<String>,
    pub added_at: Option<String>,
    pub updated_at: Option<String>,
    pub guid: Option<String>,
    pub json_extra: String,
}

/// A playlist-track membership row.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlaylistTrackRow {
    pub backend: String,
    pub playlist_id: String,
    pub track_id: String,
    pub position: i64,
    pub added_at: String,
}

/// Upsert a single playlist.
pub fn upsert(conn: &Connection, row: &PlaylistRow) -> Result<(), String> {
    conn.execute(
        "INSERT INTO playlists (backend, id, title, smart, track_count, duration,
            summary, thumb, added_at, updated_at, guid, json_extra)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)
         ON CONFLICT(backend, id) DO UPDATE SET
            title=?3, smart=?4, track_count=?5, duration=?6, summary=?7,
            thumb=?8, added_at=?9, updated_at=?10, guid=?11, json_extra=?12",
        rusqlite::params![
            row.backend, row.id, row.title, row.smart, row.track_count,
            row.duration, row.summary, row.thumb, row.added_at, row.updated_at,
            row.guid, row.json_extra,
        ],
    )
    .map_err(|e| format!("upsert playlist error: {e}"))?;
    Ok(())
}

/// Upsert many playlists in a single transaction.
pub fn upsert_bulk(conn: &Connection, rows: &[PlaylistRow]) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("begin tx error: {e}"))?;
    for row in rows {
        upsert(&tx, row)?;
    }
    tx.commit().map_err(|e| format!("commit error: {e}"))?;
    Ok(())
}

/// Get all playlists for a backend.
pub fn get_all(conn: &Connection, backend: &str) -> Result<Vec<PlaylistRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT backend, id, title, smart, track_count, duration,
                    summary, thumb, added_at, updated_at, guid, json_extra
             FROM playlists WHERE backend = ?1
             ORDER BY title COLLATE NOCASE",
        )
        .map_err(|e| format!("prepare error: {e}"))?;

    let rows: Vec<PlaylistRow> = stmt
        .query_map([backend], map_playlist_row)
        .map_err(|e| format!("query error: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

/// Add a track to a playlist.
pub fn add_track(conn: &Connection, row: &PlaylistTrackRow) -> Result<(), String> {
    conn.execute(
        "INSERT INTO playlist_tracks (backend, playlist_id, track_id, position, added_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![row.backend, row.playlist_id, row.track_id, row.position, row.added_at],
    )
    .map_err(|e| format!("add playlist track error: {e}"))?;
    Ok(())
}

/// Get all tracks in a playlist, ordered by position.
pub fn get_tracks(conn: &Connection, backend: &str, playlist_id: &str) -> Result<Vec<PlaylistTrackRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT backend, playlist_id, track_id, position, added_at
             FROM playlist_tracks
             WHERE backend = ?1 AND playlist_id = ?2
             ORDER BY position",
        )
        .map_err(|e| format!("prepare error: {e}"))?;

    let rows: Vec<PlaylistTrackRow> = stmt
        .query_map(rusqlite::params![backend, playlist_id], |row| {
            Ok(PlaylistTrackRow {
                backend: row.get(0)?,
                playlist_id: row.get(1)?,
                track_id: row.get(2)?,
                position: row.get(3)?,
                added_at: row.get(4)?,
            })
        })
        .map_err(|e| format!("query error: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

/// Count playlists for a backend.
pub fn count(conn: &Connection, backend: &str) -> Result<i64, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM playlists WHERE backend = ?1",
        [backend],
        |r| r.get(0),
    )
    .map_err(|e| format!("count error: {e}"))
}

fn map_playlist_row(row: &rusqlite::Row) -> rusqlite::Result<PlaylistRow> {
    Ok(PlaylistRow {
        backend: row.get(0)?,
        id: row.get(1)?,
        title: row.get(2)?,
        smart: row.get(3)?,
        track_count: row.get(4)?,
        duration: row.get(5)?,
        summary: row.get(6)?,
        thumb: row.get(7)?,
        added_at: row.get(8)?,
        updated_at: row.get(9)?,
        guid: row.get(10)?,
        json_extra: row.get(11)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbState;

    #[test]
    fn test_playlist_round_trip() {
        let db = DbState::open_in_memory().unwrap();
        let conn = db.0.lock().unwrap();

        let pl = PlaylistRow {
            backend: "plex".into(), id: "500".into(), title: "My Playlist".into(),
            track_count: 10, json_extra: "{}".into(), ..Default::default()
        };
        upsert(&conn, &pl).unwrap();

        let all = get_all(&conn, "plex").unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].title, "My Playlist");
    }

    #[test]
    fn test_playlist_tracks() {
        let db = DbState::open_in_memory().unwrap();
        let conn = db.0.lock().unwrap();

        // Create playlist (no FK to tracks since tracks may not be synced yet)
        upsert(&conn, &PlaylistRow {
            backend: "navidrome".into(), id: "pl-1".into(), title: "Test".into(),
            json_extra: "{}".into(), ..Default::default()
        }).unwrap();

        // Note: playlist_tracks has FK to tracks, so we need tracks to exist
        // for the FK check. In practice, sync populates tracks first.
        // For this test, disable FK temporarily.
        conn.execute_batch("PRAGMA foreign_keys = OFF").unwrap();

        for i in 0..3 {
            add_track(&conn, &PlaylistTrackRow {
                backend: "navidrome".into(), playlist_id: "pl-1".into(),
                track_id: format!("tr-{i}"), position: i,
                added_at: "2024-01-01".into(),
            }).unwrap();
        }

        let tracks = get_tracks(&conn, "navidrome", "pl-1").unwrap();
        assert_eq!(tracks.len(), 3);
        assert_eq!(tracks[0].track_id, "tr-0");
        assert_eq!(tracks[2].position, 2);

        conn.execute_batch("PRAGMA foreign_keys = ON").unwrap();
    }
}
