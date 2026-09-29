//! Blueprint D24 (board S-8, DESIGN §11.15): the exact production
//! `bbr_ui::components::media_card` in its capability/state fragments —
//! playing with a seekable length, paused, a live stream, an unknown
//! length, muted, and several sessions. Static presentation only; the real
//! WebKit control is proven by the media E2E.

use std::rc::Rc;

use bbr_ui::components::favicon::FaviconSpec;
use bbr_ui::components::media_card::{
    media_card, MediaCardControls, MediaCardModel, MediaCardTiming,
};
use gpui::{div, prelude::*, px, AnyElement};

use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page};

/// The gallery test-target selector prefix for the card fragments.
pub const MEDIA_CARD_FAMILY_SELECTOR: &str = "media-card-family-";

fn model(title: &str, source: &str, letter: char) -> MediaCardModel {
    MediaCardModel {
        title: title.to_string().into(),
        source: source.to_string().into(),
        favicon: FaviconSpec {
            letter,
            hue: letter as usize % 6,
            image: None,
        },
        playing: true,
        muted: false,
        timing: MediaCardTiming::Known {
            elapsed: 102.0,
            duration: 238.0,
            seekable: true,
        },
        controls: MediaCardControls {
            previous: true,
            play_pause: true,
            next: true,
            mute: true,
        },
        others: 0,
    }
}

pub fn render_media_card(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let playing = model("Discover Mix", "music.example · Default", 'M');
    let paused = MediaCardModel {
        playing: false,
        ..playing.clone()
    };
    let live = MediaCardModel {
        timing: MediaCardTiming::Live,
        controls: MediaCardControls {
            play_pause: true,
            mute: true,
            ..MediaCardControls::default()
        },
        ..model("Lo-fi radio", "radio.example · Work", 'R')
    };
    let unknown = MediaCardModel {
        timing: MediaCardTiming::Unknown,
        controls: MediaCardControls {
            play_pause: true,
            mute: true,
            ..MediaCardControls::default()
        },
        ..model("Podcast #112", "podcasts.example · Default", 'P')
    };
    let muted = MediaCardModel {
        muted: true,
        ..playing.clone()
    };
    let several = MediaCardModel {
        timing: MediaCardTiming::Unknown,
        controls: MediaCardControls {
            play_pause: true,
            mute: true,
            ..MediaCardControls::default()
        },
        others: 2,
        ..model("Standup call", "meet.example · Work", 'S')
    };
    let fragments = [
        ("Playing · seekable", playing),
        ("Paused", paused),
        ("Live", live),
        ("Unknown duration", unknown),
        ("Muted", muted),
        ("Several sessions", several),
    ];
    let children = fragments
        .into_iter()
        .enumerate()
        .map(|(index, (label, model))| {
            div()
                .debug_selector(move || format!("{MEDIA_CARD_FAMILY_SELECTOR}{index}"))
                .flex()
                .flex_col()
                .gap_2()
                .child(section_heading(theme, label))
                .child(
                    div()
                        .w(px(300.0))
                        .py(px(8.0))
                        .bg(theme.palette.surface_sidebar)
                        .child(media_card(theme, &model, Rc::new(|_, _, _| {}))),
                )
                .into_any()
        })
        .collect();
    story_page(ctx, "Media card", children)
}
