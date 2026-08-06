//! Real-app dense-list E2E — edge states and the bounded-window drag tick
//! (Phase-4 dense slice, DESIGN.md §11.6/§10.9).
//!
//! Inside a dense collection: long/separator titles stay on one truncated
//! line, monogram favicon tiles cover missing icons, a crashed warm tab
//! renders the error state and drops exactly one session, narrow windows
//! clamp the rail while the dense region stays bounded, a pointer drag
//! renders the §10.9 insertion tick at an on-window regular slot through the
//! bounded render and commits a region-bounded reorder without touching
//! sessions, and closing the final tab lands on the DESIGN §11.8 empty
//! New-Tab surface.
//!
//! The drag goes through the production mouse-down/move/up listener methods
//! with synthetic events (the same seam the organize suite uses), never a
//! direct reorder command. Because the regular region renders only the
//! bounded window, the tick renders only for slots inside that window — a
//! drag whose target row is off-window shows no tick until auto-scroll lands
//! (documented boundary of bounded-window rendering; drag auto-scroll is
//! out of scope for the dense slice). No sleeps/pixels: every step
//! synchronizes on typed snapshot state under a deadline. Shared harness:
//! `e2e_common`; dense-only helpers: `e2e_dense_common`.

#![cfg(target_os = "macos")]

mod e2e_common;
mod e2e_dense_common;

use bbr_testkit::protocol::Request;
use e2e_common::*;
use e2e_dense_common::*;

/// §8 normal-density tab row height (theme.density::Normal.row()); the edge
/// collection is 120 identities (< 200 regular → N density).
const ROW_H: f32 = 32.0;

/// Edge states inside dense collections: long/separator titles stay on one
/// truncated line, monogram favicon tiles cover missing icons, a crashed
/// warm tab renders the error state and drops exactly one session, narrow
/// windows clamp the rail while the dense region stays bounded, a pointer
/// drag shows the §10.9 tick inside the bounded window and commits a
/// region-bounded reorder without touching sessions, and closing the final
/// tab lands on the DESIGN §11.8 empty New-Tab surface.
#[test]
#[ignore]
fn dense_edge_states_stay_usable() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("dense-edge", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let _replay = client.attach(None).expect("attach");

        // 120 identities (110 regular < 200 → N density), 20 live sessions.
        seed_dense(&mut client, &page_one, 120, 10, 20, 60);
        client
            .send_ok(&Request::EnforceSessionBudget { cap: 20 })
            .expect("enforce budget");
        flush(&mut client);
        let seeded = client.snapshot().expect("seeded snapshot");
        assert_dense_contract(&seeded, 120, 10, 20);
        assert_eq!(seeded.shell_modes.density, "normal");

        // Long + separator titles inside the dense list: a long title stays
        // on its one rendered line (the pixel-exact cut is GPUI's
        // `text_ellipsis` backstop), and a long title with an early
        // separator cuts deterministically at the separator so the site name
        // survives (§4/§11.9). Missing favicons fall back to the
        // deterministic monogram tile (§7.2) — the favicon id is present
        // even without a page icon.
        let mut since = 0;
        let long_url = fixtures_url_long(&page_one, "/long-title.html");
        let _long_view = open_tab(&mut client, &mut since, &long_url, Some(LONG_TITLE));
        let separator_url = fixtures_url_long(&page_one, "/separator-title.html");
        let _separator_view = open_tab(
            &mut client,
            &mut since,
            &separator_url,
            Some(SEPARATOR_TITLE),
        );
        flush(&mut client);
        let long = client.snapshot().expect("long title snapshot");
        let long_index = long.tabs.len() - 2;
        let separator_index = long.tabs.len() - 1;
        assert_eq!(long.tabs[long_index].title, LONG_TITLE.to_string());
        assert_eq!(
            long.row_titles[separator_index], SEPARATOR_ROW_TITLE,
            "the §4 separator-aware cut preserves the site name"
        );
        assert_ids(
            &long,
            &[
                &format!("bbr-tab-title-{long_index}"),
                &format!("bbr-tab-title-{separator_index}"),
                &format!("bbr-tab-favicon-{long_index}"),
                &format!("bbr-tab-favicon-{separator_index}"),
            ],
        );

        // Crash of a warm tab inside the dense list: the tab renders the
        // §10.7 error state, its session is destroyed (one fewer live), and
        // the identity + discarded count survive.
        let warm_index = 115;
        let warm_view = long.tabs[warm_index].web_view.expect("warm view");
        client
            .send_ok(&Request::InjectWebEvent {
                event: WebEvent::PageCrashed(warm_view),
            })
            .expect("inject crash");
        flush(&mut client);
        let crashed = client.snapshot().expect("crashed snapshot");
        assert!(crashed.tabs[warm_index].crashed, "crash flag survives");
        assert_eq!(crashed.tabs[warm_index].lifecycle, "discarded");
        // The two title-fixture tabs opened on top of the 20-session budget,
        // so the live population is 22 before the crash destroys exactly one.
        assert_eq!(crashed.engine_sessions, 21, "crash destroys one session");
        assert_eq!(crashed.lifecycle_counts.discarded, 1);
        assert_eq!(
            crashed.tab_states[warm_index],
            TabVisualState::Error,
            "the crashed row renders the §10.7 error state"
        );

        // Narrow window: the rail clamps toward the §5 floor while the dense
        // region keeps its bounded scroll container and the sessions stay.
        client
            .send_ok(&Request::SetWindowSize {
                width: 320.0,
                height: 700.0,
            })
            .expect("narrow window");
        flush(&mut client);
        let narrow = client.snapshot().expect("narrow snapshot");
        assert!(narrow.sidebar_width > 0.0, "rail stays above zero");
        assert!(
            narrow.dense.rendered_rows <= WINDOW_ROWS,
            "window still bounded"
        );
        assert_eq!(
            narrow.engine_sessions, 21,
            "narrow width never touches sessions"
        );

        // Drag inside the dense bounded window: scroll to the top so the
        // regular window covers rows 10..74, then drag the first regular row
        // (global 10) two rows down through the production pointer listeners.
        // The §10.9 tick renders at the on-window slot 12, the drop is legal
        // within the regular region, the release commits a region-bounded
        // reorder, and the session population is untouched.
        scroll_dense(&mut client, 1_000_000.0);
        flush(&mut client);
        let at_top = client.snapshot().expect("top before drag snapshot");
        assert_eq!(
            at_top.dense.scroll_anchor, 0,
            "the drag scenario starts at the top"
        );
        let dragged_url = at_top.tabs[10].url.clone();
        client
            .send_ok(&dsl::begin_tab_drag(10, 400.0, ROW_H / 2.0))
            .expect("begin dense drag");
        client
            .send_ok(&dsl::move_tab_drag(2.0 * ROW_H))
            .expect("move dense drag");
        let during = client.snapshot().expect("during dense drag snapshot");
        assert_eq!(during.drag_source, Some(10));
        assert_eq!(
            during.drag_slot,
            Some(12),
            "the tick resolves to an on-window regular slot"
        );
        assert!(during.drag_valid, "regular-region drop is legal");
        assert_ids(&during, &["bbr-drag-tick-12"]);
        client
            .send_ok(&dsl::end_tab_drag())
            .expect("end dense drag");
        let dragged = client.snapshot().expect("after dense drag snapshot");
        assert_eq!(
            dragged.engine_sessions, 21,
            "drag reorder never touches sessions"
        );
        assert_eq!(
            dragged.tabs[11].url, dragged_url,
            "the dragged row landed one slot down (insert before slot 12)"
        );

        // Empty space: closing every tab lands on the DESIGN §11.8 pane.
        while !client.snapshot().expect("closing snapshot").tabs.is_empty() {
            client
                .send_ok(&Request::CloseTab { index: 0 })
                .expect("close tab");
        }
        flush(&mut client);
        let empty = client.snapshot().expect("empty snapshot");
        assert_ids(&empty, &["bbr-empty-new-tab"]);
        assert_eq!(empty.engine_sessions, 0, "all sessions destroyed");
        assert_eq!(empty.dense.item_count, 0, "dense region reports empty");

        client.send_ok(&Request::Quit).expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success());
        let _ = page_two;
    }));

    finish(result, &mut harness, &socket, &token);
}
