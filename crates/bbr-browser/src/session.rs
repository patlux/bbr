//! Bidirectional `TabId` ↔ `WebViewId` session registry.
//!
//! The core ownership invariant of the controller: every live tab maps to
//! exactly one disposable web session, and every session maps back to exactly
//! one tab. The registry keeps both directions unique so a bug can never
//! orphan a native view or double-map a tab.

use std::collections::{HashMap, HashSet};

use bbr_domain::TabId;
use bbr_web::WebViewId;

/// A 1:1 mapping between tabs and their live web sessions.
#[derive(Debug, Default)]
pub struct SessionRegistry {
    forward: HashMap<TabId, WebViewId>,
    reverse: HashMap<WebViewId, TabId>,
}

impl SessionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// The session currently backing `tab`, if any.
    pub fn view_for_tab(&self, tab: TabId) -> Option<WebViewId> {
        self.forward.get(&tab).copied()
    }

    /// The tab owning session `view`, if any.
    pub fn tab_for_view(&self, view: WebViewId) -> Option<TabId> {
        self.reverse.get(&view).copied()
    }

    /// Number of live sessions (== number of live tabs by invariant).
    pub fn live_sessions(&self) -> usize {
        self.forward.len()
    }

    /// All live tabs.
    pub fn tabs(&self) -> impl Iterator<Item = TabId> + '_ {
        self.forward.keys().copied()
    }

    /// All live sessions.
    pub fn views(&self) -> impl Iterator<Item = WebViewId> + '_ {
        self.reverse.keys().copied()
    }

    /// Register a fresh 1:1 mapping. Rejects either id already being mapped.
    pub fn insert(&mut self, tab: TabId, view: WebViewId) -> Result<(), SessionError> {
        if self.forward.contains_key(&tab) {
            return Err(SessionError::TabAlreadyMapped(tab));
        }
        if self.reverse.contains_key(&view) {
            return Err(SessionError::ViewAlreadyMapped(view));
        }
        self.forward.insert(tab, view);
        self.reverse.insert(view, tab);
        Ok(())
    }

    /// Remove the mapping for `tab`, returning its view (if any).
    pub fn remove_tab(&mut self, tab: TabId) -> Option<WebViewId> {
        let view = self.forward.remove(&tab)?;
        self.reverse.remove(&view);
        Some(view)
    }

    /// Remove the mapping for `view`, returning its tab (if any).
    pub fn remove_view(&mut self, view: WebViewId) -> Option<TabId> {
        let tab = self.reverse.remove(&view)?;
        self.forward.remove(&tab);
        Some(tab)
    }

    /// Structural invariant: both directions agree and are unique.
    pub fn check_invariants(&self) -> Result<(), SessionError> {
        if self.forward.len() != self.reverse.len() {
            return Err(SessionError::Asymmetric);
        }
        let fwd_tabs: HashSet<TabId> = self.forward.keys().copied().collect();
        let rev_tabs: HashSet<TabId> = self.reverse.values().copied().collect();
        if fwd_tabs != rev_tabs {
            return Err(SessionError::Asymmetric);
        }
        for (tab, view) in &self.forward {
            if self.reverse.get(view) != Some(tab) {
                return Err(SessionError::Mismatch {
                    tab: *tab,
                    view: *view,
                });
            }
        }
        Ok(())
    }
}

/// A registry structural violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    TabAlreadyMapped(bbr_domain::TabId),
    ViewAlreadyMapped(bbr_web::WebViewId),
    Asymmetric,
    Mismatch {
        tab: bbr_domain::TabId,
        view: bbr_web::WebViewId,
    },
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TabAlreadyMapped(t) => write!(f, "tab {t:?} already has a session"),
            Self::ViewAlreadyMapped(v) => write!(f, "view {v:?} already backs a tab"),
            Self::Asymmetric => write!(
                f,
                "forward and reverse session maps disagree (orphan or duplicate)"
            ),
            Self::Mismatch { tab, view } => {
                write!(
                    f,
                    "tab {tab:?} and view {view:?} do not map back to each other"
                )
            }
        }
    }
}

impl std::error::Error for SessionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_lookup_both_directions() {
        let mut r = SessionRegistry::new();
        r.insert(TabId(1), WebViewId(10)).unwrap();
        assert_eq!(r.view_for_tab(TabId(1)), Some(WebViewId(10)));
        assert_eq!(r.tab_for_view(WebViewId(10)), Some(TabId(1)));
        assert_eq!(r.live_sessions(), 1);
        r.check_invariants().unwrap();
    }

    #[test]
    fn insert_rejects_duplicate_tab_or_view() {
        let mut r = SessionRegistry::new();
        r.insert(TabId(1), WebViewId(10)).unwrap();
        assert_eq!(
            r.insert(TabId(1), WebViewId(11)),
            Err(SessionError::TabAlreadyMapped(TabId(1)))
        );
        assert_eq!(
            r.insert(TabId(2), WebViewId(10)),
            Err(SessionError::ViewAlreadyMapped(WebViewId(10)))
        );
        r.check_invariants().unwrap();
    }

    #[test]
    fn remove_tab_drops_both_directions() {
        let mut r = SessionRegistry::new();
        r.insert(TabId(1), WebViewId(10)).unwrap();
        r.insert(TabId(2), WebViewId(20)).unwrap();
        assert_eq!(r.remove_tab(TabId(1)), Some(WebViewId(10)));
        assert_eq!(r.view_for_tab(TabId(1)), None);
        assert_eq!(r.tab_for_view(WebViewId(10)), None);
        assert_eq!(r.live_sessions(), 1);
        r.check_invariants().unwrap();
    }

    #[test]
    fn remove_view_drops_both_directions() {
        let mut r = SessionRegistry::new();
        r.insert(TabId(1), WebViewId(10)).unwrap();
        assert_eq!(r.remove_view(WebViewId(10)), Some(TabId(1)));
        assert_eq!(r.live_sessions(), 0);
        r.check_invariants().unwrap();
    }

    #[test]
    fn two_sessions_are_independent() {
        let mut r = SessionRegistry::new();
        r.insert(TabId(1), WebViewId(10)).unwrap();
        r.insert(TabId(2), WebViewId(20)).unwrap();
        assert_eq!(r.view_for_tab(TabId(1)), Some(WebViewId(10)));
        assert_eq!(r.view_for_tab(TabId(2)), Some(WebViewId(20)));
        assert_eq!(r.tab_for_view(WebViewId(20)), Some(TabId(2)));
        r.check_invariants().unwrap();
    }
}
