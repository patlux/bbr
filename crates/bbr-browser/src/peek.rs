//! Transient Peek preview lifecycle (Link Preview / Peek packet).
//!
//! A peek is **transient browser state**, never a persisted `Tab`, recently
//! closed entry, sidebar row, or history item: it exists only while the
//! controller holds it, dies with the session, and never survives a restart.
//! Each preview has a typed [`PreviewId`] and a disposable source-profile
//! [`WebViewId`] owned by the controller/session registry until dismissal or
//! promotion.
//!
//! ## Lifecycle invariants
//!
//! - The preview session is created in the **source profile's** data store
//!   (validated against the resolved source tab); requests never cross
//!   profile boundaries and cookies/credentials are never copied.
//! - The origin tab stays selected and its native session stays **warm**
//!   underneath (same `WebViewId`, URL, and page state); the preview view is
//!   raised above it and covers the complete web-content area.
//! - Exactly **one preview per pane**: a new eligible request from inside
//!   the preview *navigates* the existing preview session (the deterministic
//!   documented policy); any other source while a preview is open
//!   *replaces* it (the old session is destroyed first).
//! - The preview stays only while its source tab is the active tab of the
//!   active space. Any mutation that moves the active tab/space away from
//!   the source (select, close, move, space switch, split change, reopen,
//!   new tab) closes the preview through [`BrowserController::close_preview_if_stale`].
//! - **Promotion** adopts the existing native session as a regular tab
//!   without reload or state loss: the same `WebViewId` moves from the
//!   transient state into the `TabId` bijection; the session is neither
//!   recreated nor temporarily orphaned. A crashed preview cannot promote
//!   (only a live session is adoptable without recreation).
//!
//! The regular-tab execution path and its `TabId` opener provenance stay
//! owned by `open_tab_from_new_window`; this module never duplicates the
//! new-window disposition authority.

use bbr_domain::TabId;
use bbr_web::{NewWindowSession, ProfileId, WebAction, WebEvent, WebViewId};

use crate::outcome::ControllerEvent;
use crate::pane::PaneLayout;
use crate::{BrowserController, ControllerError};

/// Typed transient preview identity. Distinct from `TabId` (a preview is not
/// a tab) and from `WebViewId` (the preview session is disposable and
/// controller-owned); it exists only while the preview is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PreviewId(pub u64);

/// The live transient preview state the controller owns.
///
/// The origin (`source_view`/`source_tab`) stays selected and warm while the
/// preview is open; the preview session (`view`) is a disposable
/// source-profile session that promotion adopts or dismissal destroys.
#[derive(Debug, Clone, PartialEq)]
pub struct PeekPreview {
    /// Typed transient preview identity.
    pub id: PreviewId,
    /// The disposable source-profile web session (never a domain tab's).
    pub view: WebViewId,
    /// The covered warm origin session.
    pub source_view: WebViewId,
    /// The origin tab: the preview stays only while it is the active tab of
    /// the active space.
    pub source_tab: TabId,
    /// The origin space (== the active space while the preview is open).
    pub source_space: bbr_domain::SpaceId,
    /// The source profile whose website data store backs the preview.
    pub profile: ProfileId,
    /// The initial URL the preview was opened at (the live URL comes from
    /// the session's navigation state).
    pub url: String,
}

/// Read-only snapshot of the transient preview for UI and E2E.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewSnapshot {
    pub id: u64,
    pub view: WebViewId,
    pub source: WebViewId,
    pub profile: ProfileId,
    pub url: String,
    pub title: Option<String>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub crashed: bool,
}

impl BrowserController {
    /// Whether a transient preview is currently open.
    pub fn preview_open(&self) -> bool {
        self.preview.is_some()
    }

    /// The live transient preview state, if any.
    pub fn preview(&self) -> Option<&PeekPreview> {
        self.preview.as_ref()
    }

    /// The pane layout while a preview is open: the preview view is the one
    /// pane (covering the complete web-content area), the covered origin is
    /// not a pane target, and the preview owns browser-command focus.
    pub(crate) fn preview_pane_layout(&self) -> Option<PaneLayout> {
        self.preview.as_ref().map(|preview| PaneLayout {
            primary: preview.view,
            secondary: None,
            focused: preview.view,
        })
    }

    /// Open a transient preview from a decided new-window request.
    ///
    /// Source resolution and profile validation are identical to regular-tab
    /// creation (`open_tab_from_new_window`): the resolved source tab's
    /// owning space/profile is authoritative and the request must agree.
    ///
    /// One-preview-per-pane policy: a request whose source is the existing
    /// preview session navigates that preview (same session, same
    /// `WebViewId`, back/forward state preserved); any other source replaces
    /// it (the old preview session is destroyed first).
    pub fn open_preview_from_new_window(
        &mut self,
        source: WebViewId,
        session: &NewWindowSession,
    ) -> Result<ControllerEvent, ControllerError> {
        // Deterministic one-preview-per-pane policy FIRST: a request whose
        // source is the existing preview session (a new eligible request
        // from inside Peek) navigates that preview — the preview session is
        // not a registry tab, so it must be resolved before source-tab
        // validation. Same session, same `WebViewId`, back/forward state
        // preserved; the URL passes verbatim (never normalized).
        if let Some(existing) = &self.preview {
            if existing.view == source {
                let url = session.url().to_string();
                self.engine
                    .borrow_mut()
                    .action(existing.view, WebAction::Load { url });
                self.nav
                    .entry(existing.view)
                    .or_default()
                    .apply(&WebEvent::UrlChanged(
                        existing.view,
                        session.url().to_string(),
                    ));
                self.check_invariants();
                return Ok(ControllerEvent::PreviewNavigated {
                    view: existing.view,
                });
            }
        }

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

        if let Some(_existing) = &self.preview {
            // Any other source while a preview is open replaces it: the old
            // session is destroyed first (never stacked).
            let _ = self.close_preview();
        }

        let view = WebViewId((self.allocate_view)());
        self.engine
            .borrow_mut()
            .create_web_view(view, source_profile, session.url());
        // The preview must sit above the warm origin: raise it to the top of
        // the native z-order (the origin view stays warm underneath).
        self.engine.borrow_mut().raise_web_view(view);
        self.visibility.insert(view, true);
        self.engine.borrow_mut().show(view, true);
        self.session_profiles.insert(view, source_profile);
        let id = PreviewId(self.next_preview_id);
        self.next_preview_id += 1;
        self.preview = Some(PeekPreview {
            id,
            view,
            source_view: source,
            source_tab,
            source_space,
            profile: source_profile,
            url: session.url().to_string(),
        });
        self.check_invariants();
        Ok(ControllerEvent::PreviewOpened {
            id,
            view,
            source,
            profile: source_profile,
            url: session.url().to_string(),
        })
    }

    /// Route one browser navigation action to the preview session (it owns
    /// browser-command focus while open).
    pub fn preview_action(
        &mut self,
        action: WebAction,
    ) -> Result<ControllerEvent, ControllerError> {
        let view = self
            .preview
            .as_ref()
            .map(|preview| preview.view)
            .ok_or(ControllerError::NoPreviewOpen)?;
        self.engine.borrow_mut().action(view, action);
        self.check_invariants();
        Ok(ControllerEvent::NavigationRouted { view })
    }

    /// Dismiss the preview: destroy its temporary native session and release
    /// the transient state. The warm origin session is untouched and the
    /// caller restores focus to it.
    pub fn close_preview(&mut self) -> Result<ControllerEvent, ControllerError> {
        let preview = self.preview.take().ok_or(ControllerError::NoPreviewOpen)?;
        self.destroy_preview_session(preview.view);
        self.check_invariants();
        Ok(ControllerEvent::PreviewClosed { view: preview.view })
    }

    /// Adopt the preview session as a regular tab in the source
    /// space/profile **without reload or state loss**: the existing native
    /// session (same `WebViewId`, same URL/history/form/scroll state) moves
    /// atomically from the transient state into the `TabId` bijection — it
    /// is neither recreated nor temporarily orphaned, and no profile
    /// boundary is crossed.
    ///
    /// The promoted tab becomes the source space's active tab (and therefore
    /// the active selection when the source space is active, which the
    /// staleness policy guarantees while the preview is open). A crashed
    /// preview refuses promotion: only a live session can be adopted without
    /// recreation.
    pub fn promote_preview(&mut self) -> Result<ControllerEvent, ControllerError> {
        let preview = self.preview.clone().ok_or(ControllerError::NoPreviewOpen)?;
        let live_url = self
            .nav
            .get(&preview.view)
            .and_then(|nav| nav.url.clone())
            .filter(|url| !url.is_empty())
            .unwrap_or_else(|| preview.url.clone());
        if self.nav.get(&preview.view).is_some_and(|nav| nav.crashed) {
            return Err(ControllerError::PreviewCrashed);
        }
        let space = self
            .browser
            .space(preview.source_space)
            .ok_or(ControllerError::UnknownSpace(preview.source_space))?;
        if space.profile != preview.profile {
            return Err(ControllerError::NewWindowProfileMismatch {
                source: space.profile,
                requested: preview.profile,
            });
        }
        // Seed the domain tab with the preview's LIVE navigation state
        // (url + title): no reload happens at promotion, so no fresh
        // `TitleChanged` event will ever arrive — the persistent tab
        // identity must carry the state the preview session already has
        // (same-document/scroll/form state stay in the adopted session).
        let live_title = self
            .nav
            .get(&preview.view)
            .and_then(|nav| nav.title.clone())
            .unwrap_or_default();
        // The domain tab is created in the source space; the EXISTING
        // session is adopted without creating a new WebViewId.
        let tab = self
            .browser
            .open_tab(preview.source_space, &live_title, &live_url)?;
        self.registry
            .insert(tab, preview.view)
            .expect("transient preview view is not mapped to any tab");
        self.session_profiles.insert(preview.view, preview.profile);
        self.preview = None;
        self.sync_views();
        self.check_invariants();
        let index = self
            .browser
            .space(preview.source_space)
            .and_then(|space| space.tabs.iter().position(|candidate| *candidate == tab))
            .expect("promoted tab belongs to the source space");
        Ok(ControllerEvent::PreviewPromoted {
            tab,
            index,
            space: preview.source_space,
            profile: preview.profile,
            view: preview.view,
        })
    }

    /// Deterministic origin-removal policy: the preview stays only while its
    /// source tab is the active tab of the active space. Callers run this
    /// **after** every mutation that can move the active tab/space away from
    /// the source (select, close, move, space switch, split change, reopen,
    /// new tab); the preview session is destroyed and the state released.
    /// Returns the `PreviewClosed` event when the preview was torn down.
    pub(crate) fn close_preview_if_stale(&mut self) -> Option<ControllerEvent> {
        let preview = self.preview.as_ref()?;
        let valid = self
            .browser
            .tab(preview.source_tab)
            .is_some_and(|state| state.space == self.active_space)
            && self
                .browser
                .space(self.active_space)
                .is_some_and(|space| space.active_tab == Some(preview.source_tab));
        if valid {
            return None;
        }
        self.close_preview().ok()
    }

    /// The read-only preview snapshot (live URL/title/loading/history from
    /// the session's navigation state).
    pub(crate) fn preview_snapshot(&self) -> Option<PreviewSnapshot> {
        let preview = self.preview.as_ref()?;
        let nav = self.nav.get(&preview.view).cloned().unwrap_or_default();
        Some(PreviewSnapshot {
            id: preview.id.0,
            view: preview.view,
            source: preview.source_view,
            profile: preview.profile,
            url: nav.url.clone().unwrap_or_else(|| preview.url.clone()),
            title: nav.title.clone(),
            loading: nav.loading,
            can_go_back: nav.can_go_back,
            can_go_forward: nav.can_go_forward,
            crashed: nav.crashed,
        })
    }

    /// Destroy the preview's disposable session (engine + visibility +
    /// profile + nav state). The origin session is never touched.
    fn destroy_preview_session(&mut self, view: WebViewId) {
        self.engine.borrow_mut().destroy_web_view(view);
        self.visibility.remove(&view);
        self.session_profiles.remove(&view);
        self.nav.remove(&view);
    }

    /// Apply a navigation event for the preview session (it has no tab): the
    /// transient navigation state is mirrored (url/title/loading/history/
    /// crash) and the event is passed through for E2E. Returns `true` when
    /// the event named the open preview session.
    pub(crate) fn apply_preview_event(&mut self, event: &WebEvent) -> bool {
        let view = match event {
            WebEvent::UrlChanged(v, _)
            | WebEvent::TitleChanged(v, _)
            | WebEvent::LoadingChanged(v, _)
            | WebEvent::CanGoBackChanged(v, _)
            | WebEvent::CanGoForwardChanged(v, _)
            | WebEvent::DidCommitNavigation(v)
            | WebEvent::PageCrashed(v)
            | WebEvent::NavigationFailed(v, _)
            | WebEvent::UnresponsiveChanged(v, _)
            | WebEvent::ContextMenuRequested(v, _)
            | WebEvent::FaviconChanged(v, _) => *v,
        };
        let is_preview = self
            .preview
            .as_ref()
            .is_some_and(|preview| preview.view == view);
        if !is_preview {
            return false;
        }
        self.nav.entry(view).or_default().apply(event);
        self.check_invariants();
        true
    }
}
