//! Blueprint D24: the sidebar media card on the wire.

use bbr_web::media::MediaCommand;
use bbr_web::WebViewId;
use serde::{Deserialize, Serialize};

/// The card while it shows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaCardSnapshot {
    /// The shown session's identity.
    pub view: WebViewId,
    pub generation: u64,
    pub title: String,
    /// "host · Profile".
    pub source: String,
    pub playing: bool,
    pub muted: bool,
    /// Seconds, when the length is known.
    pub elapsed: Option<f64>,
    pub duration: Option<f64>,
    pub live: bool,
    pub seekable: bool,
    /// The drawn controls: `previous`, `play-pause`, `next`, `mute`.
    pub controls: Vec<String>,
    /// The `+N` switch (0: none).
    pub others: usize,
}

/// One live media session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaSessionEntry {
    pub view: WebViewId,
    pub generation: u64,
    /// `playing`, `paused`, `suspended`.
    pub state: String,
}

/// The media state of the window.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MediaSnapshot {
    pub card: Option<MediaCardSnapshot>,
    /// Every session, most recently started first.
    pub sessions: Vec<MediaSessionEntry>,
    /// The last command's outcome (`Applied`, `Stale`, `Ended`,
    /// `Unsupported`, `NotControllable`).
    pub last_outcome: Option<String>,
}

/// One card press through the method its control calls, or a raw command
/// for an explicit (possibly stale) session identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MediaInput {
    Source,
    Dismiss,
    Cycle,
    Previous,
    PlayPause,
    Next,
    ToggleMute,
    Seek {
        fraction: f64,
    },
    Command {
        view: WebViewId,
        generation: u64,
        command: MediaCommand,
    },
}
