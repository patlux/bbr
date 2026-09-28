//! The G3 Saved Pages Library composition story (B3 packet
//! `saved-pages-library`, Gallery G3): the **exact production `bbr-ui`
//! `library_surface`** with the full §11.16 state matrix — empty, loading,
//! populated, no results, selection, multi, destructive, long, missing
//! favicon, large 1k, rename — driven by the story's State selector and
//! wired to the recorder/observer callbacks. No repository/browser/
//! controller/network authority, no persistence claim; gallery
//! renderability is never native-boundary or product-performance evidence.

use bbr_ui::components::library_surface;
use gpui::{div, prelude::*, px, AnyElement};

use crate::library_fixtures::library_model;
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// The full §11.16 Library composition: the production `bbr-ui` surface
/// with a fixture model per decided state. Real pointer clicks on every
/// control (close, sort, open, select, rename, collection, remove
/// confirm/cancel) report through the recorder.
pub fn render_library(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let Some(library) = ctx.library.as_ref() else {
        return story_page(ctx, "Library", Vec::new());
    };
    let model = library_model(ctx.state);
    let surface = library_surface(
        theme,
        &model,
        true,
        &bbr_ui::components::LibraryCallbacks {
            on_close: library.observer.on_close.clone(),
            on_cycle_sort: library.observer.on_cycle_sort.clone(),
            on_activate: library.observer.on_activate.clone(),
            on_toggle_select: library.observer.on_toggle_select.clone(),
            on_rename: library.observer.on_rename.clone(),
            on_rename_confirm: library.observer.on_rename_confirm.clone(),
            on_rename_cancel: library.observer.on_rename_cancel.clone(),
            on_collection_action: library.observer.on_collection_action.clone(),
            on_collection_create: library.observer.on_collection_create.clone(),
            on_collection_confirm: library.observer.on_collection_confirm.clone(),
            on_collection_cancel: library.observer.on_collection_cancel.clone(),
            on_confirm_remove: library.observer.on_confirm_remove.clone(),
            on_cancel_remove: library.observer.on_cancel_remove.clone(),
            on_clear_selection: library.observer.on_clear_selection.clone(),
            on_remove_selection: library.observer.on_remove_selection.clone(),
            on_toggle_move_sheet: library.observer.on_toggle_move_sheet.clone(),
            on_move_into: library.observer.on_move_into.clone(),
            on_move_out: library.observer.on_move_out.clone(),
            on_scroll: library.observer.on_scroll.clone(),
        },
    );
    story_page(
        ctx,
        "Library",
        vec![
            section_heading(theme, "§11.16 composition · bbr-ui library_surface"),
            value_row(theme, "surface id", "bbr-saved-pages-library"),
            value_row(theme, "state", ctx.state),
            // The surface is a full-size flex column; a fixed-height frame
            // keeps the bounded row list scrollable inside the story page.
            // The debug target is the Gallery's own E2E selector — the
            // surface's §15 ids stay the production ids.
            div()
                .debug_selector(|| "bbr-gallery-library-surface".to_string())
                .h(px(520.0))
                .w_full()
                .child(surface)
                .into_any(),
        ],
    )
}
