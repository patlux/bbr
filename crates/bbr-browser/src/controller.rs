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

use bbr_domain::{Browser, SpaceAppearanceStore, TabId, TabLifecycle};
use bbr_web::{ProfileId, WebAction, WebViewId};

mod appearance;
mod closed_tabs;
mod events;
mod favicon;
mod gesture;
mod history;
mod new_window;
mod responsiveness;
mod session;
mod setup;
mod snapshot;
mod split_groups;
mod windows;

pub use closed_tabs::ClosedTabEntry;
pub use windows::WindowKey;

use crate::nav::NavState;
use crate::outcome::ControllerEvent;
use crate::peek::PeekPreview;
use crate::provenance::OpenerProvenance;
use crate::responsiveness::ResponsivenessMonitor;
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
    /// Per-space appearance records beside the browser (the orchestration
    /// layer owns both, per the space-appearance store contract). Effective
    /// appearance derives deterministic defaults from the owning profile's
    /// identity hue when no explicit record exists.
    pub(crate) appearances: SpaceAppearanceStore,
    /// Session-local regular child-tab opener provenance (identity-based;
    /// recorded only by the productive new-window regular-tab path and
    /// invalidated when either tab closes). Drives the Back-at-root
    /// opener return decision matrix.
    pub(crate) provenance: OpenerProvenance,
    /// The one transient Peek preview; never a persisted tab or provenance row.
    pub(crate) preview: Option<PeekPreview>,
    /// Fresh deterministic transient preview identity allocation.
    pub(crate) next_preview_id: u64,
    /// The portable renderer-availability monitor (renderer-isolation lane):
    /// per-session availability, the bounded liveness-probe lifecycle, and
    /// the fail-closed deadline/backoff policy.
    pub(crate) responsiveness: ResponsivenessMonitor,
    /// The bounded per-profile, host-keyed favicon record (task packet
    /// `real-favicon-pipeline.md`): one acquisition, every presentation
    /// (tab rows, Favorites tiles, folder items derive from this same
    /// store).
    pub(crate) favicons: crate::favicon::FaviconStore,
    /// Blueprint D19: the open windows' contexts (see `controller/windows`).
    pub(crate) windows: windows::Windows,
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
            appearances: SpaceAppearanceStore::new(),
            provenance: OpenerProvenance::new(),
            preview: None,
            next_preview_id: 0,
            responsiveness: ResponsivenessMonitor::new(),
            favicons: crate::favicon::FaviconStore::default(),
            windows: windows::Windows::default(),
        }
    }

    // ---- First vertical slice ---------------------------------------------

    /// Open a tab in the active space: domain tab + fresh session + visible.
    pub fn open_tab(&mut self, url: impl Into<String>) -> Result<ControllerEvent, ControllerError> {
        let url = url.into();
        let tab = self.browser.open_tab(self.active_space, "", url)?;
        self.close_preview_if_stale();
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
        self.close_preview_if_stale();
        self.sync_views();
        self.check_invariants();
        Ok(ControllerEvent::TabSelected { index })
    }

    /// Close the tab at `index`: session destroyed, domain neighbor activated
    /// (recreating its session when needed and shown).
    pub fn close_tab(&mut self, index: usize) -> Result<ControllerEvent, ControllerError> {
        let tab = self.tab_at(index)?;
        self.destroy_session_of(tab);
        // Closing either member of a provenance pair invalidates the
        // relation (reopen restores the identity with no opener record).
        self.provenance.invalidate_for(tab);
        self.browser.close_tab(self.active_space, tab)?;
        self.close_preview_if_stale();
        self.sync_views();
        self.check_invariants();
        Ok(ControllerEvent::TabClosed { index })
    }

    /// Reopen the most recently closed tab with its preserved URL/title
    /// identity; a new session is created and shown.
    pub fn reopen_tab(&mut self) -> Result<ControllerEvent, ControllerError> {
        let tab = self.browser.reopen_tab(self.active_space)?;
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
        if self.preview.is_some() {
            return self.preview_action(action);
        }
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
        self.move_tab_id(tab, to_space)
    }

    /// Move the tab `tab` (in any space) into `to_space` — the identity form
    /// of [`Self::move_tab`] (blueprint D07: Undo moves a tab back from a
    /// space that is not active). Across profiles the session is recreated
    /// in the destination profile's store.
    pub fn move_tab_id(
        &mut self,
        tab: bbr_domain::TabId,
        to_space: bbr_domain::SpaceId,
    ) -> Result<ControllerEvent, ControllerError> {
        let from_profile = self
            .browser
            .tab(tab)
            .and_then(|t| self.browser.space(t.space))
            .map(|space| space.profile)
            .ok_or(ControllerError::Domain(
                bbr_domain::DomainError::UnknownTab(tab),
            ))?;
        let to_profile = self
            .browser
            .space(to_space)
            .ok_or(ControllerError::UnknownSpace(to_space))?
            .profile;
        self.browser.move_tab(tab, to_space)?;
        self.close_preview_if_stale();
        self.sync_views();
        self.check_invariants();
        Ok(ControllerEvent::TabMoved {
            tab,
            to_space,
            recreated: from_profile != to_profile,
        })
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
        // Any split composition change invalidates the preview's covered
        // single-layout geometry, even when the origin tab stays active.
        if self.preview.is_some() {
            let _ = self.close_preview();
        }
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
        // Closing the split changes the covered pane composition too; do not
        // leave a preview raised over geometry it no longer belongs to.
        if self.preview.is_some() {
            let _ = self.close_preview();
        }
        self.sync_views();
        self.check_invariants();
        Ok(ControllerEvent::SplitClosed)
    }

    /// Registry + engine structural agreement after every mutation.
    pub(crate) fn check_invariants(&self) {
        self.registry
            .check_invariants()
            .expect("registry bijection");
        // The transient preview is deliberately registry-free because it
        // backs no domain tab; still account for its live engine session.
        let preview_extra = usize::from(self.preview.is_some());
        assert_eq!(
            self.registry.live_sessions() + preview_extra,
            self.engine.borrow().live_session_count(),
            "registry + transient preview must agree with engine live sessions"
        );
        // The companion appearance store stays structurally consistent with
        // the live browser: every record references an existing space and
        // each space has at most one record (tampering is detectable).
        self.appearances
            .check_invariants(&self.browser)
            .expect("space-appearance invariants");
    }
}
