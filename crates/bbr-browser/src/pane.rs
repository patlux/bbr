//! The authoritative pane layout of the active space.
//!
//! [`PaneLayout`] is a plain public value the composition root (and E2E)
//! reads from [`BrowserController::pane_layout`] to frame the visible native
//! web sessions. It is derived **only** from the active space's domain
//! split, its active tab, and the session registry — the app owns no
//! independent focused-pane state. The focused pane is always the domain
//! active tab's view, and while a split is open that view is one of the two
//! split members by domain invariant.

use bbr_web::WebViewId;

/// Which live native views the shell must frame, and which one is the
/// browser command target.
///
/// - `primary` is the domain split's primary member (single-pane mode: the
///   active tab's view).
/// - `secondary` is the domain split's secondary member; `None` in
///   single-pane mode.
/// - `focused` is the domain active tab's view and is always one of the two
///   panes while a split is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaneLayout {
    pub primary: WebViewId,
    pub secondary: Option<WebViewId>,
    pub focused: WebViewId,
}

/// Blueprint D23: the visible split group of the active space with the live
/// view of every pane (tree leaves are tabs). Read with
/// [`crate::BrowserController::split_layout`]; `None` without a visible
/// group or while a preview covers the space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitLayout {
    pub group: bbr_domain::tiling::SplitGroup,
    /// Every pane's tab and view, in reading order.
    pub views: Vec<(bbr_domain::TabId, WebViewId)>,
    pub focused: WebViewId,
}

impl SplitLayout {
    pub fn view_of(&self, tab: bbr_domain::TabId) -> Option<WebViewId> {
        self.views
            .iter()
            .find(|(pane, _)| *pane == tab)
            .map(|(_, view)| *view)
    }

    pub fn tab_of(&self, view: WebViewId) -> Option<bbr_domain::TabId> {
        self.views
            .iter()
            .find(|(_, pane)| *pane == view)
            .map(|(tab, _)| *tab)
    }
}
