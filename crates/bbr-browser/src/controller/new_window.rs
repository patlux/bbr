//! Productive regular-tab execution for decided new-window requests.

use bbr_web::{NewWindowSession, WebViewId};

use crate::{BrowserController, ControllerError, ControllerEvent};

impl BrowserController {
    /// Open one regular child tab from an exact live source session.
    ///
    /// The source tab's owning space/profile is authoritative even when that
    /// space is inactive. Opening there creates a warm hidden session and does
    /// not switch or mutate the active space. Stale sources and profile
    /// mismatches reject before any domain or engine mutation.
    pub fn open_tab_from_new_window(
        &mut self,
        source: WebViewId,
        session: &NewWindowSession,
    ) -> Result<ControllerEvent, ControllerError> {
        let source_tab = self
            .registry
            .tab_for_view(source)
            .ok_or(ControllerError::UnknownWebView(source))?;
        let source_space = self
            .browser
            .tab(source_tab)
            .ok_or(ControllerError::UnknownWebView(source))?
            .space;
        let source_profile = self
            .browser
            .space(source_space)
            .ok_or(ControllerError::UnknownSpace(source_space))?
            .profile;
        if source_profile != session.source_profile() {
            return Err(ControllerError::NewWindowProfileMismatch {
                source: source_profile,
                requested: session.source_profile(),
            });
        }

        let tab = self.browser.open_tab(source_space, "", session.url())?;
        self.sync_views();
        // A new regular tab in the active source space becomes active, which
        // moves the selection away from the preview's origin: the transient
        // preview closes (deterministic origin-removal policy). When the
        // source space is inactive the active space/selection is unchanged
        // and the preview stays.
        self.close_preview_if_stale();
        let index = self
            .browser
            .space(source_space)
            .and_then(|space| space.tabs.iter().position(|candidate| *candidate == tab))
            .expect("new tab belongs to source space");
        let view = self
            .registry
            .view_for_tab(tab)
            .expect("active or warm new tab owns a live session");
        self.check_invariants();
        Ok(ControllerEvent::NewWindowOpened {
            index,
            url: session.url().to_string(),
            space: source_space,
            profile: source_profile,
            view,
        })
    }
}
