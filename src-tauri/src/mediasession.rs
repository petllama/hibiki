//! System-wide Now Playing integration via souvlaki.
//!
//! A dedicated background thread owns the `MediaControls` handle.
//! Tauri commands send updates through a bounded crossbeam channel.
//!
//! Platform support:
//! - macOS : MPRemoteCommandCenter + MPNowPlayingInfoCenter
//! - Windows: System Media Transport Controls (SMTC)
//! - Linux  : MPRIS2 via D-Bus

use crossbeam_channel::{bounded, Sender};
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
#[cfg(target_os = "windows")]
use tauri::Manager;
use tauri::{AppHandle, Emitter, Runtime};

/// How long after waking from sleep to keep suppressing media events (ms).
const WAKE_SUPPRESS_MS: u64 = 5000;

/// Spawn a thread that listens for `PrepareForSleep` on the system D-Bus.
/// Sets `suppress` to true when the system is going to sleep, and keeps it
/// true for a few seconds after waking so stale MPRIS events are ignored.
#[cfg(target_os = "linux")]
fn spawn_sleep_listener(suppress: Arc<AtomicBool>) {
    std::thread::Builder::new()
        .name("sleep-listener".into())
        .spawn(move || {
            let conn = match dbus::blocking::Connection::new_system() {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("Failed to connect to system D-Bus for sleep detection: {e}");
                    return;
                }
            };

            // Subscribe to the PrepareForSleep signal from logind
            let proxy = conn.with_proxy(
                "org.freedesktop.login1",
                "/org/freedesktop/login1",
                Duration::from_secs(5),
            );
            use dbus::message::MatchRule;
            let rule = MatchRule::new_signal("org.freedesktop.login1.Manager", "PrepareForSleep");
            if let Err(e) = conn.add_match(rule, move |_: (), _conn, msg| {
                // PrepareForSleep(bool): true = going to sleep, false = waking up
                if let Some(going_to_sleep) = msg.get1::<bool>() {
                    if going_to_sleep {
                        tracing::debug!("system going to sleep — suppressing media events");
                        suppress.store(true, Ordering::Relaxed);
                    } else {
                        tracing::debug!("system waking — suppressing media events for {}ms", WAKE_SUPPRESS_MS);
                        // Keep suppress=true, clear it after a delay
                        let suppress_clone = suppress.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(Duration::from_millis(WAKE_SUPPRESS_MS));
                            suppress_clone.store(false, Ordering::Relaxed);
                            tracing::debug!("sleep suppress window ended");
                        });
                    }
                }
                true // keep the match active
            }) {
                tracing::warn!("Failed to subscribe to PrepareForSleep: {e}");
                return;
            }

            // Block forever processing D-Bus messages
            loop {
                conn.process(Duration::from_secs(60)).ok();
            }
        })
        .ok();
}

#[cfg(not(target_os = "linux"))]
fn spawn_sleep_listener(_suppress: Arc<AtomicBool>) {
    // No-op on non-Linux platforms
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Messages sent from Tauri commands to the background souvlaki thread.
pub enum MediaUpdate {
    Metadata {
        title: String,
        artist: String,
        album: String,
        /// Full authenticated Plex thumbnail URL, built by the Rust command.
        cover_url: Option<String>,
        duration_ms: u64,
    },
    Playing {
        position_ms: u64,
    },
    Paused {
        position_ms: u64,
    },
    Stopped,
}

/// Tauri-managed state: sender half of the channel to the background thread.
pub struct MediaSessionState(pub Sender<MediaUpdate>);

impl MediaSessionState {
    /// Initialise system media controls and return the managed state.
    ///
    /// Spawns a background thread that owns the `MediaControls` instance.
    /// Media-key events from the OS are forwarded to the frontend as Tauri events.
    pub fn start<R: Runtime>(app: &AppHandle<R>) -> Self {
        let (tx, rx) = bounded::<MediaUpdate>(64);

        // Capture what we need before the thread move.
        let app_handle = app.clone();

        // On Windows souvlaki requires a window handle to attach SMTC.
        // Grab it on this thread (setup = main thread) before spawning.
        #[cfg(target_os = "windows")]
        let hwnd_isize: isize = app
            .get_webview_window("main")
            .and_then(|w| w.hwnd().ok())
            .map(|h| h.0 as isize)
            .unwrap_or(0);
        #[cfg(not(target_os = "windows"))]
        let hwnd_isize: isize = 0;

        // Sleep/wake event suppression flag
        let sleep_suppress = Arc::new(AtomicBool::new(false));
        spawn_sleep_listener(sleep_suppress.clone());
        let suppress_for_cb = sleep_suppress;

        std::thread::Builder::new()
            .name("media-session".into())
            .spawn(move || {
                let hwnd = if hwnd_isize != 0 {
                    Some(hwnd_isize as *mut std::ffi::c_void)
                } else {
                    None
                };

                let config = PlatformConfig {
                    dbus_name: "hibiki",
                    display_name: "Hibiki",
                    hwnd,
                };

                let mut controls = match MediaControls::new(config) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::error!("MediaControls::new failed: {:?}", e);
                        return;
                    }
                };

                // Forward OS media-key events to the frontend.
                controls
                    .attach(move |event: MediaControlEvent| {
                        match event {
                            MediaControlEvent::Play
                            | MediaControlEvent::Pause
                            | MediaControlEvent::Toggle => {
                                if suppress_for_cb.load(Ordering::Relaxed) {
                                    tracing::debug!("suppressing media event during sleep/wake");
                                    return;
                                }
                                app_handle.emit("media://play-pause", ()).ok();
                            }
                            MediaControlEvent::Next => {
                                app_handle.emit("media://next", ()).ok();
                            }
                            MediaControlEvent::Previous => {
                                app_handle.emit("media://previous", ()).ok();
                            }
                            MediaControlEvent::Stop => {
                                app_handle.emit("media://stop", ()).ok();
                            }
                            MediaControlEvent::SetPosition(pos) => {
                                app_handle
                                    .emit("media://seek", pos.0.as_millis() as u64)
                                    .ok();
                            }
                            _ => {}
                        }
                    })
                    .ok();

                // Process metadata/playback updates from Tauri commands.
                while let Ok(update) = rx.recv() {
                    match update {
                        MediaUpdate::Metadata {
                            title,
                            artist,
                            album,
                            cover_url,
                            duration_ms,
                        } => {
                            controls
                                .set_metadata(MediaMetadata {
                                    title: Some(title.as_str()),
                                    artist: Some(artist.as_str()),
                                    album: Some(album.as_str()),
                                    cover_url: cover_url.as_deref(),
                                    duration: Some(Duration::from_millis(duration_ms)),
                                })
                                .ok();
                        }
                        MediaUpdate::Playing { position_ms } => {
                            controls
                                .set_playback(MediaPlayback::Playing {
                                    progress: Some(MediaPosition(Duration::from_millis(
                                        position_ms,
                                    ))),
                                })
                                .ok();
                        }
                        MediaUpdate::Paused { position_ms } => {
                            controls
                                .set_playback(MediaPlayback::Paused {
                                    progress: Some(MediaPosition(Duration::from_millis(
                                        position_ms,
                                    ))),
                                })
                                .ok();
                        }
                        MediaUpdate::Stopped => {
                            controls.set_playback(MediaPlayback::Stopped).ok();
                        }
                    }
                }
            })
            .expect("Failed to spawn media-session thread");

        Self(tx)
    }
}
