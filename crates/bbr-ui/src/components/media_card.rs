//! DESIGN.md §11.15 / blueprint D24 (board S-8): the sidebar media card.
//!
//! One card above the footer while a tab plays media: the source tile,
//! title and source line (together the one source-activation target), the
//! dismiss cross, elapsed/scrubber/duration for a known seekable length
//! ("Live" for a stream, nothing for an unknown length), and the
//! capability-gated transport plus mute. Several playing tabs show the most
//! recently controlled one and a `+N` switch. The model is plain
//! presentation data the app builds from the media session registry; every
//! press goes to one pre-bound callback, so no session, tab or engine
//! identity enters this crate.

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    canvas, div, prelude::*, px, App, Bounds, MouseButton, Pixels, SharedString, Stateful, Window,
};

use crate::components::favicon::FaviconSpec;
use crate::icons::Icon;
use crate::theme::{Radius, Theme};

pub const MEDIA_CARD_ID: &str = "bbr-media-card";

/// The card's length row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaCardTiming {
    /// A known length: elapsed, scrubber and total (seconds). `seekable`
    /// makes the scrubber a control.
    Known {
        elapsed: f64,
        duration: f64,
        seekable: bool,
    },
    /// A live stream: a status dot and "Live", no total.
    Live,
    /// An unknown length: no row at all.
    Unknown,
}

/// The advertised controls (absent ones are not drawn).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MediaCardControls {
    pub previous: bool,
    pub play_pause: bool,
    pub next: bool,
    pub mute: bool,
}

/// The card's content.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaCardModel {
    pub title: SharedString,
    /// "example.com · Default" — the site and the owning profile.
    pub source: SharedString,
    pub favicon: FaviconSpec,
    pub playing: bool,
    pub muted: bool,
    pub timing: MediaCardTiming,
    pub controls: MediaCardControls,
    /// Other playing sessions (the `+N` switch; 0 hides it).
    pub others: usize,
}

/// A press on the card.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaCardAction {
    /// The title block: show the playing tab.
    Source,
    Dismiss,
    /// The `+N` switch: the next most recently controlled session.
    Cycle,
    Previous,
    PlayPause,
    Next,
    ToggleMute,
    /// Seek to this fraction of the length.
    Seek(f64),
}

pub type MediaCardCallback = Rc<dyn Fn(MediaCardAction, &mut Window, &mut App)>;

/// "m:ss" (or "h:mm:ss").
pub fn clock(seconds: f64) -> String {
    let total = seconds.max(0.0).floor() as u64;
    let (h, m, s) = (total / 3600, (total / 60) % 60, total % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// The §15 ids the card draws for `model`.
pub fn media_card_visible_ids(model: &MediaCardModel) -> Vec<String> {
    let mut ids = vec![
        MEDIA_CARD_ID,
        "bbr-media-card-title",
        "bbr-media-card-source",
        "bbr-media-card-dismiss",
    ];
    if model.others > 0 {
        ids.push("bbr-media-card-session-count");
    }
    match model.timing {
        MediaCardTiming::Known { .. } => ids.extend([
            "bbr-media-card-progress",
            "bbr-media-card-elapsed",
            "bbr-media-card-duration",
        ]),
        MediaCardTiming::Live => ids.push("bbr-media-card-live"),
        MediaCardTiming::Unknown => {}
    }
    let c = model.controls;
    for (shown, id) in [
        (c.previous, "bbr-media-card-previous"),
        (c.play_pause, "bbr-media-card-play-pause"),
        (c.next, "bbr-media-card-next"),
        (c.mute, "bbr-media-card-mute"),
    ] {
        if shown {
            ids.push(id);
        }
    }
    ids.into_iter().map(String::from).collect()
}

/// The card (placed by the sidebar above the footer).
pub fn media_card(
    theme: &Theme,
    model: &MediaCardModel,
    on_action: MediaCardCallback,
) -> Stateful<gpui::Div> {
    let p = theme.palette;
    let press = |action: MediaCardAction| {
        let on_action = on_action.clone();
        move |_e: &gpui::MouseDownEvent, window: &mut Window, cx: &mut App| {
            cx.stop_propagation();
            on_action(action, window, cx);
        }
    };
    let tile = div()
        .size(px(22.0))
        .flex_shrink_0()
        .rounded(Radius.small())
        .overflow_hidden()
        .child(crate::components::favicon::favicon_tile(
            theme,
            0,
            Some(model.favicon.clone()),
            crate::state::TabVisualState::Default,
        ));
    let header = div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(
            div()
                .id("bbr-media-card-source-target")
                .flex_1()
                .min_w_0()
                .flex()
                .items_center()
                .gap(px(10.0))
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, press(MediaCardAction::Source))
                .child(tile)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .id("bbr-media-card-title")
                                .text_size(theme.type_.label.size)
                                .font_weight(theme.type_.label.weight)
                                .text_color(p.ink)
                                .truncate()
                                .child(model.title.clone()),
                        )
                        .child(
                            div()
                                .id("bbr-media-card-source")
                                .text_size(theme.type_.key.size)
                                .text_color(p.ink_secondary)
                                .truncate()
                                .child(model.source.clone()),
                        ),
                ),
        )
        .when(model.others > 0, |row| {
            row.child(
                div()
                    .id("bbr-media-card-session-count")
                    .h(px(18.0))
                    .px(px(6.0))
                    .flex()
                    .items_center()
                    .rounded(Radius.full())
                    .bg(p.wash_active)
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, press(MediaCardAction::Cycle))
                    .text_size(theme.type_.key.size)
                    .text_color(p.ink_secondary)
                    .child(format!("+{}", model.others)),
            )
        })
        .child(
            button("bbr-media-card-dismiss", 22.0)
                .on_mouse_down(MouseButton::Left, press(MediaCardAction::Dismiss))
                .child(Icon::Close.canvas_with(theme, px(18.0), p.ink_faint)),
        );
    let timing = match model.timing {
        MediaCardTiming::Known {
            elapsed,
            duration,
            seekable,
        } => Some(
            progress_row(theme, elapsed, duration, seekable, on_action.clone()).into_any_element(),
        ),
        MediaCardTiming::Live => Some(
            div()
                .id("bbr-media-card-live")
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(
                    div()
                        .size(px(6.0))
                        .rounded(Radius.full())
                        .bg(p.status_error),
                )
                .child(
                    div()
                        .text_size(theme.type_.key.size)
                        .text_color(p.ink_secondary)
                        .child("Live"),
                )
                .into_any_element(),
        ),
        MediaCardTiming::Unknown => None,
    };
    let c = model.controls;
    let transport = div()
        .flex()
        .gap(px(4.0))
        .when(c.previous, |row| {
            row.child(
                button("bbr-media-card-previous", 28.0)
                    .on_mouse_down(MouseButton::Left, press(MediaCardAction::Previous))
                    .child(Icon::MediaPrevious.canvas(theme, px(18.0))),
            )
        })
        .when(c.play_pause, |row| {
            let glyph = if model.playing {
                Icon::MediaPause
            } else {
                Icon::MediaPlay
            };
            row.child(
                button("bbr-media-card-play-pause", 28.0)
                    .on_mouse_down(MouseButton::Left, press(MediaCardAction::PlayPause))
                    .child(glyph.canvas(theme, px(18.0))),
            )
        })
        .when(c.next, |row| {
            row.child(
                button("bbr-media-card-next", 28.0)
                    .on_mouse_down(MouseButton::Left, press(MediaCardAction::Next))
                    .child(Icon::MediaNext.canvas(theme, px(18.0))),
            )
        });
    div()
        .id(MEDIA_CARD_ID)
        .mx(px(8.0))
        .mb(px(8.0))
        .pt(px(10.0))
        .pb(px(8.0))
        .px(px(10.0))
        .rounded(Radius.large())
        .bg(p.surface_field)
        .border_1()
        .border_color(p.hairline)
        .flex()
        .flex_col()
        .gap(px(8.0))
        // The card is chrome: a press never falls through to the rail.
        .on_mouse_down(MouseButton::Left, |_e, _w, cx| cx.stop_propagation())
        .child(header)
        .when_some(timing, |card, row| card.child(row))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(transport)
                .when(c.mute, |row| {
                    row.child(
                        button("bbr-media-card-mute", 28.0)
                            .on_mouse_down(MouseButton::Left, press(MediaCardAction::ToggleMute))
                            .child(Icon::Speaker { muted: model.muted }.canvas(theme, px(18.0))),
                    )
                }),
        )
}

fn button(id: &'static str, size: f32) -> Stateful<gpui::Div> {
    div()
        .id(id)
        .size(px(size))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(Radius.small())
        .cursor_pointer()
}

/// Elapsed, the 3-pt scrubber (a knob only on hover), total.
fn progress_row(
    theme: &Theme,
    elapsed: f64,
    duration: f64,
    seekable: bool,
    on_action: MediaCardCallback,
) -> Stateful<gpui::Div> {
    let p = theme.palette;
    let fraction = if duration > 0.0 {
        (elapsed / duration).clamp(0.0, 1.0) as f32
    } else {
        0.0
    };
    let bounds: Rc<Cell<Option<Bounds<Pixels>>>> = Rc::default();
    let record = bounds.clone();
    let track = div()
        .id("bbr-media-card-progress")
        .group("bbr-media-card-progress")
        .flex_1()
        .h(px(12.0))
        .flex()
        .items_center()
        .relative()
        .when(seekable, |track| {
            track
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                    cx.stop_propagation();
                    let Some(b) = bounds.get() else {
                        return;
                    };
                    let width = f32::from(b.size.width);
                    if width <= 0.0 {
                        return;
                    }
                    let at = (f32::from(event.position.x - b.origin.x) / width).clamp(0.0, 1.0);
                    on_action(MediaCardAction::Seek(f64::from(at)), window, cx);
                })
        })
        .child(
            canvas(move |b, _w, _cx| record.set(Some(b)), |_, _, _, _| {})
                .absolute()
                .size_full(),
        )
        .child(
            div()
                .w_full()
                .h(px(3.0))
                .rounded(Radius.full())
                .bg(p.wash_active)
                .child(
                    div()
                        .w(gpui::relative(fraction))
                        .h(px(3.0))
                        .rounded(Radius.full())
                        .bg(p.accent),
                ),
        )
        .when(seekable, |track| {
            track.child(
                div()
                    .absolute()
                    .left(gpui::relative(fraction))
                    .ml(px(-5.0))
                    .size(px(10.0))
                    .rounded(Radius.full())
                    .bg(p.accent)
                    .opacity(0.0)
                    .group_hover("bbr-media-card-progress", |s| s.opacity(1.0)),
            )
        });
    let meta = |id: &'static str, text: String| {
        div()
            .id(id)
            .text_size(theme.type_.key.size)
            .text_color(p.ink_faint)
            .child(text)
    };
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .child(meta("bbr-media-card-elapsed", clock(elapsed)))
        .child(track)
        .child(meta("bbr-media-card-duration", clock(duration)))
        .id("bbr-media-card-timing")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(
        timing: MediaCardTiming,
        controls: MediaCardControls,
        others: usize,
    ) -> MediaCardModel {
        MediaCardModel {
            title: "Track".into(),
            source: "example.com · Default".into(),
            favicon: FaviconSpec {
                letter: 'E',
                hue: 0,
                image: None,
            },
            playing: true,
            muted: false,
            timing,
            controls,
            others,
        }
    }

    #[test]
    fn only_advertised_parts_are_drawn() {
        let play_mute = MediaCardControls {
            play_pause: true,
            mute: true,
            ..MediaCardControls::default()
        };
        let seekable = MediaCardTiming::Known {
            elapsed: 1.0,
            duration: 10.0,
            seekable: true,
        };
        let ids = media_card_visible_ids(&model(seekable, play_mute, 0));
        for id in [
            "bbr-media-card-progress",
            "bbr-media-card-play-pause",
            "bbr-media-card-mute",
        ] {
            assert!(ids.iter().any(|i| i == id), "{id}");
        }
        for id in [
            "bbr-media-card-previous",
            "bbr-media-card-next",
            "bbr-media-card-session-count",
            "bbr-media-card-live",
        ] {
            assert!(!ids.iter().any(|i| i == id), "{id}");
        }
        let live = media_card_visible_ids(&model(MediaCardTiming::Live, play_mute, 2));
        assert!(live.iter().any(|i| i == "bbr-media-card-live"));
        assert!(live.iter().any(|i| i == "bbr-media-card-session-count"));
        assert!(
            !live.iter().any(|i| i == "bbr-media-card-progress"),
            "no fake total"
        );
        let bare = media_card_visible_ids(&model(
            MediaCardTiming::Unknown,
            MediaCardControls::default(),
            0,
        ));
        assert_eq!(
            bare,
            [
                "bbr-media-card",
                "bbr-media-card-title",
                "bbr-media-card-source",
                "bbr-media-card-dismiss"
            ]
        );
        let all = MediaCardControls {
            previous: true,
            play_pause: true,
            next: true,
            mute: true,
        };
        let full = media_card_visible_ids(&model(seekable, all, 0));
        assert!(full.iter().any(|i| i == "bbr-media-card-previous"));
        assert!(full.iter().any(|i| i == "bbr-media-card-next"));
    }

    #[test]
    fn clock_reads_minutes_and_hours() {
        assert_eq!(clock(0.0), "0:00");
        assert_eq!(clock(102.9), "1:42");
        assert_eq!(clock(3725.0), "1:02:05");
        assert_eq!(clock(-3.0), "0:00");
    }
}
