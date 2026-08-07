//! U2 sidebar wire parts (DESIGN.md §11.10/§11.11): the favorite destination
//! tile and the folder row snapshots. Plain data with `serde` derives, kept
//! here so `snapshot` stays under the ~500-line budget; re-exported through
//! `super` so callers keep the single `bbr_testkit::protocol::*` path.

use serde::{Deserialize, Serialize};

/// Serde default for the U2 `Snapshot.spaces_visible` field: the compact
/// sidebar starts with the spaces region hidden; older clients without the
/// field deserialize to the same presentation.
pub(super) fn default_true() -> bool {
    true
}

/// One favorite destination tile on the wire (DESIGN.md §11.10): the
/// creation-order index (the `bbr-fav-<n>` identity), the concrete tab
/// owner (`TabId + SpaceId + ProfileId` — the §11.10 data-source contract:
/// a favorite is an additional presentation of one exact tab), the page
/// identity, and the derived §7.2 monogram fallback (letter + slate-hue
/// index) so E2E asserts the rendered fallback without re-deriving it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FavoriteTileSnapshot {
    /// Position in deterministic insertion order — the `bbr-fav-<n>` id.
    pub index: usize,
    /// The exact concrete tab this tile additionally presents.
    pub tab: u64,
    /// Current owner of `tab`; changes if that same tab is moved.
    pub space: u64,
    /// Derived from `space`; identifies the isolated container to activate.
    pub profile: u64,
    pub title: String,
    pub url: String,
    /// §7.2 monogram letter (first hostname character, uppercase).
    pub letter: String,
    /// §7.2 FNV slate-hue ramp index (0..6).
    pub hue: usize,
}

/// One folder row on the wire (DESIGN.md §11.11): the creation-order index
/// (the `bbr-folder-row-<n>` identity), the title, the member-tab count, and
/// the app-local expanded/collapsed value (snap semantics asserted by E2E).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderRowSnapshot {
    /// Position in creation order — the `bbr-folder-row-<n>` id.
    pub index: usize,
    /// The domain folder id (the L5 `MoveIntoFolder` command's target).
    #[serde(default)]
    pub id: u64,
    pub title: String,
    /// Member tab count (§11.11 count column, `type:meta` `tnum`).
    pub tab_count: usize,
    /// Whether the nested row list is expanded.
    pub expanded: bool,
}
