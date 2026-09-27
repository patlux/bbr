//! Folder-membership invariant across restore (task packet
//! `folder-pinned-invariant`): grouping pins a tab through the production
//! path, the pin is a structural mutation the browser-state writer commits,
//! and both a graceful restart and a force quit restore the grouped tab as
//! pinned. Folders are session-local, so a restore never recreates a
//! membership — no restored state can hold an unpinned folder member — and
//! grouping a deferred identity never hydrates its session.

use std::path::Path;

use crate::*;

/// Create a folder through the production plus-menu commit and return its
/// domain id.
fn new_folder(client: &mut Client) -> u64 {
    client
        .send_ok(&dsl::click_footer_plus())
        .expect("open plus menu");
    client
        .send_ok(&dsl::click_plus_menu_item(1))
        .expect("commit New Folder");
    client
        .send_ok(&bbr_testkit::Request::FlushFrame)
        .expect("flush after folder");
    let snapshot = client.snapshot().expect("folder snapshot");
    snapshot.folders.last().expect("folder created").id
}

fn pinned(snapshot: &bbr_testkit::protocol::Snapshot, id: u64) -> bool {
    snapshot
        .all_tabs
        .iter()
        .find(|tab| tab.id == id)
        .expect("identity survives")
        .pinned
}

/// Keep the profile directory for the next launch (a successful finish lets
/// the harness Drop remove the root) and rethrow a failed leg.
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
fn grouping_pins_durably_across_graceful_and_force_quit_restore() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let pid = std::process::id();
    let profile = harness_root("persist-folder", pid).join("profile");
    // [100 pinned, 101 page one (active), 102 page two].
    seed_browser_state(&profile, &reorder_browser_state(&page_one, &page_two));

    // Leg 1: group the deferred regular identity, flush, quit gracefully.
    let (mut harness, socket, token) = spawn_harness("persist-folder", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("restored active view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            wait_title(&mut client, first_view, PAGE_ONE_TITLE, 0);
        }
        assert_restored_chrome_first(&client.snapshot().expect("restored"), 1, 2);

        let folder = new_folder(&mut client);
        client
            .send_ok(&dsl::move_tab_into_folder(2, folder))
            .expect("group the deferred tab");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after grouping");
        let grouped = client.snapshot().expect("grouped snapshot");
        assert_eq!(grouped.folders[0].members, vec![102]);
        assert!(pinned(&grouped, 102), "grouping pins the member");
        assert_eq!(grouped.pinned_count, 2);
        assert_eq!(
            grouped.engine_sessions, 1,
            "grouping a deferred identity never hydrates its session"
        );
        client
            .send_ok(&bbr_testkit::Request::FlushBrowserState)
            .expect("flush browser state");
        client.quit().expect("graceful quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "graceful exit: {status}");
    }));
    keep_root(result, &mut harness, &socket, &token);
    drop(harness);

    // Graceful restore: the grouped tab returns pinned; the session-local
    // folder is gone, so no membership (and no unpinned member) restores.
    // Leg 2: group the live regular tab, flush, force quit.
    let (mut harness2, socket2, token2) = spawn_harness("persist-folder", &page_one);
    let result2 = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket2);
        client.hello(&token2).expect("hello");
        client.attach(None).expect("attach");
        let restored = client.snapshot().expect("graceful restore snapshot");
        assert_restored_chrome_first(&restored, 1, 2);
        assert_eq!(
            urls_in_order(&restored),
            vec![
                "https://pinned.example/".to_string(),
                page_two.clone(),
                page_one.clone(),
            ],
            "the grouped tab restored at the end of the pinned region"
        );
        assert!(
            pinned(&restored, 102),
            "the grouping pin survived the restart"
        );
        assert_eq!(restored.pinned_count, 2);
        assert!(restored.folders.is_empty(), "folders are session-local");

        let folder = new_folder(&mut client);
        client
            .send_ok(&dsl::move_tab_into_folder(2, folder))
            .expect("group the live tab");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after grouping");
        let grouped = client.snapshot().expect("second grouping snapshot");
        assert_eq!(grouped.folders[0].members, vec![101]);
        assert_eq!(grouped.pinned_count, 3);
        client
            .send_ok(&bbr_testkit::Request::FlushBrowserState)
            .expect("flush browser state");
    }));
    keep_root(result2, &mut harness2, &socket2, &token2);
    harness2.terminate();
    drop(harness2);

    // Force-quit recovery: every grouped identity restores pinned, the order
    // is the committed one, and exactly one session hydrates.
    let (mut harness3, socket3, token3) = spawn_harness("persist-folder", &page_one);
    let result3 = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket3);
        client.hello(&token3).expect("hello");
        client.attach(None).expect("attach");
        let restored = client.snapshot().expect("force-quit restore snapshot");
        assert_restored_chrome_first(&restored, 1, 2);
        assert_eq!(restored.pinned_count, 3, "both grouping pins committed");
        assert!(pinned(&restored, 101) && pinned(&restored, 102));
        assert!(restored.folders.is_empty(), "folders are session-local");
    }));
    finish(result3, &mut harness3, &socket3, &token3);
}
