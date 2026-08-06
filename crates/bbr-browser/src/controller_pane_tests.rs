//! Controller contract tests — the authoritative `pane_layout()` query
//! (split-view shared contract).
//!
//! `pane_layout()` must derive only from the active space's domain split,
//! its active tab, and the session registry: empty state → `None`, single
//! pane → one view, split → two ordered views with the domain-active member
//! as focus, and close/move/crash-adjacent mutations must leave registry,
//! engine, visibility, and pane-layout evidence consistent.

use bbr_web::{WebRect, WebViewId};

use crate::fake::EngineOp;
use crate::harness::{open, plain};
use crate::PaneLayout;

#[test]
fn fake_engine_addresses_geometry_by_live_id_and_skips_unchanged_rects() {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    h.reset_ops();
    // Unknown ids are no-ops: nothing is recorded.
    h.engine.borrow_mut().set_web_view_rect(
        WebViewId(99),
        WebRect::new(0.0, 0.0, 10.0, 10.0),
        700.0,
    );
    assert_eq!(h.ops(), vec![]);
    // Geometry is keyed by live view id, in call order.
    let primary_rect = WebRect::new(10.0, 0.0, 400.0, 600.0);
    let secondary_rect = WebRect::new(410.0, 0.0, 400.0, 600.0);
    h.engine
        .borrow_mut()
        .set_web_view_rect(WebViewId(1), primary_rect, 700.0);
    h.engine
        .borrow_mut()
        .set_web_view_rect(WebViewId(2), secondary_rect, 700.0);
    assert_eq!(
        h.ops(),
        vec![
            EngineOp::SetRect {
                view: WebViewId(1),
                rect: primary_rect,
            },
            EngineOp::SetRect {
                view: WebViewId(2),
                rect: secondary_rect,
            },
        ]
    );
    // An unchanged rect produces no repeated frame update; a changed one
    // publishes only for that view.
    h.engine
        .borrow_mut()
        .set_web_view_rect(WebViewId(1), primary_rect, 700.0);
    h.engine
        .borrow_mut()
        .set_web_view_rect(WebViewId(2), secondary_rect, 700.0);
    assert_eq!(h.ops().len(), 2, "unchanged rects are suppressed");
    h.engine.borrow_mut().set_web_view_rect(
        WebViewId(1),
        WebRect::new(20.0, 0.0, 400.0, 600.0),
        700.0,
    );
    assert_eq!(
        h.ops().len(),
        3,
        "changed rect publishes for that view only"
    );
}

#[test]
fn empty_state_has_no_pane_layout() {
    let (c, _h) = plain();
    assert_eq!(c.pane_layout(), None);
}

#[test]
fn single_pane_uses_the_active_view_as_primary_and_focus() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    assert_eq!(
        c.pane_layout(),
        Some(PaneLayout {
            primary: WebViewId(1),
            secondary: None,
            focused: WebViewId(1),
        })
    );
    open(&mut c, "u2");
    // The new tab is active; the previous tab is warm and hidden.
    assert_eq!(
        c.pane_layout(),
        Some(PaneLayout {
            primary: WebViewId(2),
            secondary: None,
            focused: WebViewId(2),
        })
    );
}

#[test]
fn split_orders_panes_by_domain_primary_secondary_and_keeps_domain_focus() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    c.open_split(0, 1).unwrap();
    // The pre-split active tab (index 1) is a split member, so the domain
    // keeps it as the active/focused pane: primary is tab 0's view, the
    // secondary pane is tab 1's view, and focus is the domain active tab.
    assert_eq!(
        c.pane_layout(),
        Some(PaneLayout {
            primary: WebViewId(1),
            secondary: Some(WebViewId(2)),
            focused: WebViewId(2),
        })
    );
}

#[test]
fn selecting_a_split_member_switches_focus_without_dissolving_the_split() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    c.open_split(0, 1).unwrap();
    c.select_tab(0).unwrap();
    assert_eq!(
        c.pane_layout(),
        Some(PaneLayout {
            primary: WebViewId(1),
            secondary: Some(WebViewId(2)),
            focused: WebViewId(1),
        }),
        "member selection focuses the pane, never the split"
    );
    // Selecting a tab outside the split dissolves it (single pane again).
    open(&mut c, "u3");
    assert_eq!(
        c.pane_layout(),
        Some(PaneLayout {
            primary: WebViewId(3),
            secondary: None,
            focused: WebViewId(3),
        })
    );
}

#[test]
fn closing_the_split_returns_a_full_single_pane_with_both_sessions_kept() {
    let (mut c, h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    c.open_split(0, 1).unwrap();
    c.select_tab(0).unwrap(); // focus the primary pane
    h.reset_ops();
    c.close_split().unwrap();
    assert_eq!(
        c.pane_layout(),
        Some(PaneLayout {
            primary: WebViewId(1),
            secondary: None,
            focused: WebViewId(1),
        })
    );
    let snap = c.snapshot();
    assert_eq!(snap.tabs.len(), 2, "both tabs stay open");
    assert_eq!(snap.engine_sessions, 2, "non-selected pane stays warm");
}

#[test]
fn closing_a_split_pane_dissolves_the_split_and_pane_layout_agrees() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    c.open_split(0, 1).unwrap();
    c.close_tab(0).unwrap();
    assert_eq!(
        c.pane_layout(),
        Some(PaneLayout {
            primary: WebViewId(2),
            secondary: None,
            focused: WebViewId(2),
        }),
        "closing a split member dissolves the split; the survivor becomes the single pane"
    );
    assert_eq!(c.snapshot().engine_sessions, 1, "no orphaned sessions");
}

#[test]
fn moving_a_split_member_dissolves_the_split_and_pane_layout_agrees() {
    let (mut c, _h) = plain();
    open(&mut c, "u1");
    open(&mut c, "u2");
    c.open_split(0, 1).unwrap();
    let (_, space_b) = {
        let profile = c.create_profile("P2");
        let space_b = c.create_space(profile, "B").unwrap();
        (profile, space_b)
    };
    c.move_tab(0, space_b).unwrap();
    assert_eq!(
        c.pane_layout(),
        Some(PaneLayout {
            primary: WebViewId(2),
            secondary: None,
            focused: WebViewId(2),
        }),
        "moving a split member dissolves the source split"
    );
}

#[test]
fn crash_of_the_focused_pane_leaves_no_pane_layout_or_orphan_session() {
    // The wired harness routes fake engine events into `apply_event` exactly
    // like the composition root, so the crash reaches the production path.
    let (controller, h) = crate::harness::wired();
    {
        let mut c = controller.borrow_mut();
        open(&mut c, "u1");
        open(&mut c, "u2");
        c.open_split(0, 1).unwrap();
    }
    // Crash the focused pane's view (the domain active tab, which is the
    // split's secondary member here): the domain discards that tab and
    // destroys its session, so the split can no longer publish any pane
    // layout (the focused member owns no live session anymore).
    h.emit(bbr_web::WebEvent::PageCrashed(WebViewId(2)));
    let layout = controller.borrow().pane_layout();
    assert!(
        layout.is_none(),
        "a crashed pane must not publish a stale two-pane layout: {layout:?}"
    );
    let snap = controller.borrow().snapshot();
    assert_eq!(snap.engine_sessions, 1, "the crashed session is destroyed");
    assert!(
        snap.tabs[1].crashed,
        "the crashed tab keeps its identity and error flag"
    );
}

#[test]
fn missing_primary_session_publishes_no_pane_layout() {
    // Regression: a split whose primary member lost its live session must
    // publish no pane layout (the `?` on the required primary lookup). The
    // focused session survives, so this exercises the split branch itself.
    let (controller, h) = crate::harness::wired();
    {
        let mut c = controller.borrow_mut();
        open(&mut c, "u1");
        open(&mut c, "u2");
        c.open_split(0, 1).unwrap();
    }
    // The domain active tab stays the secondary (view 2); crash the
    // non-focused primary member (view 1).
    h.emit(bbr_web::WebEvent::PageCrashed(WebViewId(1)));
    let layout = controller.borrow().pane_layout();
    assert!(
        layout.is_none(),
        "a split with a missing primary session must publish no layout: {layout:?}"
    );
    let snap = controller.borrow().snapshot();
    assert_eq!(snap.engine_sessions, 1, "the crashed session is destroyed");
    assert!(snap.tabs[0].crashed, "the crashed tab keeps its identity");
}

#[test]
fn missing_secondary_session_never_degrades_the_split_into_single_pane() {
    // Regression: a split whose secondary member lost its live session must
    // NOT degrade into a false single pane. Discard does not dissolve the
    // domain split, so with a missing secondary the only honest answer is
    // no pane layout at all — the shell must not frame the survivor as a
    // full-width pane while the split is still open.
    let (controller, h) = crate::harness::wired();
    {
        let mut c = controller.borrow_mut();
        open(&mut c, "u1");
        open(&mut c, "u2");
        c.open_split(0, 1).unwrap();
        // Focus the primary pane: the secondary is now genuinely
        // non-focused, so its crash cannot hide behind the focused-session
        // lookup returning `None`.
        c.select_tab(0).unwrap();
    }
    h.emit(bbr_web::WebEvent::PageCrashed(WebViewId(2)));
    let layout = controller.borrow().pane_layout();
    assert!(
        layout.is_none(),
        "a split with a missing secondary session must not degrade into a single pane: {layout:?}"
    );
    let snap = controller.borrow().snapshot();
    assert_eq!(snap.engine_sessions, 1, "the crashed session is destroyed");
    assert!(snap.tabs[1].crashed, "the crashed tab keeps its identity");
    assert!(
        snap.tabs[0].web_view == Some(WebViewId(1)),
        "the focused primary session survives"
    );
}
