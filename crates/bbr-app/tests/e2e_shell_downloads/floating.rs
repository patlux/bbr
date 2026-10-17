//! Exercise the Downloads fade while the Portal composes the floating rail.
//! The browser owns animation scheduling; the Portal only reads that frame.
//! This uses ordinary `testkit`, not the visual-capture executor configuration.

use bbr_testkit::protocol::{Request, Snapshot};
use bbr_testkit::Client;

use crate::e2e_common::*;
use crate::support::*;

fn frame(client: &mut Client) -> Snapshot {
    flush_frame(client);
    client.snapshot().expect("frame snapshot")
}

fn wait_floating(client: &mut Client) -> Snapshot {
    let until = deadline();
    loop {
        let snapshot = frame(client);
        if snapshot.rail_reveal == "floating" && snapshot.portal.native_visible {
            assert!(snapshot.floating_rail.is_some());
            assert!(!snapshot.sidebar_visible);
            return snapshot;
        }
        assert!(Instant::now() < until, "floating rail never presented");
    }
}

fn wait_painted(client: &mut Client, open: bool) -> Snapshot {
    let until = deadline();
    loop {
        let snapshot = frame(client);
        assert_eq!(snapshot.downloads.open, open);
        if open {
            if let Some(paint) = snapshot.downloads_paint.filter(|paint| paint.alpha == 1.0) {
                let rail = snapshot
                    .floating_rail
                    .as_ref()
                    .expect("floating geometry")
                    .rect;
                let [x, y, width, height] = paint.rect;
                assert!(paint.rect.into_iter().all(f32::is_finite));
                // Use the actual laid-out interior, not a second width model.
                assert!(
                    width > rail[2] / 2.0,
                    "collapsed Downloads paint: {paint:?}; rail={rail:?}"
                );
                assert!(height > 0.0);
                assert!(x >= rail[0] && y >= rail[1], "{paint:?}; rail={rail:?}");
                assert!(
                    x + width <= rail[0] + rail[2] && y + height <= rail[1] + rail[3],
                    "{paint:?}; rail={rail:?}"
                );
                eprintln!("floating Downloads painted: {paint:?}; rail={rail:?}");
                return snapshot;
            }
        } else if snapshot.downloads_paint.is_none() {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "Downloads paint endpoint absent: {:?}",
            snapshot.downloads_paint
        );
    }
}

fn same_browser(before: &Snapshot, after: &Snapshot) {
    assert_eq!(after.tabs, before.tabs);
    assert_eq!(after.active_tab_index, before.active_tab_index);
    assert_eq!(after.active_web_view, before.active_web_view);
    assert_eq!(after.engine_sessions, before.engine_sessions);
    assert_eq!(after.active_profile, before.active_profile);
    assert_eq!(after.active_space, before.active_space);
    assert_eq!(after.content_rect, before.content_rect);
    assert!(!after.sidebar_visible);
    assert!(after.floating_rail.is_some());
}

#[test]
#[ignore]
fn downloads_open_close_reopen_in_the_floating_rail_preserves_browser_identity() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, _) = fixtures();
    let (mut harness, socket, token) = spawn_visible_harness("downloads-floating", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        // §11.4: exercise the production-default slide (the VM guest runs
        // with Reduce Motion on).
        full_motion(&mut client);
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let settled = wait_for_window_settled(&mut client, deadline());
        if settled.title.as_deref() != Some(PAGE_ONE_TITLE) {
            wait_title(
                &mut client,
                settled.active_web_view.expect("active view"),
                PAGE_ONE_TITLE,
                since,
            );
        }
        keystroke(&mut client, "cmd-s");
        // §11.4: the leaving rail never floats; reveal once it settled.
        assert!(!wait_sidebar_settled(&mut client).sidebar_visible);
        for (x, time_ms) in [(500.0, 0.0), (4.0, 40.0)] {
            client
                .send_ok(&Request::RailPointer {
                    x,
                    y: 400.0,
                    time_ms,
                    pressed: false,
                })
                .expect("reveal floating rail");
        }
        let baseline = wait_floating(&mut client);
        assert!(!baseline.downloads.open);

        // Both completed endpoints and a fresh reopen: composing during each
        // transition must never request a frame on the non-drawing browser.
        for _ in 0..2 {
            open_downloads(&mut client);
            let opened = wait_painted(&mut client, true);
            assert_ids(&opened, &["bbr-downloads-panel", "bbr-downloads-empty"]);
            assert!(opened.chrome_focused);
            same_browser(&baseline, &opened);
            assert!(wait_floating(&mut client).downloads.open);

            keystroke(&mut client, "escape");
            let closed = wait_painted(&mut client, false);
            assert!(!closed.downloads.open);
            assert!(closed.chrome_focused, "footer receives focus");
            same_browser(&baseline, &closed);
            assert!(!wait_floating(&mut client).downloads.open);
        }

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
