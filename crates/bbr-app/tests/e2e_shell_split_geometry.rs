//! Real-app split-view shared-contract E2E (Phase-6 serial anchor).
//!
//! Proves the keyed per-`WebViewId` geometry seam and the authoritative pane
//! layout over real WKWebViews: two simultaneous native panes with distinct,
//! adjacent, strictly positive keyed rects; unchanged session identities;
//! focused pane equals the domain-active pre-split tab; member selection
//! switches focus without dissolving the split; a window resize reframes
//! both pane ids independently; `CloseSplit` returns to one full-width
//! focused rect while both tabs/sessions stay; no orphan geometry; and the
//! single-pane `content_rect` remains the exact sidebar contract. No
//! sleeps/pixels: every geometry step synchronizes on a paint-driven keyed
//! rect under a deadline. The shared harness lives in `e2e_common`.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::{dsl, PaneLayoutSnapshot, Snapshot};
use e2e_common::*;

/// The DESIGN §5 default rail width (mirrors `crate::sidebar`).
const SIDEBAR_DEFAULT: f32 = 260.0;

fn flush_frame(client: &mut Client) {
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush frame");
}

/// The expected single-pane native rect for a visible sidebar: the rail on
/// the left, the focused WKWebView filling the remainder at the slot's own
/// stable height.
fn single_pane_rect(sidebar: f32, window_w: f32, height: f32) -> [f32; 4] {
    [sidebar, 0.0, window_w - sidebar, height]
}

/// The expected split pane rects (equal horizontal panes, primary left):
/// both strictly positive, adjacent, full height.
fn split_rects(sidebar: f32, window_w: f32, height: f32) -> ([f32; 4], [f32; 4]) {
    let pane_w = (window_w - sidebar) / 2.0;
    (
        [sidebar, 0.0, pane_w, height],
        [sidebar + pane_w, 0.0, pane_w, height],
    )
}

fn rect_close(a: [f32; 4], b: [f32; 4]) -> bool {
    (a[0] - b[0]).abs() <= 1.0
        && (a[1] - b[1]).abs() <= 1.0
        && (a[2] - b[2]).abs() <= 1.0
        && (a[3] - b[3]).abs() <= 1.0
}

fn pane_rect_of(snapshot: &Snapshot, view: bbr_web::WebViewId) -> [f32; 4] {
    snapshot
        .pane_rects
        .iter()
        .find(|pane| pane.view == view)
        .map(|pane| pane.rect)
        .unwrap_or([0.0; 4])
}

/// Poll until the keyed pane rects reach the expected per-view rects
/// (paint-driven; deadline-bounded snapshot polling, no sleeps). Returns the
/// snapshot where both pane rects agree.
fn wait_for_pane_rects(
    client: &mut Client,
    expected: &[(bbr_web::WebViewId, [f32; 4])],
    deadline: std::time::Instant,
) -> Snapshot {
    loop {
        let snapshot = client.snapshot().expect("pane-rects snapshot");
        if expected
            .iter()
            .all(|(view, rect)| rect_close(pane_rect_of(&snapshot, *view), *rect))
        {
            return snapshot;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "keyed pane rects did not reach {expected:?}; last {:?}",
            snapshot.pane_rects
        );
        std::thread::yield_now();
    }
}

fn assert_layout(snapshot: &Snapshot, expected: PaneLayoutSnapshot) {
    assert_eq!(
        snapshot.pane_layout,
        Some(expected),
        "pane layout must match the authoritative controller query"
    );
}

#[test]
#[ignore]
fn split_geometry_keyed_panes_focus_resize_and_close() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("split-geom", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first native view");
        assert_eq!(first_view, WebViewId(1), "sequential ids: first view is 1");

        // Geometry captured only after the AppKit frame clamp settles (see
        // the sidebar suite for the same host quirk).
        let settled = wait_for_window_settled(&mut client, deadline());
        let window_w = settled.window_size[0];
        assert!(window_w > 400.0, "spawned window is comfortably wide");
        let sidebar = settled.sidebar_width;
        assert_eq!(sidebar, SIDEBAR_DEFAULT);

        // Single-pane sidebar contract is exact before any split: the keyed
        // slot id is present and the native content rect is the rail + the
        // full remainder.
        let single = wait_for_content_rect_x(&mut client, sidebar, window_w - sidebar, deadline());
        let height = single.content_rect[3];
        assert!(height > 0.0, "native content height must be positive");
        assert_ids(&single, &["bbr-native-web-slot-1"]);
        assert_layout(
            &single,
            PaneLayoutSnapshot {
                primary: WebViewId(1),
                secondary: None,
                focused: WebViewId(1),
            },
        );

        // Open the second tab: view 2 becomes active, view 1 stays warm, and
        // the single-pane layout follows the domain active tab.
        let second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        assert_eq!(second_view, WebViewId(2));
        let two_tabs = client.snapshot().expect("two-tab snapshot");
        assert_eq!(two_tabs.engine_sessions, 2, "both tabs own live sessions");
        assert_layout(
            &two_tabs,
            PaneLayoutSnapshot {
                primary: WebViewId(2),
                secondary: None,
                focused: WebViewId(2),
            },
        );
        assert_eq!(two_tabs.tabs[0].web_view, Some(WebViewId(1)));
        assert_eq!(two_tabs.tabs[1].web_view, Some(WebViewId(2)));

        // SetSplit over both tabs: two simultaneous native panes with
        // distinct, adjacent, positive keyed rects; the focused pane is the
        // domain-active pre-split tab (tab 1 → view 2); sessions unchanged.
        client.send_ok(&dsl::set_split(0, 1)).expect("set split");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SplitOpened {
                primary: 0,
                secondary: 1,
            },
            since,
        );
        flush_frame(&mut client);
        let (primary_rect, secondary_rect) = split_rects(sidebar, window_w, height);
        let split = wait_for_pane_rects(
            &mut client,
            &[(WebViewId(1), primary_rect), (WebViewId(2), secondary_rect)],
            deadline(),
        );
        assert_layout(
            &split,
            PaneLayoutSnapshot {
                primary: WebViewId(1),
                secondary: Some(WebViewId(2)),
                focused: WebViewId(2),
            },
        );
        assert_eq!(
            split.pane_rects.len(),
            2,
            "both panes publish keyed geometry"
        );
        assert_eq!(split.pane_rects[0].view, WebViewId(1), "pane-layout order");
        assert_eq!(split.pane_rects[1].view, WebViewId(2));
        for pane in &split.pane_rects {
            assert!(
                pane.rect[2] > 0.0 && pane.rect[3] > 0.0,
                "every pane rect stays strictly positive: {pane:?}"
            );
        }
        assert!(
            (split.pane_rects[1].rect[0]
                - (split.pane_rects[0].rect[0] + split.pane_rects[0].rect[2]))
                .abs()
                <= 1.0,
            "the two panes are adjacent: {:?}",
            split.pane_rects
        );
        assert!(
            (split.pane_rects[1].rect[2] - split.pane_rects[0].rect[2]).abs() <= 1.0,
            "equal horizontal split"
        );
        assert_eq!(split.engine_sessions, 2, "no session recreated by split");
        assert_eq!(split.tabs[0].web_view, Some(WebViewId(1)));
        assert_eq!(split.tabs[1].web_view, Some(WebViewId(2)));
        // The focused pane is the browser command target and its rect is the
        // legacy `content_rect` (the focused/active slot, never the secondary).
        assert_eq!(
            split.content_rect, split.pane_rects[1].rect,
            "content_rect keeps representing the focused/active view"
        );
        assert_ids(&split, &["bbr-native-web-slot-1", "bbr-native-web-slot-2"]);

        // Selecting the other split member (tab 0 → view 1) switches the
        // focused pane without dissolving the split; both panes stay framed.
        client
            .send_ok(&dsl::select_tab(0))
            .expect("select primary pane");
        flush_frame(&mut client);
        let focused_primary = client.snapshot().expect("focused primary snapshot");
        assert_layout(
            &focused_primary,
            PaneLayoutSnapshot {
                primary: WebViewId(1),
                secondary: Some(WebViewId(2)),
                focused: WebViewId(1),
            },
        );
        assert_eq!(focused_primary.engine_sessions, 2);
        assert_eq!(
            focused_primary.content_rect,
            pane_rect_of(&focused_primary, WebViewId(1)),
            "content_rect follows the focused pane"
        );

        // A window resize reframes both pane ids independently: each pane
        // takes half of the new content width and the panes stay adjacent.
        let resized_w = window_w - 120.0;
        client
            .send_ok(&bbr_testkit::Request::SetWindowSize {
                width: resized_w,
                height: 700.0,
            })
            .expect("resize window");
        flush_frame(&mut client);
        let at_width = wait_for_window_size(&mut client, resized_w, deadline());
        let (primary_after, secondary_after) =
            split_rects(sidebar, at_width.window_size[0], height);
        let resized = wait_for_pane_rects(
            &mut client,
            &[
                (WebViewId(1), primary_after),
                (WebViewId(2), secondary_after),
            ],
            deadline(),
        );
        assert_ne!(
            resized.pane_rects[0].rect[2], split.pane_rects[0].rect[2],
            "the primary pane was reframed by the resize"
        );
        assert_ne!(
            resized.pane_rects[1].rect[2], split.pane_rects[1].rect[2],
            "the secondary pane was reframed independently"
        );
        assert_eq!(resized.engine_sessions, 2);
        // `window_size` is the GPUI/AppKit window bounds while pane rects are
        // native content-view geometry. A requested resize can change their
        // one-point titlebar delta, so the post-resize pane itself is the
        // authoritative content-height baseline (the sidebar suite follows
        // the same native-slot rule rather than equating it with bounds).
        let resized_content_height = resized.pane_rects[0].rect[3];
        assert!(resized_content_height > 0.0);
        assert_eq!(
            resized.pane_rects[1].rect[3], resized_content_height,
            "both native panes share the post-resize content height"
        );

        // CloseSplit returns to one full-width focused rect while both tabs
        // and warm sessions remain; no orphan geometry stays behind.
        client.send_ok(&dsl::close_split()).expect("close split");
        since = wait_browser_change(&mut client, BrowserChange::SplitClosed, since);
        flush_frame(&mut client);
        let closed_w = resized.window_size[0];
        let closed = wait_for_content_rect(
            &mut client,
            single_pane_rect(sidebar, closed_w, resized_content_height),
            deadline(),
        );
        assert_layout(
            &closed,
            PaneLayoutSnapshot {
                primary: WebViewId(1),
                secondary: None,
                focused: WebViewId(1),
            },
        );
        assert_eq!(closed.tabs.len(), 2, "both tabs stay open");
        assert_eq!(closed.engine_sessions, 2, "non-focused pane stays warm");
        assert_eq!(closed.tabs[0].web_view, Some(WebViewId(1)));
        assert_eq!(closed.tabs[1].web_view, Some(WebViewId(2)));
        assert_eq!(
            closed.pane_rects.len(),
            1,
            "no orphan geometry: only the focused pane publishes a rect"
        );
        assert_eq!(closed.pane_rects[0].view, WebViewId(1));
        assert!(
            !closed
                .visible_ids
                .iter()
                .any(|id| id == "bbr-native-web-slot-2"),
            "the hidden pane's keyed slot id disappears from the render"
        );
        // Single-pane content_rect remains the exact sidebar contract.
        assert_eq!(closed.content_rect[0], sidebar);
        assert_eq!(closed.content_rect[2], closed_w - sidebar);
        assert_eq!(closed.content_rect[3], resized_content_height);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
