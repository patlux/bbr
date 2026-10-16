//! Blueprint D05 (DESIGN.md §11.3/§11.13): the Delete Space confirmation
//! sheet — a blocking sheet attached to the content card's top edge that
//! asks once, names what goes, and offers no undo beyond Reopen Closed Tab.
//!
//! Pure presentation: the app owns the bound `SpaceId`, the counts (from
//! the organization authority), the portal placement over the card and the
//! veil, the keyboard (Return triggers the focused button, which is Cancel
//! when the sheet opens; Tab moves between the two; Escape cancels) and
//! the commit. Delete is the danger button — its label names the action
//! and paints in `status.error`, never colour alone.

use gpui::{
    div, prelude::*, App, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Window,
};

use crate::theme::elevation::elev2;
use crate::theme::{Spacing, Theme};

/// Stable §15 ids of the sheet.
pub const SPACE_DELETE_SHEET_ID: &str = "bbr-space-delete-sheet";
pub const SPACE_DELETE_TITLE_ID: &str = "bbr-space-delete-title";
pub const SPACE_DELETE_BODY_ID: &str = "bbr-space-delete-body";
pub const SPACE_DELETE_CANCEL_ID: &str = "bbr-space-delete-cancel";
pub const SPACE_DELETE_CONFIRM_ID: &str = "bbr-space-delete-confirm";
/// The §15 id of the veil the portal paints over the content card.
pub const SPACE_DELETE_VEIL_ID: &str = "bbr-space-delete-veil";

/// The sheet width (board P-4, the §11.4 sheet family).
pub const SPACE_DELETE_SHEET_WIDTH: f32 = 400.0;

/// Which of the two buttons owns the keyboard (Return triggers it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpaceDeleteButton {
    #[default]
    Cancel,
    Delete,
}

/// The sheet's presentation model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceDeleteModel {
    pub title: SharedString,
    pub body: SharedString,
    pub focused: SpaceDeleteButton,
}

/// The sheet's title line: `Delete “<space>”?`.
pub fn delete_title(space: &str) -> String {
    format!("Delete “{space}”?")
}

/// The sheet's body copy: what closes and what stays. Counts read
/// singular/plural; favorites are named only when some of the tabs are
/// favorites (their tiles leave the grid with the tabs).
pub fn delete_body(tabs: usize, folders: usize, favorites: usize) -> String {
    let plural = |n: usize, one: &str, many: &str| {
        if n == 1 {
            format!("1 {one}")
        } else {
            format!("{n} {many}")
        }
    };
    let mut sentences = Vec::new();
    sentences.push(match (tabs, folders) {
        (0, 0) => "It is empty.".to_string(),
        (0, 1) => "Its 1 folder closes.".to_string(),
        (0, m) => format!("Its {m} folders close."),
        (1, 0) => "Its 1 tab closes.".to_string(),
        (n, 0) => format!("Its {n} tabs close."),
        (n, m) => format!(
            "Its {} and {} close.",
            plural(n, "tab", "tabs"),
            plural(m, "folder", "folders")
        ),
    });
    match favorites {
        0 => {}
        1 => sentences.push("1 of them is a favorite; its tile goes too.".to_string()),
        n => sentences.push(format!("{n} of them are favorites; their tiles go too.")),
    }
    if tabs > 0 {
        sentences.push("Reopen Closed Tab brings tabs back.".to_string());
    }
    sentences.push("History and site data stay in the profile.".to_string());
    sentences.join(" ")
}

/// The sheet card (`bbr-space-delete-sheet`): `radius:lg`, overlay surface,
/// hairline edge and `elev:2`, `sp:5` padding; the title, the body and the
/// trailing button row `[Cancel] [Delete Space]`. The caller places it and
/// stops propagation on the backdrop.
pub fn space_delete_sheet(
    theme: &Theme,
    model: &SpaceDeleteModel,
    on_cancel: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    on_confirm: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    div()
        .id(SPACE_DELETE_SHEET_ID)
        .debug_selector(|| SPACE_DELETE_SHEET_ID.to_string())
        // Testkit-only: the drawn sheet rect (`sheet_rects`).
        .children(crate::rect_probe::rect_probe(SPACE_DELETE_SHEET_ID))
        .w(gpui::px(SPACE_DELETE_SHEET_WIDTH))
        .rounded(theme.radius.large())
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .shadow(elev2(theme))
        .p(Spacing::sp5())
        .flex()
        .flex_col()
        .gap(Spacing::sp3())
        .child(
            div()
                .id(SPACE_DELETE_TITLE_ID)
                .text_size(theme.type_.title.size)
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(p.ink)
                .child(model.title.clone()),
        )
        .child(
            div()
                .id(SPACE_DELETE_BODY_ID)
                .text_size(theme.type_.label.size)
                .text_color(p.ink_secondary)
                .child(model.body.clone()),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .justify_end()
                .gap(Spacing::sp2())
                .child(button(
                    theme,
                    SPACE_DELETE_CANCEL_ID,
                    "Cancel",
                    false,
                    model.focused == SpaceDeleteButton::Cancel,
                    on_cancel,
                ))
                .child(button(
                    theme,
                    SPACE_DELETE_CONFIRM_ID,
                    "Delete Space",
                    true,
                    model.focused == SpaceDeleteButton::Delete,
                    on_confirm,
                )),
        )
}

/// One sheet button (board P-4): the `field` surface with a 1 pt
/// `border.strong` edge, `hit-min` tall; the danger button's text names the
/// action ("Delete Space") in `status.error`, so colour is never the only
/// cue. The keyboard-focused button wears the 2 pt §9 accent ring.
fn button(
    theme: &Theme,
    id: &'static str,
    label: &'static str,
    danger: bool,
    focused: bool,
    on_press: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    div()
        .id(id)
        .debug_selector(move || id.to_string())
        .h(theme.density.hit_min())
        .px(Spacing::sp3())
        .rounded(theme.radius.medium())
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .bg(p.surface_field)
        .border_1()
        // The §9 2 pt accent ring: the 1 pt edge turns accent and a 1 pt
        // spread ring completes it, so the button never changes size when
        // the keyboard moves.
        .border_color(if focused { p.accent } else { p.border_strong })
        .when(focused, |el| {
            el.shadow(vec![gpui::BoxShadow {
                color: p.accent.into(),
                offset: gpui::point(gpui::px(0.0), gpui::px(0.0)),
                blur_radius: gpui::px(0.0),
                spread_radius: gpui::px(1.0),
                inset: false,
            }])
        })
        .text_size(theme.type_.label.size)
        .font_weight(theme.type_.label.weight)
        .text_color(if danger { p.status_error } else { p.ink })
        .child(label)
        .on_mouse_down(MouseButton::Left, on_press)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_title_quotes_the_space_name() {
        assert_eq!(delete_title("Research"), "Delete “Research”?");
    }

    #[test]
    fn the_body_names_what_closes_with_singular_and_plural_counts() {
        assert_eq!(
            delete_body(14, 2, 0),
            "Its 14 tabs and 2 folders close. Reopen Closed Tab brings tabs back. \
             History and site data stay in the profile."
        );
        assert_eq!(
            delete_body(1, 1, 1),
            "Its 1 tab and 1 folder close. 1 of them is a favorite; its tile goes too. \
             Reopen Closed Tab brings tabs back. History and site data stay in the profile."
        );
        assert_eq!(
            delete_body(1, 0, 2),
            "Its 1 tab closes. 2 of them are favorites; their tiles go too. \
             Reopen Closed Tab brings tabs back. History and site data stay in the profile."
        );
        assert_eq!(
            delete_body(0, 1, 0),
            "Its 1 folder closes. History and site data stay in the profile."
        );
        assert_eq!(
            delete_body(0, 0, 0),
            "It is empty. History and site data stay in the profile."
        );
    }

    #[test]
    fn cancel_owns_the_keyboard_by_default() {
        assert_eq!(SpaceDeleteButton::default(), SpaceDeleteButton::Cancel);
    }
}
