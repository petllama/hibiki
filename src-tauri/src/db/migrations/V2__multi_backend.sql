-- V2: Multi-backend schema.
--
-- Replaces all V1 tables with backend-aware versions:
--   - Composite primary keys: (backend, id)
--   - All IDs are TEXT (Plex numeric IDs stored as strings)
--   - Backend-specific data in json_extra columns
--   - Plex-only nested tables (media, media_parts, streams, lyrics,
--     album_reviews, artist_locations) removed — data goes in json_extra
--
-- Safe to drop V1 tables: they are currently unused from the frontend.

-- Drop V1 tables in dependency order (children first)
DROP TABLE IF EXISTS play_history;
DROP TABLE IF EXISTS playlist_tracks;
DROP TABLE IF EXISTS playlists;
DROP TABLE IF EXISTS streams;
DROP TABLE IF EXISTS media_parts;
DROP TABLE IF EXISTS media;
DROP TABLE IF EXISTS lyrics;
DROP TABLE IF EXISTS album_reviews;
DROP TABLE IF EXISTS artist_locations;
DROP TABLE IF EXISTS tags;
DROP TABLE IF EXISTS tracks;
DROP TABLE IF EXISTS albums;
DROP TABLE IF EXISTS artists;
-- kv table is backend-agnostic, keep it

-- ===== ARTISTS =====
CREATE TABLE artists (
    backend    TEXT NOT NULL,
    id         TEXT NOT NULL,
    title      TEXT NOT NULL DEFAULT '',
    title_sort TEXT,
    thumb      TEXT,
    art        TEXT,
    summary    TEXT,
    user_rating REAL,
    added_at   TEXT,
    updated_at TEXT,
    guid       TEXT,
    json_extra TEXT NOT NULL DEFAULT '{}',
    PRIMARY KEY (backend, id)
);

CREATE INDEX idx_artists_title ON artists(title COLLATE NOCASE);
CREATE INDEX idx_artists_backend ON artists(backend);

-- ===== ALBUMS =====
CREATE TABLE albums (
    backend     TEXT NOT NULL,
    id          TEXT NOT NULL,
    title       TEXT NOT NULL DEFAULT '',
    artist_id   TEXT,
    artist_name TEXT NOT NULL DEFAULT '',
    year        INTEGER NOT NULL DEFAULT 0,
    track_count INTEGER NOT NULL DEFAULT 0,
    studio      TEXT,
    thumb       TEXT,
    summary     TEXT,
    user_rating REAL,
    added_at    TEXT,
    updated_at  TEXT,
    guid        TEXT,
    genre       TEXT,
    json_extra  TEXT NOT NULL DEFAULT '{}',
    PRIMARY KEY (backend, id),
    FOREIGN KEY (backend, artist_id) REFERENCES artists(backend, id) ON DELETE SET NULL
);

CREATE INDEX idx_albums_title ON albums(title COLLATE NOCASE);
CREATE INDEX idx_albums_artist ON albums(backend, artist_id);
CREATE INDEX idx_albums_backend ON albums(backend);

-- ===== TRACKS =====
CREATE TABLE tracks (
    backend       TEXT NOT NULL,
    id            TEXT NOT NULL,
    title         TEXT NOT NULL DEFAULT '',
    track_number  INTEGER NOT NULL DEFAULT 0,
    duration      INTEGER NOT NULL DEFAULT 0,
    album_id      TEXT,
    album_name    TEXT NOT NULL DEFAULT '',
    artist_id     TEXT,
    artist_name   TEXT NOT NULL DEFAULT '',
    year          INTEGER NOT NULL DEFAULT 0,
    play_count    INTEGER NOT NULL DEFAULT 0,
    thumb         TEXT,
    codec         TEXT,
    bitrate       INTEGER,
    channels      INTEGER,
    bit_depth     INTEGER,
    sampling_rate INTEGER,
    user_rating   REAL,
    added_at      TEXT,
    updated_at    TEXT,
    guid          TEXT,
    json_extra    TEXT NOT NULL DEFAULT '{}',
    PRIMARY KEY (backend, id),
    FOREIGN KEY (backend, album_id) REFERENCES albums(backend, id) ON DELETE SET NULL,
    FOREIGN KEY (backend, artist_id) REFERENCES artists(backend, id) ON DELETE SET NULL
);

CREATE INDEX idx_tracks_title ON tracks(title COLLATE NOCASE);
CREATE INDEX idx_tracks_album ON tracks(backend, album_id);
CREATE INDEX idx_tracks_artist ON tracks(backend, artist_id);
CREATE INDEX idx_tracks_backend ON tracks(backend);

-- ===== PLAYLISTS =====
CREATE TABLE playlists (
    backend     TEXT NOT NULL,
    id          TEXT NOT NULL,
    title       TEXT NOT NULL DEFAULT '',
    smart       INTEGER NOT NULL DEFAULT 0,
    track_count INTEGER NOT NULL DEFAULT 0,
    duration    INTEGER,
    summary     TEXT,
    thumb       TEXT,
    added_at    TEXT,
    updated_at  TEXT,
    guid        TEXT,
    json_extra  TEXT NOT NULL DEFAULT '{}',
    PRIMARY KEY (backend, id)
);

CREATE INDEX idx_playlists_title ON playlists(title COLLATE NOCASE);
CREATE INDEX idx_playlists_backend ON playlists(backend);

-- ===== PLAYLIST TRACKS =====
CREATE TABLE playlist_tracks (
    backend     TEXT NOT NULL,
    playlist_id TEXT NOT NULL,
    track_id    TEXT NOT NULL,
    position    INTEGER NOT NULL,
    added_at    TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (backend, playlist_id, position),
    FOREIGN KEY (backend, playlist_id) REFERENCES playlists(backend, id) ON DELETE CASCADE,
    FOREIGN KEY (backend, track_id) REFERENCES tracks(backend, id) ON DELETE CASCADE
);

CREATE INDEX idx_playlist_tracks_track ON playlist_tracks(backend, track_id);

-- ===== TAGS =====
-- No FK constraint (references multiple entity tables).
-- Orphan cleanup handled in application code.
CREATE TABLE tags (
    backend     TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id   TEXT NOT NULL,
    tag_type    TEXT NOT NULL,
    tag         TEXT NOT NULL,
    PRIMARY KEY (backend, entity_type, entity_id, tag_type, tag)
);

CREATE INDEX idx_tags_lookup ON tags(tag_type, tag);
CREATE INDEX idx_tags_entity ON tags(backend, entity_type, entity_id);

-- ===== PLAY HISTORY =====
CREATE TABLE play_history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    backend     TEXT NOT NULL,
    track_id    TEXT NOT NULL,
    started_at  TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    completed   INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_play_history_track ON play_history(backend, track_id);
CREATE INDEX idx_play_history_started ON play_history(started_at);
