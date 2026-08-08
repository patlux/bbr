//! Browser-state writer lifecycle (Phase 7, warm-restore slice): the
//! app-owned structural browser model (profiles/spaces/tabs/pinned/order/
//! active/split/recently-closed **plus** the window/sidebar presentation
//! rows) persists through the same bounded nonblocking pattern as the
//! command-history provider — the app replaces a shared slot on every
//! structural mutation and pokes a capacity-1 channel; the background
//! writer coalesces and replaces the browser tables in one transaction
//! ([`Repository::store_browser_state`]); the quit path flushes
//! synchronously. The global settings, download, and command-history rows
//! are never touched by a browser-state write.
//!
//! Exactly one AppStore/Repository authority owns both persist paths; the
//! composition root spawns both writers from the store lifecycle.

use bbr_persistence::BrowserState;
use gpui::BackgroundExecutor;

use super::AppStore;
impl AppStore {
    /// The restored browser structural state at startup (empty on a fresh
    /// or discarded store): the composition maps it into the chrome-first
    /// restore intent, or starts fresh when it is empty.
    pub fn restored_browser(&self) -> &BrowserState {
        &self.restored_browser
    }

    /// Spawns the bounded nonblocking browser-state writer on the
    /// background executor: each poke coalesces (only the latest slot value
    /// is ever written) and replaces the browser tables in one transaction.
    /// No-op on the ephemeral store. Called exactly once by the composition
    /// root after `open`, alongside the history writer.
    pub fn spawn_browser_writer(&mut self, background: &BackgroundExecutor) {
        let Some(repository) = self.repository.clone() else {
            log::warn!("ephemeral store: background browser-state writer not spawned");
            return;
        };
        let Some(browser_rx) = self.browser_rx.take() else {
            return;
        };
        let slot = self.browser_slot.clone();
        background
            .spawn(async move {
                while browser_rx.recv().await.is_ok() {
                    // Coalesce: drain any redundant pokes already queued
                    // (capacity 1, so at most one); the browser tables are
                    // replaced whole, so only the latest state matters.
                    while browser_rx.try_recv().is_ok() {}
                    let state = slot.lock().expect("browser-state slot lock").clone();
                    if let Some(state) = state {
                        if let Err(error) = repository.store_browser_state(&state) {
                            log::warn!("browser-state persist failed: {error:?}");
                        }
                    }
                }
            })
            .detach();
    }

    /// Replaces the browser-state slot and pokes the bounded writer (the
    /// app calls this after every structural mutation — open/close/pin/
    /// reorder/move/split/space/profile/sidebar — so a force quit recovers
    /// the last structurally persisted model). The poke is nonblocking: a
    /// full capacity-1 queue means a write is already pending, so the
    /// signal is coalesced — never an error and never a growing queue.
    pub fn poke_browser_state(&self, state: BrowserState) {
        *self.browser_slot.lock().expect("browser-state slot lock") = Some(state);
        let _ = self.browser_tx.try_send(());
    }

    /// Synchronously persists the latest browser-state slot in one
    /// transaction (the deterministic flush seam and the quit flush). The
    /// background writer uses the identical repository write; a flush
    /// before any poke is a no-op (nothing structural happened yet).
    pub fn flush_browser_state(&self) {
        let Some(repository) = self.repository.as_ref() else {
            log::warn!("ephemeral store: browser-state flush refused");
            return;
        };
        let state = self
            .browser_slot
            .lock()
            .expect("browser-state slot lock")
            .clone();
        let Some(state) = state else {
            return;
        };
        if let Err(error) = repository.store_browser_state(&state) {
            log::warn!("browser-state flush failed: {error:?}");
        }
    }
}
