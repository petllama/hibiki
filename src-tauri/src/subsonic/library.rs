//! Library browsing and search endpoints.

use anyhow::Result;
use url::form_urlencoded;

use super::client::SubsonicClient;
use super::models::*;

fn pct_encode(s: &str) -> String {
    form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

impl SubsonicClient {
    /// Test the connection (ping.view).
    pub async fn ping(&self) -> Result<()> {
        self.get::<PingBody>("rest/ping.view", None).await?;
        Ok(())
    }

    /// Get all artists (getArtists.view).
    pub async fn get_artists(&self) -> Result<Vec<SubsonicArtist>> {
        let resp = self.get::<ArtistsBody>("rest/getArtists.view", None).await?;
        let indexes = resp.body.artists.map(|a| a.index).unwrap_or_default();
        Ok(indexes.into_iter().flat_map(|idx| idx.artist).collect())
    }

    /// Get a single artist with their albums (getArtist.view).
    pub async fn get_artist(&self, id: &str) -> Result<ArtistDetail> {
        let resp = self.get::<ArtistDetailBody>(
            "rest/getArtist.view",
            Some(&format!("id={}", pct_encode(id))),
        ).await?;
        resp.body.artist.ok_or_else(|| anyhow::anyhow!("Artist not found"))
    }

    /// Get a single album with its songs (getAlbum.view).
    pub async fn get_album(&self, id: &str) -> Result<AlbumDetail> {
        let resp = self.get::<AlbumDetailBody>(
            "rest/getAlbum.view",
            Some(&format!("id={}", pct_encode(id))),
        ).await?;
        resp.body.album.ok_or_else(|| anyhow::anyhow!("Album not found"))
    }

    /// Get a single song (getSong.view).
    pub async fn get_song(&self, id: &str) -> Result<SubsonicSong> {
        // Subsonic wraps single song in {"song": {...}}
        #[derive(serde::Deserialize)]
        struct SongBody { song: Option<SubsonicSong> }
        let resp = self.get::<SongBody>(
            "rest/getSong.view",
            Some(&format!("id={}", pct_encode(id))),
        ).await?;
        resp.body.song.ok_or_else(|| anyhow::anyhow!("Song not found"))
    }

    /// Search across artists, albums, and songs (search3.view).
    pub async fn search3(
        &self,
        query: &str,
        artist_count: Option<i32>,
        album_count: Option<i32>,
        song_count: Option<i32>,
    ) -> Result<SearchResult3> {
        let mut params = format!("query={}", pct_encode(query));
        if let Some(n) = artist_count { params.push_str(&format!("&artistCount={}", n)); }
        if let Some(n) = album_count { params.push_str(&format!("&albumCount={}", n)); }
        if let Some(n) = song_count { params.push_str(&format!("&songCount={}", n)); }
        let resp = self.get::<SearchResult3Body>("rest/search3.view", Some(&params)).await?;
        Ok(resp.body.search_result3.unwrap_or(SearchResult3 {
            artist: vec![], album: vec![], song: vec![],
        }))
    }

    /// Get a list of albums by type (getAlbumList2.view).
    ///
    /// `list_type`: "newest", "recent", "frequent", "highest", "starred",
    /// "alphabeticalByName", "alphabeticalByArtist", "byGenre", "byYear", "random".
    pub async fn get_album_list2(
        &self,
        list_type: &str,
        size: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<SubsonicAlbum>> {
        let mut params = format!("type={}", pct_encode(list_type));
        if let Some(n) = size { params.push_str(&format!("&size={}", n)); }
        if let Some(n) = offset { params.push_str(&format!("&offset={}", n)); }
        let resp = self.get::<AlbumList2Body>("rest/getAlbumList2.view", Some(&params)).await?;
        Ok(resp.body.album_list2.map(|a| a.album).unwrap_or_default())
    }

    /// Get top songs for an artist (getTopSongs.view).
    pub async fn get_top_songs(&self, artist_name: &str, count: Option<i32>) -> Result<Vec<SubsonicSong>> {
        let mut params = format!("artist={}", pct_encode(artist_name));
        if let Some(n) = count { params.push_str(&format!("&count={}", n)); }
        let resp = self.get::<TopSongsBody>("rest/getTopSongs.view", Some(&params)).await?;
        Ok(resp.body.top_songs.map(|t| t.song).unwrap_or_default())
    }

    /// Get similar songs (getSimilarSongs2.view).
    pub async fn get_similar_songs2(&self, id: &str, count: Option<i32>) -> Result<Vec<SubsonicSong>> {
        let mut params = format!("id={}", pct_encode(id));
        if let Some(n) = count { params.push_str(&format!("&count={}", n)); }
        let resp = self.get::<SimilarSongs2Body>("rest/getSimilarSongs2.view", Some(&params)).await?;
        Ok(resp.body.similar_songs2.map(|s| s.song).unwrap_or_default())
    }

    /// Get all playlists (getPlaylists.view).
    pub async fn get_playlists(&self) -> Result<Vec<SubsonicPlaylist>> {
        let resp = self.get::<PlaylistsBody>("rest/getPlaylists.view", None).await?;
        Ok(resp.body.playlists.map(|p| p.playlist).unwrap_or_default())
    }

    /// Get a playlist with its entries (getPlaylist.view).
    pub async fn get_playlist(&self, id: &str) -> Result<PlaylistDetail> {
        let resp = self.get::<PlaylistDetailBody>(
            "rest/getPlaylist.view",
            Some(&format!("id={}", pct_encode(id))),
        ).await?;
        resp.body.playlist.ok_or_else(|| anyhow::anyhow!("Playlist not found"))
    }

    /// Create a playlist (createPlaylist.view).
    pub async fn create_playlist(&self, name: &str, song_ids: &[String]) -> Result<SubsonicPlaylist> {
        let mut params = format!("name={}", pct_encode(name));
        for id in song_ids {
            params.push_str(&format!("&songId={}", pct_encode(id)));
        }
        // createPlaylist returns the created playlist
        let resp = self.get::<PlaylistDetailBody>(
            "rest/createPlaylist.view",
            Some(&params),
        ).await?;
        let detail = resp.body.playlist.ok_or_else(|| anyhow::anyhow!("Failed to create playlist"))?;
        Ok(SubsonicPlaylist {
            id: detail.id,
            name: detail.name,
            song_count: detail.song_count,
            duration: detail.duration,
            owner: detail.owner,
            created: detail.created,
            changed: detail.changed,
            cover_art: detail.cover_art,
            is_public: None,
        })
    }

    /// Update a playlist (updatePlaylist.view).
    /// Pass song_ids_to_add and/or song_indexes_to_remove.
    pub async fn update_playlist(
        &self,
        id: &str,
        name: Option<&str>,
        song_ids_to_add: &[String],
        song_indexes_to_remove: &[i32],
    ) -> Result<()> {
        let mut params = format!("playlistId={}", pct_encode(id));
        if let Some(n) = name {
            params.push_str(&format!("&name={}", pct_encode(n)));
        }
        for sid in song_ids_to_add {
            params.push_str(&format!("&songIdToAdd={}", pct_encode(sid)));
        }
        for idx in song_indexes_to_remove {
            params.push_str(&format!("&songIndexToRemove={}", idx));
        }
        self.get::<PingBody>("rest/updatePlaylist.view", Some(&params)).await?;
        Ok(())
    }

    /// Delete a playlist (deletePlaylist.view).
    pub async fn delete_playlist(&self, id: &str) -> Result<()> {
        self.get::<PingBody>(
            "rest/deletePlaylist.view",
            Some(&format!("id={}", pct_encode(id))),
        ).await?;
        Ok(())
    }

    /// Star an item (star.view).
    pub async fn star(&self, id: &str) -> Result<()> {
        self.get::<PingBody>(
            "rest/star.view",
            Some(&format!("id={}", pct_encode(id))),
        ).await?;
        Ok(())
    }

    /// Unstar an item (unstar.view).
    pub async fn unstar(&self, id: &str) -> Result<()> {
        self.get::<PingBody>(
            "rest/unstar.view",
            Some(&format!("id={}", pct_encode(id))),
        ).await?;
        Ok(())
    }

    /// Set rating on an item (setRating.view). Rating 0-5.
    pub async fn set_rating(&self, id: &str, rating: i32) -> Result<()> {
        self.get::<PingBody>(
            "rest/setRating.view",
            Some(&format!("id={}&rating={}", pct_encode(id), rating)),
        ).await?;
        Ok(())
    }

    /// Scrobble a track (scrobble.view).
    pub async fn scrobble(&self, id: &str, submission: bool) -> Result<()> {
        self.get::<PingBody>(
            "rest/scrobble.view",
            Some(&format!("id={}&submission={}", pct_encode(id), submission)),
        ).await?;
        Ok(())
    }

    /// Get starred items (getStarred2.view).
    pub async fn get_starred2(&self) -> Result<Starred2> {
        let resp = self.get::<Starred2Body>("rest/getStarred2.view", None).await?;
        Ok(resp.body.starred2.unwrap_or(Starred2 {
            artist: vec![], album: vec![], song: vec![],
        }))
    }

    /// Get genres (getGenres.view).
    pub async fn get_genres(&self) -> Result<Vec<SubsonicGenre>> {
        let resp = self.get::<GenresBody>("rest/getGenres.view", None).await?;
        Ok(resp.body.genres.map(|g| g.genre).unwrap_or_default())
    }
}
