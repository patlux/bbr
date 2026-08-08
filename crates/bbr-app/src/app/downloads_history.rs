//! Downloads durable history mutations (DESIGN.md §17.7/§17.8/§17.10
//! settled): the Clear panel action, the row Delete action, and the
//! persist path that writes the controller's terminal history through the
//! app-owned store.
//!
//! Split out of `downloads_behavior` to keep both files within the ~500-line
//! structure gate. These are the mutations that change **persisted** state:
//! Clear empties the controller history and the repository's downloads
//! table in one transaction (so cleared history can never return from the
//! store), a row Delete removes exactly one terminal record durably, and
//! every accepted terminal transition persists the terminal history. All of
//! it runs outside render/ranking — on the main-thread event pump and on
//! the explicit pointer actions.

use bbr_browser::DownloadError;
use bbr_web::downloads::DownloadId;
use gpui::Context;

use super::BbrApp;

/// The settled Clear action (DESIGN.md §17.7): an explicit panel action
/// that empties the **controller history** — every terminal record is
/// removed and the repository's downloads table is replaced with the empty
/// slice in one transaction, so cleared history can never return from the
/// store. Live (non-terminal) attempts are kept. The panel's focus, scroll,
/// hover/press state, and the reveal cache reset with the rows.
pub fn downloads_clear(app: &mut BbrApp, cx: &mut Context<BbrApp>) {
    app.downloads.controller.clear_history();
    app.downloads.reveal.clear();
    app.downloads.focus = 0;
    app.downloads.hovered_row = None;
    app.downloads.pressed_row = None;
    app.downloads
        .scroll
        .set_offset(gpui::point(gpui::px(0.0), gpui::px(0.0)));
    app.persist_downloads();
    cx.notify();
}

/// The settled row Delete action (DESIGN.md §17.8): remove exactly one
/// terminal record through the controller and persist the remaining
/// terminal history durably (a live attempt is rejected by the controller
/// — a running transfer is never deleted from under the engine). The
/// removed row's reveal-cache entry is dropped with it.
pub fn download_delete_row(
    app: &mut BbrApp,
    id: DownloadId,
    cx: &mut Context<BbrApp>,
) -> Result<(), DownloadError> {
    app.downloads.controller.delete_history(id)?;
    app.downloads.reveal.remove(&id);
    app.persist_downloads();
    cx.notify();
    Ok(())
}

impl BbrApp {
    /// Durably persist the controller's terminal download history through
    /// the app-owned store (DESIGN.md §17.10 settled): the terminal records
    /// are exported in persisted order and the repository replaces the
    /// downloads table in one transaction. Runs outside render/ranking —
    /// on an accepted terminal event, on Clear, and on a row Delete (the
    /// same write the background command-history writer shape follows;
    /// download terminal transitions are rare, so the write is
    /// synchronous and deterministic, never a second authority).
    pub(crate) fn persist_downloads(&self) {
        let terminal = self.downloads.controller.terminal_history();
        let records = crate::store::downloads::to_persisted(&terminal);
        self.store.replace_downloads(&records);
    }
}
