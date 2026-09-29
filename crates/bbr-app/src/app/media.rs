//! Blueprint D24 (board S-8, DESIGN §11.15): the sidebar media card.
//!
//! The engine reports each live view's media ([`WebEvent::MediaObserved`]);
//! the portable [`MediaSessionRegistry`] turns those reports into sessions
//! with a generation-checked identity. A navigation commit, a crash or a
//! closed tab ends the view's session, so a command bound to the old page
//! is `Stale`/`Ended` and never reaches the new one. The card shows the
//! session the user last controlled while it lives, else the most recently
//! started one; `+N` cycles through the others, and Dismiss hides one
//! session until its identity changes (playback continues). The engine is
//! probed once a second while the window runs and right after a command.

use std::collections::BTreeSet;

use bbr_browser::GlobalTabSnapshot;
use bbr_ui::components::media_card::{
    MediaCardAction, MediaCardControls, MediaCardModel, MediaCardTiming,
};
use bbr_web::media::{
    MediaCommand, MediaCommandOutcome, MediaPlaybackState, MediaSession, MediaSessionId,
    MediaSessionRegistry,
};
use bbr_web::{WebEvent, WebViewId};
use gpui::{Context, SharedString, Window};

use crate::app::BbrApp;

/// Probe every fourth responsiveness tick (≈1 s).
const PROBE_EVERY_TICKS: u8 = 4;

#[derive(Default)]
pub(crate) struct MediaState {
    pub(crate) registry: MediaSessionRegistry,
    /// The session the user last controlled or switched to.
    pub(crate) current: Option<MediaSessionId>,
    /// Sessions the user dismissed (until their identity changes).
    pub(crate) dismissed: BTreeSet<MediaSessionId>,
    /// The last command's outcome (diagnostics and E2E).
    pub(crate) last_outcome: Option<MediaCommandOutcome>,
    ticks: u8,
}

impl BbrApp {
    /// Apply the media part of an engine event.
    pub(crate) fn media_web_event(&mut self, event: &WebEvent, cx: &mut Context<Self>) {
        let changed = match event {
            WebEvent::MediaObserved { view, observation } => self
                .media
                .registry
                .observe(*view, observation.clone())
                .is_some(),
            // A new document or a crashed page ends the view's session.
            WebEvent::DidCommitNavigation(view) | WebEvent::PageCrashed(view) => {
                self.media.registry.invalidate(*view).is_some()
            }
            _ => false,
        };
        if changed {
            self.prune_media();
            cx.notify();
        }
    }

    /// The periodic probe (driven by the responsiveness cadence).
    pub(crate) fn pump_media(&mut self) {
        self.media.ticks = (self.media.ticks + 1) % PROBE_EVERY_TICKS;
        if self.media.ticks == 0 {
            self.prune_media();
            self.engine.borrow_mut().probe_media();
        }
    }

    /// Drop sessions whose view left (a closed or discarded tab) and the
    /// dismissals of sessions that ended.
    fn prune_media(&mut self) {
        let gone: Vec<WebViewId> = self
            .media
            .registry
            .ordered()
            .iter()
            .map(|session| session.id.view)
            .filter(|view| self.media_owner(*view).is_none())
            .collect();
        for view in gone {
            self.media.registry.invalidate(view);
        }
        let registry = &self.media.registry;
        self.media
            .dismissed
            .retain(|id| registry.session(id.view).is_some_and(|s| s.id == *id));
    }

    /// The tab a view belongs to.
    fn media_owner(&self, view: WebViewId) -> Option<GlobalTabSnapshot> {
        self.controller
            .snapshot()
            .all_tabs
            .into_iter()
            .find(|tab| tab.web_view == Some(view))
    }

    /// The eligible sessions, most recent first (dismissed ones and views
    /// without a tab left out).
    fn media_sessions(&self) -> Vec<MediaSession> {
        self.media
            .registry
            .ordered()
            .into_iter()
            .filter(|session| !self.media.dismissed.contains(&session.id))
            .filter(|session| self.media_owner(session.id.view).is_some())
            .cloned()
            .collect()
    }

    /// The session the card shows.
    pub(crate) fn media_shown(&self) -> Option<(MediaSession, usize)> {
        let sessions = self.media_sessions();
        let shown = self
            .media
            .current
            .and_then(|id| sessions.iter().find(|s| s.id == id))
            .or(sessions.first())?
            .clone();
        Some((shown, sessions.len() - 1))
    }

    /// The card's presentation (`None`: no card).
    pub(crate) fn media_card_model(&self) -> Option<MediaCardModel> {
        let (session, others) = self.media_shown()?;
        let owner = self.media_owner(session.id.view)?;
        let snapshot = self.controller.snapshot();
        let profile = snapshot
            .profiles
            .iter()
            .find(|p| p.id == owner.profile)
            .map(|p| p.title.clone())
            .unwrap_or_default();
        let host = bbr_ui::text::host_of(&owner.url).to_string();
        let title = session
            .metadata
            .title
            .clone()
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| owner.title.clone());
        let caps = session.capabilities;
        let timing = match (session.timing.duration, session.timing.live) {
            (Some(duration), _) => MediaCardTiming::Known {
                elapsed: session.timing.elapsed.unwrap_or(0.0),
                duration,
                seekable: caps.seek,
            },
            (None, true) => MediaCardTiming::Live,
            (None, false) => MediaCardTiming::Unknown,
        };
        let playing = session.state == MediaPlaybackState::Playing;
        Some(MediaCardModel {
            title: SharedString::from(title),
            source: SharedString::from(match (host.is_empty(), profile.is_empty()) {
                (false, false) => format!("{host} · {profile}"),
                (false, true) => host.clone(),
                _ => profile,
            }),
            favicon: bbr_ui::components::FaviconSpec {
                letter: bbr_ui::components::favicon::monogram_letter(&owner.url),
                hue: bbr_ui::components::favicon::monogram_hue(&owner.url),
                image: self.favicon_image(owner.profile, &host),
            },
            playing,
            muted: session.muted.unwrap_or(false),
            timing,
            controls: MediaCardControls {
                previous: caps.previous,
                play_pause: if playing { caps.pause } else { caps.play },
                next: caps.next,
                mute: caps.mute,
            },
            others,
        })
    }

    /// A press on the card (the production listeners and the testkit).
    pub(crate) fn on_media_card(
        &mut self,
        action: MediaCardAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((session, _)) = self.media_shown() else {
            return;
        };
        let command = match action {
            MediaCardAction::Source => {
                self.media.current = Some(session.id);
                self.media_show_source(session.id.view, window, cx);
                return;
            }
            MediaCardAction::Dismiss => {
                self.media.dismissed.insert(session.id);
                if self.media.current == Some(session.id) {
                    self.media.current = None;
                }
                cx.notify();
                return;
            }
            MediaCardAction::Cycle => {
                let sessions = self.media_sessions();
                if let Some(at) = sessions.iter().position(|s| s.id == session.id) {
                    self.media.current = Some(sessions[(at + 1) % sessions.len()].id);
                }
                cx.notify();
                return;
            }
            MediaCardAction::Previous => MediaCommand::Previous,
            MediaCardAction::Next => MediaCommand::Next,
            MediaCardAction::PlayPause => {
                if session.state == MediaPlaybackState::Playing {
                    MediaCommand::Pause
                } else {
                    MediaCommand::Play
                }
            }
            MediaCardAction::ToggleMute => {
                if session.muted == Some(true) {
                    MediaCommand::Unmute
                } else {
                    MediaCommand::Mute
                }
            }
            MediaCardAction::Seek(fraction) => MediaCommand::Seek {
                position: (fraction.clamp(0.0, 1.0) * session.timing.duration.unwrap_or(0.0))
                    .floor(),
            },
        };
        self.media.current = Some(session.id);
        self.media_command(session.id, command);
        cx.notify();
    }

    /// Authorize `command` for `id` and hand it to the engine.
    pub(crate) fn media_command(&mut self, id: MediaSessionId, command: MediaCommand) {
        let outcome = match self.media.registry.authorize(id, &command) {
            Err(refusal) => refusal,
            Ok(_) if self.media_owner(id.view).is_none() => MediaCommandOutcome::Ended,
            Ok(_) => {
                let mut engine = self.engine.borrow_mut();
                engine.media_command(id.view, command);
                // Report the new state without waiting for the next tick.
                engine.probe_media();
                MediaCommandOutcome::Applied
            }
        };
        self.media.last_outcome = Some(outcome);
    }

    /// Show the playing tab: its space (and profile) and the exact tab.
    fn media_show_source(&mut self, view: WebViewId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(owner) = self.media_owner(view) else {
            return;
        };
        if self.controller.snapshot().active_space != owner.space {
            if self.controller.activate_space(owner.space).is_err() {
                return;
            }
            self.emit_browser(&bbr_browser::ControllerEvent::SpaceActivated { space: owner.space });
        }
        if self.controller.select_tab(owner.index).is_ok() {
            self.emit_browser(&bbr_browser::ControllerEvent::TabSelected { index: owner.index });
        }
        self.clear_command_profile_hint();
        self.sync_command_bar();
        self.ensure_active_visible();
        self.focus_tab_list(window, cx);
        cx.notify();
    }
}
