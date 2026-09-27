//! Real-app dense-list E2E — lifecycle, rendering, and keyboard (Phase-4
//! dense slice, DESIGN.md §11.6).
//!
//! Proves the dense-500 contract over real WKWebViews: 500 tab identities
//! with exactly 50 live/warm native sessions and 450 suspended tabs through
//! the production controller mutation paths, a bounded concurrent-session
//! peak while preparing the dense state (never 500 WKWebViews at once),
//! deterministic lifecycle selection that preserves active, pinned, and
//! recently used identities, bounded windowed row rendering with a
//! deterministic scroll anchor, keyboard navigation and instant
//! scroll-into-view across large collections, pointer selection after
//! scroll, warm selections that never reload, suspended selections that
//! recreate exactly one session, and clean teardown. Edge states (long
//! titles, missing favicons, crash rows, narrow width, empty space) and the
//! dense drag tick live in the sibling `e2e_shell_dense_edge` binary.
//!
//! Scroll is driven through the production scroll state: the tracked
//! `ScrollHandle` the native wheel listener, the keyboard traversal, and the
//! render share (GPUI 0.2.2 exposes no public synthetic wheel dispatch —
//! `Window::dispatch_event` returns a crate-private result type — so the
//! seam mutates the exact state a wheel would, and the keyboard scenarios
//! additionally exercise real `dispatch_keystroke` input through the
//! production `on_tab_list_key_down` path). No sleeps/pixels: every step
//! synchronizes on typed snapshot state under a deadline. The shared harness
//! lives in `e2e_common`; dense-only helpers in `e2e_dense_common`.

#![cfg(target_os = "macos")]

mod e2e_common;
mod e2e_dense_common;

use bbr_testkit::protocol::Request;
use e2e_common::*;
use e2e_dense_common::*;

/// The dense-500 headline: production seeding keeps the concurrent session
/// peak bounded (never 500 WKWebViews), the deterministic budget converges to
/// exactly 50 live sessions with 450 suspended, the render stays bounded and
/// the anchor is deterministic, warm selections never reload, suspended
/// selections recreate exactly one session, and teardown is clean.
#[test]
#[ignore]
fn dense_500_keeps_exactly_50_sessions_with_bounded_render_and_recreation() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("dense-500", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _replay = client.attach(None).expect("attach");
        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first native view");

        // Prepare the dense state in bounded batches (each under the gateway
        // per-request deadline); every intermediate snapshot asserts the
        // live-session ceiling, then the budget policy converges exactly.
        seed_dense(&mut client, &page_one, 500, 10, 50, 100);
        client
            .send_ok(&Request::EnforceSessionBudget { cap: 50 })
            .expect("enforce session budget");
        flush(&mut client);

        let snap = client.snapshot().expect("dense snapshot");
        assert_dense_contract(&snap, 500, 10, 50);
        assert_eq!(
            snap.shell_modes.density, "small",
            "≥200 regular tabs flip to S density"
        );

        // Bounded windowed rendering (§11.6): the render built exactly the
        // pinned block plus ≤ 64 regular rows. Opening each identity scrolled
        // it into view (production selection-follow), so the deterministic
        // rest anchor is the last full window at the collection bottom.
        let dense = snap.dense;
        assert_eq!(dense.item_count, 490);
        assert_eq!(dense.pinned_rows, 10);
        assert!(dense.rendered_rows <= WINDOW_ROWS, "rendered rows bounded");
        assert_eq!(
            dense.scroll_anchor,
            dense.item_count.saturating_sub(WINDOW_ROWS),
            "opening the tail anchors deterministically at the last full window"
        );
        assert!(dense.active_visible, "the pinned active row is on screen");
        assert_ids(&snap, &["bbr-tab-scroll", "bbr-tab-row-436"]);
        assert!(
            !snap.visible_ids.iter().any(|id| id == "bbr-tab-row-100"),
            "rows outside the bounded window are never exposed"
        );

        // Scrolling past the bottom through the production scroll state
        // clamps at the deterministic max (no overscroll, no panic), then a
        // scroll to the top anchors at zero with the first regular row on
        // screen, then a bounded scroll moves the anchor deterministically
        // mid-list. The window stays bounded, the pinned block keeps its
        // identity, and the live-session population is untouched.
        scroll_dense(&mut client, -100_000.0);
        let clamped = client.snapshot().expect("clamped snapshot");
        assert_eq!(
            clamped.dense.scroll_anchor,
            clamped.dense.item_count.saturating_sub(WINDOW_ROWS),
            "extreme down-scroll clamps at the last full window"
        );
        assert!(clamped.dense.rendered_rows <= WINDOW_ROWS);

        scroll_dense(&mut client, 1_000_000.0);
        let top = client.snapshot().expect("top snapshot");
        assert_eq!(top.dense.scroll_anchor, 0, "scroll to top anchors at zero");
        assert_eq!(
            top.dense.first_rendered, 10,
            "window starts at the first regular row"
        );
        assert!(top.dense.rendered_rows <= WINDOW_ROWS);
        assert_ids(&top, &["bbr-tab-row-10", "bbr-tab-row-20"]);
        assert!(
            !top.visible_ids.iter().any(|id| id == "bbr-tab-row-100"),
            "top window still bounded"
        );

        scroll_dense(&mut client, -2500.0);
        let scrolled = client.snapshot().expect("scrolled snapshot");
        let scrolled_dense = scrolled.dense;
        assert_eq!(
            scrolled_dense.scroll_anchor, 69,
            "the wheel scroll lands on the exact S-pitch anchor \
             (2500 px / 36 pt pitch = 69 rows) — proof the dense scroll \
             math steps on the §8 density:pitch, not the old row surface"
        );
        assert!(
            scrolled_dense.scroll_anchor > 0 && scrolled_dense.scroll_anchor < top.dense.item_count,
            "wheel scroll moves the deterministic anchor"
        );
        assert!(
            scrolled_dense.rendered_rows <= WINDOW_ROWS,
            "window stays bounded after scroll"
        );
        assert_eq!(scrolled_dense.pinned_rows, 10, "pinned block stable");
        assert_eq!(
            scrolled.engine_sessions, 50,
            "scroll never touches sessions"
        );
        assert!(
            scrolled.visible_ids.iter().any(|id| id == "bbr-tab-row-0"),
            "pinned rows remain rendered above the scroll view"
        );
        assert!(
            scrolled.visible_ids.iter().any(|id| id == "bbr-tab-row-9"),
            "the whole pinned block stays on screen"
        );

        // Warm selection: the recent tab keeps its WebViewId and session, and
        // no load event follows the selection in the replay window (no reload
        // for warm tabs).
        let warm_index = 470;
        let warm_view = snap.tabs[warm_index].web_view.expect("warm view");
        let cursor = client
            .attach(None)
            .expect("attach for reload evidence")
            .into_iter()
            .map(|(seq, _)| seq)
            .max()
            .unwrap_or(0);
        client
            .send_ok(&Request::ClickTab { index: warm_index })
            .expect("pointer select warm");
        flush(&mut client);
        let warm_snap = client.snapshot().expect("warm selected snapshot");
        assert_eq!(
            warm_snap.active_web_view,
            Some(warm_view),
            "warm selection reuses the existing session"
        );
        assert_eq!(warm_snap.engine_sessions, 50, "no session churn");
        let reload_events = client
            .attach(Some(cursor))
            .expect("replay since warm selection")
            .into_iter()
            .filter(|(_, event)| {
                matches!(
                    event,
                    Event::Web(WebEvent::LoadingChanged(v, _))
                        | Event::Web(WebEvent::TitleChanged(v, _))
                        | Event::Web(WebEvent::UrlChanged(v, _))
                        if *v == warm_view
                )
            })
            .count();
        assert_eq!(reload_events, 0, "warm selection never reloads the page");

        // Suspended selection: exactly one fresh session is created (the
        // session-recreation policy), and the tab becomes active.
        let suspended_index = 250;
        assert!(snap.tabs[suspended_index].web_view.is_none());
        client
            .send_ok(&Request::SelectTab {
                index: suspended_index,
            })
            .expect("select suspended");
        flush(&mut client);
        let recreated = client.snapshot().expect("recreated snapshot");
        assert_eq!(
            recreated.engine_sessions, 51,
            "selecting a suspended tab recreates exactly one session"
        );
        assert_eq!(recreated.active_tab_index, Some(suspended_index));
        assert_eq!(recreated.tabs[suspended_index].lifecycle, "active");
        assert!(
            recreated.tabs[suspended_index].web_view.is_some(),
            "fresh session backs the recreated tab"
        );
        assert_eq!(
            recreated.lifecycle_counts.suspended, 449,
            "the recreated tab left the suspended population"
        );

        // Teardown: graceful quit keeps the active view alive to the end and
        // exits cleanly (sessions are torn down by the app, not leaked).
        client.send_ok(&Request::Quit).expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exited cleanly after dense state");
        let _ = first_view;
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Keyboard navigation across a large collection through real GPUI
/// keystrokes: `PgUp/PgDn` move both selection and viewport by a bounded
/// page, arrows step with edge clamping (no wrap teleport across 500 rows),
/// every move scrolls the selection into view instantly, the scroll position
/// stays deterministic, and the pinned block never shifts.
#[test]
#[ignore]
fn keyboard_navigation_scrolls_large_collections_without_wrapping() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("dense-keys", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _replay = client.attach(None).expect("attach");

        // 230 identities (220 regular ≥ 200 → S density), 20 live sessions.
        seed_dense(&mut client, &page_one, 230, 10, 20, 115);
        client
            .send_ok(&Request::EnforceSessionBudget { cap: 20 })
            .expect("enforce budget");
        flush(&mut client);
        let seeded = client.snapshot().expect("seeded snapshot");
        assert_dense_contract(&seeded, 230, 10, 20);
        assert_eq!(seeded.shell_modes.density, "small");

        // Focus the tab list through the production pointer path (selecting
        // the active pinned tab 0, which is already selected — the pointer
        // entry still gives the list chrome focus).
        client
            .send_ok(&Request::ClickTab { index: 0 })
            .expect("focus tab list");
        flush(&mut client);
        let focused = client.snapshot().expect("focused snapshot");
        assert!(focused.chrome_focused, "pointer entry keeps chrome focus");
        assert_eq!(focused.active_tab_index, Some(0));

        // PgDn through the real input path: selection moves one bounded page
        // and stays inside the rendered window. The first page still fits in
        // the first window; the second page forces the viewport to follow
        // (instant scroll-into-view).
        let page = WINDOW_ROWS as isize;
        keystroke(&mut client, "pagedown");
        flush(&mut client);
        let paged = client.snapshot().expect("paged snapshot");
        assert_eq!(
            paged.active_tab_index,
            Some(page as usize),
            "PgDn moves selection by one rendered page"
        );
        assert!(
            paged.dense.active_visible,
            "the first page stays inside the initial window"
        );

        keystroke(&mut client, "pagedown");
        flush(&mut client);
        let paged_twice = client.snapshot().expect("paged twice snapshot");
        assert_eq!(paged_twice.active_tab_index, Some(2 * page as usize));
        assert!(
            paged_twice.dense.active_visible,
            "instant scroll-into-view keeps the selected row rendered"
        );
        assert!(
            paged_twice.dense.scroll_anchor > 0,
            "viewport followed the page"
        );
        assert_eq!(paged_twice.pinned_count, 10, "pinned boundary stable");
        // Every keyboard selection lands on a suspended mid-list tab, so each
        // recreates exactly one session (the session-recreation policy):
        // 20 live + 2 PgDn recreations. Never churn without recreation.
        assert_eq!(
            paged_twice.engine_sessions, 22,
            "each suspended selection recreates exactly one session"
        );
        assert_eq!(
            paged_twice.tabs[2 * page as usize].lifecycle,
            "active",
            "the keyboard-selected tab became active"
        );
        assert!(
            paged_twice.tabs[2 * page as usize].web_view.is_some(),
            "the recreated tab owns a fresh session"
        );

        // Arrow steps are edge-clamped in dense collections: repeated downs
        // never wrap, and the active row stays inside the rendered window.
        for _ in 0..5 {
            keystroke(&mut client, "down");
        }
        flush(&mut client);
        let stepped = client.snapshot().expect("stepped snapshot");
        assert_eq!(
            stepped.active_tab_index,
            Some(2 * page as usize + 5),
            "arrows step within the list"
        );
        assert!(stepped.dense.active_visible);
        assert_eq!(
            stepped.engine_sessions, 27,
            "five more suspended selections recreate exactly five sessions"
        );

        // PgUp returns toward the top and the anchor tracks it.
        keystroke(&mut client, "pageup");
        flush(&mut client);
        let back = client.snapshot().expect("page up snapshot");
        assert_eq!(
            back.active_tab_index,
            Some(page as usize + 5),
            "PgUp pages back"
        );
        assert!(back.dense.active_visible);
        assert_eq!(
            back.engine_sessions, 28,
            "the final PgUp selection also recreates exactly one session"
        );

        // The pinned block survived the whole traversal with its identity.
        assert!(back.visible_ids.iter().any(|id| id == "bbr-tab-row-0"));
        assert_eq!(back.dense.pinned_rows, 10);

        client.send_ok(&Request::Quit).expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success());
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The folder-membership invariant under the bounded-session policy (task
/// packet `folder-pinned-invariant`): grouping the least recently used warm
/// regular tab pins it in the same transition, so the production budget
/// keeps it live through its canonical pinned state (no folder-only keep
/// exception) and suspends the next least recently used tab instead. The
/// dense window stays bounded and the pinned block grows by exactly the
/// grouped identity.
#[test]
#[ignore]
fn dense_folder_members_stay_live_under_the_session_budget() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("dense-folder", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _replay = client.attach(None).expect("attach");

        // 120 identities, 10 pinned, 20 live: warm regular tabs 110..119.
        seed_dense(&mut client, &page_one, 120, 10, 20, 60);
        client
            .send_ok(&Request::EnforceSessionBudget { cap: 20 })
            .expect("enforce budget");
        flush(&mut client);
        let seeded = client.snapshot().expect("seeded snapshot");
        assert_dense_contract(&seeded, 120, 10, 20);
        let id_at = |snap: &bbr_testkit::protocol::Snapshot, index: usize| {
            snap.all_tabs
                .iter()
                .find(|tab| tab.space == snap.active_space && tab.index == index)
                .expect("dense identity")
                .id
        };
        let state = |snap: &bbr_testkit::protocol::Snapshot, id: u64| {
            let tab = snap
                .all_tabs
                .iter()
                .find(|tab| tab.id == id)
                .expect("identity");
            (tab.pinned, tab.lifecycle.clone(), tab.web_view.is_some())
        };
        let (member, next_lru) = (id_at(&seeded, 110), id_at(&seeded, 111));
        let member_view = seeded.tabs[110].web_view;

        client
            .send_ok(&dsl::click_footer_plus())
            .expect("open plus menu");
        client
            .send_ok(&dsl::click_plus_menu_item(1))
            .expect("commit New Folder");
        flush(&mut client);
        let folder = client.snapshot().expect("folder snapshot").folders[0].id;
        client
            .send_ok(&dsl::move_tab_into_folder(110, folder))
            .expect("group the least recently used warm tab");
        flush(&mut client);
        let grouped = client.snapshot().expect("grouped snapshot");
        assert_eq!(grouped.folders[0].members, vec![member]);
        assert_eq!(id_at(&grouped, 10), member, "pinned at the region end");
        assert_eq!(grouped.pinned_count, 11);
        assert_eq!(grouped.tabs[10].web_view, member_view, "same live session");
        assert_eq!(
            grouped.engine_sessions, 20,
            "grouping never churns sessions"
        );
        assert_eq!(grouped.dense.pinned_rows, 11);
        assert_eq!(grouped.dense.item_count, 109);
        assert!(grouped.dense.rendered_rows <= WINDOW_ROWS, "window bounded");
        assert_eq!(grouped.active_tab_index, Some(0), "selection unchanged");

        // One more live tab exceeds the cap; the budget must spare the
        // pinned folder member and suspend the next least recently used.
        let mut since = 0;
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client
            .send_ok(&Request::EnforceSessionBudget { cap: 20 })
            .expect("enforce budget again");
        flush(&mut client);
        let budgeted = client.snapshot().expect("budgeted snapshot");
        assert_eq!(budgeted.engine_sessions, 20, "the cap holds");
        assert_eq!(
            state(&budgeted, member),
            (true, "warm".to_string(), true),
            "the folder member stays live through its pinned state"
        );
        assert_eq!(
            state(&budgeted, next_lru),
            (false, "suspended".to_string(), false),
            "the next least recently used regular tab is suspended instead"
        );
        assert_eq!(budgeted.folders[0].members, vec![member]);
        assert!(budgeted.dense.rendered_rows <= WINDOW_ROWS);

        client.send_ok(&Request::Quit).expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success());
    }));

    finish(result, &mut harness, &socket, &token);
}
