//! D10 drawn pane illustrations over the real app (task packet
//! `v2-empty-states`): the offline load error, the crashed page and the
//! empty space each present their illustration when the pane appears, play
//! it (`playing`), reach `rest` on their final frame and then build no
//! animation element (no frames requested); under Reduce Motion a fresh
//! appearance starts at `rest`.

use std::time::{Duration, Instant};

use bbr_testkit::protocol::{ErrorSurfaceCategory, IllustrationSnapshot, Snapshot};
use bbr_web::web_errors::{IncidentSubReason, OfflineReason};

use crate::e2e_common::*;
use crate::support::*;

fn illustration(kind: &str, state: &str) -> Option<IllustrationSnapshot> {
    Some(IllustrationSnapshot {
        kind: kind.into(),
        state: state.into(),
    })
}

/// Poll (drawing frames) until the illustration reaches `rest`.
fn wait_rest(client: &mut Client, kind: &str) -> Snapshot {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        flush_frame(client);
        let snapshot = client.snapshot().expect("illustration snapshot");
        if snapshot.motion.illustration == illustration(kind, "rest") {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "{kind} never reached rest: {:?}",
            snapshot.motion.illustration
        );
        std::thread::yield_now();
    }
}

/// Animation elements a forced frame builds (zero: no frame requested).
fn frame_builds(client: &mut Client) -> u64 {
    let before = client
        .snapshot()
        .expect("before frame")
        .motion
        .animated_elements;
    flush_frame(client);
    client
        .snapshot()
        .expect("after frame")
        .motion
        .animated_elements
        - before
}

fn inject_offline(client: &mut Client, snapshot: &Snapshot, view: WebViewId) {
    client
        .send_ok(&dsl::inject_error_incident(
            snapshot.active_profile,
            view,
            IncidentSubReason::Offline(OfflineReason::NoNetworkConnection),
            None,
        ))
        .expect("inject offline incident");
}

#[test]
#[ignore]
fn pane_illustrations_play_two_cycles_then_rest() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (mut harness, mut client, _fixtures, socket, token) = spawn("errors-v2-illustrations");

    let result = catch_unwind(AssertUnwindSafe(|| {
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(false)))
            .expect("full motion");
        let before = client.snapshot().expect("pre-injection snapshot");
        let view = before.active_web_view.expect("active view");
        assert_eq!(before.motion.illustration, None, "a live page shows none");

        // Offline: plays, then rests and stops building animation elements.
        inject_offline(&mut client, &before, view);
        wait_surface(&mut client, view, ErrorSurfaceCategory::Offline);
        flush_frame(&mut client);
        let offline = client.snapshot().expect("offline snapshot");
        assert_eq!(
            offline.motion.illustration,
            illustration("offline", "playing")
        );
        assert!(
            frame_builds(&mut client) > 0,
            "the playing illustration animates"
        );
        wait_rest(&mut client, "offline");
        assert_eq!(
            frame_builds(&mut client),
            0,
            "at rest it requests no frames"
        );

        // Reduce Motion: a fresh appearance starts at rest.
        click_dismiss(&mut client);
        wait_surface_cleared(&mut client);
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(true)))
            .expect("reduce motion");
        let cleared = client.snapshot().expect("cleared snapshot");
        inject_offline(&mut client, &cleared, view);
        wait_surface(&mut client, view, ErrorSurfaceCategory::Offline);
        flush_frame(&mut client);
        let reduced = client.snapshot().expect("reduced offline snapshot");
        assert_eq!(reduced.motion.illustration, illustration("offline", "rest"));
        assert_eq!(frame_builds(&mut client), 0);
        click_dismiss(&mut client);
        wait_surface_cleared(&mut client);

        // Crashed: the heartbeat plays on the crash panel.
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(false)))
            .expect("full motion again");
        client
            .send_ok(&dsl::inject_web_event(WebEvent::PageCrashed(view)))
            .expect("inject crash");
        let until = Instant::now() + Duration::from_secs(30);
        let crashed = loop {
            flush_frame(&mut client);
            let snapshot = client.snapshot().expect("crash snapshot");
            if snapshot
                .visible_ids
                .iter()
                .any(|id| id == "bbr-crash-panel")
            {
                break snapshot;
            }
            assert!(Instant::now() < until, "the crash panel never presented");
            std::thread::yield_now();
        };
        assert_eq!(
            crashed.motion.illustration,
            illustration("crashed", "playing")
        );
        wait_rest(&mut client, "crashed");

        // Empty space: closing the crashed (last) tab shows the empty pane,
        // which plays from its own start.
        client
            .send_ok(&dsl::close_tab(0))
            .expect("close the last tab");
        let until = Instant::now() + Duration::from_secs(30);
        let empty = loop {
            flush_frame(&mut client);
            let snapshot = client.snapshot().expect("empty snapshot");
            if snapshot.tabs.is_empty() {
                break snapshot;
            }
            assert!(Instant::now() < until, "the space never emptied");
            std::thread::yield_now();
        };
        assert_ids(&empty, &["bbr-empty-new-tab"]);
        assert_eq!(
            empty.motion.illustration,
            illustration("empty-space", "playing")
        );
        wait_rest(&mut client, "empty-space");
        assert_eq!(frame_builds(&mut client), 0);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
