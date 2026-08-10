//! Pure Saved Pages Library model building (B3 packet
//! `saved-pages-library`): the deterministic search filter, the sort
//! application, the bounded render window, and the row/collection specs.
//!
//! Everything here is pure and unit-testable without GPUI or SQLite: it
//! consumes the loaded in-memory provider snapshot (B1/B2 authority) plus
//! the app's local search/sort/selection/scroll state and produces the
//! `bbr-ui` presentation model. The search predicate mirrors
//! `bbr_domain::SavedPage::matches` exactly (the B1 decision documents this
//! one small Unicode-lowercase authority), so Palette and Library never
//! disagree.

use bbr_persistence::{CollectionRecord, SavedPageRecord};
use bbr_ui::components::{
    library_row_spec, visible_window, LibraryCollectionAction, LibraryCollectionSpec, LibrarySort,
    LibraryStage, LibraryWindow, SavedPagesLibraryModel,
};

/// The deterministic in-memory search predicate over the persisted DTO —
/// the mirror of `bbr_domain::SavedPage::matches` (case-insensitive
/// literal substring over captured title, user label, and canonical URL;
/// an empty query matches everything). No SQL and no synchronous I/O from
/// render.
pub fn record_matches(page: &SavedPageRecord, query: &str) -> bool {
    let needle = query.to_lowercase();
    if needle.is_empty() {
        return true;
    }
    page.captured_title.to_lowercase().contains(&needle)
        || page
            .user_label
            .as_deref()
            .is_some_and(|label| label.to_lowercase().contains(&needle))
        || page.canonical_url.to_lowercase().contains(&needle)
}

/// The display-title sort key (the §11.16 ladder), so the title orders sort
/// over exactly the label the row renders.
fn display_title_key(page: &SavedPageRecord) -> String {
    if let Some(label) = &page.user_label {
        if !label.is_empty() {
            return label.clone();
        }
    }
    if !page.captured_title.trim().is_empty() {
        return page.captured_title.clone();
    }
    page.canonical_url.clone()
}

/// Applies the deterministic sort to the already-filtered result slice.
/// `UpdatedNewest` is the provider presentation order (the input arrives
/// pre-sorted most-recently-updated-first and the sort is a no-op);
/// `UpdatedOldest` is the B1 sort-key direction ascending; the title orders
/// sort by the display-title ladder with a stable tie-break (equal keys
/// keep the provider order, so the result is fully deterministic).
pub fn order_results(pages: &mut [SavedPageRecord], sort: LibrarySort) {
    match sort {
        LibrarySort::UpdatedNewest => {}
        LibrarySort::UpdatedOldest => {
            pages.sort_by(|a, b| a.updated_at.cmp(&b.updated_at).then(a.id.cmp(&b.id)));
        }
        LibrarySort::TitleAscending | LibrarySort::TitleDescending => {
            pages.sort_by(|a, b| sort.title_cmp(&display_title_key(a), &display_title_key(b)));
        }
    }
}

/// The state inputs the model builder consumes (borrowed so the builder
/// stays a pure function).
pub struct LibraryStateView<'a> {
    pub search: &'a str,
    pub sort: LibrarySort,
    pub selected: &'a [u64],
    pub remove_confirm: bool,
    pub remove_targets: &'a [u64],
    pub scroll_offset: usize,
    pub viewport_rows: usize,
    /// The row whose inline/staged rename input is presented; `None` at
    /// rest (the pointer rename entry, B3).
    pub editing_row: Option<u64>,
    /// The staged rename draft label.
    pub rename_draft: &'a str,
    /// The collection row whose pointer action is staged (`rename` /
    /// `remove` cycle); the action surface renders inline while staged.
    pub collection_action: Option<(u64, LibraryCollectionAction)>,
    /// The staged collection rename/create draft label.
    pub collection_draft: &'a str,
    /// True while the collection create input is staged.
    pub collection_create_staged: bool,
    /// True while the selection move-to sheet is open.
    pub move_sheet_open: bool,
}

/// Builds the §11.16 presentation model from the loaded in-memory snapshot
/// (B1/B2 authority) plus the app's local state: filter by search, apply
/// the deterministic sort, derive the bounded window from the production
/// scroll offset, and map only the visible records to row specs. Never
/// lays out the full collection and never touches the repository.
pub fn build_library_model(
    pages: &[SavedPageRecord],
    collections: &[CollectionRecord],
    state: &LibraryStateView<'_>,
) -> SavedPagesLibraryModel {
    let mut matching: Vec<SavedPageRecord> = pages
        .iter()
        .filter(|page| record_matches(page, state.search))
        .cloned()
        .collect();
    let total = matching.len();
    order_results(&mut matching, state.sort);
    let (first, last) = visible_window(total, state.viewport_rows, state.scroll_offset);
    let anchor = matching.get(first).map(|page| page.id);
    let rows = matching[first..last]
        .iter()
        .map(|page| {
            let collection = page
                .collection_id
                .and_then(|id| collections.iter().find(|c| c.id == id))
                .map(|c| c.title.as_str());
            library_row_spec(
                page.id,
                page.user_label.as_deref(),
                &page.captured_title,
                &page.canonical_url,
                collection,
                state.selected.contains(&page.id),
            )
        })
        .collect();
    let collections_specs = collections
        .iter()
        .map(|collection| LibraryCollectionSpec {
            id: collection.id,
            title: collection.title.clone(),
            member_count: pages
                .iter()
                .filter(|page| page.collection_id == Some(collection.id))
                .count(),
        })
        .collect();
    SavedPagesLibraryModel {
        stage: LibraryStage::Ready,
        search: state.search.to_string(),
        sort: state.sort,
        rows,
        total,
        // The unfiltered provider size: the decided-state basis, so a
        // search that matches nothing presents the no-results state, never
        // the empty state (empty means no records at all).
        records_total: pages.len(),
        window: LibraryWindow {
            first,
            last,
            anchor,
        },
        collections: collections_specs,
        remove_confirm: state.remove_confirm,
        remove_targets: state.remove_targets.to_vec(),
        editing_row: state.editing_row,
        rename_draft: state.rename_draft.to_string(),
        selected: state.selected.to_vec(),
        collection_action: state.collection_action,
        collection_draft: state.collection_draft.to_string(),
        collection_create_staged: state.collection_create_staged,
        move_sheet_open: state.move_sheet_open,
        scroll_offset: state.scroll_offset,
        viewport_rows: state.viewport_rows,
    }
}

// The §15 visible-id mirror (test/testkit only) and the model unit tests
// live in sibling modules so no file outgrows the ~500-line budget.
#[cfg(any(test, feature = "testkit"))]
mod visible_ids;

#[cfg(any(test, feature = "testkit"))]
pub(crate) use visible_ids::library_visible_ids;

#[cfg(test)]
mod tests;
