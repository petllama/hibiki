//! Album CRUD operations — backend-agnostic.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use super::artists::{get_tags, delete_tags, insert_tag, TagRow};

/// Album row — generic across all backends.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlbumRow {
    pub backend: String,
    pub id: String,
    pub title: String,
    pub artist_id: Option<String>,
    pub artist_name: String,
    pub year: i64,
    pub track_count: i64,
    pub studio: Option<String>,
    pub thumb: Option<String>,
    pub summary: Option<String>,
    pub user_rating: Option<f64>,
    pub added_at: Option<String>,
    pub updated_at: Option<String>,
    pub guid: Option<String>,
    pub genre: Option<String>,
    pub json_extra: String,
    pub tags: Vec<TagRow>,
}

/// Upsert a single album.
pub fn upsert(conn: &Connection, row: &AlbumRow) -> Result<(), String> {
    conn.execute(
        "INSERT INTO albums (backend, id, title, artist_id, artist_name, year,
            track_count, studio, thumb, summary, user_rating, added_at, updated_at,
            guid, genre, json_extra)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)
         ON CONFLICT(backend, id) DO UPDATE SET
            title=?3, artist_id=?4, artist_name=?5, year=?6, track_count=?7,
            studio=?8, thumb=?9, summary=?10, user_rating=?11, added_at=?12,
            updated_at=?13, guid=?14, genre=?15, json_extra=?16",
        rusqlite::params![
            row.backend, row.id, row.title, row.artist_id, row.artist_name,
            row.year, row.track_count, row.studio, row.thumb, row.summary,
            row.user_rating, row.added_at, row.updated_at, row.guid,
            row.genre, row.json_extra,
        ],
    )
    .map_err(|e| format!("upsert album error: {e}"))?;

    // Replace tags
    delete_tags(conn, &row.backend, "album", &row.id)?;
    for tag in &row.tags {
        insert_tag(conn, &row.backend, "album", &row.id, &tag.tag_type, &tag.tag)?;
    }

    Ok(())
}

/// Upsert many albums in a single transaction.
pub fn upsert_bulk(conn: &Connection, rows: &[AlbumRow]) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("begin tx error: {e}"))?;
    for row in rows {
        upsert(&tx, row)?;
    }
    tx.commit().map_err(|e| format!("commit error: {e}"))?;
    Ok(())
}

/// Get a single album by backend + ID.
pub fn get(conn: &Connection, backend: &str, id: &str) -> Result<Option<AlbumRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT backend, id, title, artist_id, artist_name, year, track_count,
                    studio, thumb, summary, user_rating, added_at, updated_at,
                    guid, genre, json_extra
             FROM albums WHERE backend = ?1 AND id = ?2",
        )
        .map_err(|e| format!("prepare error: {e}"))?;

    let row = stmt
        .query_row(rusqlite::params![backend, id], map_album_row)
        .optional()
        .map_err(|e| format!("query error: {e}"))?;

    match row {
        None => Ok(None),
        Some(mut a) => {
            a.tags = get_tags(conn, &a.backend, "album", &a.id)?;
            Ok(Some(a))
        }
    }
}

/// Search albums by title within a backend.
pub fn search(conn: &Connection, backend: &str, query: &str, limit: i64) -> Result<Vec<AlbumRow>, String> {
    let pattern = format!("%{query}%");
    let mut stmt = conn
        .prepare(
            "SELECT backend, id, title, artist_id, artist_name, year, track_count,
                    studio, thumb, summary, user_rating, added_at, updated_at,
                    guid, genre, json_extra
             FROM albums
             WHERE backend = ?1 AND title LIKE ?2 COLLATE NOCASE
             ORDER BY title COLLATE NOCASE
             LIMIT ?3",
        )
        .map_err(|e| format!("prepare error: {e}"))?;

    let rows: Vec<AlbumRow> = stmt
        .query_map(rusqlite::params![backend, pattern, limit], map_album_row)
        .map_err(|e| format!("query error: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

/// Get all albums for an artist within a backend.
pub fn get_by_artist(conn: &Connection, backend: &str, artist_id: &str) -> Result<Vec<AlbumRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT backend, id, title, artist_id, artist_name, year, track_count,
                    studio, thumb, summary, user_rating, added_at, updated_at,
                    guid, genre, json_extra
             FROM albums
             WHERE backend = ?1 AND artist_id = ?2
             ORDER BY year",
        )
        .map_err(|e| format!("prepare error: {e}"))?;

    let rows: Vec<AlbumRow> = stmt
        .query_map(rusqlite::params![backend, artist_id], map_album_row)
        .map_err(|e| format!("query error: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

/// Count albums for a backend.
pub fn count(conn: &Connection, backend: &str) -> Result<i64, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM albums WHERE backend = ?1",
        [backend],
        |r| r.get(0),
    )
    .map_err(|e| format!("count error: {e}"))
}

/// Delete all albums for a backend.
pub fn delete_all(conn: &Connection, backend: &str) -> Result<i64, String> {
    conn.execute(
        "DELETE FROM tags WHERE backend = ?1 AND entity_type = 'album'",
        [backend],
    )
    .map_err(|e| format!("delete album tags error: {e}"))?;

    let deleted = conn.execute(
        "DELETE FROM albums WHERE backend = ?1",
        [backend],
    )
    .map_err(|e| format!("delete albums error: {e}"))?;

    Ok(deleted as i64)
}

fn map_album_row(row: &rusqlite::Row) -> rusqlite::Result<AlbumRow> {
    Ok(AlbumRow {
        backend: row.get(0)?,
        id: row.get(1)?,
        title: row.get(2)?,
        artist_id: row.get(3)?,
        artist_name: row.get(4)?,
        year: row.get(5)?,
        track_count: row.get(6)?,
        studio: row.get(7)?,
        thumb: row.get(8)?,
        summary: row.get(9)?,
        user_rating: row.get(10)?,
        added_at: row.get(11)?,
        updated_at: row.get(12)?,
        guid: row.get(13)?,
        genre: row.get(14)?,
        json_extra: row.get(15)?,
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
    use crate::db::artists;

    #[test]
    fn test_album_round_trip() {
        let db = DbState::open_in_memory().unwrap();
        let conn = db.0.lock().unwrap();

        // Create parent artist first (FK)
        artists::upsert(&conn, &artists::ArtistRow {
            backend: "plex".into(), id: "50".into(), title: "Artist".into(),
            json_extra: "{}".into(), ..Default::default()
        }).unwrap();

        let album = AlbumRow {
            backend: "plex".into(), id: "100".into(), title: "Test Album".into(),
            artist_id: Some("50".into()), artist_name: "Artist".into(),
            year: 2024, track_count: 12, json_extra: "{}".into(),
            ..Default::default()
        };
        upsert(&conn, &album).unwrap();

        let loaded = get(&conn, "plex", "100").unwrap().expect("should exist");
        assert_eq!(loaded.title, "Test Album");
        assert_eq!(loaded.artist_id, Some("50".into()));
    }

    #[test]
    fn test_albums_by_artist() {
        let db = DbState::open_in_memory().unwrap();
        let conn = db.0.lock().unwrap();

        artists::upsert(&conn, &artists::ArtistRow {
            backend: "navidrome".into(), id: "ar-1".into(), title: "Band".into(),
            json_extra: "{}".into(), ..Default::default()
        }).unwrap();

        for i in 1..=3 {
            upsert(&conn, &AlbumRow {
                backend: "navidrome".into(), id: format!("al-{i}"),
                title: format!("Album {i}"), artist_id: Some("ar-1".into()),
                artist_name: "Band".into(), year: 2020 + i,
                json_extra: "{}".into(), ..Default::default()
            }).unwrap();
        }

        let albums = get_by_artist(&conn, "navidrome", "ar-1").unwrap();
        assert_eq!(albums.len(), 3);
        assert_eq!(albums[0].year, 2021); // ordered by year
    }
}
