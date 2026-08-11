//! Downloads panel §15 ids (DESIGN.md §11.12/§15) — the stable `bbr-*` id
//! constants and the deterministic per-row id derivations. `bbr-app`'s
//! visible-ids/snapshot derivation imports the exact same ids, so the
//! shell and the surface can never drift.

/// §11.12/§15 assert ids of the downloads panel.
pub const DOWNLOADS_PANEL_ID: &str = "bbr-downloads-panel";
pub const DOWNLOADS_ACTIVE_ID: &str = "bbr-downloads-active";
pub const DOWNLOADS_HISTORY_ID: &str = "bbr-downloads-history";
pub const DOWNLOADS_EMPTY_ID: &str = "bbr-downloads-empty";
/// The settled Clear panel action (DESIGN.md §17.7): the explicit
/// `bbr-downloads-clear` control rendered below the scroll area while any
/// history exists. It empties the controller history and the repository
/// durably.
pub const DOWNLOADS_CLEAR_ID: &str = "bbr-downloads-clear";

/// The `bbr-download-row-<n>` id of row `index` (deterministic panel
/// index).
pub fn download_row_id(index: usize) -> String {
    format!("bbr-download-row-{index}")
}

/// The `bbr-download-<slot>-<n>` ids of one row (DESIGN.md §11.12:
/// `-status`/`-name`/`-meta`/`-progress`/`-action`/`-delete`).
pub fn download_slot_id(slot: &str, index: usize) -> String {
    format!("bbr-download-{slot}-{index}")
}
