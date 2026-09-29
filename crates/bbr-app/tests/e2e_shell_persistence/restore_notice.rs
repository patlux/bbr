//! Blueprint D17 (board L-10, DESIGN §11.8): a restored session says what
//! came back, once. The notice floats over the card top through the
//! Portal's overlay layer (it takes the pointer only over itself), and it
//! leaves on the first navigation away from the restored page or on
//! Dismiss.

use std::path::Path;

use crate::*;

/// Keep the profile root for the relaunch (the harness keeps it on a
/// non-success drop).
fn keep_root(result: std::thread::Result<()>, harness: &mut Harness, socket: &Path, token: &str) {
    match result {
        Ok(()) => harness.success = false,
        Err(panic) => {
            harness.diagnostics(socket, token);
            std::panic::resume_unwind(panic);
        }
    }
}

fn notice(client: &mut Client) -> bbr_testkit::protocol::Snapshot {
    let until = deadline();
    loop {
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush");
        let snapshot = client.snapshot().expect("notice snapshot");
        if snapshot.restore_notice.is_some() && snapshot.portal.hit_regions.is_some() {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "no restore notice: {:?} regions {:?} restore {:?} tabs {}",
            snapshot.restore_notice,
            snapshot.portal.hit_regions,
            snapshot.restore,
            snapshot.tabs.len()
        );
    }
}

fn wait_gone(client: &mut Client) {
    let until = deadline();
    loop {
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush");
        if client
            .snapshot()
            .expect("gone snapshot")
            .restore_notice
            .is_none()
        {
            return;
        }
        assert!(Instant::now() < until, "the restore notice stayed");
    }
}

#[test]
#[ignore]
fn a_restored_session_says_what_came_back_until_the_first_navigation() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("persist-restore-notice", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let fresh = wait_for_window_settled(&mut client, deadline());
        assert_eq!(
            fresh.restore_notice, None,
            "a fresh start announces nothing"
        );
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client
            .send_ok(&bbr_testkit::Request::FlushBrowserState)
            .expect("flush browser state");
        client.quit().expect("graceful quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "graceful exit: {status}");
    }));
    keep_root(result, &mut harness, &socket, &token);
    drop(harness);

    // Relaunch: the notice counts the restore and never blocks the page.
    let (mut harness2, socket2, token2) = spawn_harness("persist-restore-notice", &page_one);
    let result2 = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket2);
        client.hello(&token2).expect("hello");
        let _ = client.attach(None).expect("attach");
        let _ = wait_for_window_settled(&mut client, deadline());
        let shown = notice(&mut client);
        assert_eq!(
            shown.restore_notice.as_deref(),
            Some("Restored 1 space and 2 tabs from your last session.")
        );
        for id in ["bbr-restore-notice", "bbr-restore-notice-dismiss"] {
            assert!(shown.portal.ids.iter().any(|drawn| drawn == id), "{id}");
        }
        let regions = shown.portal.hit_regions.clone().expect("hit-region mode");
        assert_eq!(regions.len(), 1, "only the notice takes the pointer");
        let card = shown.content_rect;
        assert_eq!(regions[0][0], f64::from(card[0]) + 20.0);
        assert_eq!(regions[0][1], f64::from(card[1]) + 14.0);

        // The first navigation away from the restored page ends it.
        client
            .send_ok(&dsl::navigate(format!("{page_one}?away")))
            .expect("navigate away");
        wait_gone(&mut client);
        client.quit().expect("quit");
        let status = wait_for_exit(
            &mut harness2.child,
            Instant::now() + Duration::from_secs(10),
        );
        assert!(status.success(), "graceful exit: {status}");
    }));
    keep_root(result2, &mut harness2, &socket2, &token2);
    drop(harness2);

    // Relaunch again: Dismiss ends it at once.
    let (mut harness3, socket3, token3) = spawn_harness("persist-restore-notice", &page_one);
    let result3 = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket3);
        client.hello(&token3).expect("hello");
        let _ = client.attach(None).expect("attach");
        let _ = notice(&mut client);
        client
            .send_ok(&bbr_testkit::Request::DismissRestoreNotice)
            .expect("dismiss");
        wait_gone(&mut client);
        client.quit().expect("quit");
        let status = wait_for_exit(
            &mut harness3.child,
            Instant::now() + Duration::from_secs(10),
        );
        assert!(status.success(), "graceful exit: {status}");
    }));
    finish(result3, &mut harness3, &socket3, &token3);
}
