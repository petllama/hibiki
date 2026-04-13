//! Subsonic/OpenSubsonic API client for Navidrome and compatible servers.

pub mod auth;
pub mod client;
pub mod library;
pub mod models;
pub mod streaming;

pub use client::SubsonicClient;
pub use models::*;
