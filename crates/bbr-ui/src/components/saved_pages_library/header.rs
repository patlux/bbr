//! Saved Pages Library header anatomy (DESIGN.md §11.16, B3 packet
//! `saved-pages-library`): the header (title + close), the search field,
//! the sort control, and the count line. Split from `surface` so the
//! surface composition stays under the ~500-line budget.

use gpui::{div, prelude::*, px, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Styled};

use crate::icons::Icon;
use crate::theme::{Radius, Spacing, Theme};

use super::{
    LibraryAction, LibraryCallbacks, LibrarySort, SavedPagesLibraryModel, LIBRARY_CLOSE_ID,
    LIBRARY_COUNT_ID, LIBRARY_SEARCH_ID, LIBRARY_SORT_ID,
};

/// The Library header: the §11.16 title, the search field, the sort
/// control, and the close control.
pub(super) fn library_header(
    theme: &Theme,
    model: &SavedPagesLibraryModel,
    focused: bool,
    callbacks: &LibraryCallbacks,
) -> Div {
    let p = theme.palette;
    let on_close = callbacks.on_close.clone();
    let on_cycle_sort = callbacks.on_cycle_sort.clone();
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp2())
        .px(Spacing::sp3())
        .py(Spacing::sp2())
        .border_b_1()
        .border_color(p.hairline)
        .child(
            div()
                .text_size(theme.type_.title.size)
                .font_weight(theme.type_.title.weight)
                .text_color(p.ink)
                .child(SharedString::from("Saved Pages")),
        )
        .child(library_search_field(
            theme,
            &model.search,
            focused && model.editing_row.is_none() && !model.collection_input_staged(),
        ))
        .child(library_sort_control(theme, model.sort, on_cycle_sort))
        .child(
            div()
                .id(LIBRARY_CLOSE_ID)
                .size(theme.density.hit_min())
                .rounded(Radius.small())
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_e: &MouseDownEvent, window, cx| {
                    on_close(window, cx);
                    cx.stop_propagation();
                })
                .child(Icon::Close.canvas(theme, theme.density.icon())),
        )
}

/// The §11.16 search field (`bbr-saved-pages-search`): the editable field
/// chrome with the current query text. The app owns the text model and
/// routes keyboard input; this renders the field and its focus ring. While
/// a row rename or a collection input is staged, the inline input owns the
/// field focus ring instead.
pub fn library_search_field(theme: &Theme, query: &str, focused: bool) -> Stateful<Div> {
    let p = theme.palette;
    div()
        .id(LIBRARY_SEARCH_ID)
        .flex_grow()
        .max_w(px(320.0))
        .px(Spacing::sp2())
        .py(Spacing::sp1())
        .rounded(Radius.medium())
        .bg(p.surface_field)
        .when(focused, |el| {
            el.border_1()
                .border_color(p.accent)
                .rounded(Radius.medium())
        })
        .child(
            div()
                .text_size(theme.type_.title.size)
                .text_color(if query.is_empty() { p.ink_faint } else { p.ink })
                .child(SharedString::from(if query.is_empty() {
                    "Search Saved Pages…"
                } else {
                    query
                })),
        )
}

/// The §11.16 sort control (`bbr-saved-pages-sort`): the deterministic
/// sort order label plus a disclosure chevron; clicking cycles the order.
pub fn library_sort_control(
    theme: &Theme,
    sort: LibrarySort,
    on_cycle: LibraryAction,
) -> Stateful<Div> {
    let p = theme.palette;
    div()
        .id(LIBRARY_SORT_ID)
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp1())
        .px(Spacing::sp2())
        .py(Spacing::sp1())
        .rounded(Radius.medium())
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, move |_e: &MouseDownEvent, window, cx| {
            on_cycle(window, cx);
            cx.stop_propagation();
        })
        .child(
            div()
                .text_size(theme.type_.meta.size)
                .text_color(p.ink_secondary)
                .child(SharedString::from(sort.label())),
        )
        .child(Icon::Disclosure { expanded: true }.canvas(theme, theme.density.icon()))
}

/// The §11.16 count line: the windowed presentation of the total.
pub fn library_count_line(theme: &Theme, model: &SavedPagesLibraryModel) -> Stateful<Div> {
    let p = theme.palette;
    let label = if model.total == 1 {
        "1 Saved Page".to_string()
    } else {
        format!("{} Saved Pages", model.total)
    };
    div()
        .id(LIBRARY_COUNT_ID)
        .px(Spacing::sp3())
        .py(Spacing::sp1())
        .text_size(theme.type_.meta.size)
        .text_color(p.ink_faint)
        .child(SharedString::from(label))
}
