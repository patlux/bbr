//! Blueprint D24 testkit seams: the media card on the wire and one press
//! through the method its control calls.

use bbr_testkit::protocol::{MediaCardSnapshot, MediaInput, MediaSessionEntry, MediaSnapshot};
use bbr_ui::components::media_card::{MediaCardAction, MediaCardTiming};
use bbr_web::media::{MediaPlaybackState, MediaSessionId};
use gpui::{Context, Window};

use crate::app::BbrApp;

impl BbrApp {
    pub(crate) fn media_snapshot(&self) -> MediaSnapshot {
        let card = self
            .media_card_model()
            .zip(self.media_shown())
            .map(|(model, (session, _))| {
                let (elapsed, duration, live, seekable) = match model.timing {
                    MediaCardTiming::Known {
                        elapsed,
                        duration,
                        seekable,
                    } => (Some(elapsed), Some(duration), false, seekable),
                    MediaCardTiming::Live => (None, None, true, false),
                    MediaCardTiming::Unknown => (None, None, false, false),
                };
                let c = model.controls;
                MediaCardSnapshot {
                    view: session.id.view,
                    generation: session.id.generation,
                    title: model.title.to_string(),
                    source: model.source.to_string(),
                    playing: model.playing,
                    muted: model.muted,
                    elapsed,
                    duration,
                    live,
                    seekable,
                    controls: [
                        (c.previous, "previous"),
                        (c.play_pause, "play-pause"),
                        (c.next, "next"),
                        (c.mute, "mute"),
                    ]
                    .into_iter()
                    .filter(|(shown, _)| *shown)
                    .map(|(_, name)| name.to_string())
                    .collect(),
                    others: model.others,
                }
            });
        MediaSnapshot {
            card,
            sessions: self
                .media
                .read()
                .registry
                .ordered()
                .into_iter()
                .map(|session| MediaSessionEntry {
                    view: session.id.view,
                    generation: session.id.generation,
                    state: match session.state {
                        MediaPlaybackState::Playing => "playing",
                        MediaPlaybackState::Paused => "paused",
                        MediaPlaybackState::Suspended => "suspended",
                    }
                    .to_string(),
                })
                .collect(),
            last_outcome: self
                .media
                .read()
                .last_outcome
                .as_ref()
                .map(|o| format!("{o:?}")),
        }
    }

    pub(crate) fn media_input(
        &mut self,
        input: MediaInput,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let action = match input {
            MediaInput::Source => MediaCardAction::Source,
            MediaInput::Dismiss => MediaCardAction::Dismiss,
            MediaInput::Cycle => MediaCardAction::Cycle,
            MediaInput::Previous => MediaCardAction::Previous,
            MediaInput::PlayPause => MediaCardAction::PlayPause,
            MediaInput::Next => MediaCardAction::Next,
            MediaInput::ToggleMute => MediaCardAction::ToggleMute,
            MediaInput::Seek { fraction } => MediaCardAction::Seek(fraction),
            MediaInput::Command {
                view,
                generation,
                command,
            } => {
                self.media_command(MediaSessionId { view, generation }, command);
                cx.notify();
                return;
            }
        };
        self.on_media_card(action, window, cx);
    }
}
