//! Saved Pages Library surface composition (DESIGN.md §11.16, B3 packet
//! `saved-pages-library`): the surface root that assembles the header
//! ([`super::header`]), the selection toolbar + move sheet
//! ([`super::toolbar`]), the flat collections strip with the staged
//! collection action surface ([`super::collection_surface`]), the bounded
//! row list with the production wheel listener, and the decided states
//! (loading / empty / no results / remove confirm). The app passes its
//! production listener bundle ([`LibraryCallbacks`]); this module owns the
//! composition anatomy only.

use gpui::{
    div, prelude::*, Div, MouseButton, MouseDownEvent, ScrollWheelEvent, SharedString, Stateful,
    Styled,
};

use crate::theme::{Radius, Spacing, Theme};

use super::collection_surface::{collection_action_surface, collection_create_entry};
use super::header::{library_count_line, library_header};
use super::scroll::{scroll_offset_for_wheel, wheel_rows};
use super::{
    library_collection_row, library_row, LibraryAction, LibraryCallbacks, SavedPagesLibraryModel,
    LIBRARY_EMPTY_ID, LIBRARY_ID, LIBRARY_LOADING_ID, LIBRARY_NO_RESULTS_ID,
    LIBRARY_REMOVE_CONFIRM_ID,
};

/// The Library surface: header (title + close), the §11.16 search field
/// and sort control, the selection toolbar + move sheet (while any record
/// is selected), the count line, the flat collections strip with the
/// create entry and the staged collection action surface, the bounded row
/// list with the production wheel listener, and the decided states
/// (loading / empty / no results / remove confirm). The app passes its
/// production listener bundle; this component owns the anatomy only.
pub fn library_surface(
    theme: &Theme,
    model: &SavedPagesLibraryModel,
    focused: bool,
    callbacks: &LibraryCallbacks,
) -> Stateful<Div> {
    let p = theme.palette;
    let content: Vec<gpui::AnyElement> = if model.is_loading() {
        vec![library_state_row(
            theme,
            SharedString::from(LIBRARY_LOADING_ID),
            "Loading Saved Pages…",
        )
        .into_any()]
    } else if model.is_empty() {
        vec![library_state_row(
            theme,
            SharedString::from(LIBRARY_EMPTY_ID),
            "No Saved Pages yet — press Cmd-D on any page to save it.",
        )
        .into_any()]
    } else if model.has_no_results() {
        vec![library_state_row(
            theme,
            SharedString::from(LIBRARY_NO_RESULTS_ID),
            &format!("No Saved Pages match “{}”.", model.search),
        )
        .into_any()]
    } else {
        model
            .rows
            .iter()
            .map(|row| {
                library_row(
                    theme,
                    row,
                    model.editing_row == Some(row.bookmark),
                    &model.rename_draft,
                    &callbacks.on_activate,
                    &callbacks.on_toggle_select,
                    &callbacks.on_rename,
                    &callbacks.on_rename_confirm,
                    &callbacks.on_rename_cancel,
                )
                .into_any()
            })
            .collect()
    };

    div()
        .id(LIBRARY_ID)
        .flex()
        .flex_col()
        .size_full()
        .bg(p.surface_canvas)
        .child(library_header(theme, model, focused, callbacks))
        // The selection toolbar + move sheet sit above the list (they act
        // on the multi-select set, not on a row).
        .children(super::toolbar::library_selection_strip(
            theme, model, callbacks,
        ))
        .child(
            div()
                .id(SharedString::from("bbr-saved-pages-list"))
                .flex()
                .flex_col()
                .flex_grow()
                .min_h_0()
                .overflow_y_scroll()
                .on_scroll_wheel(scroll_listener(
                    model.scroll_offset,
                    model.total,
                    model.viewport_rows,
                    callbacks.on_scroll.clone(),
                ))
                .child(library_count_line(theme, model))
                .children(
                    model
                        .collections
                        .iter()
                        .map(|collection| {
                            library_collection_row(
                                theme,
                                collection,
                                &callbacks.on_collection_action,
                            )
                            .into_any()
                        })
                        .chain(std::iter::once(
                            collection_create_entry(theme, model, callbacks).into_any(),
                        )),
                )
                .children(collection_action_surface(theme, model, callbacks))
                .children(content),
        )
        .when(model.remove_confirm, |surface| {
            surface.child(library_remove_confirm(
                theme,
                model.remove_targets.len(),
                &callbacks.on_confirm_remove,
                &callbacks.on_cancel_remove,
            ))
        })
}

/// The production wheel listener for the bounded list: converts the wheel
/// delta into row units, clamps the tracked scroll offset to the last
/// full-window start, and dispatches the new absolute offset through the
/// app's one `library_set_scroll` authority. The model window derives from
/// exactly this offset, so the wheel path and the typed scroll path never
/// disagree. All captured values are owned (the offset/total/viewport are
/// numbers and the callback is the app's `Rc` handle), so the listener is
/// `'static` exactly like the other element listeners.
fn scroll_listener(
    scroll_offset: usize,
    total: usize,
    viewport: usize,
    on_scroll: super::LibraryScrollAction,
) -> impl Fn(&ScrollWheelEvent, &mut gpui::Window, &mut gpui::App) + 'static {
    move |event: &ScrollWheelEvent, window, cx| {
        let delta_rows = wheel_rows(&event.delta);
        if delta_rows == 0 {
            return;
        }
        let next = scroll_offset_for_wheel(scroll_offset, total, viewport, delta_rows);
        on_scroll(next, window, cx);
        cx.stop_propagation();
    }
}

/// The §11.16 destructive remove confirmation: a typed surface listing the
/// target count with Confirm/Cancel. Never closes or mutates an open tab.
pub fn library_remove_confirm(
    theme: &Theme,
    count: usize,
    on_confirm: &LibraryAction,
    on_cancel: &LibraryAction,
) -> Stateful<Div> {
    let p = theme.palette;
    let confirm = on_confirm.clone();
    let cancel = on_cancel.clone();
    let noun = if count == 1 {
        "1 Saved Page".to_string()
    } else {
        format!("{count} Saved Pages")
    };
    div()
        .id(LIBRARY_REMOVE_CONFIRM_ID)
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp3())
        .px(Spacing::sp3())
        .py(Spacing::sp2())
        .border_t_1()
        .border_color(p.hairline)
        .child(
            div()
                .flex_1()
                .text_size(theme.type_.title.size)
                .text_color(p.status_error)
                .child(SharedString::from(format!(
                    "Remove {noun}? Open tabs are never touched."
                ))),
        )
        .child(
            div()
                .id(SharedString::from("bbr-saved-pages-remove-confirm-ok"))
                .px(Spacing::sp3())
                .py(Spacing::sp1())
                .rounded(Radius.medium())
                .bg(p.status_error)
                .text_size(theme.type_.meta.size)
                .text_color(p.surface_glass)
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, window, cx| {
                    confirm(window, cx)
                })
                .child(SharedString::from("Remove")),
        )
        .child(
            div()
                .id(SharedString::from("bbr-saved-pages-remove-confirm-cancel"))
                .px(Spacing::sp3())
                .py(Spacing::sp1())
                .rounded(Radius.medium())
                .bg(p.surface_field)
                .text_size(theme.type_.meta.size)
                .text_color(p.ink)
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_: &MouseDownEvent, window, cx| {
                    cancel(window, cx)
                })
                .child(SharedString::from("Cancel")),
        )
}

/// One typed state row (loading/empty/no-results) with its stable §15 id.
pub fn library_state_row(theme: &Theme, id: SharedString, text: &str) -> Stateful<Div> {
    div()
        .id(id)
        .px(Spacing::sp3())
        .py(Spacing::sp2())
        .text_size(theme.type_.meta.size)
        .text_color(theme.palette.ink_faint)
        .child(SharedString::from(text.to_string()))
}
