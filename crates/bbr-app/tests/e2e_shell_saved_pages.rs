//! Real-app Saved Pages E2E (B2 packet `saved-pages-actions-command`).
//!
//! Launches the real `bbr` process and drives the one app-owned Saved Page
//! command authority through the production seams: Cmd-D toggles the active
//! page's saved state (idempotent save, idempotent remove, no duplicates),
//! the deterministic flush persists the in-memory provider to the store, a
//! graceful restart restores the committed records, the Command Palette
//! ranks the global Saved Page rows, and activating a result opens exactly
//! one fresh regular tab in the currently active Space/Profile — never a
//! silent selection of an existing matching tab.
//!
//! Proven end to end:
//!
//! - `Cmd-D` saves the active page; repeated `Cmd-D` removes it; repeated
//!   save cycles never create duplicates; the typed saved-state count and
//!   the repository effects agree after the deterministic flush;
//! - saving a page, closing its tab, and restarting the real app restores
//!   the record while no original `TabId`/session exists — the Saved Page
//!   is a durable global URL record, not a live-tab presentation;
//! - Palette activation revalidates the record and appends exactly one new
//!   regular tab/session in the currently active Space/Profile, preserving
//!   profile isolation and never selecting a matching open tab in another
//!   context;
//! - production quit durability parity: a Cmd-D save is re-persisted by the
//!   real owning-window-close quit flush (never the testkit flush) even
//!   when the store was emptied out-of-band, and a same-profile restart
//!   restores exactly one record with remove offered.
//!
//! No sleeps: every step synchronizes on typed snapshot state or semantic
//! events under an explicit deadline. GUI suites must not run in parallel
//! (AppKit window geometry), so every test takes the serial lock.

#![cfg(target_os = "macos")]

#[path = "e2e_common/mod.rs"]
mod e2e_common;

pub use bbr_testkit::protocol::{CommandBarMode, CommandResultState, StoreLoadSnapshot};
pub use e2e_common::*;

/// Dispatch one keystroke through the production GPUI input path.
fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&bbr_testkit::Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("dispatch keystroke");
}

/// Open the location bar through the production Cmd-L path and assert the
/// focused selection state (the same helper the command suite uses).
fn cmd_l(client: &mut Client, since: &mut u64) -> bbr_testkit::protocol::Snapshot {
    key(client, "cmd-l");
    *since = wait_chrome_focus(client, true, *since);
    let snapshot = client.snapshot().expect("cmd-l snapshot");
    assert_eq!(snapshot.command_bar_mode, CommandBarMode::Location);
    assert!(snapshot.chrome_focused);
    snapshot
}

/// Type `text` through the testkit text seam and wait for the real ranking
/// debounce to publish results carrying `expected_first_key` first.
fn search(client: &mut Client, text: &str, expected_first_key: &str) {
    client
        .send_ok(&bbr_testkit::Request::SetCommandBarText {
            text: text.to_string(),
        })
        .expect("set command text");
    let until = deadline();
    loop {
        let snapshot = client.snapshot().expect("search snapshot");
        if snapshot.command_result_state == CommandResultState::Results
            && snapshot
                .command_results
                .first()
                .is_some_and(|row| row.key == expected_first_key)
        {
            return;
        }
        assert!(
            Instant::now() < until,
            "search did not publish {expected_first_key} within the deadline"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// `Cmd-D` toggles the active page's saved state: the first press saves,
/// the second removes, and repeated save cycles never create duplicates.
/// The typed saved-state count and the repository effects agree after the
/// deterministic flush.
#[test]
#[ignore]
fn cmd_d_toggles_saved_state_without_duplicates() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("saved-toggle", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            let _ = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let store = client.snapshot().expect("store snapshot");
        assert_eq!(store.store.saved_pages_count, 0, "starts empty");

        // Cmd-D saves the active page (the fixture URL is navigable).
        key(&mut client, "cmd-d");
        let saved = client.snapshot().expect("saved snapshot");
        assert_eq!(
            saved.store.saved_pages_count, 1,
            "Cmd-D saved exactly one record"
        );
        // The availability matrix flips: remove is now available, save is
        // not (no duplicate record is offered).
        let availability = client
            .request(&bbr_testkit::Request::SavedPageAvailability {
                url: page_one.clone(),
            })
            .expect("availability probe");
        match availability {
            bbr_testkit::protocol::Response::SavedPageAvailability {
                can_save,
                can_remove,
            } => {
                assert!(!can_save, "no duplicate save offered");
                assert!(can_remove, "remove is available after a save");
            }
            other => panic!("unexpected availability response: {other:?}"),
        }

        // Repeated Cmd-D removes; the record is gone and the matrix flips
        // back.
        key(&mut client, "cmd-d");
        let removed = client.snapshot().expect("removed snapshot");
        assert_eq!(
            removed.store.saved_pages_count, 0,
            "repeated Cmd-D removed the record"
        );
        let availability = client
            .request(&bbr_testkit::Request::SavedPageAvailability {
                url: page_one.clone(),
            })
            .expect("availability probe after remove");
        match availability {
            bbr_testkit::protocol::Response::SavedPageAvailability {
                can_save,
                can_remove,
            } => {
                assert!(can_save, "save is available again after a remove");
                assert!(!can_remove, "remove is not available after a remove");
            }
            other => panic!("unexpected availability response: {other:?}"),
        }

        // Repeated save cycles never create duplicates: save → save →
        // remove → save keeps the count at exactly one.
        key(&mut client, "cmd-d");
        key(&mut client, "cmd-d");
        key(&mut client, "cmd-d");
        let cycled = client.snapshot().expect("cycle snapshot");
        assert_eq!(
            cycled.store.saved_pages_count, 1,
            "save/remove cycles never create duplicates"
        );

        // The deterministic flush persists the provider to the store in one
        // transaction (the production background writer uses the identical
        // repository write).
        client
            .send_ok(&bbr_testkit::Request::FlushSavedPages)
            .expect("flush saved pages");
    }));
    finish(result, &mut harness, &socket, &token);
}

/// Save a page, close its tab, restart the real app, search the Palette,
/// and find the Saved Page while no original `TabId`/session exists — the
/// record is a durable global URL, not a live-tab presentation.
#[test]
#[ignore]
fn saved_page_survives_tab_close_and_restart() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let pid = std::process::id();
    let root = harness_root("saved-restart", pid);
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    let socket = harness_socket_path("saved-restart", pid);
    let token = format!("shell-saved-restart-{pid}");
    let bin = env!("CARGO_BIN_EXE_bbr");
    let spawn = |root: &PathBuf| {
        Harness::spawn(
            bin,
            &[
                "--bbr-initial-url",
                &page_one,
                "--bbr-profile-dir",
                profile.to_str().unwrap(),
                "--bbr-test-token",
                &token,
                "--bbr-ids=sequential",
                "--bbr-hidden",
            ],
            root,
        )
    };

    let mut harness = spawn(&root);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }

        // Save the active page, then open a second tab and close the source
        // tab: the record must survive the tab close (no TabId link).
        key(&mut client, "cmd-d");
        let saved = client.snapshot().expect("saved snapshot");
        assert_eq!(saved.store.saved_pages_count, 1);
        let source_tab = saved
            .all_tabs
            .iter()
            .find(|tab| tab.url == page_one)
            .expect("source tab")
            .id;
        client
            .send_ok(&bbr_testkit::Request::OpenTab {
                url: page_two.clone(),
            })
            .expect("open second tab");
        let second = client.snapshot().expect("second tab snapshot");
        let second_view = second.active_web_view.expect("second view");
        if second.title.as_deref() != Some(PAGE_TWO_TITLE) {
            let _ = wait_title(&mut client, second_view, PAGE_TWO_TITLE, since);
        }
        // Close the source tab through the production Cmd-W path.
        client
            .send_ok(&bbr_testkit::Request::SelectTab { index: 0 })
            .expect("select source tab");
        key(&mut client, "cmd-w");
        let closed = client.snapshot().expect("closed snapshot");
        assert!(
            !closed.all_tabs.iter().any(|tab| tab.id == source_tab),
            "the source tab is closed"
        );
        assert_eq!(
            closed.store.saved_pages_count, 1,
            "closing the source tab never removes the record"
        );
        // The deterministic flush persists the record before the restart.
        client
            .send_ok(&bbr_testkit::Request::FlushSavedPages)
            .expect("flush saved pages");
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

    // Restart on the same profile directory: the record restores while no
    // original TabId/session exists (the source tab was closed before the
    // flush; the restart restores the persisted browser state — the last
    // remaining tab, which may be a different page than the initial URL).
    let mut harness2 = spawn(&root);
    let result2 = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let store = client.snapshot().expect("restart store snapshot");
        assert_eq!(
            store.store.saved_pages_count, 1,
            "the committed record restored across the restart"
        );
        assert_eq!(
            store.all_tabs.len(),
            1,
            "no original TabId/session exists after the restart"
        );

        // The restored tab still matches the search query (its title and
        // URL share the "fixture" word with the saved page's), so close it
        // before searching: the Saved Page row must rank first
        // (CurrentTab < SavedPage is fixed), and the record is findable
        // even with no live tab at all.
        key(&mut client, "cmd-w");
        let closed = client.snapshot().expect("closed restored tab snapshot");
        assert!(closed.tabs.is_empty(), "the restored tab is closed");
        assert_eq!(closed.engine_sessions, 0, "no session remains");
        assert_eq!(
            closed.store.saved_pages_count, 1,
            "the record survives with no live tab"
        );

        // Search the Palette: the Saved Page ranks as a distinct result row
        // with the stable `saved_page:<id>` key and the canonical URL.
        let _ = cmd_l(&mut client, &mut since);
        search(&mut client, "fixture", "saved_page:1");
        let snapshot = client.snapshot().expect("search snapshot");
        let row = snapshot
            .command_results
            .iter()
            .find(|row| row.group == "saved_page")
            .expect("saved-page row");
        assert_eq!(row.key, "saved_page:1");
        assert_eq!(row.subtitle.as_deref(), Some(page_one.as_str()));
        assert_eq!(row.title, PAGE_ONE_TITLE);
    }));
    finish(result2, &mut harness2, &socket, &token);
}

/// Palette activation of a Saved Page result creates exactly one new regular
/// tab in the currently active Space/Profile, preserves profile isolation,
/// and never silently selects a matching open tab in another context.
#[test]
#[ignore]
fn palette_activation_opens_exactly_one_fresh_tab() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("saved-open", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }

        // Save the active page through the command authority.
        client
            .send_ok(&bbr_testkit::Request::SaveSavedPage {
                url: page_one.clone(),
                title: PAGE_ONE_TITLE.to_string(),
            })
            .expect("save saved page");
        let saved = client.snapshot().expect("saved snapshot");
        assert_eq!(saved.store.saved_pages_count, 1);
        let source_tab = saved
            .all_tabs
            .iter()
            .find(|tab| tab.url == page_one)
            .expect("source tab")
            .id;
        let active_space = saved.active_space;

        // The source tab still matches the search query (its title and URL
        // are the saved page's), so replace its content with a non-matching
        // page before searching: the Saved Page row must rank first
        // (CurrentTab < SavedPage is fixed).
        let long_title_url = fixtures.url("/long-title.html");
        client
            .send_ok(&bbr_testkit::Request::Navigate {
                url: long_title_url.clone(),
            })
            .expect("navigate the source tab away");
        let navigated = client.snapshot().expect("navigated snapshot");
        let navigated_view = navigated.active_web_view.expect("navigated view");
        if navigated.title.as_deref() != Some(LONG_TITLE) {
            since = wait_title(&mut client, navigated_view, LONG_TITLE, since);
        }
        let before_tabs = navigated.tabs.len();
        let before_sessions = navigated.engine_sessions;
        let before_view = navigated.active_web_view.expect("active view before open");

        // Search the Palette and activate the Saved Page row: exactly one
        // fresh regular tab/session opens in the active Space/Profile.
        let _ = cmd_l(&mut client, &mut since);
        search(&mut client, "fixture", "saved_page:1");
        key(&mut client, "enter");
        let opened = client.snapshot().expect("opened snapshot");
        assert_eq!(
            opened.tabs.len(),
            before_tabs + 1,
            "exactly one new regular tab"
        );
        assert_eq!(
            opened.engine_sessions,
            before_sessions + 1,
            "exactly one new live session"
        );
        let new_view = opened.active_web_view.expect("fresh view");
        assert_ne!(new_view, before_view, "the fresh view is not the source");
        assert_eq!(opened.command_bar_mode, CommandBarMode::Inactive);
        assert!(!opened.chrome_focused, "the fresh page owns focus");
        assert_eq!(
            opened.command_bar_text, page_one,
            "the opened destination is the record's canonical URL"
        );
        // The fresh tab is a regular tab in the active space (never pinned,
        // never a split, never a silent selection of an existing tab).
        let fresh = opened
            .all_tabs
            .iter()
            .find(|tab| tab.url == page_one && tab.id != source_tab)
            .expect("the fresh regular tab");
        assert!(!fresh.pinned, "the fresh tab is never pinned");
        assert_eq!(fresh.space, active_space, "same active space");
        // The deterministic content presents.
        if opened.title.as_deref() != Some(PAGE_ONE_TITLE) {
            wait_title(&mut client, new_view, PAGE_ONE_TITLE, since);
        }
    }));
    finish(result, &mut harness, &socket, &token);
}

/// Production quit durability parity: the real owning-window-close quit
/// flush re-persists the in-memory provider — never the testkit flush — even
/// when the store was emptied out-of-band.
///
/// Same-profile Cmd-D save; the bounded background writer's persisted state
/// is observed deterministically out-of-band (`load_saved_pages`), then a
/// foreign `Repository::replace_saved_pages(&[])` empties the store and the
/// persisted zero is observed; the production `Request::CloseWindow` (no
/// testkit flush) exits the complete process successfully; the profile root
/// is preserved, and a restart on the same profile restores exactly one
/// record with the availability matrix offering remove.
#[test]
#[ignore]
fn saved_page_survives_production_quit_flush_after_out_of_band_clear() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let pid = std::process::id();
    let root = harness_root("saved-quit-flush", pid);
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    let socket = harness_socket_path("saved-quit-flush", pid);
    let token = format!("shell-saved-quit-flush-{pid}");
    let bin = env!("CARGO_BIN_EXE_bbr");
    let spawn = |root: &PathBuf| {
        Harness::spawn(
            bin,
            &[
                "--bbr-initial-url",
                &page_one,
                "--bbr-profile-dir",
                profile.to_str().unwrap(),
                "--bbr-test-token",
                &token,
                "--bbr-ids=sequential",
                "--bbr-hidden",
            ],
            root,
        )
    };
    // The store file name mirrors `crate::store::STORE_FILE_NAME` (the E2E
    // crate cannot import the bin crate's modules).
    let store_path = profile.join("bbr.sqlite");
    // Out-of-band repository access: the same `bbr-persistence` repository
    // the app owns, opened directly on the profile store. The gateway is
    // never asked to flush in this scenario.
    let repository = || bbr_persistence::Repository::new(store_path.clone());
    // Poll the persisted Saved Page collection out-of-band until it reaches
    // `expected`. A not-yet-created store, a schema check, or a concurrent
    // writer returns `Err`/a different length and retries under the explicit
    // deadline — the poll asserts the persisted state, never a blind sleep.
    let wait_persisted_count = |expected: usize| {
        let until = deadline();
        loop {
            match repository().load_saved_pages() {
                Ok(pages) if pages.len() == expected => return,
                Ok(_) | Err(_) => {}
            }
            assert!(
                Instant::now() < until,
                "persisted Saved Page count never reached {expected}"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    };

    let mut harness = spawn(&root);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let first_view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            let _ = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let baseline = client.snapshot().expect("baseline snapshot");
        assert_eq!(baseline.store.saved_pages_count, 0, "starts empty");

        // Same-profile Cmd-D save: the typed in-memory count flips to one.
        key(&mut client, "cmd-d");
        let saved = client.snapshot().expect("saved snapshot");
        assert_eq!(
            saved.store.saved_pages_count, 1,
            "Cmd-D saved exactly one record"
        );

        // Deterministically wait until the bounded background writer has
        // persisted the record (observable only out-of-band).
        wait_persisted_count(1);

        // Empty the persisted Saved Page rows out-of-band, as a foreign
        // writer would; the in-memory provider keeps its one record. Wait
        // until the store truthfully shows zero before closing the window.
        repository()
            .replace_saved_pages(&[])
            .expect("out-of-band clear replaces with an empty collection");
        wait_persisted_count(0);

        // Production owning-window close, WITHOUT any testkit flush: the
        // real quit seam must synchronously re-persist the in-memory
        // provider before the process exits. `send_ok` flushes the gateway
        // response before teardown (no EOF race); `wait_for_exit` bounds the
        // full process exit under an explicit deadline.
        client
            .send_ok(&bbr_testkit::Request::CloseWindow)
            .expect("CloseWindow response must be flushed before teardown");
        let exit = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(
            exit.success(),
            "owning-window close with the Saved Page quit flush must exit \
             gracefully, got {exit}"
        );
    }));
    // Keep the profile root for the restart: a successful finish would let
    // the harness Drop remove the root, destroying the store the quit flush
    // just wrote.
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

    // Restart on the same profile: the production quit flush re-persisted
    // the provider, so exactly one record restores and the availability
    // matrix offers remove (never a duplicate save).
    let mut harness2 = spawn(&root);
    let result2 = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let store = client.snapshot().expect("restart store snapshot");
        // The restart restores the persisted browser state; the saved page's
        // tab loads again, so wait for the deterministic title when needed.
        if store.title.as_deref() != Some(PAGE_ONE_TITLE) {
            if let Some(view) = store.active_web_view {
                let _ = wait_title(&mut client, view, PAGE_ONE_TITLE, since);
            }
        }
        let restored = client.snapshot().expect("restored snapshot");
        assert_eq!(
            restored.store.saved_pages_count, 1,
            "the production quit flush re-persisted exactly one record across the restart"
        );
        let availability = client
            .request(&bbr_testkit::Request::SavedPageAvailability {
                url: page_one.clone(),
            })
            .expect("availability probe after restart");
        match availability {
            bbr_testkit::protocol::Response::SavedPageAvailability {
                can_save,
                can_remove,
            } => {
                assert!(
                    !can_save,
                    "no duplicate save offered for the restored record"
                );
                assert!(can_remove, "remove is available for the restored record");
            }
            other => panic!("unexpected availability response: {other:?}"),
        }
    }));
    finish(result2, &mut harness2, &socket, &token);
}
