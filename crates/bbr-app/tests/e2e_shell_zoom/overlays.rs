//! Blueprint D08 (DESIGN §6, §18.4): the transient overlays over a real
//! WKWebView. ⌘+ shows the zoom readout at the new level; its plus steps
//! the zoom through the production path; it leaves after 1.2 s. ⌘D shows
//! the "Saved to Library" toast; Undo removes the record again; a toast
//! leaves on its own after 4 s. While only overlays show, the Portal runs
//! in its hit-region mode: it captures the pointer over an overlay and lets
//! it through to the page elsewhere (the monitor's decision, routed at
//! explicit points without moving the user's cursor). Every step waits on
//! typed snapshot state under a deadline.

use crate::e2e_common::*;
use crate::support::*;

fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("keystroke");
}

fn wait(
    client: &mut Client,
    within: Duration,
    what: &str,
    ready: impl Fn(&bbr_testkit::protocol::Snapshot) -> bool,
) -> bbr_testkit::protocol::Snapshot {
    let until = Instant::now() + within;
    loop {
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush main and portal frames");
        let snapshot = client.snapshot().expect("overlay snapshot");
        if ready(&snapshot) {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "{what} never settled: readout {:?} toast {:?} visible {} native {} regions {:?}",
            snapshot.zoom_readout,
            snapshot.toast,
            snapshot.portal.visible,
            snapshot.portal.native_visible,
            snapshot.portal.hit_regions
        );
    }
}

/// Whether the Portal captures the pointer at `point` (the hit-region
/// decision, applied and read back from the native panel).
fn captures(client: &mut Client, point: [f32; 2]) -> bool {
    client
        .send_ok(&bbr_testkit::Request::PortalRoutePointer {
            x: point[0],
            y: point[1],
        })
        .expect("route pointer");
    !client
        .snapshot()
        .expect("routed snapshot")
        .portal
        .ignores_mouse_events
}

fn centre(rect: [f32; 4]) -> [f32; 2] {
    [rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0]
}

#[test]
#[ignore]
fn zoom_readout_and_toast_confirm_without_blocking_the_page() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let fixtures = zoom_fixtures();
    let (mut harness, socket, token) = spawn_harness("overlays", &fixtures.a1);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let _ = start(&mut client, PAGE_ONE_TITLE, since);
        let card = client.snapshot().expect("card").content_rect;
        let page = centre(card);

        // ⌘+: the readout confirms the new level at the top centre of the
        // card.
        client
            .send_ok(&bbr_testkit::Request::ZoomIn)
            .expect("zoom in");
        let readout = wait(&mut client, Duration::from_secs(5), "readout", |s| {
            s.zoom_readout
                .is_some_and(|level| level == u32::from(s.zoom.level))
                && s.portal.native_visible
        });
        for id in [
            "bbr-zoom-readout",
            "bbr-zoom-readout-plus",
            "bbr-zoom-readout-level",
        ] {
            assert!(readout.portal.ids.iter().any(|drawn| drawn == id), "{id}");
        }
        let regions = readout.portal.hit_regions.clone().expect("hit-region mode");
        assert_eq!(regions.len(), 1, "only the readout captures");
        let [rx, ry, rw, rh] = regions[0];
        assert_eq!((rw, rh), (120.0, 32.0));
        assert_eq!(ry, f64::from(card[1]) + 24.0, "24 pt below the card top");
        assert!((rx + rw / 2.0 - f64::from(centre(card)[0])).abs() < 0.5);
        assert!(captures(&mut client, [rx as f32 + 60.0, ry as f32 + 16.0]));
        assert!(!captures(&mut client, page), "the page takes the pointer");
        // It leaves 1.2 s after the step; the Portal hides with it.
        let gone = wait(&mut client, Duration::from_secs(5), "readout exit", |s| {
            s.zoom_readout.is_none()
        });
        assert!(!gone.portal.native_visible || gone.portal.presentation.is_some());

        // ⌘D: the toast confirms the save; it takes the pointer only over
        // itself.
        let saved_before = client.snapshot().expect("before").store.saved_pages_count;
        key(&mut client, "cmd-d");
        let toast = wait(&mut client, Duration::from_secs(5), "toast", |s| {
            s.toast.as_ref().is_some_and(|t| t.rect.is_some())
                && s.portal
                    .hit_regions
                    .as_ref()
                    .is_some_and(|regions| regions.len() == 1)
        });
        let live = toast.toast.clone().expect("toast");
        assert_eq!(live.message, "Saved to Library");
        assert_eq!(live.action.as_deref(), Some("Undo"));
        assert_eq!(toast.store.saved_pages_count, saved_before + 1);
        for id in ["bbr-toast", "bbr-toast-message", "bbr-toast-action"] {
            assert!(toast.portal.ids.iter().any(|drawn| drawn == id), "{id}");
        }
        let rect = live.rect.expect("measured toast");
        // Bottom-centre, 24 pt above the card's bottom edge, 40 pt tall.
        assert_eq!(rect[3], 40.0);
        assert_eq!(rect[1] + rect[3], card[1] + card[3] - 24.0);
        assert!((centre(rect)[0] - centre(card)[0]).abs() < 0.5);
        assert!(captures(&mut client, centre(rect)));
        assert!(!captures(&mut client, page));

        // Undo removes the saved page and the toast at once.
        client
            .send_ok(&bbr_testkit::Request::ToastAction)
            .expect("undo");
        let undone = wait(&mut client, Duration::from_secs(5), "undo", |s| {
            s.toast.is_none()
        });
        assert_eq!(undone.store.saved_pages_count, saved_before);

        // A toast leaves on its own after 4 s (the record stays).
        key(&mut client, "cmd-d");
        let again = wait(&mut client, Duration::from_secs(5), "second toast", |s| {
            s.toast.is_some()
        });
        assert_eq!(again.store.saved_pages_count, saved_before + 1);
        let left = wait(&mut client, Duration::from_secs(8), "toast exit", |s| {
            s.toast.is_none()
        });
        assert_eq!(left.store.saved_pages_count, saved_before + 1);
        assert!(!left.portal.native_visible || left.portal.presentation.is_some());

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
