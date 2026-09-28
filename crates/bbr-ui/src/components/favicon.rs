//! §7.2 favicon tile and its deterministic monogram ramp: the
//! `FaviconSpec` DTO, the FNV-1a hostname monogram derivation, and the
//! rendered tile. When a tab has no favicon yet, the slot renders the
//! monogram letter on a desaturated slate-hue tile instead of an empty
//! box; while loading it shows the §10.5 accent arc, rotating at 640 ms per
//! turn (static under Reduce Motion, §18.5). Monograms are static — no
//! spin, no pulse, no animation (§7.2).
//!
//! When the owning profile's bounded host-keyed store holds resolved
//! favicon bytes (task packet `real-favicon-pipeline.md`), the tile renders
//! the decoded image through the existing GPUI `img` element in the §8
//! density geometry (S 16 / N 18 / XL 20). The §10.5 loading spinner and
//! the §10.7/§10.11 error/unresponsive glyphs keep their precedence over
//! the image; a decode failure falls back silently to the deterministic
//! monogram (the acquisition contract's silent-fallback rule).

use std::sync::Arc;

use gpui::{
    div, img, prelude::*, px, AnimationExt, AnyElement, Div, Image, ImageSource, ObjectFit, Pixels,
    Rgba, SharedString, Stateful,
};

use crate::icons::Icon;
use crate::palette::FaviconSlate;
use crate::state::TabVisualState;
use crate::text::{host_of, scheme_of};
use crate::theme::motion::Moment;
use crate::theme::{Radius, Theme, ThemeMode};

/// §15 stable id of the tab row's favicon slot (DESIGN.md §11.1 assert
/// ids) — rendered by this component and pinned by the tab-row tests.
pub const TAB_FAVICON_ID: &str = "bbr-tab-favicon";

/// The content branch that owns a favicon slot after §10 state precedence is
/// resolved. Kept as a pure decision so image/monogram/state precedence has
/// deterministic component-level regression coverage without pixel tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FaviconContentKind {
    Monogram,
    Image,
    Loading,
    Error,
    Unresponsive,
}

fn favicon_content_kind(has_image: bool, state: TabVisualState) -> FaviconContentKind {
    match state {
        TabVisualState::Loading => FaviconContentKind::Loading,
        TabVisualState::Error => FaviconContentKind::Error,
        TabVisualState::Unresponsive => FaviconContentKind::Unresponsive,
        _ if has_image => FaviconContentKind::Image,
        _ => FaviconContentKind::Monogram,
    }
}

/// The favicon tile: the deterministic monogram (§7.2) until a real favicon
/// exists; while loading the slot shows the accent spinner. `image` carries
/// the resolved favicon bytes wrapped in gpui's [`Image`] asset type (the
/// app decodes once per acquired record); when present it replaces the
/// monogram letter while the §10 state precedence stays untouched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaviconSpec {
    pub letter: char,
    pub hue: usize,
    /// The resolved favicon image for the host, if the owning profile's
    /// bounded store holds one (task packet `real-favicon-pipeline.md`).
    pub image: Option<Arc<Image>>,
}

/// First character of the registrable hostname, uppercased — the §7.2
/// monogram tile letter. Falls back to the scheme's first letter.
pub fn monogram_letter(url: &str) -> char {
    let host = host_of(url);
    let source = if host.is_empty() {
        scheme_of(url)
    } else {
        host
    };
    source
        .chars()
        .next()
        .map(|c| c.to_ascii_uppercase())
        .unwrap_or('?')
}

/// FNV-1a of the hostname mod 6 — the deterministic slate-hue index for the
/// §7.2 monogram tile ramp. Stable per host.
pub fn monogram_hue(url: &str) -> usize {
    let host = host_of(url);
    let mut hash = 0x811c9dc5u32;
    for byte in host.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x01000193);
    }
    (hash as usize) % 6
}

/// §7.2 favicon tile: the §8 density icon size (S 16 / N 18 / XL 20)
/// square, `radius:sm`, monogram letter on the FNV-slate ramp — or the
/// decoded favicon image when the owning profile's store holds one; while
/// loading the slot shows the §10.5 accent arc ([`loading_spinner`]). The §10.7 error lifering and the §10.11
/// unresponsive lifering+clock replace the tile content exactly as before:
/// the image never overrides a state glyph.
pub fn favicon_tile(
    theme: &Theme,
    index: usize,
    spec: Option<FaviconSpec>,
    state: TabVisualState,
) -> Stateful<Div> {
    let size = theme.density.icon();
    let badge_size = theme.type_.badge.size;
    let badge_weight = theme.type_.badge.weight;
    let radius = Radius.small();
    // The §7.2 monogram identity (letter + slate tile) and the resolved
    // favicon image — one match, so the spec's fields move exactly once.
    // The monogram content is also the silent fallback when a present image
    // fails to decode (the acquisition contract's fallback rule).
    let (bg, fg, letter, image) = match spec {
        Some(spec) => {
            let bg = match theme.mode {
                ThemeMode::Dark => FaviconSlate::dark(spec.hue),
                ThemeMode::Light => FaviconSlate::light(spec.hue),
            };
            (
                bg,
                FaviconSlate::letter(theme.mode),
                spec.letter,
                spec.image.clone(),
            )
        }
        None => (
            theme.palette.surface_field,
            theme.palette.ink_faint,
            '?',
            None,
        ),
    };
    let content_kind = favicon_content_kind(image.is_some(), state);
    // The normal-branch content: the real favicon image when the record
    // holds one (decoded by the app once per acquisition through the GPUI
    // asset pipeline — never per frame), else the monogram letter.
    let content: gpui::AnyElement = match image {
        Some(image) => img(ImageSource::Image(image))
            .size(size)
            .rounded(radius)
            .object_fit(ObjectFit::Contain)
            .with_fallback(move || {
                monogram_content(size, radius, badge_size, badge_weight, bg, fg, letter).into_any()
            })
            .into_any(),
        None => monogram_content(size, radius, badge_size, badge_weight, bg, fg, letter).into_any(),
    };
    div()
        .id((TAB_FAVICON_ID, index))
        .debug_selector(move || format!("{TAB_FAVICON_ID}-{index}"))
        .size(size)
        .rounded(radius)
        .bg(bg)
        .flex()
        .items_center()
        .justify_center()
        .text_size(badge_size)
        .font_weight(badge_weight)
        .text_color(fg)
        .when(content_kind == FaviconContentKind::Loading, |el| {
            el.child(loading_spinner(theme, index, size))
        })
        .when(content_kind == FaviconContentKind::Error, |el| {
            // §10.7: the lifering replaces the favicon on an error row.
            el.child(Icon::Lifering.canvas(theme, size))
        })
        .when(content_kind == FaviconContentKind::Unresponsive, |el| {
            // §10.11: the lifering plus the static clock companion replace
            // the favicon on an unresponsive row (the renderer stopped
            // answering liveness probes; the session stays alive).
            el.flex_row()
                .gap(px(2.0))
                .child(Icon::Lifering.canvas(theme, size))
                .child(Icon::Clock.canvas(theme, size))
        })
        .when(
            matches!(
                content_kind,
                FaviconContentKind::Image | FaviconContentKind::Monogram
            ),
            |el| el.child(content),
        )
}

/// §10.5/§18.5 the loading arc: rotating at 640 ms per turn while the row
/// loads (a loop that exists only while the state holds — the element
/// leaves the tree with the state, so no frame is requested afterwards),
/// or the static arc under Reduce Motion.
pub fn loading_spinner(theme: &Theme, index: usize, size: Pixels) -> AnyElement {
    match Moment::LoadingArc.plan(theme.motion_mode) {
        Some(plan) => {
            let theme = *theme;
            div()
                .size(size)
                .with_animation(
                    ("bbr-loading-arc", index),
                    plan.animation(),
                    move |el, turn| {
                        el.child(crate::icons::spinner::loading_arc_canvas(
                            &theme, size, turn,
                        ))
                    },
                )
                .into_any_element()
        }
        None => Icon::LoadingArc.canvas(theme, size).into_any_element(),
    }
}

/// The §7.2 monogram letter tile (also the image decode-failure fallback):
/// the letter on the slate tile at the density icon size, `radius:sm`.
fn monogram_content(
    size: Pixels,
    radius: Pixels,
    badge_size: Pixels,
    badge_weight: gpui::FontWeight,
    bg: Rgba,
    fg: Rgba,
    letter: char,
) -> Div {
    div()
        .size(size)
        .rounded(radius)
        .bg(bg)
        .flex()
        .items_center()
        .justify_center()
        .text_size(badge_size)
        .font_weight(badge_weight)
        .text_color(fg)
        .child(SharedString::from(letter.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_and_state_precedence_matches_sections_7_and_10() {
        assert_eq!(
            favicon_content_kind(true, TabVisualState::Default),
            FaviconContentKind::Image
        );
        assert_eq!(
            favicon_content_kind(false, TabVisualState::Default),
            FaviconContentKind::Monogram
        );
        for has_image in [false, true] {
            assert_eq!(
                favicon_content_kind(has_image, TabVisualState::Loading),
                FaviconContentKind::Loading
            );
            assert_eq!(
                favicon_content_kind(has_image, TabVisualState::Error),
                FaviconContentKind::Error
            );
            assert_eq!(
                favicon_content_kind(has_image, TabVisualState::Unresponsive),
                FaviconContentKind::Unresponsive
            );
        }
    }

    #[test]
    fn monogram_is_deterministic_per_host() {
        assert_eq!(monogram_letter("https://example.com/x"), 'E');
        assert_eq!(monogram_letter("http://localhost:8080/x"), 'L');
        assert_eq!(monogram_letter("about:blank"), 'A');
        assert_eq!(
            monogram_letter("https://example.com/x"),
            monogram_letter("https://example.com/y")
        );
        assert_eq!(
            monogram_hue("https://example.com"),
            monogram_hue("https://example.com/other")
        );
        assert!((0..6).contains(&monogram_hue("https://example.com")));
    }
}
