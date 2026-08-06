//! Read-only snapshots of controller state for the UI and E2E harness.

use bbr_domain::SpaceId;
use bbr_web::{ProfileId, WebViewId};

use crate::nav::NavState;

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
    /// Current disposable session identity, absent when suspended/discarded.
    pub web_view: Option<WebViewId>,
}

/// A snapshot of the controller's observable state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControllerSnapshot {
    /// The profile and space all tab operations currently target.
    pub active_profile: ProfileId,
    pub active_space: SpaceId,
    /// Tabs of the active space, in order.
    pub tabs: Vec<TabSnapshot>,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_is_a_plain_value() {
        let s = ControllerSnapshot {
            active_profile: ProfileId(1),
            active_space: SpaceId(2),
            tabs: Vec::new(),
            active_index: None,
            nav: NavState::default(),
            web_view_hosted: false,
            engine_sessions: 0,
            active_view: None,
        };
        assert_eq!(s.clone(), s);
    }
}
