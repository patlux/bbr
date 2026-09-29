//! The controller half of the web-content navigation gestures (N1 packet
//! `web-content-navigation-gestures`) on the [`super::BrowserController`]:
//! the logical commit decision matrix (exactly-once history dispatch vs. the
//! atomic Back-at-settled-root child-close/opener-activate transaction) and
//! the session-local opener-provenance accessors the testkit snapshot reads.
//! Split from `controller.rs` so the gateway module stays under the
//! ~500-line budget.

use bbr_domain::{TabId, TabLifecycle};
use bbr_web::{ContentGestureCommit, ContentGestureDirection, WebAction, WebViewId};

use crate::nav::NavState;
use crate::outcome::ControllerEvent;
use crate::pane::PaneLayout;

use super::BrowserController;

impl BrowserController {
    /// The controller half of the portable content-gesture decision matrix
    /// (the native adapter reports one logical commit; this method decides
    /// exactly what it means for the exact bound view):
    ///
    /// - **Back with `can_go_back`** dispatches exactly one `Back` action
    ///   to the bound view (never a tab mutation).
    /// - **Back at a settled child-tab root** with a valid session-local
    ///   opener runs the atomic close-child/activate-opener transaction.
    /// - **Forward with `can_go_forward`** dispatches exactly one `Forward`
    ///   action to the bound view.
    /// - Anything else (disabled direction without an applicable fallback,
    ///   stale view, unknown view) is a deterministic no-op returning `None`.
    ///
    /// The matrix never closes an arbitrary tab: without provenance the
    /// Back-at-root fallback does not exist.
    pub fn on_content_gesture(&mut self, commit: ContentGestureCommit) -> Option<ControllerEvent> {
        let view = commit.view;
        let tab = self.registry.tab_for_view(view)?;
        let nav = self.nav.get(&view).cloned().unwrap_or_default();
        match commit.direction {
            ContentGestureDirection::Back => {
                if nav.can_go_back {
                    self.engine.borrow_mut().action(view, WebAction::Back);
                    self.check_invariants();
                    Some(ControllerEvent::GestureHistoryRouted { view })
                } else {
                    self.gesture_return_to_opener(tab, view, &nav)
                }
            }
            ContentGestureDirection::Forward => {
                if nav.can_go_forward {
                    self.engine.borrow_mut().action(view, WebAction::Forward);
                    self.check_invariants();
                    Some(ControllerEvent::GestureHistoryRouted { view })
                } else {
                    None
                }
            }
        }
    }

    /// The atomic Back-at-settled-root child-close/opener-activate
    /// transaction. All five contract conditions are validated before any
    /// mutation, so a failing condition is a whole no-op:
    ///
    /// 1. the bound child is the visible gesture target (active tab or a
    ///    split member — never a hidden/warm background tab);
    /// 2. the real session reports `can_go_back == false` (checked by the
    ///    caller);
    /// 3. the initial document committed or a settled failure/crash state
    ///    exists (never an ambiguous provisional load);
    /// 4. a valid session-local opener relation exists; and
    /// 5. the opener `TabId` still exists in this browser window.
    ///
    /// On success the child closes through the normal close path (session
    /// destroyed, recently-closed recorded, split dissolved), the exact
    /// opener is activated by identity at its current Space/Profile (warm
    /// session reused; suspended/discarded recreated under existing
    /// lifecycle policy), the opener's web content is presented and
    /// focused, and exactly one typed [`ControllerEvent::OpenerReturn`]
    /// names child/opener, source/destination Space/Profile, the closed
    /// session, the presented session, and whether recreation was required.
    fn gesture_return_to_opener(
        &mut self,
        child: TabId,
        child_view: WebViewId,
        nav: &NavState,
    ) -> Option<ControllerEvent> {
        if !self.is_visible_gesture_target(child_view) {
            return None;
        }
        if !nav.settled_root() {
            return None;
        }
        let opener = self.provenance.opener_of(child)?;
        if opener == child {
            return None;
        }
        // Everything below is validated before the first mutation: the
        // transaction is atomic or a whole no-op.
        let child_state = self.browser.tab(child)?;
        let child_space = child_state.space;
        let child_profile = self.browser.space(child_space)?.profile;
        let opener_state = self.browser.tab(opener)?;
        let opener_space = opener_state.space;
        let opener_profile = self.browser.space(opener_space)?.profile;
        let recreated = matches!(
            opener_state.lifecycle,
            TabLifecycle::Suspended | TabLifecycle::Discarded
        );

        // Mutation 1: close the child through the normal close path (the
        // domain records recently-closed and dissolves any split the child
        // belonged to).
        self.destroy_session_of(child);
        self.provenance.invalidate_for(child);
        self.browser.close_tab(child_space, child).ok()?;

        // Mutation 2: activate the exact opener by its current TabId owner.
        if opener_space != self.active_space {
            self.active_space = opener_space;
        }
        self.browser.select_tab(opener_space, opener).ok()?;
        self.sync_views();
        let presented_view = self.registry.view_for_tab(opener)?;
        self.engine.borrow_mut().focus_native_web(presented_view);
        self.check_invariants();
        Some(ControllerEvent::OpenerReturn {
            child,
            opener,
            from_space: child_space,
            from_profile: child_profile,
            to_space: opener_space,
            to_profile: opener_profile,
            closed_view: child_view,
            presented_view,
            recreated,
        })
    }

    /// True when `view` is the visible gesture target of the active space:
    /// the single-pane active tab's view or one of the two split member
    /// views. A hidden/warm background tab's view is never a gesture target.
    fn is_visible_gesture_target(&self, view: WebViewId) -> bool {
        let Some(layout) = self.pane_layout() else {
            return false;
        };
        layout.primary == view || layout.secondary == Some(view)
    }

    /// The session-local opener relation of `tab`, if any (typed diagnostics
    /// for tests and the testkit snapshot).
    pub fn opener_of(&self, tab: TabId) -> Option<TabId> {
        self.provenance.opener_of(tab)
    }

    /// Read-only provenance pairs for the typed testkit snapshot: the
    /// `(child, opener)` identity pairs currently recorded (sorted by child
    /// id; see [`crate::provenance::OpenerProvenance::pairs`]).
    pub fn opener_pairs(&self) -> Vec<(TabId, TabId)> {
        self.provenance.pairs()
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
        if let Some(layout) = self.preview_pane_layout() {
            return Some(layout);
        }
        self.covered_pane_layout()
    }

    /// The active space's own pane layout, ignoring an open Peek preview:
    /// what the preview covers (blueprint D09 keeps these warm origin panes
    /// framed under the inset preview). Equal to [`Self::pane_layout`]
    /// while no preview is open.
    /// Blueprint D23: the visible split group with every pane's view
    /// (`None` without a group, while a preview is open, or while a pane
    /// has no live session yet).
    pub fn split_layout(&self) -> Option<crate::pane::SplitLayout> {
        if self.preview.is_some() {
            return None;
        }
        let space = self.browser.space(self.active_space)?;
        let group = space.visible_group()?;
        let focused = space
            .active_tab
            .and_then(|tab| self.registry.view_for_tab(tab))?;
        let views = group
            .members()
            .into_iter()
            .map(|tab| Some((tab, self.registry.view_for_tab(tab)?)))
            .collect::<Option<Vec<_>>>()?;
        Some(crate::pane::SplitLayout {
            group: group.clone(),
            views,
            focused,
        })
    }

    pub fn covered_pane_layout(&self) -> Option<PaneLayout> {
        let space = self.browser.space(self.active_space)?;
        let focused = space
            .active_tab
            .and_then(|tab| self.registry.view_for_tab(tab))?;
        match space.visible_group() {
            // Blueprint D23: the first two panes stand for the group here;
            // `split_layout` carries every pane.
            Some(group) => {
                let members = group.members();
                Some(PaneLayout {
                    primary: self.registry.view_for_tab(members[0])?,
                    // A split must own **every** live session: a missing
                    // pane never degrades the split into a false single
                    // pane — the honest answer is no pane layout at all.
                    secondary: Some(self.registry.view_for_tab(members[1])?),
                    focused,
                })
            }
            None => Some(PaneLayout {
                primary: focused,
                secondary: None,
                focused,
            }),
        }
    }
}
