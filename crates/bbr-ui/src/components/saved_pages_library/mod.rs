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
//! # Structure
//!
//! The module is split by responsibility so no file outgrows the ~500-line
//! budget: [`ids`] owns the stable §15 ids; [`model`] owns the plain
//! presentation data, the deterministic sort orders, and the bounded-render
//! math; [`callbacks`] owns the production listener bundle; [`scroll`]
//! owns the wheel-delta → row clamp math; [`rows`] owns the
//! row/collection-row anatomy; [`toolbar`] owns the selection toolbar and
//! move-to sheet; [`header`] owns the header/search/sort/count anatomy;
//! [`collection_surface`] owns the collection create/rename/remove action
//! surface; [`surface`] owns the surface composition.
//!
//! # Bounded rendering
//!
//! The surface renders exactly [`SavedPagesLibraryModel::rows`] — the
//! bounded window the app derives from the deterministic full result order
//! and the production scroll offset. The pure [`visible_window`] math is
//! owned here so the "the Library does not lay out all 1,000 rows" contract
//! is testable without GPUI.

mod callbacks;
mod collection_surface;
mod header;
mod ids;
mod model;
mod rows;
mod scroll;
mod surface;
mod toolbar;

pub use callbacks::*;
pub use header::{library_count_line, library_search_field, library_sort_control};
pub use ids::*;
pub use model::*;
pub use rows::{favicon_tile, library_collection_row, library_row, library_row_ids, selection_box};
pub use scroll::{scroll_offset_for_wheel, wheel_rows, LIBRARY_ROW_HEIGHT_PX};
pub use surface::{library_remove_confirm, library_state_row, library_surface};
pub use toolbar::{library_move_sheet, library_selection_strip, library_selection_toolbar};
