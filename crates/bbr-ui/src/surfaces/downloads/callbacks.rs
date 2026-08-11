//! Downloads panel callbacks (DESIGN.md §11.12, G2 packet
//! `component-gallery-browser-components.md`, Group 5): the typed listener
//! bundle `bbr-app` wires onto the `bbr-ui` surface and the gallery
//! recorder wires onto the same surface. The surface never holds a
//! controller, engine, or portal — every interactive path routes through
//! one of these callbacks.

use std::rc::Rc;

use gpui::{App, KeyDownEvent, MouseDownEvent, Window};

/// The §11.12 panel keyboard listener (`↑`/`↓`/`PgUp`/`PgDn`/Enter/
/// Tab/Escape — the app's `on_downloads_key_down`).
pub type DownloadsKeyDown = Rc<dyn Fn(&KeyDownEvent, &mut Window, &mut App)>;
/// The §13 inside-press marker: a pointer press inside the panel marks
/// itself so the root outside-press handler never dismisses the panel,
/// and stops propagation (one pointer action).
pub type DownloadsInsidePress = Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>;
/// The contextual row action by deterministic panel index (Cancel / Retry /
/// Restart / Reveal-in-Finder — the app's `download_click_row_action`).
pub type DownloadsRowAction = Rc<dyn Fn(usize, &mut Window, &mut App)>;
/// The row hover/press state by panel index (the §10.2/§10.3 washes).
pub type DownloadsRowState = Rc<dyn Fn(usize, bool, &mut Window, &mut App)>;
/// The §17.8 row Delete control by attempt id (`bbr-download-delete-<n>`).
pub type DownloadsRowDelete = Rc<dyn Fn(u64, &mut Window, &mut App)>;
/// The §17.7 Clear control.
pub type DownloadsClear = Rc<dyn Fn(&mut Window, &mut App)>;

/// The full production listener bundle of the §11.12 downloads panel:
/// every interactive path the surface draws routes through one of these
/// callbacks, so the normal binary reaches every packet operation
/// (cancel/retry/restart/reveal/delete/clear, the keyboard cursor, the
/// inside-press marker) through the same composition the E2E asserts.
pub struct DownloadsCallbacks {
    /// The panel keyboard listener (§11.12).
    pub on_key_down: DownloadsKeyDown,
    /// A pointer press inside the panel (§13 one pointer action).
    pub on_inside_press: DownloadsInsidePress,
    /// Activate the contextual row action by deterministic panel index.
    pub on_row_action: DownloadsRowAction,
    /// The row pressed wash (§10.3): `(index, pressed)`.
    pub on_row_press: DownloadsRowState,
    /// The row hover wash (§10.2): `(index, hovered)`.
    pub on_row_hover: DownloadsRowState,
    /// The §17.8 row Delete control (attempt id).
    pub on_row_delete: DownloadsRowDelete,
    /// The §17.7 Clear panel control.
    pub on_clear: DownloadsClear,
}
