//! Blueprint D05 across restart: a space deleted through the space menu's
//! confirmation sheet stays deleted after a graceful restart — the space
//! and its appearance record are gone — and its closed tabs survive on the
//! fallback space's recently-closed stack, so Reopen Closed Tab still
//! brings them back after the restart.

use std::path::Path;

use crate::*;

fn keep_root(result: std::thread::Result<()>, harness: &mut Harness, socket: &Path, token: &str) {
    match result {
        Ok(()) => harness.success = false,
        Err(panic) => {
            harness.diagnostics(socket, token);
            std::panic::resume_unwind(panic);
        }
    }
}

#[test]
#[ignore]
fn a_deleted_space_stays_deleted_and_its_tabs_stay_reopenable_after_restart() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("persist-space-delete", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let initial = wait_for_window_settled(&mut client, deadline());
        client
            .send_ok(&dsl::create_space(initial.active_profile, "Doomed"))
            .expect("create space");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = seq;
        let BrowserChange::SpaceCreated { space: doomed, .. } = created else {
            unreachable!()
        };
        client.send_ok(&dsl::click_space(doomed)).expect("activate");
        open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client
            .send_ok(&dsl::set_space_theme(doomed, "pine"))
            .expect("give the space an appearance record");

        client
            .send_ok(&bbr_testkit::Request::RightClickSpaceChip {
                space: doomed,
                x: 120.0,
                y: 600.0,
            })
            .expect("open the space menu");
        client
            .send_ok(&bbr_testkit::Request::TabMenuActivate { index: 4 })
            .expect("Delete Space…");
        client
            .send_ok(&bbr_testkit::Request::SpaceDeleteConfirm)
            .expect("confirm");
        let _ = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceDeleted { .. }),
            since,
        );
        let deleted = client.snapshot().expect("deleted snapshot");
        assert!(deleted.spaces.iter().all(|s| s.title != "Doomed"));

        client
            .send_ok(&bbr_testkit::Request::FlushBrowserState)
            .expect("flush browser state");
        client.quit().expect("graceful quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "graceful exit: {status}");
    }));
    keep_root(result, &mut harness, &socket, &token);
    drop(harness);

    let (mut harness2, socket2, token2) = spawn_harness("persist-space-delete", &page_one);
    let result2 = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket2);
        client.hello(&token2).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let restored = wait_for_window_settled(&mut client, deadline());
        assert!(
            matches!(restored.restore, RestoreDisposition::Restored { .. }),
            "restored from the store: {:?}",
            restored.restore
        );
        let titles: Vec<&str> = restored.spaces.iter().map(|s| s.title.as_str()).collect();
        assert_eq!(
            restored.spaces.len(),
            1,
            "the deleted space stays deleted: {titles:?}"
        );
        assert!(restored.spaces.iter().all(|s| s.title != "Doomed"));

        client
            .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                keystroke: "cmd-shift-t".into(),
            })
            .expect("reopen closed tab");
        let _ = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::TabReopened { .. }),
            since,
        );
        let reopened = client.snapshot().expect("reopened snapshot");
        assert!(
            reopened.tabs.iter().any(|tab| tab.url == page_two),
            "the deleted space's tab reopens in the fallback after restart"
        );
    }));
    finish(result2, &mut harness2, &socket2, &token2);
}
