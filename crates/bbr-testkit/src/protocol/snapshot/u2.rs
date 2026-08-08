//! U2 sidebar wire parts (DESIGN.md §11.3/§11.10/§11.11): the permanent
//! space-strip chip snapshot, the favorite destination tile, and the folder
//! row snapshots. Plain data with `serde` derives, kept here so `snapshot`
//! stays under the ~500-line budget; re-exported through `super` so callers
//! keep the single `bbr_testkit::protocol::*` path.

use serde::{Deserialize, Serialize};

use crate::protocol::TestSpaceId;

/// One §11.3 identity chip on the wire (DESIGN.md §11.3/§15): the persistent
/// space identity plus the space/identity strings the strip renders, in the
/// strip's canonical display order (the vector order is display order; the
/// `bbr-space-chip-<n>`/`bbr-space-identity-<n>` ids derive from the
/// persistent `space` id alone — never render position, creation-order
/// index, or profile index). The strip is always part of the rail, so the
/// chips expose the exact selected evidence the active space renders. Each
/// chip also carries the effective curated theme token name so the active
/// appearance (explicit controller record or derived profile-hue default)
/// is provable per chip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceChipSnapshot {
    /// The persistent space identity — the only identity of the chip ids.
    pub space: TestSpaceId,
    /// The space title after §4 truncation.
    pub title: String,
    /// The validated curated identity in its canonical string form (the
    /// emoji scalar or the glyph name).
    pub identity: String,
    /// The effective curated theme token name (`cobalt`…`ink`), from the
    /// one controller appearance authority — explicit record or derived
    /// profile-hue default.
    #[serde(default)]
    pub theme: String,
    /// Whether this is the active space (the chip renders selected).
    pub selected: bool,
}

/// The resolved appearance evidence of the active space on the wire
/// (DESIGN.md §11.3/§15): the effective theme/identity names plus the
/// **resolved chrome/chip tint state** the render actually paints — typed
/// `#RRGGBB` hex derived from the one central presentation model, never raw
/// native/web pixels and never a parallel render store. E2E proves that
/// cross-space/cross-profile activation through production activation
/// changes this evidence exactly when the effective appearance changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceAppearanceSnapshot {
    /// The space whose appearance this is (the active space at snapshot
    /// time).
    pub space: TestSpaceId,
    /// The effective curated theme token name (explicit record or derived
    /// profile-hue default).
    pub theme: String,
    /// The effective validated identity in its canonical string form.
    pub identity: String,
    /// The resolved sidebar-rail chrome tint (`#RRGGBB`) for the active
    /// space's theme in the shell's mode/contrast.
    pub chrome_tint: String,
    /// The resolved active-chip surface tint (`#RRGGBB`) for the active
    /// space's theme in the shell's mode/contrast.
    pub chip_tint: String,
}

/// §11.3/§11.4 strip scroll evidence on the wire: the strip's measured
/// viewport plus the tracked-scroll state and one raw layout bounds rect
/// per chip in canonical display order, read-only from the production
/// `space_strip_scroll` [`gpui::ScrollHandle`] the render tracks (never a
/// snapshot mutation). The E2E combines a raw chip rect with the signed
/// `offset` to compute the chip's visual position (`visual = raw + offset`)
/// and asserts the selected chip lies fully inside the viewport after an
/// activation — the §11.3 overflow rule fails loudly when an offscreen chip
/// is not brought fully into view.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SpaceStripScrollSnapshot {
    /// The strip viewport's window-relative bounds `[x, y, w, h]` in points.
    pub viewport: [f32; 4],
    /// The current scroll offset `[x, y]` in points. GPUI's scroll offset is
    /// signed and non-positive: `[0, 0]` means no scroll, a negative `x`
    /// means the content has been scrolled left.
    pub offset: [f32; 2],
    /// The maximum scroll offset `[x, y]` in points (non-negative); a zero
    /// `x` means the chip row fits the viewport and cannot scroll.
    pub max_offset: [f32; 2],
    /// One raw window-relative layout bounds `[x, y, w, h]` per chip in
    /// canonical display order (the same order as [`SpaceChipSnapshot`]s).
    /// The raw rects ignore the scroll offset; combine with `offset` for
    /// the visual position.
    pub chip_bounds: Vec<[f32; 4]>,
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
