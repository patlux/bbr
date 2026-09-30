//! Blueprint D17 (board L-11): the startup recovery pane. When the last
//! session could not be restored, the card's empty state explains why once
//! (instead of starting silently empty) with the one useful action.

use bbr_ui::components::recovery::RecoveryKind;
use gpui::Context;

use super::BbrApp;
use crate::store::StoreRecovery;

impl BbrApp {
    /// The pane to show in place of the empty state, if any.
    pub(crate) fn recovery_pane_kind(&self) -> Option<RecoveryKind> {
        if self.recovery_dismissed.value() {
            return None;
        }
        self.store
            .read()
            .recovery
            .as_ref()
            .map(|recovery| match recovery {
                StoreRecovery::Unreadable { kept } => RecoveryKind::Unreadable {
                    kept: kept.is_some(),
                },
                StoreRecovery::Newer { .. } => RecoveryKind::Newer,
            })
    }

    /// The pane's action (its button and the testkit seam): reveal the kept
    /// session file in Finder, or continue with the fresh session.
    pub fn recovery_action(&mut self, cx: &mut Context<Self>) -> bool {
        let recovery = self.store.read().recovery.clone();
        match recovery {
            Some(StoreRecovery::Unreadable { kept: Some(path) }) => {
                #[cfg(target_os = "macos")]
                if let Some(adapter) = self.downloads_adapter.as_ref() {
                    if let Err(error) = adapter.reveal(&path.to_string_lossy()) {
                        log::warn!("could not reveal the kept session: {error:?}");
                    }
                }
                let _ = path;
                true
            }
            Some(StoreRecovery::Newer { .. }) => {
                self.recovery_dismissed.set(true);
                cx.notify();
                true
            }
            _ => false,
        }
    }
}

/// Blueprint D18: the About window.
impl BbrApp {
    /// Open About BBR (the menu item's action).
    pub fn show_about(&mut self, cx: &mut Context<Self>) {
        let theme = self.theme();
        let about = self.about_window.value();
        self.about_window
            .set(crate::about::open(about, theme, !self.windows_hidden, cx));
    }
}
