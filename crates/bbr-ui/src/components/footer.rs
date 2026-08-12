//! DESIGN.md §11.4 footer row — exactly `[Downloads] [Space Strip] [+]`.
//!
//! The footer is the rail's bottom row, separated from the list above by a
//! hairline. The fixed controls keep stable §15 ids; `[Downloads]` opens the
//! downloads surface, the middle flex region is the permanent §11.3 space
//! strip (rendered by the app's `space_strip` composition, which stays
//! app-owned until the G3 strip row), and `[+]` opens the plus menu (see
//! [`super::menu`]). There is no `[Spaces]` toggle and no revealable
//! space-row region — the strip is always part of the rail.
//!
//! This module owns the control identity contract (ids, labels, glyphs) and
//! the row anatomy. The rail composition step wires the production
//! listeners; the strip/chip behavior lives in the app layer.

use gpui::{div, prelude::*, App, Div, MouseButton, MouseDownEvent, Stateful, Styled, Window};

use crate::icons::Icon;
use crate::theme::{Spacing, Theme};

/// The §15 id of the footer row container.
pub const FOOTER_ID: &str = "bbr-footer";
/// The §15 id of the `[Downloads]` control.
pub const FOOTER_DOWNLOADS_ID: &str = "bbr-downloads";
/// The §15 id of the permanent space strip (DESIGN.md §11.3/§11.4).
pub const FOOTER_STRIP_ID: &str = "bbr-space-strip";
/// The §15 id of the `[+]` control.
pub const FOOTER_PLUS_ID: &str = "bbr-plus";
/// The §7 text alternative of the glyph-only `[+]` control (the label is
/// the glyph itself; the semantic tree exposes this name).
#[allow(dead_code)] // §15 semantic seam
pub const FOOTER_PLUS_NAME: &str = "More actions";

/// One fixed footer control (§11.4: exactly two, the strip is the middle
/// flex region).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FooterControl {
    Downloads,
    Plus,
}

impl FooterControl {
    /// The stable §15 id.
    pub fn id(self) -> &'static str {
        match self {
            FooterControl::Downloads => FOOTER_DOWNLOADS_ID,
            FooterControl::Plus => FOOTER_PLUS_ID,
        }
    }

    /// The semantic label. Both fixed footer controls render glyph-only at
    /// rest; the typed semantic tree exposes this text alternative.
    pub fn label(self) -> &'static str {
        match self {
            FooterControl::Downloads => "Downloads",
            FooterControl::Plus => "+",
        }
    }

    /// The §7 glyph of the control.
    pub fn glyph(self) -> Icon {
        match self {
            FooterControl::Downloads => Icon::Downloads,
            FooterControl::Plus => Icon::Plus,
        }
    }
}

/// The footer row anatomy: a hairline above a `type:meta` row of the two
/// fixed §11.4 controls with the permanent §11.3 space strip between them.
/// `on_downloads`/`on_plus` are the production view listeners; `strip` is
/// the caller-built space strip element (chips + scroll container). The
/// plus control is glyph-only per §11.4's `[+]` — its text alternative
/// rides the semantic tree.
///
/// `downloads_count` renders the §11.4 `type:meta` `tnum` count on the
/// `[Downloads]` control when content exists (which attempts feed the count
/// is DESIGN §17.11 — open; v1 reports the in-flight attempts).
/// `downloads_focus` makes the control keyboard-operable: it tracks the
/// handle (so dismissal can return focus here, §11.12) and Enter/Space
/// activate it like a pointer press.
pub fn footer_surface(
    theme: &Theme,
    downloads_focus: gpui::FocusHandle,
    downloads_focused: bool,
    downloads_count: Option<usize>,
    strip: Stateful<Div>,
    on_downloads: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_plus: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    // The listener is shared between the pointer path and the keyboard
    // activation without requiring `Clone` (GPUI's `cx.listener` closures
    // are not `Clone` in this version): an `Rc` owns the one listener and
    // both paths call through it.
    let on_downloads = std::rc::Rc::new(on_downloads);
    let on_downloads_pointer = on_downloads.clone();
    let on_downloads_pointer = move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
        on_downloads_pointer(event, window, cx)
    };
    let on_downloads_keyboard = on_downloads.clone();
    let downloads_keyboard =
        move |event: &gpui::KeyDownEvent, window: &mut Window, cx: &mut App| {
            if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                cx.stop_propagation();
                on_downloads_keyboard(&MouseDownEvent::default(), window, cx);
            }
        };
    let focused = downloads_focused;
    div()
        .id(FOOTER_ID)
        .debug_selector(|| FOOTER_ID.to_string())
        .h(theme.density.row())
        .border_t_1()
        .border_color(p.hairline)
        .bg(p.surface_footer)
        .px(Spacing::sp2())
        .gap(Spacing::sp3())
        .items_center()
        .flex()
        .flex_row()
        .child(
            footer_control(theme, FooterControl::Downloads, on_downloads_pointer)
                .track_focus(&downloads_focus)
                // §9 focus ring on the keyboard-focused control (structural cue —
                // never color-only).
                .when(focused, |el| {
                    el.border_2()
                        .border_color(theme.palette.accent)
                        .rounded(theme.radius.small())
                })
                .when(downloads_count.is_some(), |el| {
                    el.child(
                        div()
                            .text_size(theme.type_.meta.size)
                            .font_weight(theme.type_.meta.weight)
                            .text_color(p.ink_faint)
                            .child(downloads_count.unwrap_or(0).to_string()),
                    )
                })
                .on_key_down(downloads_keyboard),
        )
        // §11.3/§11.4: the permanent space strip consumes all remaining
        // footer width, clips to one line, and scrolls horizontally — it is
        // never a disclosure control.
        .child(strip)
        .child(footer_control(theme, FooterControl::Plus, on_plus))
}

/// One fixed footer control: the §7 glyph at the density icon size. Both
/// controls are visually glyph-only in the compact reference footer; their
/// full labels remain semantic names. The id sits on the control div;
/// `on_select` is the caller's production pointer path.
pub fn footer_control(
    theme: &Theme,
    control: FooterControl,
    on_select: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let glyph = div()
        .size(theme.density.icon())
        .flex()
        .items_center()
        .justify_center()
        .child(control.glyph().canvas(theme, theme.density.icon()));
    div()
        .id(control.id())
        .debug_selector(move || control.id().to_string())
        .h(theme.density.row())
        .gap(Spacing::sp1())
        .items_center()
        .flex()
        .flex_row()
        .cursor_pointer()
        .text_size(theme.type_.meta.size)
        .font_weight(theme.type_.meta.weight)
        .text_color(theme.palette.ink_secondary)
        .child(glyph)
        .on_mouse_down(MouseButton::Left, on_select)
}

/// The complete footer id set for the §15 `visible_ids` derivation (the
/// container, the two fixed controls, and the permanent strip).
#[allow(dead_code)] // §15 visible-ids seam
pub fn footer_ids() -> Vec<String> {
    [
        FOOTER_ID,
        FOOTER_DOWNLOADS_ID,
        FOOTER_STRIP_ID,
        FOOTER_PLUS_ID,
    ]
    .map(String::from)
    .to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footer_controls_are_exactly_the_two_fixed_contract_controls() {
        assert_eq!(FooterControl::Downloads.id(), FOOTER_DOWNLOADS_ID);
        assert_eq!(FooterControl::Plus.id(), FOOTER_PLUS_ID);
        assert_eq!(FooterControl::Downloads.label(), "Downloads");
        assert_eq!(FooterControl::Plus.label(), "+");
        assert_eq!(FooterControl::Plus.glyph(), Icon::Plus);
        assert_eq!(FooterControl::Downloads.glyph(), Icon::Downloads);
        // The glyph-only plus control keeps a non-empty text alternative.
        assert_eq!(FOOTER_PLUS_NAME, "More actions");
    }

    #[test]
    fn footer_ids_are_stable_and_exact() {
        assert_eq!(
            footer_ids(),
            vec![
                "bbr-footer".to_string(),
                "bbr-downloads".to_string(),
                "bbr-space-strip".to_string(),
                "bbr-plus".to_string(),
            ]
        );
        // No toggle control: the §11.4 footer is the two fixed controls
        // plus the permanent strip (the legacy `bbr-spaces` id is gone).
        assert_eq!(footer_ids().len(), 4);
        assert!(!footer_ids().contains(&"bbr-spaces".to_string()));
    }
}
