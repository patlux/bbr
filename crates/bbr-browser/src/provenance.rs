//! Session-local regular child-tab opener provenance.
//!
//! When the canonical new-window/disposition policy resolves a user-originated
//! request to a **regular tab**, the controller records a relation from the
//! child [`TabId`] to the exact opener [`TabId`]. The relation is
//! **identity-based** — never index-, URL-, title-, `WebViewId`-, or
//! render-position-based — so it follows either tab through reorder,
//! same-window moves between spaces/profiles, navigation, reload, suspension,
//! crash, and native-session replacement.
//!
//! Lifetime rules (browser-interactions.md / task packet N1):
//!
//! - Closing **either** tab invalidates the relation (a reopened tab is a
//!   fresh identity with no provenance — reopen never invents one).
//! - Ending the browser process invalidates everything (in-memory only;
//!   v1 never persists provenance across restart).
//! - `Cmd-T`, Saved Page activation, reopen-closed-tab, external URL
//!   handling, duplicate/bulk operations, and tabs created without a
//!   concrete source tab never record provenance: only the new-window
//!   regular-tab path calls [`OpenerProvenance::record`].
//!
//! The controller is one browser window, so the "same browser window"
//! requirement of the contract holds by construction here; a future
//! multi-window controller would key the map by window identity.

use std::collections::HashMap;

use bbr_domain::TabId;

/// Identity-based child → opener relations within one browser window.
#[derive(Debug, Default)]
pub struct OpenerProvenance {
    /// child `TabId` → exact opener `TabId`.
    relations: HashMap<TabId, TabId>,
}

impl OpenerProvenance {
    /// A new, empty provenance store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record that `child` was opened from `opener` (identity-based). Only
    /// the productive new-window regular-tab path may call this; ordinary
    /// tab creation never does.
    pub fn record(&mut self, child: TabId, opener: TabId) {
        debug_assert_ne!(
            child, opener,
            "a tab cannot be its own opener (defensive; never produced by the new-window path)"
        );
        self.relations.insert(child, opener);
    }

    /// The exact opener `TabId` of `child`, if recorded and not invalidated.
    pub fn opener_of(&self, child: TabId) -> Option<TabId> {
        self.relations.get(&child).copied()
    }

    /// Remove every relation involving `tab` — as a child (it closed) or as
    /// an opener (its children lose their provenance). Called whenever a tab
    /// closes.
    pub fn invalidate_for(&mut self, tab: TabId) {
        self.relations.remove(&tab);
        self.relations.retain(|_, opener| *opener != tab);
    }

    /// The number of live relations (diagnostics/tests only).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn len(&self) -> usize {
        self.relations.len()
    }

    /// True when no relation is recorded.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn is_empty(&self) -> bool {
        self.relations.is_empty()
    }

    /// Read-only snapshot of the current relations for tests and typed
    /// testkit evidence: `(child, opener)` identity pairs sorted by child
    /// `TabId` so the evidence is deterministic (`HashMap` iteration order
    /// is unspecified and is never relied on).
    pub fn pairs(&self) -> Vec<(TabId, TabId)> {
        let mut pairs: Vec<(TabId, TabId)> = self.relations.iter().map(|(c, o)| (*c, *o)).collect();
        pairs.sort_by_key(|(child, _)| *child);
        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tab(id: u64) -> TabId {
        TabId(id)
    }

    #[test]
    fn record_and_lookup_are_identity_based() {
        let mut p = OpenerProvenance::new();
        assert!(p.is_empty());
        p.record(tab(2), tab(1));
        assert_eq!(p.opener_of(tab(2)), Some(tab(1)));
        assert_eq!(p.len(), 1);
        assert_eq!(p.pairs(), vec![(tab(2), tab(1))]);
    }

    #[test]
    fn closing_the_child_invalidates_only_its_relation() {
        let mut p = OpenerProvenance::new();
        p.record(tab(2), tab(1));
        p.record(tab(3), tab(1));
        p.invalidate_for(tab(2));
        assert_eq!(p.opener_of(tab(2)), None, "closed child loses provenance");
        assert_eq!(
            p.opener_of(tab(3)),
            Some(tab(1)),
            "a sibling child keeps its provenance"
        );
    }

    #[test]
    fn closing_the_opener_invalidates_its_children() {
        let mut p = OpenerProvenance::new();
        p.record(tab(2), tab(1));
        p.record(tab(3), tab(1));
        p.record(tab(5), tab(4));
        p.invalidate_for(tab(1));
        assert_eq!(p.opener_of(tab(2)), None);
        assert_eq!(p.opener_of(tab(3)), None);
        assert_eq!(
            p.opener_of(tab(5)),
            Some(tab(4)),
            "relations under a different opener survive"
        );
    }

    #[test]
    fn reorder_and_move_do_not_touch_relations() {
        // Identity-based by construction: nothing here observes index,
        // URL, title, or WebViewId. Recording, then "moving" (a no-op for
        // the map) keeps the same relation.
        let mut p = OpenerProvenance::new();
        p.record(tab(7), tab(1));
        assert_eq!(p.opener_of(tab(7)), Some(tab(1)));
        p.record(tab(7), tab(1)); // a re-record is idempotent
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn replaced_relation_updates_the_opener() {
        let mut p = OpenerProvenance::new();
        p.record(tab(2), tab(1));
        p.record(tab(2), tab(9));
        assert_eq!(p.opener_of(tab(2)), Some(tab(9)));
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn pairs_are_deterministic_sorted_by_child_id() {
        let mut p = OpenerProvenance::new();
        p.record(tab(7), tab(1));
        p.record(tab(3), tab(1));
        p.record(tab(9), tab(4));
        // Never HashMap iteration order: the evidence is sorted by child id.
        assert_eq!(
            p.pairs(),
            vec![(tab(3), tab(1)), (tab(7), tab(1)), (tab(9), tab(4))]
        );
    }
}
