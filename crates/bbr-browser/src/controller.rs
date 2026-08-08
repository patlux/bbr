//! The portable [`BrowserController`]: owns the domain [`Browser`], maps
//! every live tab to exactly one disposable web session, routes navigation
//! commands to the focused tab or split pane, and applies engine
//! [`WebEvent`]s to the owning tab.
//!
//! Ownership shape (see `lib.rs`): the controller drives the engine through
//! `Rc<RefCell<dyn WebEngine>>` and allocates `WebViewId`s through an
//! injected generator, so E2E runs can rely on deterministic sequential ids.
//!
//! Session policy (the crate's core invariant, also enforced by
//! [`SessionRegistry`]):
//!
//! - Every `Active`/`Warm` tab owns exactly one live session in its profile.
//! - `Suspended`/`Discarded` tabs own no session; activation recreates one.
//! - Closing, suspending, or crashing a tab destroys its session.
//! - Moving across profiles destroys and recreates in the destination profile.
//! - Only visible tabs (single-pane selection or split panes) are shown.

use std::collections::HashMap;

use bbr_domain::{Browser, TabId, TabLifecycle};
use bbr_web::{ProfileId, WebAction, WebEvent, WebViewId};

use crate::nav::NavState;
use crate::outcome::ControllerEvent;
use crate::pane::PaneLayout;
use crate::session::SessionRegistry;
use crate::{ControllerError, EngineRef, ViewIdAllocator};

/// Platform-neutral owner of the browser model and its web sessions.
pub struct BrowserController {
    pub(crate) engine: EngineRef,
    pub(crate) browser: Browser,
    pub(crate) registry: SessionRegistry,
    /// Fresh `WebViewId` allocation (injected so E2E is deterministic).
    pub(crate) allocate_view: ViewIdAllocator,
    /// The space all tab operations currently target.
    pub(crate) active_space: bbr_domain::SpaceId,
    /// Profile each live session was created in (cross-profile move policy).
    pub(crate) session_profiles: HashMap<WebViewId, ProfileId>,
    /// Desired engine visibility per live session (avoids redundant shows).
    pub(crate) visibility: HashMap<WebViewId, bool>,
    /// Live-session navigation state (url/title/loading/history/crashed),
    /// reset whenever a session is recreated.
    pub(crate) nav: HashMap<WebViewId, NavState>,
    /// Tabs whose web process terminated. Survives the session destruction
    /// (the crash destroys the session but the tab identity stays); cleared
    /// when a fresh session is created or loads.
    pub(crate) crashed_tabs: std::collections::HashSet<TabId>,
}

impl std::fmt::Debug for BrowserController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrowserController")
            .field("active_space", &self.active_space)
            .field("live_sessions", &self.registry.live_sessions())
            .field("tabs", &self.browser.tabs().count())
            .finish_non_exhaustive()
    }
}

impl BrowserController {
    /// Create the controller and its initial profile + space through setup.
    pub fn new(engine: EngineRef, allocate_view: ViewIdAllocator) -> Self {
        let mut browser = Browser::new();
        let profile = browser.create_profile("Default");
        let space = browser
            .create_space(profile, "Default")
            .expect("fresh profile exists");
        Self {
            engine,
            browser,
            registry: SessionRegistry::new(),
            allocate_view,
            active_space: space,
            session_profiles: HashMap::new(),
            visibility: HashMap::new(),
            nav: HashMap::new(),
            crashed_tabs: std::collections::HashSet::new(),
        }
    }

    // ---- Setup helpers (profile/space creation for multi-profile tests) ----

    pub fn create_profile(&mut self, title: impl Into<String>) -> ProfileId {
        self.browser.create_profile(title)
    }

    /// Create a profile with one initial empty space, ready for activation.
    pub fn create_profile_space(
        &mut self,
        profile_title: impl Into<String>,
        space_title: impl Into<String>,
    ) -> Result<ControllerEvent, ControllerError> {
        let profile = self.browser.create_profile(profile_title);
        let space = self.browser.create_space(profile, space_title)?;
        self.check_invariants();
        Ok(ControllerEvent::ProfileCreated { profile, space })
    }

    pub fn create_space(
        &mut self,
        profile: ProfileId,
        title: impl Into<String>,
    ) -> Result<bbr_domain::SpaceId, ControllerError> {
        Ok(self.browser.create_space(profile, title)?)
    }

    /// Create another space in an existing profile with a typed outcome.
    pub fn create_space_event(
        &mut self,
        profile: ProfileId,
        title: impl Into<String>,
    ) -> Result<ControllerEvent, ControllerError> {
        let space = self.browser.create_space(profile, title)?;
        self.check_invariants();
        Ok(ControllerEvent::SpaceCreated { profile, space })
    }

    /// Switch the workspace whose tabs are visible and receive commands.
    pub fn activate_space(
        &mut self,
        space: bbr_domain::SpaceId,
    ) -> Result<ControllerEvent, ControllerError> {
        if self.browser.space(space).is_none() {
            return Err(ControllerError::UnknownSpace(space));
        }
        self.active_space = space;
        self.sync_views();
        self.check_invariants();
        Ok(ControllerEvent::SpaceActivated { space })
    }

    /// The profile backing the active space.
    pub fn active_profile(&self) -> ProfileId {
        self.browser
            .space(self.active_space)
            .expect("active space exists")
            .profile
    }

    // ---- First vertical slice ---------------------------------------------

    /// Open a tab in the active space: domain tab + fresh session + visible.
    pub fn open_tab(&mut self, url: impl Into<String>) -> Result<ControllerEvent, ControllerError> {
        let url = url.into();
        let tab = self.browser.open_tab(self.active_space, "", url)?;
        self.sync_views();
        let index = self.index_of(tab).expect("tab was just opened");
        let url = self
            .browser
            .tab(tab)
            .map(|t| t.url.clone())
            .unwrap_or_default();
        self.check_invariants();
        Ok(ControllerEvent::TabOpened { index, url })
    }

    /// Select the tab at `index` in the active space: previous session hidden
    /// and warm, selected session shown (recreated when suspended/discarded).
    pub fn select_tab(&mut self, index: usize) -> Result<ControllerEvent, ControllerError> {
        let tab = self.tab_at(index)?;
        self.browser.select_tab(self.active_space, tab)?;
        self.sync_views();
        self.check_invariants();
        Ok(ControllerEvent::TabSelected { index })
    }

    /// Close the tab at `index`: session destroyed, domain neighbor activated
    /// (recreating its session when needed and shown).
    pub fn close_tab(&mut self, index: usize) -> Result<ControllerEvent, ControllerError> {
        let tab = self.tab_at(index)?;
        self.destroy_session_of(tab);
        self.browser.close_tab(self.active_space, tab)?;
        self.sync_views();
        self.check_invariants();
        Ok(ControllerEvent::TabClosed { index })
    }

    /// Reopen the most recently closed tab with its preserved URL/title
    /// identity; a new session is created and shown.
    pub fn reopen_tab(&mut self) -> Result<ControllerEvent, ControllerError> {
        let tab = self.browser.reopen_tab(self.active_space)?;
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

    /// Pin the tab at `index` (moves it into the pinned region) and keep it
    /// active; the live session is preserved.
    pub fn pin_tab(&mut self, index: usize) -> Result<ControllerEvent, ControllerError> {
        let tab = self.tab_at(index)?;
        self.browser.pin_tab(self.active_space, tab)?;
        self.sync_views();
        self.check_invariants();
        let index = self.index_of(tab).expect("tab stays in the space");
        Ok(ControllerEvent::TabPinned { index })
    }

    /// Unpin the tab at `index` (moves it to the end of the regular region)
    /// and keep it active; the live session is preserved.
    pub fn unpin_tab(&mut self, index: usize) -> Result<ControllerEvent, ControllerError> {
        let tab = self.tab_at(index)?;
        self.browser.unpin_tab(self.active_space, tab)?;
        self.sync_views();
        self.check_invariants();
        let index = self.index_of(tab).expect("tab stays in the space");
        Ok(ControllerEvent::TabUnpinned { index })
    }

    /// Reorder the tab at `from` to `to` within the active space. The domain
    /// rejects a move across the pinned/regular boundary and out-of-range
    /// indices; the selected tab stays selected and the session is preserved.
    pub fn reorder_tab(
        &mut self,
        from: usize,
        to: usize,
    ) -> Result<ControllerEvent, ControllerError> {
        self.browser.reorder_tab(self.active_space, from, to)?;
        self.sync_views();
        self.check_invariants();
        Ok(ControllerEvent::TabReordered { from, to })
    }

    /// Route one navigation action to the focused tab (the active tab, or the
    /// focused split pane). A suspended/discarded focused tab is reactivated
    /// first, recreating its session.
    pub fn navigate(&mut self, action: WebAction) -> Result<ControllerEvent, ControllerError> {
        let tab = self.focused_tab().ok_or(ControllerError::NoActiveTab)?;
        let needs_reactivate = match self.browser.tab(tab) {
            Some(state) => matches!(
                state.lifecycle,
                TabLifecycle::Suspended | TabLifecycle::Discarded
            ),
            None => {
                return Err(ControllerError::Domain(
                    bbr_domain::DomainError::UnknownTab(tab),
                ))
            }
        };
        if needs_reactivate {
            self.browser.select_tab(self.active_space, tab)?;
        }
        self.sync_views();
        let view = self
            .registry
            .view_for_tab(tab)
            .ok_or(ControllerError::NoSession(tab))?;
        self.engine.borrow_mut().action(view, action);
        self.check_invariants();
        Ok(ControllerEvent::NavigationRouted { view })
    }

    // ---- Lifecycle and moves ----------------------------------------------

    /// Suspend a hidden tab: session destroyed, identity kept suspended.
    pub fn suspend_tab(&mut self, index: usize) -> Result<ControllerEvent, ControllerError> {
        let tab = self.tab_at(index)?;
        self.browser
            .transition_tab_lifecycle(tab, TabLifecycle::Suspended)?;
        self.sync_views();
        self.check_invariants();
        Ok(ControllerEvent::TabSuspended { index })
    }

    /// Deterministic live-session budget for the active space (the dense-list
    /// session cap). Keeps visible, pinned, and recently used identities live
    /// and suspends the least recently used warm tabs so the total
    /// Active + Warm population never exceeds `cap`; the suspended sessions
    /// are destroyed. A no-op when the population already fits.
    pub fn enforce_session_budget(
        &mut self,
        cap: usize,
    ) -> Result<ControllerEvent, ControllerError> {
        let suspended = self.browser.suspend_least_recently_used(cap);
        self.sync_views();
        self.check_invariants();
        let snapshot = self.snapshot();
        Ok(ControllerEvent::LifecycleBudgetApplied {
            live: snapshot.lifecycle_counts.live(),
            suspended: suspended.len(),
        })
    }

    /// Move the tab at `index` into `to_space`. Within one profile the live
    /// session is kept; across profiles it is destroyed and recreated in the
    /// destination profile's store.
    pub fn move_tab(
        &mut self,
        index: usize,
        to_space: bbr_domain::SpaceId,
    ) -> Result<ControllerEvent, ControllerError> {
        let tab = self.tab_at(index)?;
        let from_profile = self.active_profile();
        let to_profile = self
            .browser
            .space(to_space)
            .ok_or(ControllerError::UnknownSpace(to_space))?
            .profile;
        self.browser.move_tab(tab, to_space)?;
        self.sync_views();
        self.check_invariants();
        Ok(ControllerEvent::TabMoved {
            tab,
            to_space,
            recreated: from_profile != to_profile,
        })
    }

    /// Handle a WebKit process termination: mark the owning tab discarded and
    /// destroy its session; the next activation recreates it. The tab's crash
    /// flag survives so the shell can render the §10.7 error state.
    fn on_page_crash(&mut self, view: WebViewId) -> ControllerEvent {
        let Some(tab) = self.registry.tab_for_view(view) else {
            return ControllerEvent::UnknownWebView { view };
        };
        let _ = self.browser.discard_tab(tab);
        self.destroy_session_of(tab);
        self.crashed_tabs.insert(tab);
        self.check_invariants();
        ControllerEvent::TabDiscarded { tab }
    }

    // ---- Split behavior ---------------------------------------------------

    /// Open a two-pane split over the two given tab indices: both pane
    /// sessions are activated and shown.
    pub fn open_split(
        &mut self,
        primary_index: usize,
        secondary_index: usize,
    ) -> Result<ControllerEvent, ControllerError> {
        let primary = self.tab_at(primary_index)?;
        let secondary = self.tab_at(secondary_index)?;
        self.browser
            .set_split(self.active_space, primary, secondary)?;
        self.sync_views();
        self.check_invariants();
        Ok(ControllerEvent::SplitOpened {
            primary: primary_index,
            secondary: secondary_index,
        })
    }

    /// Close the active space's split: the non-selected pane is hidden and
    /// warmed without closing its tab; the focused pane keeps its session.
    pub fn close_split(&mut self) -> Result<ControllerEvent, ControllerError> {
        self.browser.close_split(self.active_space)?;
        self.sync_views();
        self.check_invariants();
        Ok(ControllerEvent::SplitClosed)
    }

    // ---- Event application ------------------------------------------------

    /// Apply one engine event to its owning tab. Events for unknown/stale
    /// `WebViewId`s are reported without corrupting any state.
    pub fn apply_event(&mut self, event: WebEvent) -> ControllerEvent {
        let view = event_view(&event);
        let Some(tab) = self.registry.tab_for_view(view) else {
            return ControllerEvent::UnknownWebView { view };
        };
        match &event {
            WebEvent::PageCrashed(_) => return self.on_page_crash(view),
            WebEvent::UrlChanged(_, url) => {
                let _ = self.browser.apply_navigation(tab, url, "");
            }
            WebEvent::TitleChanged(_, title) => {
                let _ = self.browser.apply_navigation(tab, "", title);
            }
            _ => {}
        }
        // Loading/history/crashed live only in the session's nav state.
        self.nav.entry(view).or_default().apply(&event);
        self.check_invariants();
        ControllerEvent::WebEvent { event }
    }

    /// The authoritative pane layout of the active space: the native views
    /// the shell must frame and the focused pane (the browser command
    /// target). Derived only from the domain split, the active tab, and the
    /// session registry — the app owns no independent focus value.
    ///
    /// - No active tab or no live session → `None`.
    /// - Single pane → `primary` is the active tab's view, no `secondary`.
    /// - Split → `primary`/`secondary` follow the domain split order and
    ///   `focused` is the active tab's view (a split member by domain
    ///   invariant).
    pub fn pane_layout(&self) -> Option<PaneLayout> {
        let space = self.browser.space(self.active_space)?;
        let focused = space
            .active_tab
            .and_then(|tab| self.registry.view_for_tab(tab))?;
        match space.split {
            Some(split) => Some(PaneLayout {
                primary: self.registry.view_for_tab(split.primary)?,
                // A split must own **both** live sessions: a missing
                // secondary never degrades the split into a false single
                // pane — the honest answer is no pane layout at all.
                secondary: Some(self.registry.view_for_tab(split.secondary)?),
                focused,
            }),
            None => Some(PaneLayout {
                primary: focused,
                secondary: None,
                focused,
            }),
        }
    }

    // ---- Snapshots --------------------------------------------------------

    /// Read-only snapshot of the controller's observable state.
    pub fn snapshot(&self) -> crate::snapshot::ControllerSnapshot {
        let tabs: Vec<bbr_domain::TabId> = self
            .browser
            .space(self.active_space)
            .map(|s| s.tabs.clone())
            .unwrap_or_default();
        let active_tab = self
            .browser
            .space(self.active_space)
            .and_then(|s| s.active_tab);
        let tab_snapshots: Vec<crate::snapshot::TabSnapshot> = tabs
            .iter()
            .enumerate()
            .filter_map(|(index, &tab)| {
                let state = self.browser.tab(tab)?;
                let view = self.registry.view_for_tab(tab);
                let session = view.and_then(|v| self.nav.get(&v));
                Some(crate::snapshot::TabSnapshot {
                    index,
                    url: state.url.clone(),
                    title: state.title.clone(),
                    pinned: state.pinned,
                    loading: session.is_some_and(|n| n.loading),
                    crashed: self.crashed_tabs.contains(&tab),
                    active: active_tab == Some(tab),
                    web_view: view,
                    lifecycle: state.lifecycle,
                    last_used: state.last_used,
                })
            })
            .collect();
        let all_tab_snapshots: Vec<crate::snapshot::GlobalTabSnapshot> = self
            .browser
            .spaces()
            .flat_map(|space| {
                space
                    .tabs
                    .iter()
                    .copied()
                    .enumerate()
                    .filter_map(|(index, tab)| {
                        let state = self.browser.tab(tab)?;
                        Some(crate::snapshot::GlobalTabSnapshot {
                            id: tab,
                            space: space.id,
                            profile: space.profile,
                            index,
                            url: state.url.clone(),
                            title: state.title.clone(),
                            pinned: state.pinned,
                            web_view: self.registry.view_for_tab(tab),
                            lifecycle: state.lifecycle,
                            last_used: state.last_used,
                        })
                    })
            })
            .collect();
        let active_index = active_tab.and_then(|t| tabs.iter().position(|x| *x == t));
        let active_view = active_tab.and_then(|t| self.registry.view_for_tab(t));
        let nav = active_view
            .and_then(|v| self.nav.get(&v).cloned())
            .unwrap_or_default();
        let profiles = self
            .browser
            .profiles()
            .enumerate()
            .map(|(index, p)| crate::snapshot::ProfileSnapshot {
                index,
                id: p.id,
                title: p.title.clone(),
                hue: p.hue,
            })
            .collect();
        let spaces = self
            .browser
            .spaces()
            .enumerate()
            .map(|(index, s)| {
                let active_tab = s
                    .active_tab
                    .and_then(|t| s.tabs.iter().position(|x| *x == t));
                crate::snapshot::SpaceSnapshot {
                    index,
                    id: s.id,
                    profile: s.profile,
                    title: s.title.clone(),
                    active_tab,
                    active_view: s.active_tab.and_then(|t| self.registry.view_for_tab(t)),
                    active: s.id == self.active_space,
                    tab_count: s.tabs.len(),
                }
            })
            .collect();
        let active_profile = self.active_profile();
        let lifecycle_counts = crate::snapshot::LifecycleCounts::tally(&tab_snapshots);
        crate::snapshot::ControllerSnapshot {
            active_profile,
            active_profile_title: self
                .browser
                .profile(active_profile)
                .map(|p| p.title.clone()),
            active_profile_hue: self.browser.profile(active_profile).map(|p| p.hue),
            active_space: self.active_space,
            tabs: tab_snapshots,
            all_tabs: all_tab_snapshots,
            profiles,
            spaces,
            active_index,
            nav,
            web_view_hosted: active_view.is_some(),
            engine_sessions: self.engine.borrow().live_session_count(),
            active_view,
            lifecycle_counts,
        }
    }

    // ---- Internal session policy -------------------------------------------

    /// Reconcile engine state with the domain: live sessions for every
    /// Active/Warm tab in its profile, destruction for Suspended/Discarded
    /// tabs, and show/hide matching domain visibility.
    ///
    /// Domain order is used everywhere so the recorded engine op log stays
    /// deterministic (map iteration order is not).
    fn sync_views(&mut self) {
        let tabs: Vec<TabId> = self.browser.tabs().map(|t| t.id).collect();
        for tab in tabs.iter().copied() {
            let _ = self.ensure_session(tab);
        }
        for tab in tabs {
            let Some(view) = self.registry.view_for_tab(tab) else {
                continue;
            };
            let in_active_space = self
                .browser
                .tab(tab)
                .is_some_and(|state| state.space == self.active_space);
            let want = in_active_space && self.browser.is_visible(tab);
            if self.visibility.get(&view) != Some(&want) {
                self.engine.borrow_mut().show(view, want);
                self.visibility.insert(view, want);
            }
        }
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
    /// longer in the §10.7 error state).
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
        self.session_profiles.insert(view, profile);
        self.registry
            .insert(tab, view)
            .expect("freshly allocated web view id cannot collide");
        view
    }

    /// Destroy a tab's session if it has one (engine + registry + nav state).
    pub(crate) fn destroy_session_of(&mut self, tab: TabId) {
        if let Some(view) = self.registry.remove_tab(tab) {
            self.engine.borrow_mut().destroy_web_view(view);
            self.visibility.remove(&view);
            self.session_profiles.remove(&view);
            self.nav.remove(&view);
        }
    }

    /// The focused navigation target: the active tab, which is also the
    /// focused pane while a split is open.
    fn focused_tab(&self) -> Option<TabId> {
        self.browser
            .space(self.active_space)
            .and_then(|s| s.active_tab)
    }

    /// Resolve a space-local tab index to its [`TabId`].
    fn tab_at(&self, index: usize) -> Result<TabId, ControllerError> {
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

    fn index_of(&self, tab: TabId) -> Option<usize> {
        self.browser
            .space(self.active_space)
            .and_then(|s| s.tabs.iter().position(|t| *t == tab))
    }

    /// Registry + engine structural agreement after every mutation.
    pub(crate) fn check_invariants(&self) {
        self.registry
            .check_invariants()
            .expect("registry bijection");
        assert_eq!(
            self.registry.live_sessions(),
            self.engine.borrow().live_session_count(),
            "registry and engine must agree on live sessions"
        );
    }
}

/// The [`WebViewId`] an event names.
fn event_view(event: &WebEvent) -> WebViewId {
    match event {
        WebEvent::UrlChanged(v, _) => *v,
        WebEvent::TitleChanged(v, _) => *v,
        WebEvent::LoadingChanged(v, _) => *v,
        WebEvent::CanGoBackChanged(v, _) => *v,
        WebEvent::CanGoForwardChanged(v, _) => *v,
        WebEvent::PageCrashed(v) => *v,
        WebEvent::NavigationFailed(v, _) => *v,
        // A context-menu request names the view it was requested on (the
        // menu is BBR chrome over that view; the request carries no
        // navigation state).
        WebEvent::ContextMenuRequested(v, _) => *v,
    }
}
