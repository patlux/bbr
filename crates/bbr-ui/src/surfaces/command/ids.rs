//! Stable floating-command §15 ids (DESIGN.md §11.4/§15) and the
//! render-internal animation element id.
//!
//! These are the ids of the **floating** command presentations — the
//! centered Cmd-T palette and the field-anchored Cmd-L/click overlay —
//! rendered inside the one native portal window. They moved here with the
//! production renderer (G3 Command cutover); the in-rail command surface
//! (`bbr-command-results` family) stays app-owned. The same constants feed
//! the renderer, the app's `visible_ids` derivation, and the portal
//! snapshot, so E2E ids can never drift from the shell.

/// The centered Cmd-T palette container (DESIGN.md §11.4).
pub const PALETTE_ID: &str = "bbr-palette";
/// The palette's field row.
pub const PALETTE_FIELD_ID: &str = "bbr-palette-field";
/// The palette's `type:mono` field text.
pub const PALETTE_TEXT_ID: &str = "bbr-palette-text";
/// The palette's mode chip (New Tab staged per the interaction contract).
pub const PALETTE_MODE_ID: &str = "bbr-palette-mode";
/// The palette's block cursor (while the surface owns chrome focus).
pub const PALETTE_CURSOR_ID: &str = "bbr-palette-cursor";
/// Result rows of the centered palette keep the §11.5 row anatomy under
/// `bbr-palette-result-<k>`.
pub const PALETTE_RESULT_PREFIX: &str = "bbr-palette-result";
/// Typed state rows of the centered palette (loading / empty / error).
pub const PALETTE_STATE_LOADING: &str = "bbr-palette-loading";
pub const PALETTE_STATE_EMPTY: &str = "bbr-palette-empty";
pub const PALETTE_STATE_ERROR: &str = "bbr-palette-error";

/// The field-anchored overlay container (DESIGN.md §11.4); its results
/// keep the §11.5 `bbr-command-result-<k>` ids of the in-rail surface.
pub const OVERLAY_ID: &str = "bbr-command-overlay";
/// The overlay's own field row (DESIGN.md §11.4): the full current URL is
/// preselected here, so one keystroke replaces it.
pub const OVERLAY_FIELD_ID: &str = "bbr-command-overlay-field";
pub const OVERLAY_TEXT_ID: &str = "bbr-command-overlay-text";
pub const OVERLAY_CURSOR_ID: &str = "bbr-command-overlay-cursor";
/// Typed state rows of the field-anchored overlay — the same §11.5 state
/// strings the in-rail surface uses (`bbr-command-loading` / `-empty` /
/// `-error`), rendered by the floating surface with the same ids.
pub const OVERLAY_STATE_LOADING: &str = "bbr-command-loading";
pub const OVERLAY_STATE_EMPTY: &str = "bbr-command-empty";
pub const OVERLAY_STATE_ERROR: &str = "bbr-command-error";

/// The §11.4 open-animation element id of the field-anchored overlay: its
/// entrance opacity fade at `dur:fine` (DESIGN.md §6/§11.4) — opacity only,
/// never translate or scale. A render-internal element id, not a §15
/// semantic id: `visible_ids` and the portal snapshot derive from the
/// surface ids, so this id can never drift into E2E assertions.
pub const OVERLAY_FADE_ELEMENT_ID: &str = "bbr-command-overlay-fade";
