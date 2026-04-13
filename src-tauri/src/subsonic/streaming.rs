//! Streaming and cover art URL builders.
//!
//! These methods never make network requests — they build authenticated URLs
//! that the frontend audio element can hit directly.

use super::client::SubsonicClient;
use url::form_urlencoded;

fn pct_encode(s: &str) -> String {
    form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

impl SubsonicClient {
    /// Build a stream URL for a song.
    pub fn stream_url(&self, id: &str) -> String {
        self.build_url(
            "rest/stream.view",
            Some(&format!("id={}&format=raw", pct_encode(id))),
        )
    }

    /// Build a cover art URL.
    pub fn cover_art_url(&self, cover_art_id: &str, size: Option<i32>) -> String {
        let mut params = format!("id={}", pct_encode(cover_art_id));
        if let Some(s) = size {
            params.push_str(&format!("&size={}", s));
        }
        self.build_url("rest/getCoverArt.view", Some(&params))
    }
}
