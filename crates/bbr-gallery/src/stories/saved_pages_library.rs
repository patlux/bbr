//! The G2 Saved Pages Library row/control stories (B3 packet
//! `saved-pages-library`, Gallery G2): the **exact production `bbr-ui`
//! Library rows and controls** — `library_row`, `library_collection_row`,
//! `library_search_field`, `library_sort_control`, `library_count_line`,
//! `library_remove_confirm`, and `library_state_row` — rendered with plain
//! fixture models and the recorder/observer callbacks. No gallery-local
//! reimplementation of anatomy, no copied tokens, no repository/browser/
//! controller/network authority, no persistence claim.

use bbr_ui::components::{
    library_collection_row, library_count_line, library_remove_confirm, library_row,
    library_search_field, library_sort_control, library_state_row, LibrarySort, LIBRARY_EMPTY_ID,
    LIBRARY_LOADING_ID, LIBRARY_NO_RESULTS_ID,
};
use gpui::{div, prelude::*, AnyElement, SharedString};

use crate::library_fixtures::{library_collections, library_count_model, library_row_fixture};
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// The shared no-op body for a Library story rendered without its context
/// (defensive only — the app always supplies the context).
fn missing_context(ctx: &StoryContext, title: &str) -> AnyElement {
    story_page(ctx, title, Vec::new())
}

/// A `self_start` container that hugs its child so its debug bounds equal
/// the real control's bounds — the real-pointer E2E target. Pure layout;
/// the control itself is untouched.
fn control_slot(selector: &str, child: impl IntoElement) -> AnyElement {
    div()
        .self_start()
        .debug_selector(move || selector.to_string())
        .child(child)
        .into_any()
}

/// A full-width debug target around a Library row: the row's inner controls
/// (result block, rename button, select box) sit at deterministic positions
/// inside the row bounds, so the real-pointer E2E clicks them at their
/// computed centers. Pure layout; the row itself is untouched.
fn row_slot(bookmark: u64, child: impl IntoElement) -> AnyElement {
    div()
        .debug_selector(move || format!("bbr-gallery-library-row-{bookmark}"))
        .child(child)
        .into_any()
}

/// §11.16 Library row: the production `bbr-ui` row anatomy with fixture
/// specs — default, selected, long title/URL, missing title (hostname
/// fallback), missing favicon (monogram), collection label, and the staged
/// rename input. Real pointer clicks report through the recorder.
pub fn render_library_row(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let Some(library) = ctx.library.as_ref() else {
        return missing_context(ctx, "Library row");
    };
    let (rows, editing, draft) = library_row_fixture(ctx.state);
    let row_elements: Vec<AnyElement> = rows
        .iter()
        .map(|row| {
            let bookmark = row.bookmark;
            row_slot(
                bookmark,
                library_row(
                    theme,
                    row,
                    editing == Some(bookmark),
                    &draft,
                    &library.observer.on_activate,
                    &library.observer.on_toggle_select,
                    &library.observer.on_rename,
                    &library.observer.on_rename_confirm,
                    &library.observer.on_rename_cancel,
                ),
            )
        })
        .collect();
    let mut children = vec![
        section_heading(theme, "§11.16 row anatomy · bbr-ui library_row"),
        value_row(theme, "row id", "bbr-saved-pages-row-<bookmark>"),
        value_row(theme, "select id", "bbr-saved-pages-select-<bookmark>"),
        value_row(
            theme,
            "result ids",
            "bbr-saved-page-result(-title|-url)-<bookmark>",
        ),
        value_row(
            theme,
            "rename ids",
            "bbr-saved-pages-rename(-input|-confirm|-cancel)-<bookmark>",
        ),
    ];
    children.extend(row_elements);
    story_page(ctx, "Library row", children)
}

/// §11.16 flat collection row: the production `bbr-ui` collection row with
/// fixture collections. Real pointer clicks report through the recorder
/// (the move/rename/remove entry).
pub fn render_library_collection_row(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let Some(library) = ctx.library.as_ref() else {
        return missing_context(ctx, "Library collection row");
    };
    let rows: Vec<AnyElement> = library_collections()
        .iter()
        .map(|spec| {
            let id = spec.id;
            control_slot(
                &format!("bbr-gallery-library-collection-{id}"),
                library_collection_row(theme, spec, &library.observer.on_collection_action),
            )
        })
        .collect();
    let mut children = vec![
        section_heading(
            theme,
            "§11.16 flat collections · bbr-ui library_collection_row",
        ),
        value_row(theme, "collection id", "bbr-saved-pages-collection-<id>"),
    ];
    children.extend(rows);
    story_page(ctx, "Library collection row", children)
}

/// §11.16 search field: the production `bbr-ui` search field with the
/// empty (placeholder), typed, and focused states. The query is the model's
/// search text; the app owns the text model in production.
pub fn render_library_search(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let query = match ctx.state {
        "typed" => "recipes",
        _ => "",
    };
    let focused = ctx.state == "focused";
    story_page(
        ctx,
        "Library search",
        vec![
            section_heading(theme, "§11.16 search field · bbr-ui library_search_field"),
            control_slot(
                "bbr-gallery-library-search",
                library_search_field(theme, query, focused),
            ),
            value_row(theme, "id", "bbr-saved-pages-search"),
            value_row(
                theme,
                "query",
                if query.is_empty() {
                    "(empty placeholder)"
                } else {
                    query
                },
            ),
            value_row(theme, "focused", if focused { "true" } else { "false" }),
        ],
    )
}

/// §11.16 sort control: the production `bbr-ui` sort control for all four
/// deterministic orders. Real pointer clicks cycle through the recorder.
pub fn render_library_sort(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let Some(library) = ctx.library.as_ref() else {
        return missing_context(ctx, "Library sort");
    };
    let orders = [
        LibrarySort::UpdatedNewest,
        LibrarySort::UpdatedOldest,
        LibrarySort::TitleAscending,
        LibrarySort::TitleDescending,
    ];
    let controls: Vec<AnyElement> = orders
        .iter()
        .map(|sort| {
            control_slot(
                &format!("bbr-gallery-library-sort-{}", sort.as_str()),
                library_sort_control(theme, *sort, library.observer.on_cycle_sort.clone()),
            )
        })
        .collect();
    let mut children = vec![
        section_heading(
            theme,
            "§11.16 deterministic sort · bbr-ui library_sort_control",
        ),
        value_row(theme, "id", "bbr-saved-pages-sort"),
    ];
    children.extend(controls);
    story_page(ctx, "Library sort", children)
}

/// §11.16 count line: the production `bbr-ui` count line for 1, many, and
/// the windowed 1k total.
pub fn render_library_count(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let lines: Vec<AnyElement> = [1usize, 24, 1000]
        .into_iter()
        .map(|total| {
            let model = library_count_model(total);
            control_slot(
                &format!("bbr-gallery-library-count-{total}"),
                library_count_line(theme, &model),
            )
        })
        .collect();
    let mut children = vec![
        section_heading(theme, "§11.16 count line · bbr-ui library_count_line"),
        value_row(theme, "id", "bbr-saved-pages-count"),
    ];
    children.extend(lines);
    story_page(ctx, "Library count", children)
}

/// §11.16 destructive remove confirmation: the production `bbr-ui` confirm
/// for one and many targets. Real pointer clicks report through the
/// recorder.
pub fn render_library_remove_confirm(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let Some(library) = ctx.library.as_ref() else {
        return missing_context(ctx, "Library remove confirm");
    };
    let count = if ctx.state == "multi" { 3 } else { 1 };
    story_page(
        ctx,
        "Library remove confirm",
        vec![
            section_heading(
                theme,
                "§11.16 destructive remove · bbr-ui library_remove_confirm",
            ),
            control_slot(
                "bbr-gallery-library-remove-confirm",
                library_remove_confirm(
                    theme,
                    count,
                    &library.observer.on_confirm_remove,
                    &library.observer.on_cancel_remove,
                ),
            ),
            value_row(theme, "id", "bbr-saved-pages-remove-confirm"),
            value_row(theme, "targets", if count == 1 { "1" } else { "3" }),
        ],
    )
}

/// §11.16 decided state rows: the production `bbr-ui` state rows for
/// loading, empty, and no-results.
pub fn render_library_state_rows(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let rows = [
        (LIBRARY_LOADING_ID, "Loading Saved Pages…"),
        (
            LIBRARY_EMPTY_ID,
            "No Saved Pages yet — press Cmd-D on any page to save it.",
        ),
        (LIBRARY_NO_RESULTS_ID, "No Saved Pages match “query”."),
    ];
    let elements: Vec<AnyElement> = rows
        .iter()
        .map(|(id, text)| {
            control_slot(
                &format!("bbr-gallery-library-state-{id}"),
                library_state_row(theme, SharedString::from(*id), text),
            )
        })
        .collect();
    let mut children = vec![
        section_heading(theme, "§11.16 decided states · bbr-ui library_state_row"),
        value_row(theme, "loading id", LIBRARY_LOADING_ID),
        value_row(theme, "empty id", LIBRARY_EMPTY_ID),
        value_row(theme, "no-results id", LIBRARY_NO_RESULTS_ID),
    ];
    children.extend(elements);
    story_page(ctx, "Library state rows", children)
}
