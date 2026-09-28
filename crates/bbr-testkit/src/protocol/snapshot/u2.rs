//! U2 sidebar wire parts (DESIGN.md §11.3/§11.10/§11.11): the permanent
//! space-strip chip snapshot, the favorite destination tile, and the folder
//! row snapshots. Plain data with `serde` derives, kept here so `snapshot`
//! stays under the ~500-line budget; re-exported through `super` so callers
//! keep the single `bbr_testkit::protocol::*` path.

use serde::{Deserialize, Serialize};

use crate::protocol::TestSpaceId;

/// The §12 destination-profile companion of a chip during a drag (DESIGN.md
/// §10.9/§11.3): the destination space's profile identity rendered inside
/// the targeted chip so isolation is never communicated by color alone. The
/// `bbr-profile-chip-<n>` id derives from the profile's creation-order
/// index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChipCompanionSnapshot {
    /// The destination profile's creation-order index (the §12 chip id).
    pub profile: usize,
    /// The §12 monogram (the non-color identity).
    pub monogram: String,
    /// The profile hue name (DESIGN.md §12 allowed values).
    pub hue: String,
}

/// One §11.3 identity chip on the wire (DESIGN.md §11.3/§15): the persistent
/// space identity plus the space/identity strings the strip renders, in the
/// strip's canonical display order (the vector order is display order; the
/// `bbr-space-chip-<n>`/`bbr-space-identity-<n>` ids derive from the
/// persistent `space` id alone — never render position, creation-order
/// index, or profile index). The strip is always part of the rail, so the
/// chips expose the exact selected evidence the active space renders. Each
/// chip also carries the effective curated theme token name so the active
/// appearance (explicit controller record or derived profile-hue default)
/// is provable per chip. While a drag targets this exact chip, the wire
/// reports the §10.9 drag-target state and its §12 destination-profile
/// companion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceChipSnapshot {
    /// The persistent space identity — the only identity of the chip ids.
    pub space: TestSpaceId,
    /// The space title after §4 truncation.
    pub title: String,
    /// The validated curated identity in its canonical string form (the
    /// emoji scalar or the glyph name).
    pub identity: String,
    /// §11.3/§7 the rendered identity form on the wire: `"emoji"` (the
    /// scalar renders directly), `"glyph"` (the curated name renders its
    /// drawn §7 path), or `"badge"` (a non-curated value renders the
    /// two-letter §7 fallback). Typed so E2E asserts the glyph-rendered
    /// chips without stringly-typed probing.
    #[serde(default)]
    pub identity_kind: String,
    /// The effective curated theme token name (`cobalt`…`ink`), from the
    /// one controller appearance authority — explicit record or derived
    /// profile-hue default.
    #[serde(default)]
    pub theme: String,
    /// Whether this is the active space (the chip renders selected).
    pub selected: bool,
    /// §10.9: whether a pointer drag currently targets this exact chip (the
    /// chip renders the drag-target outline + tint plus the §12 companion).
    #[serde(default)]
    pub drag_target: bool,
    /// The §12 destination-profile companion rendered while this chip is
    /// the drag target; `None` at rest.
    #[serde(default)]
    pub companion: Option<ChipCompanionSnapshot>,
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
    /// v2 D7: the effective tint intensity step (`subtle`/`standard`/`rich`).
    #[serde(default = "standard_intensity")]
    pub intensity: String,
    /// v2 D6: whether the rail and chassis paint the static grain.
    #[serde(default)]
    pub grain: bool,
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
    /// §7.2/§15 favicon presentation of the tile's slot (task packet
    /// `real-favicon-pipeline.md`): `"monogram"` until the owning profile's
    /// bounded host-keyed store holds resolved bytes for the tile's host,
    /// then `"image"` — the same record the tab row derives from.
    pub favicon: String,
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
    /// §11.11 hover/focus-reveal: whether the count paints right now
    /// (the pointer hovers the row, or the row is the keyboard focus
    /// anchor while the folders region owns chrome focus). At rest
    /// `false` — the count stays quiet while its id remains in the tree.
    #[serde(default)]
    pub count_revealed: bool,
    /// The member `TabId`s in folder order (the exact grouped identities;
    /// every member is pinned by the folder-membership invariant).
    #[serde(default)]
    pub members: Vec<u64>,
}

/// DESIGN.md §11.3 sidebar-swipe evidence on the wire (F2a swipe kernel):
/// the truthful active space plus the gesture's source/destination
/// persistent space ids, the display-order direction resolved at begin, the
/// signed progress (clamped to [-1, 1], `0.0` while Idle), and — only while
/// tracking — the resolved chrome/chip tints the render interpolates from
/// the one central `space_tint` resolver (typed `#RRGGBB` hex, never raw
/// pixels). The gesture never changes browser state: the selected chip and
/// `active_appearance` stay truthful until a committed release reaches the
/// destination through the existing `activate_space` authority.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SwipeSnapshot {
    /// The active space at snapshot time — truthful at every phase: a
    /// tracking gesture never changes the selected space.
    pub active: TestSpaceId,
    /// `tracking` while a gesture is in progress, `idle` otherwise.
    pub phase: String,
    /// The source space of a tracking gesture (the space the gesture began
    /// from). `None` while Idle.
    pub source: Option<TestSpaceId>,
    /// The one adjacent persistent destination space id resolved at begin.
    /// `None` while Idle (boundary gestures never begin).
    pub destination: Option<TestSpaceId>,
    /// The display-order direction resolved at begin: `next` (positive
    /// delta) or `prev` (negative). `None` while Idle.
    pub direction: Option<String>,
    /// The signed progress, clamped to [-1, 1]: positive toward the
    /// destination, negative back toward/past the source. `0.0` while Idle.
    pub progress: f64,
    /// The interpolated sidebar-rail chrome tint (`#RRGGBB`) while tracking
    /// — the exact color the render paints during the gesture. `None` while
    /// Idle (cancel returns the exact source presentation).
    pub interpolated_chrome_tint: Option<String>,
    /// The interpolated active-chip surface tint (`#RRGGBB`) while tracking.
    /// `None` while Idle.
    pub interpolated_chip_tint: Option<String>,
}

impl Default for SwipeSnapshot {
    fn default() -> Self {
        Self {
            active: TestSpaceId(0),
            phase: "idle".to_string(),
            source: None,
            destination: None,
            direction: None,
            progress: 0.0,
            interpolated_chrome_tint: None,
            interpolated_chip_tint: None,
        }
    }
}

/// The §11.3 Space-configuration surface on the wire (DESIGN.md §11.3 /
/// packet `space-creation-appearance-ui.md`): the bound exact persistent
/// `SpaceId`, the header title, the rename draft, and the selected curated
/// theme/identity names — the same plain model the production surface
/// renders and the gallery fixtures drive. `None` while closed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpaceConfigSnapshot {
    /// The exact persistent SpaceId the surface is bound to (captured at
    /// open; never re-derived, never a render position).
    pub space: TestSpaceId,
    /// The header title: the bound space's title at open time.
    pub title: String,
    /// The rename draft (what the `bbr-space-config-name` field shows).
    pub name: String,
    /// The selected curated theme token name (`cobalt`…`ink`).
    pub theme: String,
    /// The selected validated identity in its canonical string form (the
    /// emoji scalar or the glyph name).
    pub identity: String,
    /// v2 D7: the selected tint intensity step in the draft.
    #[serde(default = "standard_intensity")]
    pub intensity: String,
    /// v2 D6: whether the draft turns the static grain on.
    #[serde(default)]
    pub grain: bool,
    /// Whether the surface owns chrome focus (the §9 ring + keyboard
    /// domain of the rename field).
    pub focused: bool,
}

/// The wire default of an older client's missing intensity (v2 D7).
fn standard_intensity() -> String {
    "standard".into()
}
