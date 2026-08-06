//! Testkit-only keyed per-pane geometry observation for the split seam.
//!
//! The performance probe publishes the focused slot's legacy `content_rect`;
//! this focused module additionally records the per-`WebViewId` native rect
//! of every visible pane so the semantic snapshot can prove keyed split
//! geometry (`pane_rects`) and prune destroyed/hidden panes. It is a
//! separate responsibility from presentation timing, kept in its own module
//! so `performance.rs` stays under the ~500-line budget.
//!
//! The whole module is testkit-gated through its parent (`performance` is
//! only compiled under the `testkit` feature).

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use bbr_web::{WebRect, WebViewId};

/// Keyed per-pane native geometry: pane id → the last engine-received
/// WKWebView frame of that pane, pruned to the current pane layout each
/// render so destroyed/hidden panes never leave orphan geometry evidence.
#[derive(Clone, Default)]
pub struct PaneGeometry {
    inner: Rc<RefCell<BTreeMap<WebViewId, WebRect>>>,
}

impl PaneGeometry {
    /// Observe the rect one pane slot handed to the engine, keyed by its
    /// view id (the real native WKWebView frame of that pane).
    pub fn observe(&self, id: WebViewId, rect: WebRect) {
        self.inner.borrow_mut().insert(id, rect);
    }

    /// The keyed per-pane geometry observed since the last prune.
    pub fn snapshot(&self) -> BTreeMap<WebViewId, WebRect> {
        self.inner.borrow().clone()
    }

    /// Drop keyed geometry for every view outside the current pane layout,
    /// so destroyed/hidden panes leave no orphan geometry evidence.
    pub fn prune(&self, keep: &[WebViewId]) {
        self.inner.borrow_mut().retain(|id, _| keep.contains(id));
    }

    /// A callback that observes every pane slot rect keyed by view id
    /// (testkit-only keyed geometry observation for the split seam).
    pub fn callback(&self) -> Rc<dyn Fn(WebViewId, WebRect)> {
        let geometry = self.clone();
        Rc::new(move |id, rect| geometry.observe(id, rect))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observes_snapshots_and_prunes_keyed_geometry() {
        let geometry = PaneGeometry::default();
        let primary = WebRect::new(10.0, 0.0, 400.0, 600.0);
        let secondary = WebRect::new(410.0, 0.0, 400.0, 600.0);
        geometry.observe(WebViewId(1), primary);
        geometry.observe(WebViewId(2), secondary);
        assert_eq!(geometry.snapshot().len(), 2);
        geometry.observe(WebViewId(1), primary); // overwrite is fine (latest frame)
        assert_eq!(geometry.snapshot().get(&WebViewId(1)), Some(&primary));

        // Pruning to the current pane layout drops hidden/destroyed panes.
        geometry.prune(&[WebViewId(1)]);
        assert_eq!(geometry.snapshot().len(), 1);
        assert!(!geometry.snapshot().contains_key(&WebViewId(2)));
    }

    #[test]
    fn callback_records_by_view_id() {
        let geometry = PaneGeometry::default();
        let cb = geometry.callback();
        cb(WebViewId(7), WebRect::new(0.0, 0.0, 100.0, 80.0));
        assert_eq!(
            geometry.snapshot().get(&WebViewId(7)),
            Some(&WebRect::new(0.0, 0.0, 100.0, 80.0))
        );
    }
}
