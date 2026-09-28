//! Real-app persistence + warm-restore E2E (Phase 7, warm-restore slice).
//!
//! Launches the real `bbr` process against profile directories whose browser
//! state was written through the actual `bbr-persistence` repository
//! (`Repository::store_browser_state`) — never a test-only in-memory seed or
//! product bypass. The deterministic fixtures/helpers live in
//! [`support`](e2e_shell_persistence/support.rs). Proven end to end:
//!
//! - chrome-first restore: persisted identities (profiles/spaces/tabs/
//!   recently-closed) appear with their persisted ids before any native
//!   session, and only the active space's visible set hydrates a session;
//! - a dense persisted model (500+ tab identities) restores exactly **one**
//!   live session (two for an active split), with the remaining tabs
//!   session-less (`Suspended`) until activation recreates exactly one;
//! - truthful restore disposition/deferred evidence on the testkit snapshot
//!   (`restore` + `engine_sessions` together are the fast-but-wrong guard:
//!   a restore that eagerly instantiated every persisted session would
//!   report hundreds of sessions, never one);
//! - window geometry and sidebar state restore through the production
//!   window options and restore seam;
//! - a graceful restart (deterministic flush then process exit) restores
//!   exactly the committed model;
//! - a force-quit/crash-style exit (SIGKILL, no flush) recovers exactly the
//!   last committed repository state and never a partial/fast-but-wrong
//!   model — the app presents precisely what the repository holds — for the
//!   open **and** the tab-reorder mutation classes;
//! - grouping pins durably: a graceful restart and a force quit restore the
//!   grouped tabs pinned and never a folder membership
//!   ([`folder_pins`](e2e_shell_persistence/folder_pins.rs));
//! - the curated space paper (v2 S10) edits as a draft, commits, and
//!   restores with the theme across a restart
//!   ([`space_paper`](e2e_shell_persistence/space_paper.rs)).
//!
//! No sleeps: every step synchronizes on typed snapshot state or semantic
//! events under an explicit deadline. GUI suites must not run in parallel
//! (AppKit window geometry), so every test takes the serial lock.

#![cfg(target_os = "macos")]

#[path = "e2e_common/mod.rs"]
mod e2e_common;

#[path = "e2e_shell_persistence/support.rs"]
mod support;

#[path = "e2e_shell_persistence/folder_pins.rs"]
mod folder_pins;

#[path = "e2e_shell_persistence/space_paper.rs"]
mod space_paper;

pub use bbr_testkit::protocol::{RestoreDisposition, StoreLoadSnapshot};
pub use e2e_common::*;
pub use support::*;
#[test]
#[ignore]
fn dense_persisted_restore_shows_identities_first_one_session_and_deferred_activation() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let pid = std::process::id();
    let root = harness_root("persist-dense", pid);
    let profile = root.join("profile");
    seed_browser_state(&profile, &dense_browser_state(&page_one));

    let (mut harness, socket, token) = spawn_harness("persist-dense", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("restored active view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            wait_title(&mut client, first_view, PAGE_ONE_TITLE, 0);
        }
        let restored = client.snapshot().expect("restored snapshot");

        // Chrome first: every persisted live identity with its persisted id
        // (503 live tabs across the three spaces), never fewer.
        assert_eq!(restored.all_tabs.len(), 503, "503 live identities");
        assert!(
            restored.all_tabs.iter().any(|tab| tab.id == 1000),
            "the dense identity 1000 restored with its persisted id"
        );
        // Exactly one live session; the dense space and every other tab are
        // session-less (the fast-but-wrong guard: never 500 sessions).
        // 503 live tabs − 1 hydrated session = 502 deferred identities.
        assert_restored_chrome_first(&restored, 1, 502);
        // The restored window row: the seeded geometry through the window
        // options reproduces the exact native content area (GPUI applies
        // the persisted bounds as the content rect; `window_size` is the
        // frame and includes the title bar).
        assert_eq!(restored.window_content_size, [1080.0, 700.0]);
        assert_eq!(restored.window_size[0], 1080.0);

        // Activating a deferred tab recreates exactly one new session (the
        // previously active session stays live): 2, never more.
        client
            .send_ok(&bbr_testkit::Request::SelectTab { index: 0 })
            .expect("select deferred pinned tab");
        let activated = client.snapshot().expect("activation snapshot");
        assert_eq!(
            activated.engine_sessions, 2,
            "deferred activation recreates exactly one session"
        );
        assert_eq!(
            activated
                .all_tabs
                .iter()
                .filter(|tab| tab.lifecycle == "suspended")
                .count(),
            501,
            "the remaining deferred identities stay session-less (503 − 2 sessions)"
        );
    }));
    finish(result, &mut harness, &socket, &token);
}

/// A persisted model whose active space carries a split restores exactly two
/// sessions (the visible set is both panes), never the per-space count.
#[test]
#[ignore]
fn persisted_split_active_space_restores_exactly_two_sessions() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let pid = std::process::id();
    let root = harness_root("persist-split", pid);
    let profile = root.join("profile");
    seed_browser_state(&profile, &split_active_browser_state(&page_one));

    let (mut harness, socket, token) = spawn_harness("persist-split", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        let restored = client.snapshot().expect("restored snapshot");
        // The active space 10 carries the split (panes 101/102): exactly two
        // sessions, everything else deferred (502 deferred of 504 live tabs).
        assert_restored_chrome_first(&restored, 2, 502);
        assert_eq!(restored.all_tabs.len(), 504, "504 live identities");
        let pane_layout = restored.pane_layout.expect("split panes presented");
        assert_eq!(
            pane_layout.primary,
            bbr_web::WebViewId(1),
            "primary pane is the first hydrated session view"
        );
        assert_eq!(pane_layout.secondary, Some(bbr_web::WebViewId(2)));
    }));
    finish(result, &mut harness, &socket, &token);
}

/// A graceful restart on the same profile directory restores exactly the
/// committed browser model: a structural mutation flushed deterministically
/// through the repository is present after the process restarts, chrome-first
/// with the visible set only.
#[test]
#[ignore]
fn graceful_restart_restores_committed_browser_state() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let pid = std::process::id();
    let root = harness_root("persist-restart", pid);
    let profile = root.join("profile");
    seed_browser_state(&profile, &small_browser_state(&page_one));

    let (mut harness, socket, token) = spawn_harness("persist-restart", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("restored active view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            wait_title(&mut client, first_view, PAGE_ONE_TITLE, 0);
        }
        let restored = client.snapshot().expect("restored snapshot");
        // 2 live tabs − 1 hydrated session = 1 deferred identity.
        assert_restored_chrome_first(&restored, 1, 1);
        // The seeded window row restores the exact native content area.
        assert_eq!(restored.window_content_size, [1080.0, 700.0]);
        assert_eq!(restored.window_size[0], 1080.0);

        // A structural mutation through the production path, then the
        // deterministic flush: the committed model gains the new tab.
        client
            .send_ok(&bbr_testkit::Request::OpenTab {
                url: page_two.clone(),
            })
            .expect("open tab");
        let second = client.snapshot().expect("after open tab");
        assert!(
            second.all_tabs.iter().any(|tab| tab.url == page_two),
            "the new tab is part of the live model"
        );
        client
            .send_ok(&bbr_testkit::Request::FlushBrowserState)
            .expect("flush browser state");
    }));
    // Keep the profile directory for the restart: a successful finish would
    // let the harness Drop remove the root, destroying the store we reload.
    match result {
        Ok(()) => {
            harness.success = false;
        }
        Err(panic) => {
            harness.diagnostics(&socket, &token);
            std::panic::resume_unwind(panic);
        }
    }
    drop(harness);

    // Restart on the same profile directory: the committed model (including
    // the flushed new tab) restores chrome-first with one live session.
    let (mut harness2, socket2, token2) = spawn_harness("persist-restart", &page_one);
    let result2 = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket2);
        client.hello(&token2).expect("hello");
        client.attach(None).expect("attach");
        let restored = client.snapshot().expect("restart snapshot");
        // 3 live tabs − 1 hydrated session = 2 deferred identities.
        assert_restored_chrome_first(&restored, 1, 2);
        assert!(
            restored.all_tabs.iter().any(|tab| tab.url == page_two),
            "the flushed new tab returned with the committed model"
        );
        assert_eq!(
            restored
                .all_tabs
                .iter()
                .filter(|tab| tab.id == 100 || tab.id == 101)
                .count(),
            2,
            "the original identities survived the restart"
        );
    }));
    finish(result2, &mut harness2, &socket2, &token2);
}

/// A force-quit/crash-style exit (SIGKILL, no graceful flush) recovers
/// exactly the last committed repository state: the app presents precisely
/// what the repository holds, chrome-first and without partial/fast-but-
/// wrong state — an unflushed structural mutation is only present when the
/// repository actually committed it.
#[test]
#[ignore]
fn force_quit_recovers_exactly_the_committed_state() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let pid = std::process::id();
    let root = harness_root("persist-force", pid);
    let profile = root.join("profile");
    seed_browser_state(&profile, &small_browser_state(&page_one));

    // Leg 1: flush a mutation, then force-quit with no graceful path.
    let (mut harness, socket, token) = spawn_harness("persist-force", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("restored active view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            wait_title(&mut client, first_view, PAGE_ONE_TITLE, 0);
        }
        assert_restored_chrome_first(&client.snapshot().expect("restored"), 1, 1);
        client
            .send_ok(&bbr_testkit::Request::OpenTab {
                url: page_two.clone(),
            })
            .expect("open tab");
        client
            .send_ok(&bbr_testkit::Request::FlushBrowserState)
            .expect("flush browser state");
    }));
    match result {
        Ok(()) => {
            harness.success = false;
        }
        Err(panic) => {
            harness.diagnostics(&socket, &token);
            std::panic::resume_unwind(panic);
        }
    }
    // Force quit: SIGKILL, no graceful quit flush (the committed state was
    // written by the deterministic flush before the crash).
    harness.terminate();
    drop(harness);

    // The committed model returned: the flushed tab is present, chrome-first,
    // exactly one live session.
    let (mut harness2, socket2, token2) = spawn_harness("persist-force", &page_one);
    let result2 = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket2);
        client.hello(&token2).expect("hello");
        client.attach(None).expect("attach");
        let restored = client.snapshot().expect("crash-recovery snapshot");
        // 3 committed live tabs − 1 hydrated session = 2 deferred.
        assert_restored_chrome_first(&restored, 1, 2);
        assert!(
            restored.all_tabs.iter().any(|tab| tab.url == page_two),
            "the committed flushed state survived the force quit"
        );
    }));
    match result2 {
        Ok(()) => {
            harness2.success = false;
        }
        Err(panic) => {
            harness2.diagnostics(&socket2, &token2);
            std::panic::resume_unwind(panic);
        }
    }
    drop(harness2);

    // Leg 2: a structural mutation that is never flushed, then a force quit.
    // Whether or not the background writer committed it, the next restore
    // presents exactly the repository's committed state — never a partial or
    // fast-but-wrong model.
    let (mut harness3, socket3, token3) = spawn_harness("persist-force", &page_one);
    let result3 = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket3);
        client.hello(&token3).expect("hello");
        client.attach(None).expect("attach");
        let restored = client.snapshot().expect("pre-crash snapshot");
        // 3 committed live tabs − 1 hydrated session = 2 deferred.
        assert_restored_chrome_first(&restored, 1, 2);
        // Unflushed mutation: open a third tab and force-quit immediately.
        client
            .send_ok(&bbr_testkit::Request::OpenTab {
                url: page_two.clone(),
            })
            .expect("open unflushed tab");
    }));
    match result3 {
        Ok(()) => {
            harness3.success = false;
        }
        Err(panic) => {
            harness3.diagnostics(&socket3, &token3);
            std::panic::resume_unwind(panic);
        }
    }
    harness3.terminate();
    drop(harness3);

    // The repository's committed truth at the crash instant.
    let committed = committed_browser_state(&profile);
    let committed_tab_count = committed
        .profiles
        .iter()
        .flat_map(|p| p.spaces.iter().flat_map(|s| s.tabs.iter()))
        .count();

    // Restart: the app presents exactly the committed repository state.
    let (mut harness4, socket4, token4) = spawn_harness("persist-force", &page_one);
    let result4 = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket4);
        client.hello(&token4).expect("hello");
        client.attach(None).expect("attach");
        let restored = client.snapshot().expect("post-crash snapshot");
        // The restored live identity count equals the repository's committed
        // live tab count exactly — never more, never a partial eager model.
        assert_eq!(
            restored.all_tabs.len(),
            committed_tab_count,
            "the app presents exactly the repository's committed state"
        );
        assert!(
            matches!(restored.restore, RestoreDisposition::Restored { .. }),
            "the committed model restored chrome-first"
        );
        assert_eq!(
            restored.engine_sessions, 1,
            "exactly one live session after crash recovery — never a partial eager restore"
        );
    }));
    finish(result4, &mut harness4, &socket4, &token4);
}

/// A force-quit (SIGKILL) after a **tab reorder** recovers exactly the
/// committed reordered model: the reorder pokes the one browser-state writer
/// through the production mutation path, the deterministic flush commits it,
/// and the restart presents the reordered tab order — never the pre-reorder
/// order, never a partial model. This closes the Phase-7 exit evidence for
/// the reorder class (the open class is covered by
/// [`force_quit_recovers_exactly_the_committed_state`]).
#[test]
#[ignore]
fn force_quit_after_reorder_recovers_committed_order() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let pid = std::process::id();
    let root = harness_root("persist-reorder", pid);
    let profile = root.join("profile");
    seed_browser_state(&profile, &reorder_browser_state(&page_one, &page_two));

    let (mut harness, socket, token) = spawn_harness("persist-reorder", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("restored active view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            wait_title(&mut client, first_view, PAGE_ONE_TITLE, 0);
        }
        let restored = client.snapshot().expect("restored snapshot");
        // 3 live tabs − 1 hydrated session = 2 deferred identities.
        assert_restored_chrome_first(&restored, 1, 2);
        assert_eq!(
            urls_in_order(&restored),
            vec![
                "https://pinned.example/".to_string(),
                page_one.clone(),
                page_two.clone(),
            ],
            "seeded order: pinned, page-one, page-two"
        );

        // Reorder the last regular tab before the first regular tab through
        // the production reorder path (which pokes the browser-state writer):
        // [pinned, one, two] → [pinned, two, one].
        client
            .send_ok(&bbr_testkit::Request::ReorderTab { from: 2, to: 1 })
            .expect("reorder tab");
        let reordered = client.snapshot().expect("reordered snapshot");
        assert_eq!(
            urls_in_order(&reordered),
            vec![
                "https://pinned.example/".to_string(),
                page_two.clone(),
                page_one.clone(),
            ],
            "the live model shows the reordered tabs"
        );
        client
            .send_ok(&bbr_testkit::Request::FlushBrowserState)
            .expect("flush reordered browser state");
    }));
    match result {
        Ok(()) => {
            harness.success = false;
        }
        Err(panic) => {
            harness.diagnostics(&socket, &token);
            std::panic::resume_unwind(panic);
        }
    }
    // Force quit: SIGKILL, no graceful flush (the reordered order was
    // committed by the deterministic flush before the crash).
    harness.terminate();
    drop(harness);

    // The committed reordered model returned: the restart presents the
    // reordered order, chrome-first, exactly one live session.
    let (mut harness2, socket2, token2) = spawn_harness("persist-reorder", &page_one);
    let result2 = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket2);
        client.hello(&token2).expect("hello");
        client.attach(None).expect("attach");
        let restored = client.snapshot().expect("post-crash snapshot");
        // 3 committed live tabs − 1 hydrated session = 2 deferred identities.
        assert_restored_chrome_first(&restored, 1, 2);
        assert_eq!(
            urls_in_order(&restored),
            vec![
                "https://pinned.example/".to_string(),
                page_two.clone(),
                page_one.clone(),
            ],
            "the committed reordered model returned after the force quit"
        );
        assert!(
            matches!(restored.restore, RestoreDisposition::Restored { .. }),
            "the reordered model restored chrome-first"
        );
    }));
    finish(result2, &mut harness2, &socket2, &token2);
}
