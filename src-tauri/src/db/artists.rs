//! Artist CRUD operations — backend-agnostic.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

/// Artist row — generic across all backends.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtistRow {
    pub backend: String,
    pub id: String,
    pub title: String,
    pub title_sort: Option<String>,
    pub thumb: Option<String>,
    pub art: Option<String>,
    pub summary: Option<String>,
    pub user_rating: Option<f64>,
    pub added_at: Option<String>,
    pub updated_at: Option<String>,
    pub guid: Option<String>,
    /// Backend-specific data (JSON). Opaque to the DB layer.
    pub json_extra: String,
    pub tags: Vec<TagRow>,
}

/// A tag row from the shared `tags` table.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TagRow {
    pub tag_type: String,
    pub tag: String,
}

/// Upsert a single artist.
pub fn upsert(conn: &Connection, row: &ArtistRow) -> Result<(), String> {
    conn.execute(
        "INSERT INTO artists (backend, id, title, title_sort, thumb, art, summary,
            user_rating, added_at, updated_at, guid, json_extra)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
         ON CONFLICT(backend, id) DO UPDATE SET
            title=?3, title_sort=?4, thumb=?5, art=?6, summary=?7,
            user_rating=?8, added_at=?9, updated_at=?10, guid=?11, json_extra=?12",
        rusqlite::params![
            row.backend, row.id, row.title, row.title_sort,
            row.thumb, row.art, row.summary, row.user_rating,
            row.added_at, row.updated_at, row.guid, row.json_extra,
        ],
    )
    .map_err(|e| format!("upsert artist error: {e}"))?;

    // Replace tags
    delete_tags(conn, &row.backend, "artist", &row.id)?;
    for tag in &row.tags {
        insert_tag(conn, &row.backend, "artist", &row.id, &tag.tag_type, &tag.tag)?;
    }

    Ok(())
}

/// Upsert many artists in a single transaction.
pub fn upsert_bulk(conn: &Connection, rows: &[ArtistRow]) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("begin tx error: {e}"))?;
    for row in rows {
        upsert(&tx, row)?;
    }
    tx.commit().map_err(|e| format!("commit error: {e}"))?;
    Ok(())
}

/// Get a single artist by backend + ID.
pub fn get(conn: &Connection, backend: &str, id: &str) -> Result<Option<ArtistRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT backend, id, title, title_sort, thumb, art, summary,
                    user_rating, added_at, updated_at, guid, json_extra
             FROM artists WHERE backend = ?1 AND id = ?2",
        )
        .map_err(|e| format!("prepare error: {e}"))?;

    let row = stmt
        .query_row(rusqlite::params![backend, id], |row| {
            Ok(ArtistRow {
                backend: row.get(0)?,
                id: row.get(1)?,
                title: row.get(2)?,
                title_sort: row.get(3)?,
                thumb: row.get(4)?,
                art: row.get(5)?,
                summary: row.get(6)?,
                user_rating: row.get(7)?,
                added_at: row.get(8)?,
                updated_at: row.get(9)?,
                guid: row.get(10)?,
                json_extra: row.get(11)?,
                tags: Vec::new(),
            })
        })
        .optional()
        .map_err(|e| format!("query error: {e}"))?;

    match row {
        None => Ok(None),
        Some(mut a) => {
            a.tags = get_tags(conn, &a.backend, "artist", &a.id)?;
            Ok(Some(a))
        }
    }
}

/// Search artists by title within a backend.
pub fn search(conn: &Connection, backend: &str, query: &str, limit: i64) -> Result<Vec<ArtistRow>, String> {
    let pattern = format!("%{query}%");
    let mut stmt = conn
        .prepare(
            "SELECT backend, id, title, title_sort, thumb, art, summary,
                    user_rating, added_at, updated_at, guid, json_extra
             FROM artists
             WHERE backend = ?1 AND title LIKE ?2 COLLATE NOCASE
             ORDER BY title COLLATE NOCASE
             LIMIT ?3",
        )
        .map_err(|e| format!("prepare error: {e}"))?;

    let rows: Vec<ArtistRow> = stmt
        .query_map(rusqlite::params![backend, pattern, limit], |row| {
            Ok(ArtistRow {
                backend: row.get(0)?,
                id: row.get(1)?,
                title: row.get(2)?,
                title_sort: row.get(3)?,
                thumb: row.get(4)?,
                art: row.get(5)?,
                summary: row.get(6)?,
                user_rating: row.get(7)?,
                added_at: row.get(8)?,
                updated_at: row.get(9)?,
                guid: row.get(10)?,
                json_extra: row.get(11)?,
                tags: Vec::new(),
            })
        })
        .map_err(|e| format!("query error: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rows)
}

/// Count artists for a backend.
pub fn count(conn: &Connection, backend: &str) -> Result<i64, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM artists WHERE backend = ?1",
        [backend],
        |r| r.get(0),
    )
    .map_err(|e| format!("count error: {e}"))
}

/// Delete all artists for a backend.
pub fn delete_all(conn: &Connection, backend: &str) -> Result<i64, String> {
    // Tags must be cleaned manually (no FK from tags to artists).
    conn.execute(
        "DELETE FROM tags WHERE backend = ?1 AND entity_type = 'artist'",
        [backend],
    )
    .map_err(|e| format!("delete artist tags error: {e}"))?;

    let deleted = conn.execute(
        "DELETE FROM artists WHERE backend = ?1",
        [backend],
    )
    .map_err(|e| format!("delete artists error: {e}"))?;

    Ok(deleted as i64)
}

// ---- Tag helpers (shared across entity types) ----

/// Get tags for any entity.
pub fn get_tags(conn: &Connection, backend: &str, entity_type: &str, entity_id: &str) -> Result<Vec<TagRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT tag_type, tag FROM tags
             WHERE backend = ?1 AND entity_type = ?2 AND entity_id = ?3",
        )
        .map_err(|e| format!("prepare error: {e}"))?;
    let rows: Vec<TagRow> = stmt
        .query_map(rusqlite::params![backend, entity_type, entity_id], |row| {
            Ok(TagRow {
                tag_type: row.get(0)?,
                tag: row.get(1)?,
            })
        })
        .map_err(|e| format!("query error: {e}"))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// Insert a single tag.
pub fn insert_tag(
    conn: &Connection,
    backend: &str,
    entity_type: &str,
    entity_id: &str,
    tag_type: &str,
    tag: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT OR IGNORE INTO tags (backend, entity_type, entity_id, tag_type, tag)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![backend, entity_type, entity_id, tag_type, tag],
    )
    .map_err(|e| format!("insert tag error: {e}"))?;
    Ok(())
}

/// Delete all tags for an entity.
pub fn delete_tags(conn: &Connection, backend: &str, entity_type: &str, entity_id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM tags WHERE backend = ?1 AND entity_type = ?2 AND entity_id = ?3",
        rusqlite::params![backend, entity_type, entity_id],
    )
    .map_err(|e| format!("delete tags error: {e}"))?;
    Ok(())
}

/// rusqlite optional helper
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

    fn make_artist(backend: &str, id: &str, title: &str) -> ArtistRow {
        ArtistRow {
            backend: backend.to_string(),
            id: id.to_string(),
            title: title.to_string(),
            json_extra: "{}".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_artist_round_trip() {
        let db = DbState::open_in_memory().unwrap();
        let conn = db.0.lock().unwrap();

        let artist = make_artist("plex", "100", "Test Artist");
        upsert(&conn, &artist).unwrap();

        let loaded = get(&conn, "plex", "100").unwrap().expect("should exist");
        assert_eq!(loaded.id, "100");
        assert_eq!(loaded.backend, "plex");
        assert_eq!(loaded.title, "Test Artist");
    }

    #[test]
    fn test_no_cross_backend_collision() {
        let db = DbState::open_in_memory().unwrap();
        let conn = db.0.lock().unwrap();

        upsert(&conn, &make_artist("plex", "100", "Plex Artist")).unwrap();
        upsert(&conn, &make_artist("navidrome", "100", "Navidrome Artist")).unwrap();

        let plex = get(&conn, "plex", "100").unwrap().unwrap();
        let navi = get(&conn, "navidrome", "100").unwrap().unwrap();
        assert_eq!(plex.title, "Plex Artist");
        assert_eq!(navi.title, "Navidrome Artist");
    }

    #[test]
    fn test_upsert_overwrites_within_backend() {
        let db = DbState::open_in_memory().unwrap();
        let conn = db.0.lock().unwrap();

        upsert(&conn, &make_artist("plex", "200", "Original")).unwrap();
        upsert(&conn, &make_artist("plex", "200", "Updated")).unwrap();

        let loaded = get(&conn, "plex", "200").unwrap().unwrap();
        assert_eq!(loaded.title, "Updated");
        assert_eq!(count(&conn, "plex").unwrap(), 1);
    }

    #[test]
    fn test_search_scoped_to_backend() {
        let db = DbState::open_in_memory().unwrap();
        let conn = db.0.lock().unwrap();

        upsert(&conn, &make_artist("plex", "1", "Radiohead")).unwrap();
        upsert(&conn, &make_artist("navidrome", "2", "Radiohead")).unwrap();
        upsert(&conn, &make_artist("plex", "3", "Beach House")).unwrap();

        let plex_results = search(&conn, "plex", "radio", 10).unwrap();
        assert_eq!(plex_results.len(), 1);
        assert_eq!(plex_results[0].backend, "plex");

        let navi_results = search(&conn, "navidrome", "radio", 10).unwrap();
        assert_eq!(navi_results.len(), 1);
        assert_eq!(navi_results[0].backend, "navidrome");
    }

    #[test]
    fn test_artist_not_found() {
        let db = DbState::open_in_memory().unwrap();
        let conn = db.0.lock().unwrap();
        assert!(get(&conn, "plex", "999").unwrap().is_none());
    }

    #[test]
    fn test_tags() {
        let db = DbState::open_in_memory().unwrap();
        let conn = db.0.lock().unwrap();

        let mut artist = make_artist("plex", "300", "Tagged Artist");
        artist.tags = vec![
            TagRow { tag_type: "genre".into(), tag: "Rock".into() },
            TagRow { tag_type: "genre".into(), tag: "Alternative".into() },
        ];
        upsert(&conn, &artist).unwrap();

        let loaded = get(&conn, "plex", "300").unwrap().unwrap();
        assert_eq!(loaded.tags.len(), 2);
    }
}
