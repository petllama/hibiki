//! Database schema management — runs SQL migrations on startup.

use rusqlite::Connection;

/// Run all pending migrations against the given connection.
///
/// Uses a simple `schema_version` pragma to track which migrations have run.
/// Each migration is an idempotent SQL script applied in order.
pub fn run_migrations(conn: &Connection) -> Result<(), String> {
    let current_version: i64 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|e| format!("Failed to read schema version: {e}"))?;

    let migrations: &[(i64, &str)] = &[
        (1, include_str!("migrations/V1__initial.sql")),
        (2, include_str!("migrations/V2__multi_backend.sql")),
    ];

    for &(version, sql) in migrations {
        if current_version < version {
            conn.execute_batch(sql)
                .map_err(|e| format!("Migration V{version} failed: {e}"))?;
            conn.pragma_update(None, "user_version", version)
                .map_err(|e| format!("Failed to update schema version to {version}: {e}"))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();

        // Run once
        run_migrations(&conn).expect("first run");
        let v: i64 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(v, 2);

        // Run again — should be a no-op
        run_migrations(&conn).expect("second run");
        let v2: i64 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(v2, 2);
    }

    #[test]
    fn test_all_tables_created() {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        run_migrations(&conn).expect("migrations");

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        let expected = vec![
            "albums",
            "artists",
            "kv",
            "play_history",
            "playlist_tracks",
            "playlists",
            "tags",
            "tracks",
        ];

        for name in &expected {
            assert!(
                tables.contains(&name.to_string()),
                "missing table: {name}"
            );
        }
    }
}
