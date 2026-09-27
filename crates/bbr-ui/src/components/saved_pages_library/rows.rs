//! Saved Pages Library rows (DESIGN.md §11.16, B3 packet
//! `saved-pages-library`): the §11.16 row anatomy (glyph, monogram favicon
//! tile, display title, mid-ellipsized URL, optional collection label,
//! selection state), the §10 selection box, the §7.2 favicon tile, and the
//! one-level flat collection row.

use gpui::{div, prelude::*, px, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Styled};

use crate::components::favicon::FaviconSpec;
use crate::icons::Icon;
use crate::theme::{Radius, Spacing, Theme};

use super::{
    library_collection_id, library_row_id, library_row_select_id, saved_page_result_id,
    saved_page_result_title_id, saved_page_result_url_id, saved_page_row_rename_cancel_id,
    saved_page_row_rename_confirm_id, saved_page_row_rename_id, saved_page_row_rename_input_id,
    LibraryAction, LibraryCollectionSpec, LibraryItemAction, LibraryRowSpec,
};

/// The §11.16 row id `bbr-saved-pages-row-<bookmark-id>` plus the §15
/// result ids the row folds in — the shared derivation the visible-id
/// mirror uses, kept next to the render. While the row's rename is staged
/// (`editing`), the title/result block and the rename button are replaced
/// by the inline rename input field with its Confirm/Cancel controls, so
/// the mirror advertises exactly the ids the editing render draws.
pub fn library_row_ids(bookmark: u64, editing: bool) -> Vec<SharedString> {
    let mut ids = vec![library_row_id(bookmark), library_row_select_id(bookmark)];
    if editing {
        ids.push(saved_page_row_rename_input_id(bookmark));
        ids.push(saved_page_row_rename_confirm_id(bookmark));
        ids.push(saved_page_row_rename_cancel_id(bookmark));
    } else {
        ids.push(saved_page_result_id(bookmark));
        ids.push(saved_page_result_title_id(bookmark));
        ids.push(saved_page_result_url_id(bookmark));
        ids.push(saved_page_row_rename_id(bookmark));
    }
    ids
}

/// One §11.16 Library row: the drawn §7 bookmark glyph, the monogram
/// favicon tile, the display title, the mid-ellipsized URL, the optional
/// collection label, and the selection state. `on_activate` opens exactly
/// one fresh regular tab through B2; `on_toggle_select` toggles
/// multi-select; `on_rename` stages the inline rename input for this row
/// (the pointer rename entry). While `editing`, the title block and the
/// rename button are replaced by the staged rename input
/// (`saved_page_row_rename_input_id`) with its Confirm/Cancel controls
/// (`on_rename_confirm` / `on_rename_cancel`), so the pointer rename entry
/// is a real inline input, never a no-op.
#[allow(clippy::too_many_arguments)]
pub fn library_row(
    theme: &Theme,
    row: &LibraryRowSpec,
    editing: bool,
    draft: &str,
    on_activate: &LibraryItemAction,
    on_toggle_select: &LibraryItemAction,
    on_rename: &LibraryItemAction,
    on_rename_confirm: &LibraryAction,
    on_rename_cancel: &LibraryAction,
) -> Stateful<Div> {
    let p = theme.palette;
    let bookmark = row.bookmark;
    let activate = on_activate.clone();
    let toggle = on_toggle_select.clone();
    let rename = on_rename.clone();
    let confirm = on_rename_confirm.clone();
    let cancel = on_rename_cancel.clone();
    div()
        .id(library_row_id(bookmark))
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp2())
        .px(Spacing::sp3())
        .py(Spacing::sp1())
        .cursor_pointer()
        .when(row.selected, |el| el.bg(selected_row_fill(&p)))
        .child(selection_box(theme, row.selected))
        .child(favicon_tile(theme, row.favicon.clone()))
        .when(editing, |el| {
            // The staged rename input replaces the title block and the
            // rename button: the draft field plus its Confirm/Cancel
            // controls (the smallest coherent inline rename entry).
            el.child(rename_input_field(theme, bookmark, draft))
                .child(rename_confirm_control(theme, bookmark, confirm))
                .child(rename_cancel_control(theme, bookmark, cancel))
        })
        .when(!editing, |el| {
            el.child(
                div()
                    .id(saved_page_result_id(bookmark))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(Spacing::sp1())
                    .flex_grow()
                    .min_w_0()
                    .on_mouse_down(
                        MouseButton::Left,
                        move |_event: &MouseDownEvent, window, cx| {
                            activate(bookmark, window, cx);
                            cx.stop_propagation();
                        },
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .child(Icon::Bookmark.canvas(theme, theme.density.icon())),
                    )
                    .child(
                        div()
                            .id(saved_page_result_title_id(bookmark))
                            .flex_1()
                            .overflow_hidden()
                            .text_ellipsis()
                            .text_size(theme.type_.title.size)
                            .font_weight(theme.type_.title.weight)
                            .text_color(p.ink)
                            .child(SharedString::from(row.title.clone())),
                    )
                    .child(
                        div()
                            .id(saved_page_result_url_id(bookmark))
                            .max_w(px(280.0))
                            .overflow_hidden()
                            .text_ellipsis()
                            .text_size(theme.type_.mono.size)
                            .text_color(p.ink_secondary)
                            .child(SharedString::from(row.url.clone())),
                    )
                    .when(row.collection.is_some(), |el| {
                        let label = row.collection.clone().unwrap_or_default();
                        el.child(
                            div()
                                .max_w(px(96.0))
                                .overflow_hidden()
                                .text_ellipsis()
                                .text_size(theme.type_.meta.size)
                                .text_color(p.ink_faint)
                                .child(SharedString::from(label)),
                        )
                    }),
            )
            .child(
                div()
                    .id(saved_page_row_rename_id(bookmark))
                    .size(theme.density.hit_min_close())
                    .rounded(Radius.small())
                    .flex()
                    .items_center()
                    .justify_center()
                    .on_mouse_down(
                        MouseButton::Left,
                        move |_event: &MouseDownEvent, window, cx| {
                            rename(bookmark, window, cx);
                            cx.stop_propagation();
                        },
                    )
                    .child(Icon::Pencil.canvas(theme, theme.density.icon())),
            )
        })
        .child(
            div()
                .id(library_row_select_id(bookmark))
                .size(theme.density.hit_min_close())
                .rounded(Radius.small())
                .flex()
                .items_center()
                .justify_center()
                .on_mouse_down(
                    MouseButton::Left,
                    move |_event: &MouseDownEvent, window, cx| {
                        toggle(bookmark, window, cx);
                        cx.stop_propagation();
                    },
                )
                .child(selection_box(theme, row.selected)),
        )
}

/// The staged rename input field: the editable draft chrome with the
/// current draft text (a placeholder when empty) and the accent border
/// that marks the active inline entry. The app owns the draft text model;
/// this renders the field only.
fn rename_input_field(theme: &Theme, bookmark: u64, draft: &str) -> Stateful<Div> {
    let p = theme.palette;
    div()
        .id(saved_page_row_rename_input_id(bookmark))
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
            "Rename…"
        } else {
            draft
        }))
}

/// The staged rename Confirm control (commits the draft through the app's
/// one `library_rename` authority).
fn rename_confirm_control(
    theme: &Theme,
    bookmark: u64,
    on_confirm: LibraryAction,
) -> Stateful<Div> {
    div()
        .id(saved_page_row_rename_confirm_id(bookmark))
        .size(theme.density.hit_min_close())
        .rounded(Radius.small())
        .flex()
        .items_center()
        .justify_center()
        .on_mouse_down(
            MouseButton::Left,
            move |_event: &MouseDownEvent, window, cx| {
                on_confirm(window, cx);
                cx.stop_propagation();
            },
        )
        .child(Icon::Check.canvas(theme, theme.density.icon()))
}

/// The staged rename Cancel control (leaves the input without committing;
/// the row's label is untouched).
fn rename_cancel_control(theme: &Theme, bookmark: u64, on_cancel: LibraryAction) -> Stateful<Div> {
    div()
        .id(saved_page_row_rename_cancel_id(bookmark))
        .size(theme.density.hit_min_close())
        .rounded(Radius.small())
        .flex()
        .items_center()
        .justify_center()
        .on_mouse_down(
            MouseButton::Left,
            move |_event: &MouseDownEvent, window, cx| {
                on_cancel(window, cx);
                cx.stop_propagation();
            },
        )
        .child(Icon::Close.canvas(theme, theme.density.icon()))
}

/// The §7.2 favicon tile for a row (monogram fallback; the normal favicon
/// presentation path — no favicon blob is stored in the record).
pub fn favicon_tile(theme: &Theme, spec: FaviconSpec) -> Div {
    let slate = match theme.mode {
        crate::theme::ThemeMode::Dark => crate::palette::FaviconSlate::dark(spec.hue),
        crate::theme::ThemeMode::Light => crate::palette::FaviconSlate::light(spec.hue),
    };
    div()
        .size(theme.density.icon())
        .rounded(Radius.small())
        .bg(slate)
        .flex()
        .items_center()
        .justify_center()
        .text_size(theme.type_.badge.size)
        .font_weight(theme.type_.badge.weight)
        .text_color(crate::palette::FaviconSlate::letter(theme.mode))
        .child(SharedString::from(spec.letter.to_string()))
}

/// The §10.4 selection surface of a selected Library row. Children keep
/// their own ink colors, so the fill must be the quiet `tint.selected`
/// surface — never an accent fill, which drops ink and URL text below §3.1.
fn selected_row_fill(p: &crate::palette::Palette) -> gpui::Rgba {
    p.tint_selected
}

/// The §10 selection box: a filled accent square when selected, a hollow
/// ring when not.
pub fn selection_box(theme: &Theme, selected: bool) -> Div {
    let p = theme.palette;
    div()
        .size(theme.density.icon())
        .rounded(Radius.small())
        .flex()
        .items_center()
        .justify_center()
        .when(selected, |el| el.bg(p.accent))
        .when(!selected, |el| {
            el.border_1()
                .border_color(p.ink_secondary)
                .rounded(Radius.small())
        })
        .when(selected, |el| {
            el.child(div().size(px(6.0)).rounded_full().bg(p.on_accent))
        })
}

/// One §11.16 flat collection row: the collection label and its member
/// count. `on_action` opens the collection action surface (rename/remove)
/// for this collection.
pub fn library_collection_row(
    theme: &Theme,
    spec: &LibraryCollectionSpec,
    on_action: &LibraryItemAction,
) -> Stateful<Div> {
    let p = theme.palette;
    let id = spec.id;
    let action = on_action.clone();
    div()
        .id(library_collection_id(id))
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
                action(id, window, cx);
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
                .font_weight(theme.type_.title.weight)
                .text_color(p.ink)
                .child(SharedString::from(spec.title.clone())),
        )
        .child(
            div()
                .text_size(theme.type_.meta.size)
                .text_color(p.ink_faint)
                .child(SharedString::from(format!("{}", spec.member_count))),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{contrast, Palette};

    fn palettes() -> [Palette; 4] {
        [
            Palette::light_standard(),
            Palette::light_increased(),
            Palette::dark_standard(),
            Palette::dark_increased(),
        ]
    }

    #[test]
    fn a_selected_row_keeps_its_text_readable() {
        // Regression: the selected row filled with accent and kept ink/URL
        // text (≈ 3.5 : 1 and ≈ 1.6 : 1).
        for p in palettes() {
            let fill = selected_row_fill(&p);
            assert_eq!(fill, p.tint_selected, "§10.4 selection surface");
            assert!(contrast(p.ink, fill) >= 4.5, "title on the selected row");
            assert!(
                contrast(p.ink_secondary, fill) >= 4.5,
                "URL on the selected row"
            );
        }
    }

    #[test]
    fn the_selection_box_mark_is_visible_on_its_accent_fill() {
        // Regression: the selected mark painted accent.ink on accent.
        for p in palettes() {
            assert!(contrast(p.on_accent, p.accent) >= 3.0);
        }
    }
}
