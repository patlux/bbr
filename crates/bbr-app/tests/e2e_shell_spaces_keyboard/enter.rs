//! Real-app §11.3 keyboard Enter re-activation evidence (F2a strip E2E
//! layer): with a chip focused, Enter activates the focused chip again
//! through the one production strip authority (`on_space_strip_key_down`
//! → `activate_space`, which re-emits `SpaceActivated`). The exact active
//! space, the warm WebViewId, the unchanged engine session count, the
//! chrome focus domain, the exactly-one-selected chip binding, and the
//! same-profile hint lifetime are asserted, and the replay window proves
//! no reload. This is chip activation, not the New-Space staging commit.
//!
//! Split out of the legacy `e2e_shell_spaces_keyboard` aggregate so the
//! lane's new selected-chip/Enter evidence stays a focused sibling module
//! instead of enlarging the pre-existing suite file.

use super::*;

/// The §11.3/§15 typed chip evidence on the keyboard path: exactly one chip
/// renders selected, and that chip is exactly the active space — the strip's
/// selection can never drift from the activation authority.
fn assert_chip_evidence(
    snapshot: &bbr_testkit::protocol::Snapshot,
    active: bbr_testkit::protocol::TestSpaceId,
) {
    assert_eq!(
        snapshot.space_chips.iter().filter(|c| c.selected).count(),
        1,
        "exactly one chip renders selected"
    );
    let chip = snapshot
        .space_chips
        .iter()
        .find(|c| c.selected)
        .expect("selected chip");
    assert_eq!(
        chip.space, active,
        "the selected chip is exactly the active space"
    );
}

/// Enter re-activation of the focused same-profile chip: a keyboard-created
/// "Reading" space with a warm tab has the strip focused on its chip; Enter
/// re-emits `SpaceActivated` for Reading with the warm view, unchanged
/// session count, chrome focus, no profile hint, the exactly-one-selected
/// chip binding, and a zero-reload replay window.
#[test]
#[ignore]
fn keyboard_enter_reactivates_the_focused_same_profile_chip() {
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

    let (mut harness, socket, token) = spawn_harness("spaces-kbd-enter-same", &page_one);

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

        // §11.3/§11.4: the permanent space strip is always part of the rail
        // (the strip presence seam verifies it; there is no reveal step).
        strip_presence(&mut client);

        let keystroke = |client: &mut Client, key: &str| {
            client
                .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                    keystroke: key.into(),
                })
                .expect("keystroke")
        };

        // Keyboard create: Cmd-Shift-N stages a space name, Enter commits
        // and activates the fresh space (chrome focus stays in the strip).
        keystroke(&mut client, "cmd-shift-n");
        since = wait_chrome_focus(&mut client, true, since);
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
        assert_eq!(
            created.active_space, reading,
            "committed space is activated"
        );

        // Warm a tab in reading, then focus the strip on the reading chip.
        let second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client
            .send_ok(&dsl::click_space(reading))
            .expect("focus the space strip on reading");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        assert!(
            client.snapshot().expect("strip focused").chrome_focused,
            "pointer focus keeps the strip chrome domain"
        );

        // Enter re-activation through the one strip authority: with Reading's
        // chip focused, Enter activates the focused chip again (the
        // controller re-emits `SpaceActivated`) — the exact active space,
        // the warm WebViewId, the unchanged session count, the chrome focus
        // domain, and the selected chip all stay bound to Reading, and the
        // replay window proves no reload.
        let before_enter = since;
        keystroke(&mut client, "enter");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        let enter = client.snapshot().expect("enter re-activation snapshot");
        assert_eq!(
            enter.active_space, reading,
            "Enter re-activates the focused chip"
        );
        assert_eq!(
            enter.active_web_view,
            Some(second_view),
            "warm view preserved on re-activation"
        );
        assert_eq!(
            enter.engine_sessions, 2,
            "re-activation never recreates sessions"
        );
        assert!(
            enter.chrome_focused,
            "Enter keeps the strip's chrome focus domain"
        );
        assert_eq!(
            enter.command_profile_hint, None,
            "same-profile re-activation never announces"
        );
        assert_chip_evidence(&enter, reading);
        let reloads = client
            .attach(Some(before_enter))
            .expect("replay enter window")
            .into_iter()
            .filter(|(seq, event)| {
                *seq < since
                    && matches!(
                        event,
                        Event::Web(WebEvent::LoadingChanged(view, true))
                            | Event::Web(WebEvent::UrlChanged(view, _))
                            if *view == second_view
                    )
            })
            .count();
        assert_eq!(reloads, 0, "Enter re-activation must not reload the tab");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// Enter re-activation of the focused cross-profile chip: with the Private
/// space's chip focused after a cross-profile activation (which set the
/// one-interaction hint), Enter re-activates Private through the same
/// authority — the warm view and session count stay untouched, chrome focus
/// stays in the strip, the same-profile re-activation clears the hint
/// (source profile == destination profile), the exactly-one-selected chip
/// binding holds, and the replay window proves no reload.
#[test]
#[ignore]
fn keyboard_enter_reactivates_the_focused_cross_profile_chip() {
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

    let (mut harness, socket, token) = spawn_harness("spaces-kbd-enter-cross", &page_one);

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

        // §11.3/§11.4: the permanent space strip is always part of the rail
        // (the strip presence seam verifies it; there is no reveal step).
        strip_presence(&mut client);

        let keystroke = |client: &mut Client, key: &str| {
            client
                .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                    keystroke: key.into(),
                })
                .expect("keystroke")
        };

        // Same-profile "Reading" and cross-profile "Private", both warm.
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
        let _reading_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
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

        // Leave private for the default profile, then cross into Private
        // again: the cross-profile activation sets the one-interaction
        // hint and leaves the strip focused on the Private chip — the
        // same state the legacy traversal reached before its Enter block.
        client
            .send_ok(&dsl::click_space(default_space))
            .expect("back to default");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        let back = client.snapshot().expect("back to default snapshot");
        assert_eq!(
            back.active_web_view,
            Some(first_view),
            "the default tab stayed warm"
        );
        client
            .send_ok(&dsl::click_space(private_space))
            .expect("cross into private");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let focused = client.snapshot().expect("strip focused on private");
        assert!(focused.chrome_focused, "pointer focus keeps chrome domain");
        assert_eq!(
            focused.command_profile_hint.as_deref(),
            Some("in Profile · Private"),
            "the cross-profile activation announced the destination"
        );

        // Enter re-activation of the focused cross-profile chip routes
        // through the same authority: the controller re-emits
        // `SpaceActivated` for the focused chip, the warm view and session
        // count stay untouched, chrome focus stays in the strip, and the
        // same-profile re-activation clears the one-interaction hint (source
        // profile == destination profile) — the exact hint lifecycle the
        // pointer path follows. The replay window proves no reload.
        let before_enter = since;
        keystroke(&mut client, "enter");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let enter = client
            .snapshot()
            .expect("enter cross re-activation snapshot");
        assert_eq!(
            enter.active_space, private_space,
            "Enter re-activates the focused cross-profile chip"
        );
        assert_eq!(
            enter.active_web_view,
            Some(private_view),
            "warm view preserved on re-activation"
        );
        assert_eq!(
            enter.engine_sessions, sessions,
            "re-activation never recreates sessions"
        );
        assert!(enter.chrome_focused, "Enter keeps chrome focus");
        assert_eq!(
            enter.command_profile_hint, None,
            "same-profile re-activation clears the one-interaction hint"
        );
        assert_chip_evidence(&enter, private_space);
        let reloads = client
            .attach(Some(before_enter))
            .expect("replay enter window")
            .into_iter()
            .filter(|(seq, event)| {
                *seq < since
                    && matches!(
                        event,
                        Event::Web(WebEvent::LoadingChanged(view, true))
                            | Event::Web(WebEvent::UrlChanged(view, _))
                            if *view == private_view
                    )
            })
            .count();
        assert_eq!(reloads, 0, "Enter re-activation must not reload the tab");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
