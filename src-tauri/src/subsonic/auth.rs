//! Persistent settings management for Subsonic connections.
//!
//! Settings are stored as a JSON file in the OS app config directory,
//! separate from the Plex settings file.

use super::models::SubsonicSettings;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn settings_path(config_dir: &Path) -> PathBuf {
    config_dir.join("subsonic_settings.json")
}

pub fn load(config_dir: &Path) -> Result<SubsonicSettings> {
    let path = settings_path(config_dir);
    if !path.exists() {
        return Ok(SubsonicSettings::default());
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read subsonic settings from {:?}", path))?;
    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse subsonic settings from {:?}", path))
}

pub fn save(config_dir: &Path, settings: &SubsonicSettings) -> Result<()> {
    std::fs::create_dir_all(config_dir)
        .with_context(|| format!("Failed to create config dir {:?}", config_dir))?;
    let path = settings_path(config_dir);
    let content = serde_json::to_string_pretty(settings)
        .context("Failed to serialize subsonic settings")?;
    std::fs::write(&path, content)
        .with_context(|| format!("Failed to write subsonic settings to {:?}", path))
}
