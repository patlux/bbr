//! The §15 state vocabulary (DESIGN.md §10/§11.3/§11.12) — plain `bbr-ui`
//! DTOs shared by the production render, the app's typed snapshot, the
//! semantic tree, and the future gallery stories. No browser, session,
//! profile, or space authority lives here: these are presentation states
//! and rendered identities only.

use bbr_domain::{ProfileHue, SpaceId, SpaceTheme};

/// §10 row states the shell renders and E2E asserts. The full §15
/// vocabulary; hover/active/focused/attention are derived from render
/// state the polish passes introduce, so they legitimately stay
/// unconstructed until then.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TabVisualState {
    Default,
    Hover,
    Active,
    Selected,
    Loading,
    Attention,
    /// §10.7: the tab's web process terminated (crash authority; the
    /// session is destroyed and the row shows the lifering in `status.error`).
    Error,
    /// §10.11: the session's renderer stopped answering liveness probes
    /// (fail-closed; the session and tab stay alive under the §11.8a
    /// recovery surface). A sibling of `Error` — the row shows the lifering
    /// plus the static clock companion.
    Unresponsive,
    Disabled,
    /// §10.9: the dragged row (92% opacity, elev:2 shadow, strong outline).
    Dragging,
    /// §10.9: the drop target (accent tick + target outline + tint; the
    /// space target replaces the tick with the destination profile chip).
    DragTarget,
    Focused,
}

/// The §10 visual state of a download row (DESIGN.md §11.12: the applicable
/// names for the panel — focus is the keyboard cursor, no persistent
/// selection).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadVisual {
    Default,
    Hover,
    Pressed,
    Focused,
    Disabled,
}

/// The §12 destination-profile companion of a chip during a drag (DESIGN.md
/// §10.9/§11.3): the destination space's profile identity — creation-order
/// index (the `bbr-profile-chip-<n>` id), monogram, and hue — rendered
/// inside the targeted chip so the cross-profile destination is unmistakable
/// without relying on color. Derived from the controller snapshot in the
/// same `space_chip_visuals` derivation the render, the snapshot, and the
/// semantic tree consume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileCompanion {
    /// The destination profile's creation-order index (the §12 chip id).
    pub profile_index: usize,
    /// The §12 monogram (the non-color identity).
    pub letter: String,
    /// The §12 hue (the tile's identity; never the only cue).
    pub hue: ProfileHue,
}

/// One rendered space chip's identity, derived from the controller snapshot
/// plus the effective space appearance (the portable appearance contract's
/// validated identity) and the app's drag state. The render, the snapshot,
/// and the semantic tree all consume this same source so the chip ids can
/// never drift.
///
/// Display ordering is the caller's vector order (canonical grouped order);
/// the persistent id is [`Self::space`] alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceChipVisuals {
    /// The persistent space identity — the only identity of the chip ids
    /// (`bbr-space-chip-<persistent-space-id>`). Never the render position,
    /// the creation-order index, or the profile index.
    pub space: SpaceId,
    /// The space title after §4 truncation.
    pub title: String,
    /// The owning profile's title — part of the chip's accessible name
    /// (DESIGN.md §11.3: the stable accessible name carries the Space and
    /// Profile names, so identity is never color-only).
    pub profile_title: String,
    /// The validated curated identity in its canonical string form (the
    /// emoji scalar or the glyph name).
    pub identity: String,
    /// The effective curated theme token of this space (from the one
    /// controller appearance authority; derived defaults included). The
    /// chip resolves its tint through [`crate::theme::space_tint`], so the
    /// render, the snapshot, and the semantic tree share one theme source.
    pub theme: SpaceTheme,
    /// Whether this is the active space (the chip renders selected).
    pub selected: bool,
    /// §10.9: whether a pointer drag currently targets this exact chip (the
    /// chip renders the drag-target outline + tint plus the §12
    /// destination-profile companion). Derived from the app's drag state,
    /// never a render position.
    pub drag_target: bool,
    /// The §12 destination-profile companion rendered while this chip is
    /// the drag target; `None` at rest. Never the only identity cue — the
    /// chip always carries its curated identity and title.
    pub companion: Option<ProfileCompanion>,
}
