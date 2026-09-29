//! Portable media-session capability contract (task packet
//! `media-session-capability-contract.md`).
//!
//! An engine adapter observes media playback per live web view and reports
//! it as a [`MediaObservation`]; the [`MediaSessionRegistry`] turns those
//! observations into sessions with an opaque, generation-checked
//! [`MediaSessionId`], and authorizes every [`MediaCommand`] against the
//! session's explicit [`MediaCapabilities`] before an adapter may apply it.
//!
//! Capabilities are facts the source proved, never guesses: the public
//! WebKit surface proves only a per-view playback state and Pause (see the
//! packet's API inventory); richer controls need an isolated content-world
//! bridge and arrive as further capability bits. A command carrying an old
//! generation is `Stale` and can never reach a newer page or session in the
//! same tab. This module is portable: no framework, no domain identities
//! (the app maps `WebViewId` to its tab/space/profile), no `unsafe`.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

use crate::WebViewId;

/// Opaque identity of one media session: the owning web view plus a
/// generation that changes whenever the session is replaced (navigation,
/// reload, crash, close, or playback ending and starting again).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MediaSessionId {
    pub view: WebViewId,
    pub generation: u64,
}

/// The playback fact an adapter observed for one web view.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaPlayback {
    /// No media playback exists in the view.
    None,
    Playing,
    Paused,
    /// Playback is suspended by the host (the page and the user cannot
    /// resume it until the host lifts the suspension).
    Suspended,
}

/// The state of an existing session (a view without playback has none).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaPlaybackState {
    Playing,
    Paused,
    Suspended,
}

/// The controls a session's source proved it supports. Absent controls are
/// absent in the UI; they are never simulated.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MediaCapabilities {
    pub play: bool,
    pub pause: bool,
    pub seek: bool,
    pub previous: bool,
    pub next: bool,
    pub mute: bool,
}

impl MediaCapabilities {
    /// What the public per-web-view WebKit API proves: Pause only.
    pub fn native_web_view() -> Self {
        Self {
            pause: true,
            ..Self::default()
        }
    }

    /// Whether `command` is covered by these capabilities.
    pub fn allows(&self, command: &MediaCommand) -> bool {
        match command {
            MediaCommand::Play => self.play,
            MediaCommand::Pause => self.pause,
            MediaCommand::Seek { .. } => self.seek,
            MediaCommand::Previous => self.previous,
            MediaCommand::Next => self.next,
            MediaCommand::Mute | MediaCommand::Unmute => self.mute,
        }
    }
}

/// Elapsed/duration in seconds, only when the source knows them. A live or
/// unknown duration is `None`, never a fake total.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MediaTiming {
    pub elapsed: Option<f64>,
    pub duration: Option<f64>,
    /// Blueprint D24: the source is a live stream (its length is endless,
    /// not merely unknown); the card says "Live" instead of a total.
    #[cfg_attr(feature = "serde", serde(default))]
    pub live: bool,
}

/// Display metadata the source reported (title, subtitle/site).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MediaMetadata {
    pub title: Option<String>,
    pub subtitle: Option<String>,
}

/// One adapter observation for a web view.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct MediaObservation {
    pub playback: MediaPlayback,
    pub capabilities: MediaCapabilities,
    pub timing: MediaTiming,
    pub metadata: MediaMetadata,
    /// Mute state, only when the source supports reading it.
    pub muted: Option<bool>,
}

impl MediaObservation {
    /// A native per-web-view observation: playback state plus the proven
    /// native capabilities, no timing, metadata, or mute state.
    pub fn native(playback: MediaPlayback) -> Self {
        Self {
            playback,
            capabilities: MediaCapabilities::native_web_view(),
            timing: MediaTiming::default(),
            metadata: MediaMetadata::default(),
            muted: None,
        }
    }
}

/// A live media session as the UI may present it.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct MediaSession {
    pub id: MediaSessionId,
    pub state: MediaPlaybackState,
    pub capabilities: MediaCapabilities,
    pub timing: MediaTiming,
    pub metadata: MediaMetadata,
    pub muted: Option<bool>,
}

/// Typed media commands. Every command targets one exact session.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaCommand {
    Play,
    Pause,
    Seek { position: f64 },
    Previous,
    Next,
    Mute,
    Unmute,
}

/// The outcome of a media command.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaCommandOutcome {
    /// The adapter applied the command to the bound session.
    Applied,
    /// The session's source does not support this command.
    Unsupported,
    /// The command carried an older generation than the view's session.
    Stale,
    /// No session exists for the view any more.
    Ended,
    /// The session exists but cannot take the command in its state (a seek
    /// position outside the known duration, or Play while the host holds a
    /// suspension).
    NotControllable,
    /// The adapter failed (typed message for diagnostics, never page data).
    Failed(String),
}

/// A registry change the app forwards to its read model.
#[derive(Debug, Clone, PartialEq)]
pub enum MediaSessionChange {
    /// A new session (new generation) appeared.
    Started(MediaSession),
    /// The existing session's state, metadata, or timing changed.
    Updated(MediaSession),
    /// The session ended or was invalidated; commands for it now fail.
    Ended(MediaSessionId),
}

#[derive(Debug, Clone)]
struct Entry {
    session: MediaSession,
    /// Monotonic stamp of the last start of playback (the recency order).
    recency: u64,
}

/// Generation-checked media sessions of one engine, keyed by web view.
#[derive(Debug, Default)]
pub struct MediaSessionRegistry {
    sessions: BTreeMap<WebViewId, Entry>,
    next_generation: u64,
    clock: u64,
}

impl MediaSessionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply one adapter observation for `view`. A view without a session
    /// that starts playback gets a fresh generation; `MediaPlayback::None`
    /// ends the session. Unknown/live durations never carry a seek
    /// capability.
    pub fn observe(
        &mut self,
        view: WebViewId,
        observation: MediaObservation,
    ) -> Option<MediaSessionChange> {
        let state = match observation.playback {
            MediaPlayback::None => return self.invalidate(view),
            MediaPlayback::Playing => MediaPlaybackState::Playing,
            MediaPlayback::Paused => MediaPlaybackState::Paused,
            MediaPlayback::Suspended => MediaPlaybackState::Suspended,
        };
        let mut capabilities = observation.capabilities;
        let known_duration = observation
            .timing
            .duration
            .is_some_and(|d| d.is_finite() && d > 0.0);
        if !known_duration {
            capabilities.seek = false;
        }
        if observation.muted.is_none() {
            capabilities.mute = false;
        }
        let playing = state == MediaPlaybackState::Playing;
        match self.sessions.get_mut(&view) {
            Some(entry) => {
                let was_playing = entry.session.state == MediaPlaybackState::Playing;
                let updated = MediaSession {
                    id: entry.session.id,
                    state,
                    capabilities,
                    timing: observation.timing,
                    metadata: observation.metadata,
                    muted: observation.muted,
                };
                if updated == entry.session {
                    return None;
                }
                if playing && !was_playing {
                    self.clock += 1;
                    entry.recency = self.clock;
                }
                entry.session = updated.clone();
                Some(MediaSessionChange::Updated(updated))
            }
            None => {
                self.next_generation += 1;
                self.clock += 1;
                let session = MediaSession {
                    id: MediaSessionId {
                        view,
                        generation: self.next_generation,
                    },
                    state,
                    capabilities,
                    timing: observation.timing,
                    metadata: observation.metadata,
                    muted: observation.muted,
                };
                self.sessions.insert(
                    view,
                    Entry {
                        session: session.clone(),
                        recency: self.clock,
                    },
                );
                Some(MediaSessionChange::Started(session))
            }
        }
    }

    /// End the view's session because its page or session was replaced
    /// (navigation, reload, crash, close, WebView replacement). The next
    /// observation starts a new generation.
    pub fn invalidate(&mut self, view: WebViewId) -> Option<MediaSessionChange> {
        self.sessions
            .remove(&view)
            .map(|entry| MediaSessionChange::Ended(entry.session.id))
    }

    /// The current session of `view`, if any.
    pub fn session(&self, view: WebViewId) -> Option<&MediaSession> {
        self.sessions.get(&view).map(|entry| &entry.session)
    }

    /// All sessions, most recently started playback first; ties resolve by
    /// view id so the order is deterministic.
    pub fn ordered(&self) -> Vec<&MediaSession> {
        let mut entries: Vec<&Entry> = self.sessions.values().collect();
        entries.sort_by(|a, b| {
            b.recency
                .cmp(&a.recency)
                .then(a.session.id.view.cmp(&b.session.id.view))
        });
        entries.into_iter().map(|entry| &entry.session).collect()
    }

    /// Authorize `command` for exactly `id` before an adapter applies it:
    /// the session must still exist with the same generation, its source
    /// must support the command, and its state must accept it.
    pub fn authorize(
        &self,
        id: MediaSessionId,
        command: &MediaCommand,
    ) -> Result<&MediaSession, MediaCommandOutcome> {
        let Some(session) = self.session(id.view) else {
            return Err(MediaCommandOutcome::Ended);
        };
        if session.id.generation != id.generation {
            return Err(MediaCommandOutcome::Stale);
        }
        if !session.capabilities.allows(command) {
            return Err(MediaCommandOutcome::Unsupported);
        }
        match command {
            MediaCommand::Seek { position } => {
                let duration = session.timing.duration.unwrap_or(0.0);
                if !position.is_finite() || *position < 0.0 || *position > duration {
                    return Err(MediaCommandOutcome::NotControllable);
                }
            }
            MediaCommand::Play if session.state == MediaPlaybackState::Suspended => {
                return Err(MediaCommandOutcome::NotControllable);
            }
            _ => {}
        }
        Ok(session)
    }
}

#[cfg(test)]
#[path = "media_tests.rs"]
mod tests;
