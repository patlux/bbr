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
    /// A transient preview session opened over its warm origin (Link
    /// Preview / Peek packet): the source view stays warm underneath, the
    /// preview owns the pane and browser-command focus, and no regular tab
    /// was created.
    PreviewOpened {
        id: crate::peek::PreviewId,
        view: WebViewId,
        source: WebViewId,
        profile: ProfileId,
        url: String,
    },
    /// The transient preview session was destroyed (dismissal, origin
    /// removal, or teardown).
    PreviewClosed { view: WebViewId },
    /// A new eligible request from inside the preview navigated the existing
    /// preview session (the deterministic one-preview-per-pane policy).
    PreviewNavigated { view: WebViewId },
    /// The preview session was adopted as a regular tab in the source
    /// space/profile without recreating the native session: the same
    /// `WebViewId` now backs a real tab and the active selection changed to
    /// it.
    PreviewPromoted {
        tab: bbr_domain::TabId,
        index: usize,
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
    /// One committed logical content gesture dispatched exactly one real
    /// history action (Back/Forward) against the exact bound view — never a
    /// tab mutation. The gesture targets the pane under the pointer, which
    /// may differ from the focused command target in split mode.
    GestureHistoryRouted { view: WebViewId },
    /// A committed logical Back gesture at a settled child-tab root closed
    /// the child through the normal close path and activated the exact
    /// opener: `child`/`opener` are the persistent `TabId`s, source is the
    /// child's Space/Profile, destination the opener's current Space/Profile,
    /// `closed_view` the destroyed child session, `presented_view` the
    /// opener session presented/focused, and `recreated` whether the opener's
    /// session had to be rebuilt (suspended/discarded lifecycle).
    OpenerReturn {
        child: bbr_domain::TabId,
        opener: bbr_domain::TabId,
        from_space: SpaceId,
        from_profile: ProfileId,
        to_space: SpaceId,
        to_profile: ProfileId,
        closed_view: WebViewId,
        presented_view: WebViewId,
        recreated: bool,
    },
    /// A raw engine event was applied to its owning tab's state.
    WebEvent { event: WebEvent },
    /// An engine event named a view with no owning tab (stale/unknown).
    UnknownWebView { view: WebViewId },
}
