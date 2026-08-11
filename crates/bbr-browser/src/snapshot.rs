//! Read-only snapshots of controller state for the UI and E2E harness.

use bbr_domain::{ProfileHue, SpaceId, TabId, TabLifecycle};
use bbr_web::{ProfileId, WebViewId};

use crate::nav::NavState;

/// Per-tab lifecycle counts of the active model (the dense-list session
/// budget evidence): `active + warm` is the live-session population and must
/// never exceed the enforced budget; `suspended + discarded` own no session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LifecycleCounts {
    pub active: usize,
    pub warm: usize,
    pub suspended: usize,
    pub discarded: usize,
}

impl LifecycleCounts {
    /// Tally the lifecycle of every tab in `tabs`.
    pub fn tally(tabs: &[TabSnapshot]) -> Self {
        let mut counts = Self::default();
        for tab in tabs {
            match tab.lifecycle {
                TabLifecycle::Active => counts.active += 1,
                TabLifecycle::Warm => counts.warm += 1,
                TabLifecycle::Suspended => counts.suspended += 1,
                TabLifecycle::Discarded => counts.discarded += 1,
            }
        }
        counts
    }

    /// The number of tabs that own a live session.
    pub fn live(self) -> usize {
        self.active + self.warm
    }
}

/// One tab of the active space as presented to the UI/E2E.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabSnapshot {
    /// Position in the space's tab list.
    pub index: usize,
    pub url: String,
    pub title: String,
    /// The pinned region occupies the leading `pinned` tabs of the space.
    pub pinned: bool,
    /// Live-session loading flag (url/title/loading live on the session).
    pub loading: bool,
    /// The tab's web process terminated; its identity survives until a fresh
    /// session loads (DESIGN.md §10.7 error state).
    pub crashed: bool,
    /// Whether this tab is the active one.
    pub active: bool,
    /// The session's renderer is reported unresponsive (renderer-isolation
    /// lane; DESIGN.md §10.11/§11.8a): a sibling of `crashed` — the session
    /// and tab stay alive under the bounded recovery surface.
    pub unresponsive: bool,
    /// Current disposable session identity, absent when suspended/discarded.
    pub web_view: Option<WebViewId>,
    /// Resource policy state of the tab's disposable session (the dense-list
    /// lifecycle contract).
    pub lifecycle: TabLifecycle,
    /// Deterministic recency stamp (the session-budget policy's "recently
    /// used" order).
    pub last_used: u64,
}

/// One persistent tab identity across the complete space collection.
///
/// This compact read model supports command/search composition without
/// exposing the mutable domain model or duplicating session ownership.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalTabSnapshot {
    pub id: TabId,
    pub space: SpaceId,
    pub profile: ProfileId,
    /// Position in the owning space's tab list.
    pub index: usize,
    pub url: String,
    pub title: String,
    pub pinned: bool,
    pub web_view: Option<WebViewId>,
    pub lifecycle: TabLifecycle,
    pub last_used: u64,
}

/// One profile's identity as presented to the UI/E2E (DESIGN.md §12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSnapshot {
    /// Position in creation order — the stable `bbr-profile-chip-<n>` identity.
    pub index: usize,
    pub id: ProfileId,
    pub title: String,
    /// Stable round-robin identity hue (DESIGN.md §12).
    pub hue: ProfileHue,
}

/// One space as presented to the UI/E2E, including its own active tab and
/// live session — the "each space remembers its active tab" contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceSnapshot {
    /// Position in creation order — the collection-order key (the chip ids
    /// derive from the persistent [`SpaceId`], never this index).
    pub index: usize,
    pub id: SpaceId,
    pub profile: ProfileId,
    pub title: String,
    /// The space's own selected tab (restored on activation), space-local.
    pub active_tab: Option<usize>,
    /// Live session of the space's active tab (`None` when empty/suspended).
    pub active_view: Option<WebViewId>,
    /// Whether this is the controller's active space.
    pub active: bool,
    /// Tab count for the §11.3 count column.
    pub tab_count: usize,
}

/// A snapshot of the controller's observable state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControllerSnapshot {
    /// The profile and space all tab operations currently target.
    pub active_profile: ProfileId,
    /// Display title of the active profile (DESIGN.md §11.8 pane naming).
    pub active_profile_title: Option<String>,
    /// The active profile's identity hue (DESIGN.md §12).
    pub active_profile_hue: Option<ProfileHue>,
    pub active_space: SpaceId,
    /// Tabs of the active space, in order.
    pub tabs: Vec<TabSnapshot>,
    /// Complete tab collection in space order, then tab order. This preserves
    /// persistent identity for cross-space command/search composition without
    /// exposing the mutable domain model.
    pub all_tabs: Vec<GlobalTabSnapshot>,
    /// Complete profile collection in creation order.
    pub profiles: Vec<ProfileSnapshot>,
    /// Complete space collection in creation order.
    pub spaces: Vec<SpaceSnapshot>,
    /// Index of the active tab, if any.
    pub active_index: Option<usize>,
    /// Navigation state of the active session (mirrors the active tab).
    pub nav: NavState,
    /// Whether the active tab is backed by a hosted web view.
    pub web_view_hosted: bool,
    /// The engine's live-session count (native destruction evidence). The
    /// registry invariant says it equals the number of live-tab sessions the
    /// controller has registered.
    pub engine_sessions: usize,
    /// Active session id (when a session backs the active tab).
    pub active_view: Option<WebViewId>,
    /// Lifecycle population counts of the active space (the dense-list
    /// session-budget evidence).
    pub lifecycle_counts: LifecycleCounts,
    /// The transient Peek preview (Link Preview / Peek packet): `None` at
    /// rest. The preview is transient controller state, never a persisted
    /// `Tab`/recently-closed/history row.
    pub preview: Option<crate::peek::PreviewSnapshot>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_is_a_plain_value() {
        let s = ControllerSnapshot {
            active_profile: ProfileId(1),
            active_profile_title: Some("Default".into()),
            active_profile_hue: Some(ProfileHue::Cobalt),
            active_space: SpaceId(2),
            tabs: Vec::new(),
            all_tabs: Vec::new(),
            profiles: Vec::new(),
            spaces: Vec::new(),
            active_index: None,
            nav: NavState::default(),
            web_view_hosted: false,
            engine_sessions: 0,
            active_view: None,
            lifecycle_counts: LifecycleCounts::default(),
            preview: None,
        };
        assert_eq!(s.clone(), s);
    }
}
