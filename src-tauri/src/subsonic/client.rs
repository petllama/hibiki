//! Subsonic API HTTP client with token-based authentication.
//!
//! Authentication: every request includes `u`, `t` (md5(password+salt)), `s` (salt),
//! `v` (API version), `c` (client name), `f=json`.

use anyhow::{Context, Result, bail};
use md5::{Md5, Digest};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::RetryTransientMiddleware;
use reqwest_retry::policies::ExponentialBackoff;
use serde::de::DeserializeOwned;
use tracing::debug;
use url::form_urlencoded;

fn pct_encode(s: &str) -> String {
    form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

use super::models::{SubsonicEnvelope, SubsonicResponse};

const API_VERSION: &str = "1.16.1";
const CLIENT_NAME: &str = "Hibiki";

/// Subsonic API client.
#[derive(Debug, Clone)]
pub struct SubsonicClient {
    pub base_url: String,
    pub username: String,
    /// md5(password + salt)
    token: String,
    salt: String,
    /// Raw password for re-keying with fresh salts if needed.
    password: String,
    client: ClientWithMiddleware,
}

impl SubsonicClient {
    /// Create a new client. Generates a fresh salt and token from the password.
    pub fn new(base_url: String, username: String, password: String) -> Result<Self> {
        let salt = Self::generate_salt();
        let token = Self::make_token(&password, &salt);

        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(
                std::time::Duration::from_millis(100),
                std::time::Duration::from_secs(3),
            )
            .build_with_max_retries(1);

        let raw = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .danger_accept_invalid_certs(true)
            .build()
            .context("Failed to build HTTP client")?;

        let client = ClientBuilder::new(raw)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Ok(Self { base_url, username, token, salt, password, client })
    }

    /// Build the auth query string appended to every request.
    fn auth_params(&self) -> String {
        format!(
            "u={}&t={}&s={}&v={}&c={}&f=json",
            pct_encode(&self.username),
            &self.token,
            &self.salt,
            API_VERSION,
            CLIENT_NAME,
        )
    }

    /// Make a GET request to a Subsonic endpoint and parse the response.
    ///
    /// `path` should be the endpoint path (e.g. "rest/ping.view").
    /// `extra_params` are additional query params (e.g. "id=123&count=50").
    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        extra_params: Option<&str>,
    ) -> Result<SubsonicResponse<T>> {
        let url = self.build_url(path, extra_params);
        debug!("GET {}", url);

        let resp = self.client.get(&url).send().await
            .context("Subsonic request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            bail!("Subsonic returned HTTP {}: {}", status, body);
        }

        let envelope: SubsonicEnvelope<T> = resp.json().await
            .context("Failed to parse Subsonic JSON response")?;

        if envelope.response.status != "ok" {
            if let Some(err) = &envelope.response.error {
                bail!("Subsonic error {}: {}", err.code, err.message);
            }
            bail!("Subsonic returned status: {}", envelope.response.status);
        }

        Ok(envelope.response)
    }

    /// Build a full URL for an endpoint.
    pub fn build_url(&self, path: &str, extra_params: Option<&str>) -> String {
        let base = self.base_url.trim_end_matches('/');
        let clean_path = path.trim_start_matches('/');
        match extra_params {
            Some(params) if !params.is_empty() =>
                format!("{}/{}?{}&{}", base, clean_path, self.auth_params(), params),
            _ =>
                format!("{}/{}?{}", base, clean_path, self.auth_params()),
        }
    }

    fn generate_salt() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("{:x}", nanos)
    }

    fn make_token(password: &str, salt: &str) -> String {
        let mut hasher = Md5::new();
        hasher.update(format!("{}{}", password, salt));
        format!("{:x}", hasher.finalize())
    }

    /// Get the raw password (needed for settings persistence).
    pub fn password(&self) -> &str { &self.password }
    pub fn token(&self) -> &str { &self.token }
    pub fn salt(&self) -> &str { &self.salt }
}
