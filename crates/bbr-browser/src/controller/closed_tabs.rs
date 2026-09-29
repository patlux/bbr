//! Blueprint D10: the active space's recently closed tabs for History ›
//! Recently closed, and reopening one of them by identity. Blueprint D11:
//! the profile a live view belongs to (a permission request names its
//! view).

use bbr_domain::TabId;

use crate::outcome::ControllerEvent;
use crate::ControllerError;

use super::BrowserController;

/// One recently closed tab as History lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedTabEntry {
    pub tab: TabId,
    pub title: String,
    pub url: String,
}

impl BrowserController {
    /// The profile whose data store backs the live `view` (a tab's session
    /// or the open Peek preview); `None` for an unknown view.
    pub fn profile_of_view(&self, view: bbr_web::WebViewId) -> Option<bbr_web::ProfileId> {
        if let Some(preview) = self.preview.as_ref().filter(|p| p.view == view) {
            return Some(preview.profile);
        }
        let tab = self.registry.tab_for_view(view)?;
        let space = self.browser.tab(tab)?.space;
        Some(self.browser.space(space)?.profile)
    }

    /// The active space's recently closed tabs, most recent first.
    pub fn recently_closed(&self) -> Vec<ClosedTabEntry> {
        self.browser
            .space(self.active_space)
            .map(|space| {
                space
                    .recently_closed
                    .iter()
                    .rev()
                    .map(|closed| ClosedTabEntry {
                        tab: closed.tab.id,
                        title: closed.tab.title.clone(),
                        url: closed.tab.url.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Reopen the recently closed `tab` of the active space with its
    /// preserved URL/title identity — a new session is created and shown,
    /// exactly as ⇧⌘T does for the latest one.
    pub fn reopen_closed_tab(&mut self, tab: TabId) -> Result<ControllerEvent, ControllerError> {
        let tab = self.browser.reopen_closed_tab(self.active_space, tab)?;
        self.close_preview_if_stale();
        self.sync_views();
        self.check_invariants();
        let index = self.index_of(tab).expect("tab was just reopened");
        let url = self
            .browser
            .tab(tab)
            .map(|t| t.url.clone())
            .unwrap_or_default();
        Ok(ControllerEvent::TabReopened { index, url })
    }
}
