//! Downloads panel surface (DESIGN.md §11.12, G2 packet
//! `component-gallery-browser-components.md`, Group 5).
//!
//! The anchored popover (`bbr-downloads-panel`) confined to the sidebar
//! column: an **Active** group (non-terminal attempts, ascending
//! `DownloadId`) above a **History** group (terminal attempts,
//! most-recent-terminal first), with `type:badge` group labels hidden when
//! a group is empty, per-row `[status glyph] [name + secondary] [action]`
//! anatomy, the §10.2/§10.3 hover/press washes, exactly one contextual
//! primary action per row, the bounded 6-row window (§11.6 overscan +
//! scrollbar), the empty hint, and the `dur:fine` opacity open/close fade.
//!
//! The panel is ordinary GPUI sidebar chrome: it is a flex child of the
//! sidebar column placed directly above the footer row, so it never
//! overlaps the native web view and never uses the NativeAboveWeb Portal
//! host. All geometry derives from the settled S1 tokens — no invented
//! sizes. Rows render exactly the L6 machine states; the surface never
//! invents a transition — the controller is the authority and `bbr-app`
//! maps records to the [`model::DownloadRowModel`] DTOs.
//!
//! The surface takes a plain presentation model ([`model::DownloadsPanelModel`]:
//! open/focused/alpha, the deterministic row list, the bounded window, the
//! layout math, the tracked scroll handle, and the panel width — all built
//! by `bbr-app` from its controller + sidebar geometry) plus a typed
//! callback bundle ([`callbacks::DownloadsCallbacks`]) that wires the
//! production controller/behavior paths. Gallery stories drive the exact
//! same surface with fixture models and recorder callbacks — no fake
//! engine, controller, portal, or network.

pub mod callbacks;
mod check_moment;
pub mod ids;
pub mod model;
pub mod render;
pub mod row_detail;

pub use callbacks::{
    DownloadsCallbacks, DownloadsClear, DownloadsInsidePress, DownloadsKeyDown, DownloadsRowAction,
    DownloadsRowDelete, DownloadsRowState,
};
pub use ids::{
    download_row_id, download_slot_id, DOWNLOADS_ACTIVE_ID, DOWNLOADS_CLEAR_ID, DOWNLOADS_EMPTY_ID,
    DOWNLOADS_HISTORY_ID, DOWNLOADS_PANEL_ID,
};
pub use model::{
    action_label, DownloadActionKind, DownloadGroupKind, DownloadMachine, DownloadRowModel,
    DownloadsPanelModel, PanelLayout, PanelWindowItems, DOWNLOADS_MAX_VISIBLE, DOWNLOADS_PAGE_ROWS,
    DOWNLOADS_WINDOW_ROWS,
};
pub use render::downloads_panel_surface;

#[cfg(test)]
mod tests;
