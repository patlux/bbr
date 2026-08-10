//! DESIGN.md §11.16 Saved Pages Library: the one dedicated ordinary GPUI
//! Library surface (B3 packet `saved-pages-library`).
//!
//! The full collection lives in one GPUI surface — never a permanent
//! Sidebar region, never a WebKit page or Portal. The surface presents the
//! settled states (empty, loading, populated, no results, single/multi
//! selection, destructive removal confirmation, long title/URL, missing
//! title, missing favicon, flat collections, and 1k bounded rendering),
//! search, deterministic sort, rename, remove, multi-select, and one-level
//! flat collections through the shared Chassis tokens.
//!
//! This module is **presentation only**: the model is a `bbr-ui` DTO built
//! by `bbr-app` from the loaded in-memory snapshot (B2 provider), and the
//! callbacks are the GPUI handler closures the app passes. No repository,
//! controller, engine, or network appears here.
//!
//! # Bounded rendering
//!
//! The surface renders exactly [`SavedPagesLibraryModel::rows`] — the
//! bounded window the app derives from the deterministic full result order
//! and the production scroll offset. The pure [`visible_window`] math is
//! owned here so the "the Library does not lay out all 1,000 rows" contract
//! is testable without GPUI.

use std::rc::Rc;

use gpui::{
    div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Styled,
    Window,
};

use crate::components::favicon::{monogram_hue, monogram_letter, FaviconSpec};
use crate::icons::Icon;
use crate::text::mid_ellipsis;
use crate::theme::{Radius, Spacing, Theme};

/// The §15 id of the Library surface.
pub const LIBRARY_ID: &str = "bbr-saved-pages-library";
/// The §15 id of the Library search field.
pub const LIBRARY_SEARCH_ID: &str = "bbr-saved-pages-search";
/// The §15 id of the Library sort control.
pub const LIBRARY_SORT_ID: &str = "bbr-saved-pages-sort";
/// The §15 id of the destructive remove confirmation.
pub const LIBRARY_REMOVE_CONFIRM_ID: &str = "bbr-saved-pages-remove-confirm";
/// The §15 id of the empty Library state.
pub const LIBRARY_EMPTY_ID: &str = "bbr-saved-pages-empty";
/// The §15 id of the loading state.
pub const LIBRARY_LOADING_ID: &str = "bbr-saved-pages-loading";
/// The §15 id of the no-search-results state.
pub const LIBRARY_NO_RESULTS_ID: &str = "bbr-saved-pages-no-results";
/// The §15 id of the count line.
pub const LIBRARY_COUNT_ID: &str = "bbr-saved-pages-count";
/// The §15 id of the close control.
pub const LIBRARY_CLOSE_ID: &str = "bbr-saved-pages-close";
/// The §15 id of the Collections section header.
pub const LIBRARY_COLLECTIONS_ID: &str = "bbr-saved-pages-collections";

/// The §15 result-row id `bbr-saved-page-result-<bookmark-id>` (DESIGN.md
/// §11.16 — shared by the Palette result and the Library row).
pub fn saved_page_result_id(bookmark: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-page-result-{bookmark}"))
}

/// The §15 result-title id `bbr-saved-page-result-title-<bookmark-id>`.
pub fn saved_page_result_title_id(bookmark: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-page-result-title-{bookmark}"))
}

/// The §15 result-url id `bbr-saved-page-result-url-<bookmark-id>`.
pub fn saved_page_result_url_id(bookmark: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-page-result-url-{bookmark}"))
}

/// The §15 Library-row id `bbr-saved-pages-row-<bookmark-id>`.
pub fn library_row_id(bookmark: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-pages-row-{bookmark}"))
}

/// The §15 collection-row id `bbr-saved-pages-collection-<collection-id>`.
pub fn library_collection_id(collection: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-pages-collection-{collection}"))
}

/// The §11.16 Library stage: the repository snapshot is loaded once at
/// startup (B2 provider), so production opens `Ready`; `Loading` is the
/// typed contract state the surface renders while a snapshot is being
/// attached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryStage {
    Loading,
    Ready,
}

/// Deterministic Library sort orders (§11.16 "deterministic sort"; the
/// order never invents a second sort rule — `UpdatedNewest` is exactly the
/// provider's presentation order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibrarySort {
    /// Most recently updated first (the B2 provider presentation order).
    #[default]
    UpdatedNewest,
    /// Least recently updated first.
    UpdatedOldest,
    /// Display title ascending (bytewise, deterministic).
    TitleAscending,
    /// Display title descending.
    TitleDescending,
}

/// The bounded render window: `[first, last)` indices into the
/// deterministic full result order plus the stable scroll anchor (the
/// `BookmarkId` of the first visible row).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LibraryWindow {
    pub first: usize,
    /// Exclusive end of the rendered window.
    pub last: usize,
    /// The stable `BookmarkId` of the first visible row; `None` when the
    /// window is empty.
    pub anchor: Option<u64>,
}

/// One rendered Library row (§11.16 row contract): bookmark/fallback glyph
/// or normal favicon presentation, user label or captured-title fallback,
/// mid-ellipsized URL, optional collection label, and selection state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryRowSpec {
    /// The stable `BookmarkId` (the §15 id suffix).
    pub bookmark: u64,
    /// Display title: user label → captured title → hostname/URL fallback
    /// (the §11.16 ladder, already derived).
    pub title: String,
    /// Mid-ellipsized canonical URL (already budgeted).
    pub url: String,
    /// Optional one-level collection label (`type:meta`).
    pub collection: Option<String>,
    /// §7.2 monogram fallback (the normal favicon path; no favicon blob is
    /// ever stored in a Saved Page record).
    pub favicon: FaviconSpec,
    /// The §10 `selected` state.
    pub selected: bool,
}

/// One flat collection row (§11.16 flat collections, one level only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryCollectionSpec {
    pub id: u64,
    pub title: String,
    pub member_count: usize,
}

/// The full Library presentation model: plain data, no repository. The app
/// builds it from the loaded in-memory snapshot plus its local search/
/// sort/selection/scroll state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedPagesLibraryModel {
    pub stage: LibraryStage,
    /// The active search query (empty = no filter).
    pub search: String,
    pub sort: LibrarySort,
    /// The bounded window the surface renders — never the full collection.
    pub rows: Vec<LibraryRowSpec>,
    /// The total number of matching records behind the window.
    pub total: usize,
    /// The windowed render bounds (the bounded-render evidence).
    pub window: LibraryWindow,
    /// The flat collections, one level only.
    pub collections: Vec<LibraryCollectionSpec>,
    /// True while the destructive remove confirmation is presented.
    pub remove_confirm: bool,
    /// The ids targeted by the pending destructive remove.
    pub remove_targets: Vec<u64>,
}

impl SavedPagesLibraryModel {
    /// The §11.16 decided states as booleans the surface and the snapshot
    /// share: loading, empty (ready with no records), and no-search-results
    /// (ready with records but none matching).
    pub fn is_loading(&self) -> bool {
        self.stage == LibraryStage::Loading
    }

    pub fn is_empty(&self) -> bool {
        self.stage == LibraryStage::Ready && self.total == 0
    }

    pub fn has_no_results(&self) -> bool {
        self.stage == LibraryStage::Ready && self.total > 0 && self.rows.is_empty()
    }
}

/// The deterministic bounded window into `total` results for a fixed
/// viewport (rows visible) at `scroll_offset` (rows scrolled past). The
/// window is `[scroll_offset, min(total, scroll_offset + viewport))` with
/// both bounds clamped — the app derives its production window from the
/// tracked scroll offset with exactly this math, so the "no full-list
/// layout" contract is testable here.
pub fn visible_window(total: usize, viewport_rows: usize, scroll_offset: usize) -> (usize, usize) {
    if total == 0 || viewport_rows == 0 {
        return (0, 0);
    }
    // The offset clamps to the last full-window position, so a scrolled-
    // past-end offset still renders a complete viewport.
    let max_first = total.saturating_sub(viewport_rows);
    let first = scroll_offset.min(max_first);
    let last = first.saturating_add(viewport_rows).min(total);
    (first, last)
}

/// The §11.16 display-title ladder: user label, then captured title, then
/// the hostname/URL fallback (mirrors the B2 `saved_page_to_entry` ladder
/// so Palette and Library never disagree).
pub fn library_display_title(user_label: Option<&str>, captured_title: &str, url: &str) -> String {
    if let Some(label) = user_label {
        if !label.is_empty() {
            return label.to_string();
        }
    }
    if !captured_title.trim().is_empty() {
        return captured_title.to_string();
    }
    let host = crate::text::host_of(url);
    if !host.is_empty() {
        return host;
    }
    crate::text::scheme_of(url)
}

/// The §7.2 monogram fallback for a Saved Page URL (deterministic per
/// host). The normal production favicon presentation path — a Saved Page
/// record stores no favicon blob.
pub fn library_monogram(url: &str) -> FaviconSpec {
    FaviconSpec {
        letter: monogram_letter(url),
        hue: monogram_hue(url),
    }
}

/// The §11.16 URL budget for the Library row (`type:mono`, §4 mid-ellipsis).
pub const LIBRARY_URL_BUDGET: usize = 48;

/// Builds one row spec from the record ladder (pure; the app derives the
/// bounded window first and maps only the visible records).
pub fn library_row_spec(
    bookmark: u64,
    user_label: Option<&str>,
    captured_title: &str,
    url: &str,
    collection: Option<&str>,
    selected: bool,
) -> LibraryRowSpec {
    LibraryRowSpec {
        bookmark,
        title: library_display_title(user_label, captured_title, url),
        url: mid_ellipsis(url, LIBRARY_URL_BUDGET),
        collection: collection.map(str::to_owned),
        favicon: library_monogram(url),
        selected,
    }
}

/// A Library callback carrying a stable `BookmarkId`/`CollectionId` (the
/// per-row/per-collection listeners the app wires through its production
/// entity handle; `Rc` so the surface clones one handler into every row).
pub type LibraryItemAction = Rc<dyn Fn(u64, &mut Window, &mut App)>;
/// A Library callback with no payload (close, sort cycle, remove confirm).
pub type LibraryAction = Rc<dyn Fn(&mut Window, &mut App)>;

/// The Library surface: header (title + close), the §11.16 search field
/// and sort control, the count line, the flat collections strip, the
/// bounded row list, and the decided states (loading / empty / no results /
/// remove confirm). The app passes its production listener bundles; this
/// component owns the anatomy only.
#[allow(clippy::too_many_arguments)]
pub fn library_surface(
    theme: &Theme,
    model: &SavedPagesLibraryModel,
    focused: bool,
    on_close: LibraryAction,
    on_cycle_sort: LibraryAction,
    on_activate: LibraryItemAction,
    on_toggle_select: LibraryItemAction,
    on_rename: LibraryItemAction,
    on_collection_action: LibraryItemAction,
    on_confirm_remove: LibraryAction,
    on_cancel_remove: LibraryAction,
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
                library_row(theme, row, &on_activate, &on_toggle_select, &on_rename)
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
        .child(library_header(
            theme,
            model,
            focused,
            on_close.clone(),
            on_cycle_sort.clone(),
        ))
        .child(
            div()
                .id(SharedString::from("bbr-saved-pages-list"))
                .flex()
                .flex_col()
                .flex_grow()
                .min_h_0()
                .overflow_y_scroll()
                .child(library_count_line(theme, model))
                .children(model.collections.iter().map(|collection| {
                    library_collection_row(theme, collection, &on_collection_action).into_any()
                }))
                .children(content),
        )
        .when(model.remove_confirm, |surface| {
            surface.child(library_remove_confirm(
                theme,
                model.remove_targets.len(),
                &on_confirm_remove,
                &on_cancel_remove,
            ))
        })
}

/// The Library header: the §11.16 title, the search field, the sort
/// control, and the close control.
#[allow(clippy::too_many_arguments)]
fn library_header(
    theme: &Theme,
    model: &SavedPagesLibraryModel,
    focused: bool,
    on_close: LibraryAction,
    on_cycle_sort: LibraryAction,
) -> Div {
    let p = theme.palette;
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
        .child(library_search_field(theme, &model.search, focused))
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
/// routes keyboard input; this renders the field and its focus ring.
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
            el.border_1().border_color(p.accent).rounded(Radius.medium())
        })
        .child(
            div()
                .text_size(theme.type_.title.size)
                .text_color(if query.is_empty() {
                    p.ink_faint
                } else {
                    p.ink
                })
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

impl LibrarySort {
    /// The deterministic order label (the snapshot's machine name is the
    /// serde name; the display label is here).
    pub fn label(self) -> &'static str {
        match self {
            LibrarySort::UpdatedNewest => "Recently updated",
            LibrarySort::UpdatedOldest => "Oldest first",
            LibrarySort::TitleAscending => "Title A–Z",
            LibrarySort::TitleDescending => "Title Z–A",
        }
    }

    /// The next order in the deterministic cycle.
    pub fn next(self) -> LibrarySort {
        match self {
            LibrarySort::UpdatedNewest => LibrarySort::UpdatedOldest,
            LibrarySort::UpdatedOldest => LibrarySort::TitleAscending,
            LibrarySort::TitleAscending => LibrarySort::TitleDescending,
            LibrarySort::TitleDescending => LibrarySort::UpdatedNewest,
        }
    }

    /// The stable machine name (§15 snapshot; deterministic).
    pub fn as_str(self) -> &'static str {
        match self {
            LibrarySort::UpdatedNewest => "updated_newest",
            LibrarySort::UpdatedOldest => "updated_oldest",
            LibrarySort::TitleAscending => "title_ascending",
            LibrarySort::TitleDescending => "title_descending",
        }
    }

    /// Builds the deterministic comparison over display titles
    /// (`TitleAscending`/`TitleDescending`; the updated orders are the
    /// provider's presentation direction and need no key here).
    pub fn title_cmp(self, a: &str, b: &str) -> std::cmp::Ordering {
        match self {
            LibrarySort::TitleAscending => a.cmp(b),
            LibrarySort::TitleDescending => b.cmp(a),
            _ => std::cmp::Ordering::Equal,
        }
    }
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

/// One §11.16 Library row: the bookmark/fallback glyph (the §11.16 settled
/// `★` mark), the monogram favicon tile, the display title, the
/// mid-ellipsized URL, the optional collection label, and the selection
/// state. `on_activate` opens exactly one fresh regular tab through B2;
/// `on_toggle_select` toggles multi-select; `on_rename` opens the rename
/// surface for this row.
pub fn library_row(
    theme: &Theme,
    row: &LibraryRowSpec,
    on_activate: &LibraryItemAction,
    on_toggle_select: &LibraryItemAction,
    on_rename: &LibraryItemAction,
) -> Stateful<Div> {
    let p = theme.palette;
    let bookmark = row.bookmark;
    let activate = on_activate.clone();
    let toggle = on_toggle_select.clone();
    let rename = on_rename.clone();
    div()
        .id(library_row_id(bookmark))
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::sp2())
        .px(Spacing::sp3())
        .py(Spacing::sp1())
        .cursor_pointer()
        .when(row.selected, |el| {
            el.bg(p.accent).text_color(p.accent_ink)
        })
        .child(selection_box(theme, row.selected))
        .child(favicon_tile(theme, row.favicon))
        .child(
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
                        .text_size(theme.type_.title.size)
                        .text_color(p.ink)
                        .child(SharedString::from("★ ")),
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
                .child(Icon::Check.canvas(theme, theme.density.icon())),
        )
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

/// The §15 id of the row rename control.
pub fn saved_page_row_rename_id(bookmark: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-pages-rename-{bookmark}"))
}

/// The §15 id of the row multi-select toggle.
pub fn library_row_select_id(bookmark: u64) -> SharedString {
    SharedString::from(format!("bbr-saved-pages-select-{bookmark}"))
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
        .text_color(theme.palette.surface_glass)
        .child(SharedString::from(spec.letter.to_string()))
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
            el.child(
                div()
                    .size(px(6.0))
                    .rounded_full()
                    .bg(p.accent_ink),
            )
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
        .child(Icon::Folder.canvas(theme, theme.density.icon()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_window_is_bounded_and_clamped() {
        // Empty / zero viewport never produces a range.
        assert_eq!(visible_window(0, 10, 0), (0, 0));
        assert_eq!(visible_window(1000, 0, 5), (0, 0));
        // Full window on a short collection.
        assert_eq!(visible_window(3, 10, 0), (0, 3));
        // Bounded window into 1,000 rows: never lays out the full list.
        assert_eq!(visible_window(1000, 24, 0), (0, 24));
        assert_eq!(visible_window(1000, 24, 100), (100, 124));
        // The offset clamps to the last visible position; the window still
        // renders only the viewport.
        assert_eq!(visible_window(1000, 24, 5000), (976, 1000));
        assert_eq!(visible_window(1000, 24, 999), (976, 1000));
    }

    #[test]
    fn display_title_ladder_is_label_then_captured_then_host() {
        assert_eq!(
            library_display_title(Some("Work"), "Captured", "https://example.com/a"),
            "Work"
        );
        assert_eq!(
            library_display_title(None, "Captured", "https://example.com/a"),
            "Captured"
        );
        assert_eq!(
            library_display_title(None, "  ", "https://example.com/a"),
            "example.com"
        );
        assert_eq!(library_display_title(None, "", "about:blank"), "about");
    }

    #[test]
    fn row_spec_uses_mid_ellipsis_and_the_monogram() {
        let long = "https://verylongsubdomain.example.com/a/very/long/path?with=a+query";
        let row = library_row_spec(7, None, "Title", long, Some("Work"), true);
        assert_eq!(row.bookmark, 7);
        assert_eq!(row.title, "Title");
        assert!(row.url.contains('…'));
        assert!(row.url.chars().count() < long.chars().count());
        assert!(row.url.ends_with("query"), "the tail survives: {}", row.url);
        assert_eq!(row.collection.as_deref(), Some("Work"));
        assert!(row.selected);
        assert_eq!(row.favicon.letter, 'V');
    }

    #[test]
    fn sort_orders_cycle_deterministically_and_compare_titles() {
        let mut sort = LibrarySort::default();
        let order: Vec<&str> = (0..4)
            .map(|_| {
                let label = sort.label();
                sort = sort.next();
                label
            })
            .collect();
        assert_eq!(
            order,
            vec![
                "Recently updated",
                "Oldest first",
                "Title A–Z",
                "Title Z–A",
            ]
        );
        assert_eq!(sort, LibrarySort::default(), "the cycle closes");
        assert_eq!(
            LibrarySort::TitleAscending.title_cmp("beta", "alpha"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            LibrarySort::TitleDescending.title_cmp("beta", "alpha"),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            LibrarySort::UpdatedNewest.title_cmp("beta", "alpha"),
            std::cmp::Ordering::Equal,
            "updated orders use the provider presentation, never a title key"
        );
    }

    #[test]
    fn decided_states_are_derived_from_the_model() {
        let base = SavedPagesLibraryModel {
            stage: LibraryStage::Loading,
            search: String::new(),
            sort: LibrarySort::default(),
            rows: Vec::new(),
            total: 0,
            window: LibraryWindow {
                first: 0,
                last: 0,
                anchor: None,
            },
            collections: Vec::new(),
            remove_confirm: false,
            remove_targets: Vec::new(),
        };
        let loading = base.clone();
        assert!(loading.is_loading());
        assert!(!loading.is_empty());

        let empty = SavedPagesLibraryModel {
            stage: LibraryStage::Ready,
            ..base.clone()
        };
        assert!(empty.is_empty());
        assert!(!empty.has_no_results());

        let no_results = SavedPagesLibraryModel {
            stage: LibraryStage::Ready,
            total: 1000,
            rows: Vec::new(),
            ..base.clone()
        };
        assert!(!no_results.is_empty());
        assert!(no_results.has_no_results());

        let populated = SavedPagesLibraryModel {
            stage: LibraryStage::Ready,
            total: 1000,
            rows: vec![library_row_spec(1, None, "A", "https://a.test/", None, false)],
            ..base
        };
        assert!(!populated.is_empty());
        assert!(!populated.has_no_results());
    }
}
