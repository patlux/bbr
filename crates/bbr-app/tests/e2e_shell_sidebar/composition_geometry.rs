//! G3 Sidebar composition geometry evidence (packet
//! `component-gallery-compositions`): the native WKWebView geometry stays
//! exactly rail-width/content-floor correct in the existing split layout
//! with collapse/expand restore and no session/WebView identity churn, and
//! the bounded dense 500-tab/50-session production path renders through
//! the sidebar composition with the full rail coherent and the regular
//! region windowed. Every step reuses the existing typed requests, events,
//! snapshots, deterministic fixtures, and the existing dense seed path —
//! no sleeps, pixels, controller mutation, fake engine, or new testkit
//! seam.

use super::*;
use crate::e2e_semantics_common::find;
use bbr_testkit::protocol::dsl;
use e2e_dense_common::{flush, seed_dense, WINDOW_ROWS};

#[path = "../e2e_dense_common/mod.rs"]
mod e2e_dense_common;

/// The reference-authoritative settled rail order; mirrors
/// `super::composition::RAIL_ORDER_AT_REST` for the dense branch.
const RAIL_ORDER_AT_REST: [&str; 6] = [
    "bbr-titlebar",
    "bbr-address",
    "bbr-active-space-title",
    "bbr-tab-list",
    "bbr-footer",
    "bbr-sidebar-divider",
];

fn assert_rail_order(snapshot: &bbr_testkit::protocol::Snapshot, expected: &[&str]) {
    let sidebar = find(&snapshot.semantics, "bbr-sidebar")
        .unwrap_or_else(|| panic!("semantic tree missing bbr-sidebar"));
    let order: Vec<&str> = sidebar
        .children
        .iter()
        .map(|node| node.id.as_str())
        .collect();
    assert_eq!(order, expected, "§11.4 rail region order");
}

fn pane_rect_of(snapshot: &bbr_testkit::protocol::Snapshot, view: bbr_web::WebViewId) -> [f32; 4] {
    snapshot
        .pane_rects
        .iter()
        .find(|pane| pane.view == view)
        .map(|pane| pane.rect)
        .unwrap_or([0.0; 4])
}

/// Poll until the keyed pane rects reach the expected per-view rects
/// (paint-driven; deadline-bounded snapshot polling, no sleeps).
fn wait_for_pane_rects(
    client: &mut Client,
    expected: &[(bbr_web::WebViewId, [f32; 4])],
    deadline: Instant,
) -> bbr_testkit::protocol::Snapshot {
    let close = |a: [f32; 4], b: [f32; 4]| {
        (a[0] - b[0]).abs() <= 1.0
            && (a[1] - b[1]).abs() <= 1.0
            && (a[2] - b[2]).abs() <= 1.0
            && (a[3] - b[3]).abs() <= 1.0
    };
    loop {
        let snapshot = client.snapshot().expect("pane-rects snapshot");
        if expected
            .iter()
            .all(|(view, rect)| close(pane_rect_of(&snapshot, *view), *rect))
        {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "keyed pane rects did not reach {expected:?}; last {:?}",
            snapshot.pane_rects
        );
        std::thread::yield_now();
    }
}

/// The sidebar composition keeps the native WKWebView geometry exactly
/// rail-width/content-floor correct in the existing split layout: both
/// panes reflow to the full content width on collapse and back to the
/// rail-offset split on expand, the focused pane's rect stays the
/// `content_rect` contract, and no session/WebView identity churns.
#[test]
#[ignore]
fn split_collapse_expand_keeps_native_geometry_and_identity() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("sidebar-split-collapse", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let settled = wait_for_window_settled(&mut client, deadline());
        let window_w = settled.window_size[0];
        let initial_rect = wait_for_content_rect_x(
            &mut client,
            SIDEBAR_DEFAULT,
            window_w - SIDEBAR_DEFAULT,
            deadline(),
        );
        let height = initial_rect.content_rect[3];

        // Open the split over two tabs: both panes live, adjacent, and
        // strictly positive at the rail offset.
        let second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client.send_ok(&dsl::set_split(0, 1)).expect("set split");
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::SplitOpened {
                primary: 0,
                secondary: 1,
            },
            since,
        );
        flush_frame(&mut client);
        let pane_w = (window_w - SIDEBAR_DEFAULT) / 2.0;
        let split = wait_for_pane_rects(
            &mut client,
            &[
                (first_view, [SIDEBAR_DEFAULT, 0.0, pane_w, height]),
                (second_view, [SIDEBAR_DEFAULT + pane_w, 0.0, pane_w, height]),
            ],
            deadline(),
        );
        assert_eq!(split.engine_sessions, 2);
        assert_eq!(split.tabs[0].web_view, Some(first_view));
        assert_eq!(split.tabs[1].web_view, Some(second_view));

        // Collapse: both panes reflow to the full content width (rail 0),
        // the focused pane's rect is the full window, and both sessions
        // stay live.
        keystroke(&mut client, "cmd-b");
        flush_frame(&mut client);
        let collapsed = client.snapshot().expect("collapsed split snapshot");
        assert!(!collapsed.sidebar_visible);
        assert_eq!(collapsed.sidebar_width, 0.0);
        let collapsed_pane_w = window_w / 2.0;
        let collapsed_split = wait_for_pane_rects(
            &mut client,
            &[
                (first_view, [0.0, 0.0, collapsed_pane_w, height]),
                (
                    second_view,
                    [collapsed_pane_w, 0.0, collapsed_pane_w, height],
                ),
            ],
            deadline(),
        );
        assert_eq!(collapsed_split.engine_sessions, 2, "no session churn");
        assert_eq!(collapsed_split.tabs[0].web_view, Some(first_view));
        assert_eq!(collapsed_split.tabs[1].web_view, Some(second_view));
        assert_eq!(
            collapsed_split.content_rect, collapsed_split.pane_rects[1].rect,
            "content_rect follows the focused pane"
        );

        // Expand: the rail returns to the default width and both panes
        // reflow back to the rail-offset split; identity is untouched.
        keystroke(&mut client, "cmd-b");
        flush_frame(&mut client);
        let expanded = client.snapshot().expect("expanded split snapshot");
        assert!(expanded.sidebar_visible);
        assert_eq!(expanded.sidebar_width, SIDEBAR_DEFAULT);
        let restored = wait_for_pane_rects(
            &mut client,
            &[
                (first_view, [SIDEBAR_DEFAULT, 0.0, pane_w, height]),
                (second_view, [SIDEBAR_DEFAULT + pane_w, 0.0, pane_w, height]),
            ],
            deadline(),
        );
        assert_eq!(restored.engine_sessions, 2);
        assert_eq!(restored.tabs[0].web_view, Some(first_view));
        assert_eq!(restored.tabs[1].web_view, Some(second_view));
        assert_eq!(
            restored.content_rect, restored.pane_rects[1].rect,
            "focused pane rect is the exact sidebar contract"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The bounded dense 500-tab/50-session production path renders through
/// the sidebar composition: the full rail (titlebar, active Space, New Tab,
/// favorites, folders, conditional status, footer/strip/plus) stays coherent
/// at density, the regular region stays windowed (never 500 rows), and the
/// pinned block keeps its identity. Reuses the existing dense seed path — no
/// new unbounded seed/render path.
#[test]
#[ignore]
fn dense_window_renders_through_the_sidebar_composition() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("sidebar-dense-composition", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _replay = client.attach(None).expect("attach");

        // Seed the dense state through the existing production path in
        // bounded batches, then converge to exactly 50 live sessions.
        seed_dense(&mut client, &page_one, 500, 10, 50, 100);
        client
            .send_ok(&bbr_testkit::Request::EnforceSessionBudget { cap: 50 })
            .expect("enforce session budget");
        flush(&mut client);

        let snap = client.snapshot().expect("dense composition snapshot");
        assert_eq!(snap.tabs.len(), 500);
        assert_eq!(snap.engine_sessions, 50);
        assert_eq!(snap.pinned_count, 10);
        // The full rail composition holds at density: titlebar controls,
        // command slot, active Space, New Tab, footer/strip/plus and divider.
        assert_ids(
            &snap,
            &[
                "bbr-sidebar",
                "bbr-titlebar",
                "bbr-titlebar-sidebar-toggle",
                "bbr-address",
                "bbr-active-space-title",
                "bbr-new-tab",
                "bbr-nav-back",
                "bbr-nav-forward",
                "bbr-nav-reload",
                "bbr-footer",
                "bbr-downloads",
                "bbr-space-strip",
                "bbr-plus",
                "bbr-sidebar-divider",
            ],
        );
        assert!(
            !snap.visible_ids.iter().any(|id| id == "bbr-brand"),
            "the dense rail never restores the obsolete standalone brand"
        );
        let mut expected_order = RAIL_ORDER_AT_REST.to_vec();
        if snap.visible_ids.iter().any(|id| id == "bbr-status") {
            expected_order.insert(expected_order.len() - 2, "bbr-status");
        }
        // The regular region stays windowed: the scroll container and a
        // bounded window of rows render, never 500 rows.
        assert_ids(&snap, &["bbr-tab-scroll", "bbr-tab-row-436"]);
        assert!(
            !snap.visible_ids.iter().any(|id| id == "bbr-tab-row-100"),
            "rows outside the bounded window are never exposed"
        );
        assert!(
            snap.visible_ids.iter().any(|id| id == "bbr-tab-row-0"),
            "the pinned block stays rendered"
        );
        assert!(
            snap.dense.rendered_rows <= WINDOW_ROWS,
            "the regular window stays bounded"
        );
        // The semantic tree still mirrors the §11.4 region order at
        // density (favorites/folders empty → hidden).
        assert_rail_order(&snap, &expected_order);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
