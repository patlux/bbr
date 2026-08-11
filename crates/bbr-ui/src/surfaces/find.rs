//! The find-in-page bar (task packet `find-in-page-portal-bar`; DESIGN.md
//! §11.4 command-surface family).
//!
//! The bar renders inside the one native `NativeAboveWeb` portal window
//! (never a second host, never a GPUI overlay behind WebKit, never a
//! screenshot composite). This module is the plain presentation half: the
//! app's `FindController` keeps the portable `FindSession`, the native
//! WebKit adapter, the bound view, the lifecycle, and the DTO building;
//! the portal root resolves the presentation rect and passes the
//! production `cx.listener` closures; this surface only renders.
//!
//! The bar is the §11.4 command-surface family: `material:overlay`,
//! `radius:lg`, `elev:2`, `density:row` tall, anchored top-right of the
//! content area. Controls: previous/next/close. The card stops propagation
//! at the portal-root composition level (the backdrop owns the §13
//! one-pointer-action dismissal); the control callbacks themselves never
//! stop — the same contract the pre-cutover portal render had.

use gpui::{
    div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Styled,
    Window,
};

use crate::icons::Icon;
use crate::theme::elevation::elev2;
use crate::theme::{Spacing, Theme};

/// Stable §15 ids of the find bar (DESIGN.md §15; exposed through the
/// portal snapshot's typed `ids` field). The components render these exact
/// strings; the tests pin them so a label edit can never shift an id.
pub const FIND_BAR_ID: &str = "bbr-find-bar";
pub const FIND_FIELD_ID: &str = "bbr-find-field";
pub const FIND_TEXT_ID: &str = "bbr-find-text";
pub const FIND_COUNT_ID: &str = "bbr-find-count";
pub const FIND_PREVIOUS_ID: &str = "bbr-find-previous";
pub const FIND_NEXT_ID: &str = "bbr-find-next";
pub const FIND_CLOSE_ID: &str = "bbr-find-close";

/// The find bar's presentation model: the plain fields one portal-window
/// render consumes. The app derives this DTO from its live
/// [`crate::app::find::FindState`] per render, so the count/query can
/// never drift from the app. The `theme` is a separate parameter (the
/// caller owns theme resolution, like every other `bbr-ui` surface).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindSurface {
    /// §9: whether the find field owns keyboard focus (the chrome-focus
    /// ring on the field).
    pub focused: bool,
    /// The live query text (what the field renders; empty renders the
    /// "Find in page" placeholder).
    pub query: String,
    /// Total matches reported by WebKit (`None` until a result lands).
    pub match_count: Option<usize>,
    /// Current 1-based ordinal (`None` for zero matches / no result).
    pub current_match: Option<usize>,
}

/// The resolved card rect in points (top-left origin) the caller resolved
/// from the portal viewport. Plain data; the caller (the portal root or a
/// gallery fixture) owns the geometry authority.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FindBarRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// §11.5 count copy: "n of m"; zero matches (and any report with no
/// current ordinal) reads "0 of 0"; no report yet renders nothing. WebKit
/// reports count and ordinal together (both or neither), so the ordinal
/// arms can never show without a total.
pub fn count_label(model: &FindSurface) -> String {
    match (model.current_match, model.match_count) {
        (Some(ordinal), Some(total)) => format!("{ordinal} of {total}"),
        (None, Some(_)) => "0 of 0".to_string(),
        (_, None) => "".to_string(),
    }
}

/// Build the find bar at the resolved card rect. The caller (the portal
/// root) owns the full-window backdrop and pointer propagation policy; the
/// card stops propagation at that level. `on_previous`/`on_next`/`on_close`
/// are the production `cx.listener` closures routing through the portal
/// root's deferred app update — exactly like the command surfaces.
pub fn find_bar(
    theme: &Theme,
    model: &FindSurface,
    rect: FindBarRect,
    on_previous: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_next: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_close: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    div()
        .id(FIND_BAR_ID)
        .absolute()
        .left(px(rect.x))
        .top(px(rect.y))
        .w(px(rect.width))
        .h(px(rect.height))
        .rounded(theme.radius.large())
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .shadow(elev2(theme))
        .px(Spacing::sp2())
        .gap(Spacing::sp1())
        .flex()
        .flex_row()
        .items_center()
        .child(field(theme, model))
        .child(
            div()
                .id(FIND_COUNT_ID)
                .min_w(px(64.0))
                .text_size(theme.type_.meta.size)
                .text_color(p.ink_secondary)
                .child(count_label(model)),
        )
        .child(
            control_button(theme, FIND_PREVIOUS_ID, Icon::ChevronLeft)
                .on_mouse_down(MouseButton::Left, on_previous),
        )
        .child(
            control_button(theme, FIND_NEXT_ID, Icon::ChevronRight)
                .on_mouse_down(MouseButton::Left, on_next),
        )
        .child(
            control_button(theme, FIND_CLOSE_ID, Icon::Close)
                .on_mouse_down(MouseButton::Left, on_close),
        )
}

/// The query field: the live text plus a placeholder when empty. The find
/// field owns focus while the presentation is open; the §9 ring marks it
/// without color-only feedback.
fn field(theme: &Theme, model: &FindSurface) -> Stateful<Div> {
    let p = theme.palette;
    let text = if model.query.is_empty() {
        SharedString::from("Find in page")
    } else {
        SharedString::from(model.query.clone())
    };
    div()
        .id(FIND_FIELD_ID)
        .flex_1()
        .h(theme.density.row())
        .px(Spacing::sp2())
        .rounded(theme.radius.medium())
        .border_2()
        .border_color(if model.focused { p.accent } else { p.hairline })
        .bg(p.surface_field)
        .flex()
        .flex_row()
        .items_center()
        .child(
            div()
                .id(FIND_TEXT_ID)
                .flex_1()
                .text_size(theme.type_.mono.size)
                .text_color(if model.query.is_empty() {
                    p.ink_faint
                } else {
                    p.ink
                })
                .child(text),
        )
}

/// One bar control (previous / next / close), rendered with the §7 drawn
/// catalog glyph (`Icon::ChevronLeft` / `Icon::ChevronRight` / `Icon::Close`)
/// at the `icon:<density>` size. The close control uses the §8
/// `density:hit-min.close` hit square; the step controls use
/// `density:hit-min`. The owning div keeps the id, hit square, cursor, and
/// listeners — the glyph is pure presentation (§14: no Unicode substitutes
/// in chrome).
fn control_button(theme: &Theme, id: &'static str, icon: Icon) -> Stateful<Div> {
    let is_close = id == FIND_CLOSE_ID;
    let hit = if is_close {
        theme.density.hit_min_close()
    } else {
        theme.density.hit_min()
    };
    div()
        .id(id)
        .w(hit)
        .h(hit)
        .rounded(theme.radius.medium())
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .child(icon.canvas(theme, theme.density.icon()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_the_stable_section_15_ids() {
        // DESIGN.md §15 assert ids — unchangeable contract.
        assert_eq!(FIND_BAR_ID, "bbr-find-bar");
        assert_eq!(FIND_FIELD_ID, "bbr-find-field");
        assert_eq!(FIND_TEXT_ID, "bbr-find-text");
        assert_eq!(FIND_COUNT_ID, "bbr-find-count");
        assert_eq!(FIND_PREVIOUS_ID, "bbr-find-previous");
        assert_eq!(FIND_NEXT_ID, "bbr-find-next");
        assert_eq!(FIND_CLOSE_ID, "bbr-find-close");
    }

    #[test]
    fn find_surface_round_trips_its_fields() {
        let model = FindSurface {
            focused: true,
            query: "bbr".into(),
            match_count: Some(5),
            current_match: Some(2),
        };
        assert!(model.focused);
        assert_eq!(model.query, "bbr");
        assert_eq!(model.match_count, Some(5));
        assert_eq!(model.current_match, Some(2));
        // Plain data: clone/equality (anchor model contract).
        assert_eq!(model, model.clone());
        assert_ne!(
            model,
            FindSurface {
                focused: true,
                query: "bbr".into(),
                match_count: Some(5),
                current_match: None,
            }
        );
    }

    #[test]
    fn find_bar_rect_is_plain_geometry() {
        let rect = FindBarRect {
            x: 12.0,
            y: 8.0,
            width: 360.0,
            height: 36.0,
        };
        assert_eq!(rect, rect.clone());
        assert_eq!(rect.x, 12.0);
        assert_eq!(rect.y, 8.0);
        assert_eq!(rect.width, 360.0);
        assert_eq!(rect.height, 36.0);
    }

    #[test]
    fn count_label_derives_the_section_11_5_copy() {
        // §11.5 count copy: "n of m"; zero matches reads "0 of 0"; no
        // report yet renders nothing (WebKit reports count and ordinal
        // together, both or neither).
        let with_report = FindSurface {
            focused: true,
            query: "bbr".into(),
            match_count: Some(5),
            current_match: Some(2),
        };
        assert_eq!(count_label(&with_report), "2 of 5");
        let last = FindSurface {
            focused: true,
            query: "bbr".into(),
            match_count: Some(5),
            current_match: Some(5),
        };
        assert_eq!(count_label(&last), "5 of 5");
        let zero = FindSurface {
            focused: true,
            query: "zzz".into(),
            match_count: Some(0),
            current_match: None,
        };
        assert_eq!(count_label(&zero), "0 of 0");
        // A report with a total but no ordinal reads "0 of 0".
        let no_ordinal = FindSurface {
            focused: true,
            query: "bbr".into(),
            match_count: Some(3),
            current_match: None,
        };
        assert_eq!(count_label(&no_ordinal), "0 of 0");
        // No report yet (count still pending): the count area renders empty.
        let pending = FindSurface {
            focused: true,
            query: "bbr".into(),
            match_count: None,
            current_match: None,
        };
        assert_eq!(count_label(&pending), "");
        let empty_query = FindSurface {
            focused: true,
            query: "".into(),
            match_count: None,
            current_match: None,
        };
        assert_eq!(count_label(&empty_query), "");
    }
}
