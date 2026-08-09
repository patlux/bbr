//! §7.2 favicon tile and its deterministic monogram ramp: the
//! `FaviconSpec` DTO, the FNV-1a hostname monogram derivation, and the
//! rendered tile. When a tab has no favicon yet, the slot renders the
//! monogram letter on a desaturated slate-hue tile instead of an empty
//! box; while loading it shows the §10.5 accent arc (drawn, static —
//! rotation is a later polish pass). Monograms are static — no spin, no
//! pulse, no animation (§7.2).

use gpui::{div, prelude::*, Div, SharedString, Stateful};

use crate::icons::Icon;
use crate::palette::FaviconSlate;
use crate::state::TabVisualState;
use crate::text::{host_of, scheme_of};
use crate::theme::{Radius, Theme, ThemeMode};

/// The favicon tile: the deterministic monogram (§7.2) until a real favicon
/// exists; while loading the slot shows the accent spinner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaviconSpec {
    pub letter: char,
    pub hue: usize,
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
/// square, `radius:sm`, monogram letter on the FNV-slate ramp; while
/// loading the slot shows the §10.5 accent arc (drawn, static — rotation
/// is a later polish pass).
pub fn favicon_tile(
    theme: &Theme,
    index: usize,
    spec: Option<FaviconSpec>,
    state: TabVisualState,
) -> Stateful<Div> {
    let size = theme.density.icon();
    let (bg, fg) = match spec {
        Some(spec) => {
            let bg = match theme.mode {
                ThemeMode::Dark => FaviconSlate::dark(spec.hue),
                ThemeMode::Light => FaviconSlate::light(spec.hue),
            };
            (bg, theme.palette.surface_glass)
        }
        None => (theme.palette.surface_field, theme.palette.ink_faint),
    };
    let letter = spec.map(|s| s.letter).unwrap_or('?');
    div()
        .id(("bbr-tab-favicon", index))
        .size(size)
        .rounded(Radius.small())
        .bg(bg)
        .flex()
        .items_center()
        .justify_center()
        .text_size(theme.type_.badge.size)
        .font_weight(theme.type_.badge.weight)
        .text_color(fg)
        .when(state == TabVisualState::Loading, |el| {
            el.child(Icon::LoadingArc.canvas(theme, size))
        })
        .when(state != TabVisualState::Loading, |el| {
            el.child(SharedString::from(letter.to_string()))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

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
