//! Real-app keyboard profile/space navigation E2E (Phase-4 space slice).
//!
//! The keyboard half of DESIGN.md §11.3/§12 with real keystrokes:
//! Cmd-Shift-N stages a space name (Escape cancels, Enter commits and
//! activates), `↑/↓` traverse the focused space list through the production
//! activation path, and moves between spaces preserve or recreate sessions
//! exactly per the profile-boundary policy. The pointer counterpart lives in
//! `e2e_shell_spaces.rs`; the shared harness in `e2e_common`.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::CommandBarMode;
use e2e_common::*;

/// Keyboard control of the visible space surface with real keystrokes:
/// Cmd-Shift-N stages a space name (Escape cancels, Enter commits and
/// activates), `↑/↓` traverse the focused space list through the production
/// activation path, and moves between spaces preserve or recreate sessions
/// exactly per the profile-boundary policy.
#[test]
#[ignore]
fn keyboard_selects_and_creates_spaces_and_moves_tabs_between_spaces() {
    let _serial = serial_gui();
    if std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false") {
        return;
    }

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");
    let page_two = fixtures.url("/page2.html");
    let set_beta = fixtures.url("/__fx/cookie/set-beta");
    let read_cookie = fixtures.url("/__fx/cookie/read");

    let (mut harness, socket, token) = spawn_harness("spaces-keyboard", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let default_space = initial.active_space;

        let keystroke = |client: &mut Client, key: &str| {
            client
                .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                    keystroke: key.into(),
                })
                .expect("keystroke")
        };

        // §11.4 compact rail: reveal the §11.3 space rows through the footer
        // `[Spaces]` control first (the rows hide by default; the keyboard
        // traversal and the row ids require the revealed region).
        reveal_spaces_region(&mut client);

        // Keyboard create: Cmd-Shift-N stages a space name in the command
        // bar without creating anything; a typed name commits on Enter and
        // the fresh space is activated (chrome focus stays in the space
        // list — an empty space has no page to hand focus to).
        keystroke(&mut client, "cmd-shift-n");
        since = wait_chrome_focus(&mut client, true, since);
        let staged = client.snapshot().expect("staged space snapshot");
        assert_eq!(staged.command_bar_mode, CommandBarMode::NewSpace);
        assert_eq!(staged.spaces.len(), 1, "staging creates nothing");
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "Reading".into(),
            })
            .expect("set space name");
        keystroke(&mut client, "enter");
        let (seq, _) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = seq;
        let created = client.snapshot().expect("created space snapshot");
        let reading = created.spaces[1].space;
        assert_eq!(created.spaces.len(), 2);
        assert_eq!(
            created.active_space, reading,
            "committed space is activated"
        );
        assert_eq!(created.command_bar_mode, CommandBarMode::Inactive);
        assert!(created.command_bar_text.is_empty());
        assert!(created.chrome_focused, "fresh space keeps chrome focus");
        assert_eq!(created.active_web_view, None);
        assert_ids(&created, &["bbr-empty-new-tab", "bbr-space-row-1"]);

        // Escape cancels a staged name: nothing is created, the previous
        // command-bar mode and focus domain are restored.
        keystroke(&mut client, "cmd-shift-n");
        let _ = wait_chrome_focus(&mut client, true, since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "Cancelled".into(),
            })
            .expect("set cancelled name");
        keystroke(&mut client, "escape");
        let cancelled = client.snapshot().expect("cancelled snapshot");
        assert_eq!(cancelled.spaces.len(), 2, "Escape creates nothing");
        assert_eq!(cancelled.command_bar_mode, CommandBarMode::Inactive);
        assert_eq!(cancelled.active_space, reading);
        assert!(cancelled.command_bar_text.is_empty());

        // Warm a tab in reading, then drive the space list with real
        // arrow keys from the production pointer-focused list: down/up
        // activate the next/previous space (wrap-around), restoring each
        // space's own warm tab without recreating sessions.
        let second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client
            .send_ok(&dsl::click_space(default_space))
            .expect("focus the space list on default");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        assert!(client.snapshot().expect("list focused").chrome_focused);

        keystroke(&mut client, "down");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        let down = client.snapshot().expect("down snapshot");
        assert_eq!(down.active_space, reading);
        assert_eq!(
            down.active_web_view,
            Some(second_view),
            "reading's own tab restored"
        );
        assert_eq!(down.engine_sessions, 2, "keyboard switch keeps sessions");
        assert!(down.chrome_focused, "keyboard traversal stays in chrome");

        keystroke(&mut client, "down");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        let wrapped = client.snapshot().expect("wrapped snapshot");
        assert_eq!(
            wrapped.active_space, default_space,
            "down wraps to the first space"
        );
        assert_eq!(wrapped.active_web_view, Some(first_view));
        assert_eq!(wrapped.engine_sessions, 2);
        assert!(wrapped.chrome_focused);

        keystroke(&mut client, "up");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        let up = client.snapshot().expect("up snapshot");
        assert_eq!(up.active_space, reading);
        assert_eq!(up.active_web_view, Some(second_view));
        assert_eq!(up.engine_sessions, 2);
        assert!(up.chrome_focused);

        // Same-profile move preserves the live session (recreated: false) and
        // the destination space activates the moved tab with the same
        // WebViewId.
        client
            .send_ok(&dsl::move_tab(0, default_space))
            .expect("same-profile move");
        since = wait_browser_change(
            &mut client,
            BrowserChange::TabMoved {
                to_space: default_space,
                recreated: false,
            },
            since,
        );
        let moved_same = client.snapshot().expect("same-profile move snapshot");
        assert!(moved_same.tabs.is_empty(), "reading emptied by the move");
        assert_eq!(moved_same.engine_sessions, 2);
        client
            .send_ok(&dsl::click_space(default_space))
            .expect("view the moved tab");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        let destination = client.snapshot().expect("destination snapshot");
        assert_eq!(destination.tabs.len(), 2);
        assert_eq!(
            destination.active_web_view,
            Some(second_view),
            "same WebViewId, no recreation"
        );
        assert_eq!(destination.engine_sessions, 2);

        // Cross-profile move follows the recreation policy: destroy the old
        // session and create a fresh one in the destination profile's store
        // (recreated: true, new WebViewId).
        client
            .send_ok(&dsl::create_profile("Private", "Private"))
            .expect("create private profile");
        let (seq, created_profile) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = seq;
        let (_, private_space) = match created_profile {
            BrowserChange::ProfileCreated { profile, space } => (profile, space),
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&dsl::click_space(private_space))
            .expect("activate private space");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let private_view = client.snapshot().expect("private snapshot");
        assert_eq!(private_view.active_profile_hue.as_deref(), Some("violet"));
        let private_view_id = open_tab(&mut client, &mut since, &set_beta, Some("Cookie Set beta"));
        client
            .send_ok(&dsl::navigate(&read_cookie))
            .expect("read beta");
        since = wait_title(&mut client, private_view_id, "Cookie beta", since);
        let before_move = since;
        client
            .send_ok(&dsl::move_tab(0, default_space))
            .expect("cross-profile move");
        since = wait_browser_change(
            &mut client,
            BrowserChange::TabMoved {
                to_space: default_space,
                recreated: true,
            },
            since,
        );
        client
            .send_ok(&dsl::click_space(default_space))
            .expect("view the recreated tab");
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        let recreated = client.snapshot().expect("recreated tab snapshot");
        assert_eq!(recreated.tabs.len(), 3);
        assert_eq!(
            recreated.engine_sessions, 3,
            "destroy + recreate keeps the count"
        );
        let moved_view = recreated.active_web_view.expect("fresh incarnation");
        assert_ne!(
            moved_view, private_view_id,
            "cross-profile move recreates the session"
        );
        // The fresh default-store session has no private cookie. The title
        // event fires during the move, so the wait cursor predates it.
        wait_title(&mut client, moved_view, "Cookie none", before_move);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The §11.3 cross-context rule through the keyboard traversal: real `↑`/`↓`
/// on the focused space list set the command-field profile hint only when the
/// destination crosses a profile boundary, and clear it when a same-profile
/// activation follows — the deterministic interaction boundary on the
/// keyboard path.
#[test]
#[ignore]
fn keyboard_cross_profile_traversal_sets_and_clears_the_profile_hint() {
    let _serial = serial_gui();
    if std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false") {
        return;
    }

    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("bbr-fixtures")
        .join("html");
    let fixtures = FixtureServer::start(&fixtures_dir).expect("fixture server");
    let page_one = fixtures.url("/page1.html");
    let page_two = fixtures.url("/page2.html");

    let (mut harness, socket, token) = spawn_harness("spaces-hint-kbd", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first view");
        if initial.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, first_view, PAGE_ONE_TITLE, since);
        }
        let default_space = initial.active_space;
        let default_profile = initial.active_profile;

        // §11.4 compact rail: reveal the §11.3 space rows through the footer
        // `[Spaces]` control before the pointer-focus and `↑`/`↓` traversal
        // (the rows hide by default).
        reveal_spaces_region(&mut client);

        let keystroke = |client: &mut Client, key: &str| {
            client
                .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                    keystroke: key.into(),
                })
                .expect("keystroke")
        };

        // Same-profile space "Reading" and cross-profile space "Private",
        // both with warm tabs so the traversed space presents a session.
        client
            .send_ok(&dsl::create_space(default_profile, "Reading"))
            .expect("create reading space");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = seq;
        let reading = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&dsl::create_profile("Private", "Private"))
            .expect("create private profile");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = seq;
        let private_space = match created {
            BrowserChange::ProfileCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&dsl::click_space(reading))
            .expect("activate reading");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        let reading_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client
            .send_ok(&dsl::click_space(private_space))
            .expect("activate private");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let private_view = open_tab(&mut client, &mut since, &page_one, Some(PAGE_ONE_TITLE));
        let sessions = client.snapshot().expect("sessions").engine_sessions;

        // Focus the space list on the default space (render order: Default,
        // Reading, Private), then traverse with real keys.
        client
            .send_ok(&dsl::click_space(default_space))
            .expect("focus list on default");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        assert!(client.snapshot().expect("list focused").chrome_focused);

        // down → Reading (same profile): no profile transition announced.
        keystroke(&mut client, "down");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        let same = client.snapshot().expect("same-profile down snapshot");
        assert_eq!(
            same.command_profile_hint, None,
            "same-profile keyboard activation never announces"
        );
        assert_eq!(same.active_web_view, Some(reading_view));

        // down → Private (cross profile): the hint names the destination.
        keystroke(&mut client, "down");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let cross = client.snapshot().expect("cross-profile down snapshot");
        assert_eq!(
            cross.command_profile_hint.as_deref(),
            Some("in Profile · Private")
        );
        assert_eq!(cross.active_web_view, Some(private_view));
        assert_eq!(
            cross.engine_sessions, sessions,
            "keyboard cross-profile switch keeps sessions"
        );
        assert!(cross.chrome_focused);

        // down wraps to Default — cross-profile in the reverse direction.
        keystroke(&mut client, "down");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        let wrapped = client.snapshot().expect("wrapped snapshot");
        assert_eq!(
            wrapped.command_profile_hint.as_deref(),
            Some("in Profile · Default")
        );
        assert_eq!(wrapped.active_web_view, Some(first_view));

        // up → Private (cross), then up → Reading (same profile) clears.
        keystroke(&mut client, "up");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let up_cross = client.snapshot().expect("up cross snapshot");
        assert_eq!(
            up_cross.command_profile_hint.as_deref(),
            Some("in Profile · Private")
        );
        keystroke(&mut client, "up");
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        // Reading belongs to the default profile, so Private → Reading
        // crosses the profile boundary again: the hint names the new
        // destination ("in Profile · Default") instead of clearing.
        let up_same = client.snapshot().expect("up cross snapshot");
        assert_eq!(
            up_same.command_profile_hint.as_deref(),
            Some("in Profile · Default"),
            "crossing back into the default profile re-announces it"
        );
        assert_eq!(up_same.active_web_view, Some(reading_view));

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
