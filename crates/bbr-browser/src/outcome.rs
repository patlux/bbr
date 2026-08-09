//! Typed mutation outcomes published by the controller.
//!
//! Every controller mutation returns exactly one [`ControllerEvent`] so the
//! composition root can translate it into UI/E2E events without re-reading
//! state. Semantic navigation events pass through unchanged; unknown/stale
//! [`WebViewId`]s are reported explicitly rather than corrupting state.

use bbr_domain::SpaceId;
use bbr_web::{ProfileId, WebEvent, WebViewId};

/// Outcome of one controller operation (mutation, routing, or event).
#[derive(Debug, Clone, PartialEq)]
pub enum ControllerEvent {
    /// A profile and its initial space were created.
    ProfileCreated { profile: ProfileId, space: SpaceId },
    /// A space was created in an existing profile.
    SpaceCreated { profile: ProfileId, space: SpaceId },
    /// The controller switched its active workspace.
    SpaceActivated { space: SpaceId },
    /// A tab was opened at `index` in the active space.
    TabOpened { index: usize, url: String },
    /// A user-initiated new-window request opened one regular tab in the
    /// source tab's owning space/profile. `index` is source-space-local and
    /// the active space may remain unchanged.
    NewWindowOpened {
        index: usize,
        url: String,
        space: SpaceId,
        profile: ProfileId,
        view: WebViewId,
    },
    /// The tab at `index` became the active tab.
    TabSelected { index: usize },
    /// The tab at `index` was closed (session destroyed).
    TabClosed { index: usize },
    /// The tab at `index` was pinned (moved into the pinned region).
    TabPinned { index: usize },
    /// The tab at `index` was unpinned (moved out of the pinned region).
    TabUnpinned { index: usize },
    /// The tab at `from` was reordered within its region to `to` (same
    /// region only; the pinned-first invariant is enforced by the domain).
    TabReordered { from: usize, to: usize },
    /// The most recently closed tab was reopened at `index`.
    TabReopened { index: usize, url: String },
    /// A tab was suspended (session destroyed, identity kept).
    TabSuspended { index: usize },
    /// The deterministic dense-list session budget was applied: `live` is the
    /// resulting Active + Warm population and `suspended` the number of
    /// sessions destroyed to reach it (0 when already within budget).
    LifecycleBudgetApplied { live: usize, suspended: usize },
    /// A tab's WebKit process terminated; its session was destroyed.
    TabDiscarded { tab: bbr_domain::TabId },
    /// The tab moved into another space; `recreated` says whether the session
    /// had to be destroyed and rebuilt in the destination profile.
    TabMoved {
        tab: bbr_domain::TabId,
        to_space: SpaceId,
        recreated: bool,
    },
    /// A two-pane split was opened over the given tab indices.
    SplitOpened { primary: usize, secondary: usize },
    /// The active space's split was closed.
    SplitClosed,
    /// A navigation action was issued against the focused session.
    NavigationRouted { view: WebViewId },
    /// A raw engine event was applied to its owning tab's state.
    WebEvent { event: WebEvent },
    /// An engine event named a view with no owning tab (stale/unknown).
    UnknownWebView { view: WebViewId },
}
