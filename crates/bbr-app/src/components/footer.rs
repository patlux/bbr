//! DESIGN.md §11.4 footer row — exactly `[Downloads] [Spaces] [+]`.
//!
//! The footer is the rail's bottom row, separated from the list above by a
//! hairline. The three controls keep stable §15 ids; `[Downloads]` is a
//! placeholder control in the U2 slice (the downloads *surface* is Phase 8,
//! the portable capability contract the L6 lane), `[Spaces]` toggles the
//! §11.3 spaces region, and `[+]` opens the plus menu (see `menu`).
//!
//! This module owns the control identity contract (ids, labels, glyphs) and
//! the row anatomy. The rail composition step wires the production
//! listeners; the toggle/menu behavior lives in the app layer.

use gpui::{div, prelude::*, App, Div, MouseButton, MouseDownEvent, Stateful, Styled, Window};

use crate::theme::{Spacing, Theme};

use super::Icon;

/// The §15 id of the footer row container.
pub const FOOTER_ID: &str = "bbr-footer";
/// The §15 id of the `[Downloads]` control.
pub const FOOTER_DOWNLOADS_ID: &str = "bbr-downloads";
/// The §15 id of the `[Spaces]` control.
pub const FOOTER_SPACES_ID: &str = "bbr-spaces";
/// The §15 id of the `[+]` control.
pub const FOOTER_PLUS_ID: &str = "bbr-plus";
/// The §7 text alternative of the glyph-only `[+]` control (the label is
/// the glyph itself; the semantic tree exposes this name).
#[cfg_attr(not(feature = "testkit"), allow(dead_code))] // §15 semantic seam
pub const FOOTER_PLUS_NAME: &str = "More actions";

/// One footer control (§11.4: exactly three).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FooterControl {
    Downloads,
    Spaces,
    Plus,
}

impl FooterControl {
    /// The three §11.4 footer controls in fixed order.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // semantic footer node
    pub fn all() -> [FooterControl; 3] {
        [
            FooterControl::Downloads,
            FooterControl::Spaces,
            FooterControl::Plus,
        ]
    }

    /// The stable §15 id.
    pub fn id(self) -> &'static str {
        match self {
            FooterControl::Downloads => FOOTER_DOWNLOADS_ID,
            FooterControl::Spaces => FOOTER_SPACES_ID,
            FooterControl::Plus => FOOTER_PLUS_ID,
        }
    }

    /// The visual label: `[Downloads]`/`[Spaces]` show the label next to
    /// the §7 glyph; `[+]` is glyph-only (its label is the glyph).
    pub fn label(self) -> &'static str {
        match self {
            FooterControl::Downloads => "Downloads",
            FooterControl::Spaces => "Spaces",
            FooterControl::Plus => "+",
        }
    }

    /// The §7 glyph of the control.
    pub fn glyph(self) -> Icon {
        match self {
            FooterControl::Downloads => Icon::Downloads,
            FooterControl::Spaces => Icon::Spaces,
            FooterControl::Plus => Icon::Plus,
        }
    }

    /// The §7 text alternative exposed as the semantic name (the plus
    /// control's label is its glyph, so it needs an explicit name).
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // §15 semantic seam
    pub fn semantic_name(self) -> &'static str {
        match self {
            FooterControl::Downloads => "Downloads",
            FooterControl::Spaces => "Spaces",
            FooterControl::Plus => FOOTER_PLUS_NAME,
        }
    }
}

/// The footer row anatomy: a hairline above a `type:meta` row of the three
/// §11.4 controls, each with its §7 glyph (plus a label for Downloads and
/// Spaces). `on_downloads`/`on_spaces`/`on_plus` are the production view
/// listeners; the rail composition wires them. The plus control is
/// glyph-only per §11.4's `[+]` — its text alternative rides the semantic
/// tree.
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
    on_downloads: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_spaces: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
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
        .h(theme.density.row())
        .border_t_1()
        .border_color(p.hairline)
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
        .child(footer_control(theme, FooterControl::Spaces, on_spaces))
        .child(footer_control(theme, FooterControl::Plus, on_plus))
}

/// One footer control: the §7 glyph at the density icon size plus the
/// `type:meta` label (glyph-only for `[+]`, whose label is the glyph). The
/// id sits on the control div; `on_select` is the caller's production
/// pointer path.
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
        .when(control != FooterControl::Plus, |el| {
            el.child(control.label())
        })
        .on_mouse_down(MouseButton::Left, on_select)
}

/// The complete footer id set for the §15 `visible_ids` derivation (the
/// container plus exactly the three §11.4 controls).
#[cfg_attr(not(feature = "testkit"), allow(dead_code))] // §15 visible-ids seam
pub fn footer_ids() -> Vec<String> {
    [
        FOOTER_ID,
        FOOTER_DOWNLOADS_ID,
        FOOTER_SPACES_ID,
        FOOTER_PLUS_ID,
    ]
    .map(String::from)
    .to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footer_controls_are_exactly_the_three_contract_controls() {
        assert_eq!(FooterControl::Downloads.id(), FOOTER_DOWNLOADS_ID);
        assert_eq!(FooterControl::Spaces.id(), FOOTER_SPACES_ID);
        assert_eq!(FooterControl::Plus.id(), FOOTER_PLUS_ID);
        assert_eq!(FooterControl::Downloads.label(), "Downloads");
        assert_eq!(FooterControl::Spaces.label(), "Spaces");
        assert_eq!(FooterControl::Plus.label(), "+");
        assert_eq!(FooterControl::Plus.glyph(), Icon::Plus);
        assert_eq!(FooterControl::Downloads.glyph(), Icon::Downloads);
        assert_eq!(FooterControl::Spaces.glyph(), Icon::Spaces);
        // The glyph-only plus control keeps a non-empty text alternative.
        assert!(!FooterControl::Plus.semantic_name().is_empty());
    }

    #[test]
    fn footer_ids_are_stable_and_exact() {
        assert_eq!(
            footer_ids(),
            vec![
                "bbr-footer".to_string(),
                "bbr-downloads".to_string(),
                "bbr-spaces".to_string(),
                "bbr-plus".to_string(),
            ]
        );
        // No fourth control: the §11.4 footer is exactly three controls.
        assert_eq!(footer_ids().len(), 4);
    }
}
