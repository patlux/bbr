//! Internal session policy on the [`super::BrowserController`]: the
//! registry/engine reconciliation (`sync_views`), per-tab session
//! create/destroy across profiles, and the focused-tab/index resolution
//! helpers. Split from `controller.rs` so the gateway module stays under
//! the ~500-line budget (the same shape as the other `controller/<concern>`
//! modules).

use bbr_domain::{TabId, TabLifecycle};
use bbr_web::{ProfileId, WebViewId};

use crate::ControllerError;

use super::BrowserController;

impl BrowserController {
    /// Reconcile engine state with the domain: live sessions for every
    /// Active/Warm tab in its profile, destruction for Suspended/Discarded
    /// tabs, and show/hide matching domain visibility.
    ///
    /// Domain order is used everywhere so the recorded engine op log stays
    /// deterministic (map iteration order is not).
    pub(crate) fn sync_views(&mut self) {
        // Blueprint D19: what the other windows select and show.
        let elsewhere = self.reconcile_windows();
        let tabs: Vec<TabId> = self.browser.tabs().map(|t| t.id).collect();
        for tab in tabs.iter().copied() {
            let _ = self.ensure_session(tab);
        }
        let current = self.windows.current;
        let here: Vec<TabId> = self
            .browser
            .space(self.active_space)
            .map(|space| space.visible_tabs())
            .unwrap_or_default();
        for tab in tabs {
            let Some(view) = self.registry.view_for_tab(tab) else {
                continue;
            };
            let holder = if here.contains(&tab) {
                Some(current)
            } else {
                elsewhere.get(&tab).copied()
            };
            let want = holder.is_some();
            // The session sits in the root of the window that shows it
            // (re-parented, never reloaded, when it moves).
            if let Some(window) = holder {
                let placed = self
                    .windows
                    .placement
                    .get(&view)
                    .copied()
                    .unwrap_or_default();
                if placed != window {
                    self.engine.borrow_mut().place_web_view(view, window.0);
                    self.windows.placement.insert(view, window);
                }
            }
            if self.visibility.get(&view) != Some(&want) {
                self.engine.borrow_mut().show(view, want);
                self.visibility.insert(view, want);
            }
        }
        // Arm the bounded liveness probes for the visible sessions (a
        // freshly shown session is probed immediately; the transitions are
        // observable through the snapshot — the app's cadence tick and the
        // `UnresponsiveChanged` event path report them as observations).
        let _ = self.pump_responsiveness();
    }

    /// Create or destroy a tab's session to match its domain lifecycle and
    /// profile. Returns the tab's live view, if any.
    fn ensure_session(&mut self, tab: TabId) -> Result<Option<WebViewId>, ControllerError> {
        let info = {
            let state = self.browser.tab(tab).ok_or(ControllerError::Domain(
                bbr_domain::DomainError::UnknownTab(tab),
            ))?;
            let profile = self
                .browser
                .space(state.space)
                .expect("table tab's space exists")
                .profile;
            (state.lifecycle, state.url.clone(), profile)
        };
        let (lifecycle, url, profile) = info;
        match lifecycle {
            TabLifecycle::Suspended | TabLifecycle::Discarded => {
                self.destroy_session_of(tab);
                Ok(None)
            }
            TabLifecycle::Active | TabLifecycle::Warm => {
                if let Some(view) = self.registry.view_for_tab(tab) {
                    if self.session_profiles.get(&view) != Some(&profile) {
                        // Cross-profile move: destroy and recreate in the
                        // destination profile's store.
                        self.destroy_session_of(tab);
                        let view = self.create_session(tab, profile, &url);
                        return Ok(Some(view));
                    }
                    Ok(Some(view))
                } else {
                    let view = self.create_session(tab, profile, &url);
                    Ok(Some(view))
                }
            }
        }
    }

    /// Allocate a fresh session for `tab`: engine create + registry mapping.
    /// A fresh session clears the tab's crash flag (the recovered page is no
    /// longer in the §10.7 error state) and registers a fresh availability
    /// record (a stale completion for the previous incarnation is dropped).
    pub(crate) fn create_session(
        &mut self,
        tab: TabId,
        profile: ProfileId,
        url: &str,
    ) -> WebViewId {
        self.crashed_tabs.remove(&tab);
        let view = WebViewId((self.allocate_view)());
        self.engine.borrow_mut().create_web_view(view, profile, url);
        self.visibility.insert(view, false);
        // Blueprint D19: a new session starts in the current window's root.
        let current = self.windows.current;
        if current != crate::WindowKey::default() {
            self.engine.borrow_mut().place_web_view(view, current.0);
        }
        self.windows.placement.insert(view, current);
        self.session_profiles.insert(view, profile);
        self.registry
            .insert(tab, view)
            .expect("freshly allocated web view id cannot collide");
        self.responsiveness.register(view);
        view
    }

    /// Destroy a tab's session if it has one (engine + registry + nav state).
    pub(crate) fn destroy_session_of(&mut self, tab: TabId) {
        if let Some(view) = self.registry.remove_tab(tab) {
            self.engine.borrow_mut().destroy_web_view(view);
            self.visibility.remove(&view);
            self.windows.placement.remove(&view);
            self.session_profiles.remove(&view);
            self.nav.remove(&view);
            // The session's availability record (and any in-flight probe or
            // pending deadline) is dropped with the session.
            self.responsiveness.forget(view);
        }
    }

    /// The focused navigation target: the active tab, which is also the
    /// focused pane while a split is open.
    pub(crate) fn focused_tab(&self) -> Option<TabId> {
        self.browser
            .space(self.active_space)
            .and_then(|s| s.active_tab)
    }

    /// Resolve a space-local tab index to its [`TabId`].
    pub(crate) fn tab_at(&self, index: usize) -> Result<TabId, ControllerError> {
        let space = self
            .browser
            .space(self.active_space)
            .ok_or(ControllerError::NoActiveSpace)?;
        space
            .tabs
            .get(index)
            .copied()
            .ok_or(ControllerError::IndexOutOfBounds {
                index,
                len: space.tabs.len(),
            })
    }

    pub(crate) fn index_of(&self, tab: TabId) -> Option<usize> {
        self.browser
            .space(self.active_space)
            .and_then(|s| s.tabs.iter().position(|t| *t == tab))
    }
}
