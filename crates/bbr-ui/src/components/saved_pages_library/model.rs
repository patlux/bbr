//! Saved Pages Library model (DESIGN.md §11.16, B3 packet
//! `saved-pages-library`): the plain presentation data, the deterministic
//! sort orders, and the bounded-render math (the stable §15 ids live in the
//! sibling `ids` module).
//!
//! This module is **presentation only**: the model is a `bbr-ui` DTO built
//! by `bbr-app` from the loaded in-memory snapshot (B2 provider), and the
//! callbacks are the GPUI handler closures the app passes. No repository,
//! controller, engine, or network appears here.

use crate::components::favicon::{monogram_hue, monogram_letter, FaviconSpec};
use crate::text::mid_ellipsis;

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

/// The staged flat-collection action the pointer cycle stages (rename →
/// remove → none): presentation data owned here so the `bbr-ui` surface
/// renders the staged action surface (create/rename/remove) and the app
/// commits through the one AppStore authority. The cycle is deterministic
/// and testable without GPUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryCollectionAction {
    /// Stage the inline rename input for the clicked collection (seeded
    /// with its current title).
    Rename,
    /// Stage the destructive remove confirmation for the clicked
    /// collection (members move out first).
    Remove,
}

impl LibraryCollectionAction {
    /// The next action in the deterministic pointer cycle.
    pub fn next(self) -> Option<Self> {
        match self {
            LibraryCollectionAction::Rename => Some(LibraryCollectionAction::Remove),
            LibraryCollectionAction::Remove => None,
        }
    }

    /// The stable §15 machine name (the typed testkit snapshot's
    /// `collection_action` value), mirroring [`LibrarySort::as_str`].
    pub fn as_str(self) -> &'static str {
        match self {
            LibraryCollectionAction::Rename => "rename",
            LibraryCollectionAction::Remove => "remove",
        }
    }
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
    /// The total number of records in the provider (never filtered): the
    /// basis of the §11.16 decided states, so `empty` (no records at all)
    /// and `no results` (records exist, none match the query) stay
    /// distinct even when a search matches nothing (`total` is then 0).
    pub records_total: usize,
    /// The windowed render bounds (the bounded-render evidence).
    pub window: LibraryWindow,
    /// The flat collections, one level only.
    pub collections: Vec<LibraryCollectionSpec>,
    /// True while the destructive remove confirmation is presented.
    pub remove_confirm: bool,
    /// The ids targeted by the pending destructive remove.
    pub remove_targets: Vec<u64>,
    /// The row whose inline/staged rename input is presented (the pointer
    /// rename entry, B3); `None` at rest.
    pub editing_row: Option<u64>,
    /// The staged rename draft label (empty at rest).
    pub rename_draft: String,
    /// The full multi-select set (the selection toolbar's source of truth;
    /// rows outside the window stay selected).
    pub selected: Vec<u64>,
    /// The collection row whose pointer action is staged (`rename` /
    /// `remove` cycle); the action surface renders inline while staged.
    pub collection_action: Option<(u64, LibraryCollectionAction)>,
    /// The staged collection rename/create draft label.
    pub collection_draft: String,
    /// True while the collection create input is staged.
    pub collection_create_staged: bool,
    /// True while the selection move-to sheet is open.
    pub move_sheet_open: bool,
    /// The tracked production scroll offset (rows scrolled past) the wheel
    /// listener updates and the bounded window derives from.
    pub scroll_offset: usize,
    /// The bounded viewport (rows rendered) the wheel clamp uses.
    pub viewport_rows: usize,
}

impl SavedPagesLibraryModel {
    /// The §11.16 decided states as booleans the surface and the snapshot
    /// share: loading, empty (ready with no records), and no-search-results
    /// (ready with records but none matching).
    pub fn is_loading(&self) -> bool {
        self.stage == LibraryStage::Loading
    }

    pub fn is_empty(&self) -> bool {
        self.stage == LibraryStage::Ready && self.records_total == 0
    }

    pub fn has_no_results(&self) -> bool {
        self.stage == LibraryStage::Ready && self.records_total > 0 && self.rows.is_empty()
    }

    /// True while a collection rename or create input is staged (the
    /// keyboard owns the collection draft then).
    pub fn collection_input_staged(&self) -> bool {
        self.collection_create_staged
            || matches!(
                self.collection_action,
                Some((_, LibraryCollectionAction::Rename))
            )
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
        // Saved Pages store no favicon blob (§11.16): the monogram is the
        // only Library favicon presentation.
        image: None,
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

#[cfg(test)]
mod tests;
