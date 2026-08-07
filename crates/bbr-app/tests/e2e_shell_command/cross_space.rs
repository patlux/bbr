//! Cross-space command E2E: selecting standalone spaces through the command
//! surface preserves warm sessions and profile isolation, and — the
//! cross-space tab evidence — warm tabs in other spaces/profiles rank with
//! their persistent identity and activate with exact view reuse.

use super::*;

#[test]
#[ignore]
fn cross_space_selection_preserves_warm_sessions_and_profile_isolation() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-spaces", &page_one);

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
        let default_profile = first.active_profile;
        let default_space = first.active_space;

        // A second warm space in the default profile.
        client
            .send_ok(&dsl::create_space(default_profile, "Reading"))
            .expect("create reading");
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
            .send_ok(&dsl::activate_space(reading))
            .expect("activate reading");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        let reading_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));

        // A private profile + space, also warm.
        client
            .send_ok(&dsl::create_profile("Private", "Private"))
            .expect("create private");
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
            .send_ok(&dsl::activate_space(private_space))
            .expect("activate private");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let private_view = open_tab(&mut client, &mut since, &page_one, Some(PAGE_ONE_TITLE));
        let warm = client.snapshot().expect("warm state snapshot");
        let sessions = warm.engine_sessions;
        assert_eq!(sessions, 3);

        // Back to the default space: warm view, no session churn.
        client
            .send_ok(&dsl::activate_space(default_space))
            .expect("activate default");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );

        // Same-profile space selection through the command surface: the
        // Reading row activates with the same warm WebViewId.
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "reading".into(),
            })
            .expect("query reading");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush reading");
        let rows = client.snapshot().expect("reading rows");
        let row = rows
            .command_results
            .iter()
            .find(|row| row.key == format!("space:{}", reading.0))
            .expect("reading row");
        assert_eq!(row.subtitle.as_deref(), Some("Space"));
        key(&mut client, "enter");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        let reading_active = client.snapshot().expect("reading active");
        assert_eq!(reading_active.active_space, reading);
        assert_eq!(reading_active.active_web_view, Some(reading_view));
        assert_eq!(
            reading_active.engine_sessions, sessions,
            "warm space selection never recreates sessions"
        );
        assert!(reading_active.chrome_focused);

        // Cross-profile space selection: the Private row exposes its profile
        // identity and activates with the private space's warm view.
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "private".into(),
            })
            .expect("query private");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush private");
        let rows = client.snapshot().expect("private rows");
        let row = rows
            .command_results
            .iter()
            .find(|row| row.key == format!("space:{}", private_space.0))
            .expect("private space row");
        assert_eq!(
            row.subtitle.as_deref(),
            Some("Profile Private"),
            "cross-profile identity is never hidden"
        );
        key(&mut client, "enter");
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let private_active = client.snapshot().expect("private active");
        assert_eq!(
            private_active.active_profile,
            private_active
                .spaces
                .iter()
                .find(|space| space.space == private_space)
                .expect("private space entry")
                .profile
        );
        assert_eq!(private_active.active_space, private_space);
        assert_eq!(private_active.active_web_view, Some(private_view));
        assert_eq!(
            private_active.engine_sessions, sessions,
            "profile-isolated spaces stay warm across the command-surface switch"
        );
        assert!(private_active.chrome_focused);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn other_space_warm_tabs_activate_with_exact_identity() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-warmtabs", &page_one);

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
        let default_profile = first.active_profile;
        let default_space = first.active_space;
        let _ = first_view;

        // A second warm space in the default profile with a warm page_two tab;
        // capture its persistent identity from the complete snapshot.
        client
            .send_ok(&dsl::create_space(default_profile, "Reading"))
            .expect("create reading");
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
            .send_ok(&dsl::activate_space(reading))
            .expect("activate reading");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        let reading_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let reading_tab = client
            .snapshot()
            .expect("reading identity snapshot")
            .all_tabs
            .iter()
            .find(|tab| tab.space == reading && tab.web_view == Some(reading_view))
            .cloned()
            .expect("reading tab identity");

        // A private profile + space with a warm page_one tab (the profile
        // store isolation evidence).
        client
            .send_ok(&dsl::create_profile("Private", "Private"))
            .expect("create private");
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
            .send_ok(&dsl::activate_space(private_space))
            .expect("activate private");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let private_view = open_tab(&mut client, &mut since, &page_one, Some(PAGE_ONE_TITLE));
        let private_tab = client
            .snapshot()
            .expect("private identity snapshot")
            .all_tabs
            .iter()
            .find(|tab| tab.space == private_space && tab.web_view == Some(private_view))
            .cloned()
            .expect("private tab identity");
        let private_profile = private_tab.profile;
        let warm = client.snapshot().expect("warm snapshot");
        let sessions = warm.engine_sessions;
        assert_eq!(sessions, 3);

        // Back to the default space.
        client
            .send_ok(&dsl::activate_space(default_space))
            .expect("activate default");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );

        // Same-profile other-space tab through the command surface: the row
        // carries the persistent tab id; Enter activates the owning space and
        // selects the exact owning-space index with the exact warm view.
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "page2".into(),
            })
            .expect("query page2");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush page2");
        let rows = client.snapshot().expect("page2 rows");
        let row = rows
            .command_results
            .iter()
            .find(|row| row.key == format!("tab:{}", reading_tab.id))
            .expect("reading tab row");
        assert_eq!(row.group, "other_space");
        assert_eq!(row.subtitle.as_deref(), Some("Space Reading"));
        key(&mut client, "enter");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        let reading_active = client.snapshot().expect("reading active snapshot");
        assert_eq!(reading_active.active_space, reading);
        assert_eq!(
            reading_active.active_web_view,
            Some(reading_view),
            "the exact warm view is reused"
        );
        assert_eq!(
            reading_active.active_tab_index,
            Some(reading_tab.index),
            "the exact owning-space index is selected"
        );
        assert_eq!(
            reading_active.engine_sessions, sessions,
            "no session recreation for the other-space warm tab"
        );
        assert!(
            !reading_active.chrome_focused,
            "Enter selects the tab into web focus"
        );
        assert_eq!(reading_active.command_bar_mode, CommandBarMode::Inactive);

        // Cross-profile other-space tab: the row exposes profile identity and
        // activates the private space with the private tab's warm view.
        let _ = cmd_l(&mut client, &mut since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "page1".into(),
            })
            .expect("query page1");
        client
            .send_ok(&bbr_testkit::Request::FlushCommandRanking)
            .expect("flush page1");
        let rows = client.snapshot().expect("page1 rows");
        // "page1" matches two other-space tabs (the default space's page_one
        // tab sorts before the private one), so move the selection to the
        // intended private tab row before Enter — the production Up/Down
        // path.
        let private_index = rows
            .command_results
            .iter()
            .position(|row| row.key == format!("tab:{}", private_tab.id))
            .expect("private tab row");
        assert_eq!(rows.command_results[private_index].group, "other_space");
        assert_eq!(
            rows.command_results[private_index].subtitle.as_deref(),
            Some("Profile Private · Space Private"),
            "cross-profile tab identity is never hidden"
        );
        for _ in 0..private_index {
            key(&mut client, "down");
        }
        key(&mut client, "enter");
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let private_active = client.snapshot().expect("private active snapshot");
        assert_eq!(private_active.active_space, private_space);
        assert_eq!(
            private_active.active_profile,
            bbr_web::ProfileId(private_profile),
            "the active profile is the tab's owning profile (store isolation)"
        );
        assert_eq!(
            private_active.active_web_view,
            Some(private_view),
            "the exact cross-profile warm view is reused"
        );
        assert_eq!(private_active.active_tab_index, Some(private_tab.index));
        assert_eq!(
            private_active.engine_sessions, sessions,
            "cross-profile warm tabs stay warm"
        );
        assert!(!private_active.chrome_focused);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
