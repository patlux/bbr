//! Saved Pages Library collection action surface (DESIGN.md §11.16, B3
//! packet `saved-pages-library`): the collection create entry, the staged
//! create/rename input surface, and the destructive collection remove
//! confirmation. Split from `surface` so the surface composition stays
//! under the ~500-line budget.

use gpui::{div, prelude::*, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Styled};

use crate::icons::Icon;
use crate::theme::{Radius, Spacing, Theme};

use super::{
    library_collection_remove_cancel_id, library_collection_remove_confirm_id,
    library_collection_remove_ok_id, library_collection_rename_cancel_id,
    library_collection_rename_confirm_id, library_collection_rename_input_id, LibraryAction,
    LibraryCallbacks, LibraryCollectionAction, SavedPagesLibraryModel,
    LIBRARY_COLLECTION_CREATE_CANCEL_ID, LIBRARY_COLLECTION_CREATE_CONFIRM_ID,
    LIBRARY_COLLECTION_CREATE_ID, LIBRARY_COLLECTION_CREATE_INPUT_ID,
};

/// The collection create entry (`+ New collection`) at the end of the flat
/// collections strip: stages the inline create input (the collection
/// action surface's create operation). While the create input is staged it
/// is replaced by the staged create input with its Confirm/Cancel
/// controls (the `bbr-saved-pages-collection-create` slot keeps its stable
/// id through both forms).
pub(super) fn collection_create_entry(
    theme: &Theme,
    model: &SavedPagesLibraryModel,
    callbacks: &LibraryCallbacks,
) -> gpui::AnyElement {
    let p = theme.palette;
    if model.collection_create_staged {
        return collection_input_surface(
            theme,
            SharedString::from(LIBRARY_COLLECTION_CREATE_INPUT_ID),
            SharedString::from(LIBRARY_COLLECTION_CREATE_CONFIRM_ID),
            SharedString::from(LIBRARY_COLLECTION_CREATE_CANCEL_ID),
            &model.collection_draft,
            callbacks.on_collection_confirm.clone(),
            callbacks.on_collection_cancel.clone(),
        )
        .into_any();
    }
    let on_create = callbacks.on_collection_create.clone();
    div()
        .id(LIBRARY_COLLECTION_CREATE_ID)
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp2())
        .px(Spacing::sp3())
        .py(Spacing::sp1())
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            move |_event: &MouseDownEvent, window, cx| {
                on_create(window, cx);
                cx.stop_propagation();
            },
        )
        .child(Icon::Plus.canvas(theme, theme.density.icon()))
        .child(
            div()
                .text_size(theme.type_.title.size)
                .text_color(p.ink_secondary)
                .child(SharedString::from("New collection")),
        )
        .into_any()
}

/// The staged collection action surface: the inline rename input (seeded
/// with the collection's current title) with Confirm/Cancel, or the
/// destructive remove confirmation (members move out first), rendered
/// below the collections strip while the pointer cycle stages an action.
pub(super) fn collection_action_surface(
    theme: &Theme,
    model: &SavedPagesLibraryModel,
    callbacks: &LibraryCallbacks,
) -> Vec<gpui::AnyElement> {
    let Some((id, action)) = model.collection_action else {
        return Vec::new();
    };
    match action {
        LibraryCollectionAction::Rename => vec![collection_input_surface(
            theme,
            library_collection_rename_input_id(id),
            library_collection_rename_confirm_id(id),
            library_collection_rename_cancel_id(id),
            &model.collection_draft,
            callbacks.on_collection_confirm.clone(),
            callbacks.on_collection_cancel.clone(),
        )
        .into_any()],
        LibraryCollectionAction::Remove => {
            vec![collection_remove_surface(theme, id, callbacks).into_any()]
        }
    }
}

/// The inline collection rename/create input: the draft field chrome plus
/// its Confirm/Cancel controls (the collection action surface's rename and
/// create operations). The app owns the draft text model; this renders the
/// field only (the root is a plain container — the interactive Confirm /
/// Cancel controls carry their own stable ids).
fn collection_input_surface(
    theme: &Theme,
    input_id: SharedString,
    confirm_id: SharedString,
    cancel_id: SharedString,
    draft: &str,
    on_confirm: LibraryAction,
    on_cancel: LibraryAction,
) -> Div {
    let p = theme.palette;
    let confirm = on_confirm.clone();
    let cancel = on_cancel.clone();
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp2())
        .px(Spacing::sp3())
        .py(Spacing::sp1())
        .child(
            div()
                .id(input_id)
                .flex_grow()
                .min_w_0()
                .px(Spacing::sp2())
                .py(Spacing::sp1())
                .rounded(Radius.medium())
                .border_1()
                .border_color(p.accent)
                .bg(p.surface_field)
                .text_size(theme.type_.title.size)
                .text_color(if draft.is_empty() { p.ink_faint } else { p.ink })
                .child(SharedString::from(if draft.is_empty() {
                    "Collection name…"
                } else {
                    draft
                })),
        )
        .child(
            div()
                .id(confirm_id)
                .size(theme.density.hit_min_close())
                .rounded(Radius.small())
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, window, cx| {
                    confirm(window, cx);
                    cx.stop_propagation();
                })
                .child(Icon::Check.canvas(theme, theme.density.icon())),
        )
        .child(
            div()
                .id(cancel_id)
                .size(theme.density.hit_min_close())
                .rounded(Radius.small())
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, window, cx| {
                    cancel(window, cx);
                    cx.stop_propagation();
                })
                .child(Icon::Close.canvas(theme, theme.density.icon())),
        )
}

/// The staged destructive collection remove: a typed row (members move out
/// first, records are never lost) with Confirm/Cancel.
fn collection_remove_surface(
    theme: &Theme,
    id: u64,
    callbacks: &LibraryCallbacks,
) -> Stateful<Div> {
    let p = theme.palette;
    let confirm = callbacks.on_collection_confirm.clone();
    let cancel = callbacks.on_collection_cancel.clone();
    div()
        .id(library_collection_remove_confirm_id(id))
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp3())
        .px(Spacing::sp3())
        .py(Spacing::sp2())
        .border_b_1()
        .border_color(p.hairline)
        .child(
            div()
                .flex_1()
                .text_size(theme.type_.title.size)
                .text_color(p.status_error)
                .child(SharedString::from(
                    "Remove collection? Members move out first — records are never lost.",
                )),
        )
        .child(
            div()
                .id(library_collection_remove_ok_id(id))
                .px(Spacing::sp3())
                .py(Spacing::sp1())
                .rounded(Radius.medium())
                .bg(p.status_error)
                .text_size(theme.type_.meta.size)
                .text_color(p.surface_glass)
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, window, cx| {
                    confirm(window, cx);
                })
                .child(SharedString::from("Remove")),
        )
        .child(
            div()
                .id(library_collection_remove_cancel_id(id))
                .px(Spacing::sp3())
                .py(Spacing::sp1())
                .rounded(Radius.medium())
                .bg(p.surface_field)
                .text_size(theme.type_.meta.size)
                .text_color(p.ink)
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, window, cx| {
                    cancel(window, cx);
                })
                .child(SharedString::from("Cancel")),
        )
}
