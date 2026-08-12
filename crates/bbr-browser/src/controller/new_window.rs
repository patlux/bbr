//! Productive regular-tab execution for decided new-window requests.
//!
//! Two dispositions execute here: [`NewWindowDisposition::OpenTab`] opens an
//! active regular tab in the source space (the ordinary popup/new-window
//! path), and [`NewWindowDisposition::OpenBackgroundTab`] opens one regular
//! child tab in the background immediately after the exact opener while the
//! opener stays selected, visible, and web-focused (Command-primary /
//! middle-click background link tabs). Both resolve the exact live source
//! session and validate its owning profile against the decided session
//! before any domain or engine mutation.

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
        let (source_tab, source_space, source_profile) =
            self.resolve_new_window_source(source, session)?;
        let tab = self.browser.open_tab(source_space, "", session.url())?;
        // The productive new-window regular-tab path is the **only** creator
        // of opener provenance: identity-based child → exact opener relation.
        self.provenance.record(tab, source_tab);
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

    /// Open one regular child tab in the background from an exact live
    /// source session (Command-primary / middle-click background link tabs).
    ///
    /// The identity-bound grouped insertion policy inserts the first child
    /// immediately after the exact opener and every later child after the
    /// last surviving member of the opener's contiguous child group,
    /// preserving click order. The opener stays selected, visible, and
    /// web-focused: no selection change, no split mutation, no focus
    /// bounce — the new child is a warm hidden session in the source
    /// space/profile. Stale sources and profile mismatches reject before
    /// any domain or engine mutation.
    pub fn open_background_tab_from_new_window(
        &mut self,
        source: WebViewId,
        session: &NewWindowSession,
    ) -> Result<ControllerEvent, ControllerError> {
        let (source_tab, source_space, source_profile) =
            self.resolve_new_window_source(source, session)?;
        let index = self.background_child_index(source_space, source_tab);
        let tab = self
            .browser
            .insert_tab(source_space, index, "", session.url())?;
        self.provenance.record(tab, source_tab);
        self.sync_views();
        // The opener is still the active tab, so an open preview over it is
        // not stale and stays; the hidden child never moves selection.
        self.close_preview_if_stale();
        let index = self
            .browser
            .space(source_space)
            .and_then(|space| space.tabs.iter().position(|candidate| *candidate == tab))
            .expect("background child belongs to source space");
        let view = self
            .registry
            .view_for_tab(tab)
            .expect("warm background child owns a live session");
        self.check_invariants();
        Ok(ControllerEvent::NewWindowOpened {
            index,
            url: session.url().to_string(),
            space: source_space,
            profile: source_profile,
            view,
        })
    }

    /// Resolve the exact live source tab/space/profile of a decided
    /// new-window request and validate it against the decided session. Both
    /// dispositions fail closed on a stale source view or a profile
    /// mismatch before any domain or engine mutation.
    fn resolve_new_window_source(
        &self,
        source: WebViewId,
        session: &NewWindowSession,
    ) -> Result<(bbr_domain::TabId, bbr_domain::SpaceId, bbr_web::ProfileId), ControllerError> {
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
        Ok((source_tab, source_space, source_profile))
    }

    /// The deterministic grouped insertion index for a background child of
    /// `opener` in `space` (background link tabs packet).
    ///
    /// Identity-bound, never adjacency-inferred: scan right from the opener
    /// while the consecutive tabs are exact provenance children of the
    /// opener and stop at the first non-child. The first child therefore
    /// lands immediately after the opener and every later child after the
    /// last surviving member of its contiguous group, preserving click
    /// order. A child that was closed, moved away, or manually separated is
    /// never inferred: the next insertion resumes after the contiguous run
    /// that still directly follows the opener. A pinned opener's child is
    /// regular, so the scan starts at the pinned/regular boundary (the
    /// nearest legal position after a pinned opener).
    fn background_child_index(
        &self,
        space: bbr_domain::SpaceId,
        opener: bbr_domain::TabId,
    ) -> usize {
        let tabs = &self
            .browser
            .space(space)
            .expect("validated source space")
            .tabs;
        let len = tabs.len();
        let pinned = self
            .browser
            .space(space)
            .expect("validated source space")
            .pinned_count;
        let opener_index = tabs
            .iter()
            .position(|candidate| *candidate == opener)
            .expect("opener is in the validated source space");
        let mut index = if opener_index >= pinned {
            opener_index + 1
        } else {
            pinned
        };
        while index < len && self.provenance.opener_of(tabs[index]) == Some(opener) {
            index += 1;
        }
        index
    }
}
