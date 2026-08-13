//! Saved Pages Library selection toolbar and move-to sheet (DESIGN.md
//! §11.16, B3 packet `saved-pages-library`): the production controls the
//! multi-select set exposes — the count, the Clear control, the Remove
//! control (stages the destructive confirmation), the Move control
//! (opens/closes the move-to sheet), and the sheet's per-collection and
//! "No collection" options (move the selection into/out of a flat
//! collection). Split from `surface` so no surface module outgrows the
//! ~500-line budget.

use gpui::{div, prelude::*, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Styled};

use crate::icons::Icon;
use crate::theme::{Radius, Spacing, Theme};

use super::{
    LibraryAction, LibraryCallbacks, LibraryCollectionSpec, LIBRARY_MOVE_NONE_ID,
    LIBRARY_SELECTION_CLEAR_ID, LIBRARY_SELECTION_ID, LIBRARY_SELECTION_MOVE_ID,
    LIBRARY_SELECTION_REMOVE_ID,
};

/// The §11.16 selection toolbar: the selected count plus the Clear, Remove,
/// and Move controls. Rendered only while the multi-select set is
/// non-empty; the Remove control stages the destructive confirmation for
/// exactly the selected records and the Move control toggles the move-to
/// sheet. Every control dispatches through the app's production methods
/// (the normal binary reaches selection clear/remove-stage/move).
pub fn library_selection_toolbar(
    theme: &Theme,
    selected_count: usize,
    move_sheet_open: bool,
    callbacks: &LibraryCallbacks,
) -> Stateful<Div> {
    let p = theme.palette;
    let noun = if selected_count == 1 {
        "1 selected".to_string()
    } else {
        format!("{selected_count} selected")
    };
    div()
        .id(LIBRARY_SELECTION_ID)
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp2())
        .px(Spacing::sp3())
        .py(Spacing::sp1())
        .border_b_1()
        .border_color(p.hairline)
        .child(
            div()
                .flex_1()
                .text_size(theme.type_.meta.size)
                .text_color(p.ink_secondary)
                .child(SharedString::from(noun)),
        )
        .child(toolbar_button(
            theme,
            LIBRARY_SELECTION_CLEAR_ID,
            "Clear",
            callbacks.on_clear_selection.clone(),
        ))
        .child(toolbar_button(
            theme,
            LIBRARY_SELECTION_REMOVE_ID,
            "Remove",
            callbacks.on_remove_selection.clone(),
        ))
        .child(toolbar_button(
            theme,
            LIBRARY_SELECTION_MOVE_ID,
            if move_sheet_open {
                "Done"
            } else {
                "Move to…"
            },
            callbacks.on_toggle_move_sheet.clone(),
        ))
}

/// The move-to sheet: one option per flat collection (moves the selection
/// into it) plus the "No collection" option (moves the selection out of any
/// collection). Rendered below the selection toolbar while open; every
/// option dispatches through the app's production
/// `library_move_selection` authority.
pub fn library_move_sheet(
    theme: &Theme,
    collections: &[LibraryCollectionSpec],
    callbacks: &LibraryCallbacks,
) -> Div {
    let p = theme.palette;
    div()
        .flex()
        .flex_col()
        .px(Spacing::sp3())
        .py(Spacing::sp1())
        .border_b_1()
        .border_color(p.hairline)
        .children(
            collections
                .iter()
                .map(|collection| {
                    let id = super::library_move_id(collection.id);
                    let action = callbacks.on_move_into.clone();
                    let collection_id = collection.id;
                    div()
                        .id(id)
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(Spacing::sp2())
                        .px(Spacing::sp2())
                        .py(Spacing::sp1())
                        .rounded(Radius.small())
                        .cursor_pointer()
                        .on_mouse_down(
                            MouseButton::Left,
                            move |_event: &MouseDownEvent, window, cx| {
                                action(collection_id, window, cx);
                                cx.stop_propagation();
                            },
                        )
                        .child(Icon::Folder { open: false }.canvas(theme, theme.density.icon()))
                        .child(
                            div()
                                .flex_1()
                                .overflow_hidden()
                                .text_ellipsis()
                                .text_size(theme.type_.title.size)
                                .text_color(p.ink)
                                .child(SharedString::from(collection.title.clone())),
                        )
                        .child(
                            div()
                                .text_size(theme.type_.meta.size)
                                .text_color(p.ink_faint)
                                .child(SharedString::from(format!("{}", collection.member_count))),
                        )
                        .into_any()
                })
                .chain(std::iter::once(
                    move_sheet_option(theme, callbacks.on_move_out.clone()).into_any(),
                )),
        )
}

/// One toolbar control: the shared button anatomy (hit target, radius,
/// label) with its deterministic §15 id.
fn toolbar_button(
    theme: &Theme,
    id: &'static str,
    label: &str,
    on_click: LibraryAction,
) -> Stateful<Div> {
    let p = theme.palette;
    div()
        .id(id)
        .px(Spacing::sp2())
        .py(Spacing::sp1())
        .rounded(Radius.medium())
        .bg(p.surface_field)
        .text_size(theme.type_.meta.size)
        .text_color(p.ink)
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            move |_event: &MouseDownEvent, window, cx| {
                on_click(window, cx);
                cx.stop_propagation();
            },
        )
        .child(SharedString::from(label.to_string()))
}

/// The move-to sheet's "No collection" option: moves the selection out of
/// any collection (clears the membership through the one AppStore
/// authority).
fn move_sheet_option(theme: &Theme, on_move_out: LibraryAction) -> Stateful<Div> {
    let p = theme.palette;
    div()
        .id(LIBRARY_MOVE_NONE_ID)
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp2())
        .px(Spacing::sp2())
        .py(Spacing::sp1())
        .rounded(Radius.small())
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            move |_event: &MouseDownEvent, window, cx| {
                on_move_out(window, cx);
                cx.stop_propagation();
            },
        )
        .child(Icon::Stopped.canvas(theme, theme.density.icon()))
        .child(
            div()
                .flex_1()
                .text_size(theme.type_.title.size)
                .text_color(p.ink_secondary)
                .child(SharedString::from("No collection")),
        )
}

/// The §11.16 selection toolbar plus the move-to sheet (while open): the
/// whole multi-select action strip the surface places above the list.
pub fn library_selection_strip(
    theme: &Theme,
    model: &super::SavedPagesLibraryModel,
    callbacks: &LibraryCallbacks,
) -> Vec<gpui::AnyElement> {
    let mut strip = Vec::new();
    if model.selected.is_empty() {
        return strip;
    }
    strip.push(
        library_selection_toolbar(
            theme,
            model.selected.len(),
            model.move_sheet_open,
            callbacks,
        )
        .into_any(),
    );
    if model.move_sheet_open {
        strip.push(library_move_sheet(theme, &model.collections, callbacks).into_any());
    }
    strip
}
