//! Real-app visible profile/space navigation E2E (Phase-4 space slice).
//!
//! Covers DESIGN.md §11.3/§12 over real WKWebViews: complete profile/space
//! collections with deterministic identity (round-robin hues, chips, stable
//! `bbr-space-row-<n>` / `bbr-profile-chip-<n>` ids), active/selected space
//! states, pointer activation through the production listener path,
//! per-space active-tab restoration with warm WebViewIds and no reloads,
//! empty-space pane naming, cross-profile view-only switches that keep
//! cookie/data-store isolation without recreating sessions, and the
//! recreation policy for tabs that actually cross profiles. The §11.3
//! cross-context command-field hint lives in `e2e_shell_spaces_hint.rs`;
//! the keyboard counterpart in `e2e_shell_spaces_keyboard.rs`. No sleeps:
//! every step synchronizes on semantic events or snapshot state under a
//! deadline. The shared harness lives in `e2e_common`.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::{CommandBarMode, SpaceVisualState, TestProfileId};
use e2e_common::*;

/// Pointer activation of the visible space surface: complete collections with
/// identity, per-space active-tab restoration (warm sessions, no reloads),
/// empty-space pane naming, and cross-profile switches that preserve
/// cookie/data-store isolation without recreating sessions.
#[test]
#[ignore]
fn pointer_switches_spaces_with_identity_and_warm_sessions() {
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
    let set_alpha = fixtures.url("/__fx/cookie/set-alpha");
    let set_beta = fixtures.url("/__fx/cookie/set-beta");
    let read_cookie = fixtures.url("/__fx/cookie/read");

    let (mut harness, socket, token) = spawn_harness("spaces-pointer", &page_one);

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

        // §11.4 compact rail: the footer `[Spaces]` control reveals the
        // §11.3 space rows; drive it before asserting any space-row id or
        // clicking a row. The region hides its rows by default.
        assert!(
            !initial.spaces_visible,
            "compact rail starts with the spaces region hidden"
        );
        let revealed = reveal_spaces_region(&mut client);

        // Complete collections at launch: one profile (Default, cobalt — the
        // first §12 hue) owning one space with its tab. Identity is stable
        // and typed, never a hardcoded label.
        assert_eq!(revealed.profiles.len(), 1);
        assert_eq!(revealed.profiles[0].index, 0);
        assert_eq!(revealed.profiles[0].id, TestProfileId(1));
        assert_eq!(revealed.profiles[0].title, "Default");
        assert_eq!(revealed.profiles[0].hue, "cobalt");
        assert_eq!(revealed.spaces.len(), 1);
        assert_eq!(revealed.spaces[0].index, 0);
        assert_eq!(revealed.spaces[0].space, default_space);
        assert_eq!(revealed.spaces[0].profile, TestProfileId(1));
        assert_eq!(revealed.spaces[0].title, "Default");
        assert!(revealed.spaces[0].active);
        assert_eq!(revealed.spaces[0].tab_count, 1);
        assert_eq!(revealed.spaces[0].active_tab, Some(0));
        assert_eq!(revealed.spaces[0].active_view, Some(first_view));
        assert_eq!(revealed.active_profile_title.as_deref(), Some("Default"));
        assert_eq!(revealed.active_profile_hue.as_deref(), Some("cobalt"));
        assert_eq!(revealed.space_states, vec![SpaceVisualState::Selected]);
        assert_eq!(revealed.space_titles, vec!["Default"]);
        // §11.3/§12/§15 ids for the space surface.
        assert_ids(
            &revealed,
            &[
                "bbr-space-list",
                "bbr-space-new",
                "bbr-space-row-0",
                "bbr-space-title-0",
                "bbr-space-count-0",
                "bbr-profile-chip-0",
            ],
        );
        assert!(
            !initial
                .visible_ids
                .iter()
                .any(|id| id == "bbr-space-list" || id.starts_with("bbr-space-row")),
            "hidden region renders no space rows before the reveal"
        );

        // A second space in the same profile: grouped under Default's group
        // with its own row id, chip, and count; creation does not activate.
        client
            .send_ok(&dsl::create_space(default_profile, "Reading"))
            .expect("create second space");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = seq;
        let (_, reading) = match created {
            BrowserChange::SpaceCreated { profile, space } => (profile, space),
            other => panic!("unexpected change: {other:?}"),
        };
        let two = client.snapshot().expect("two-space snapshot");
        assert_eq!(two.spaces.len(), 2);
        assert_eq!(two.spaces[1].index, 1);
        assert_eq!(two.spaces[1].space, reading);
        assert_eq!(two.spaces[1].title, "Reading");
        assert!(!two.spaces[1].active);
        assert_eq!(two.spaces[1].tab_count, 0);
        assert_eq!(two.spaces[1].active_tab, None);
        assert_eq!(
            two.space_states,
            vec![SpaceVisualState::Selected, SpaceVisualState::Default]
        );
        assert_eq!(two.space_titles, vec!["Default", "Reading"]);
        assert_ids(
            &two,
            &["bbr-space-row-1", "bbr-space-title-1", "bbr-space-count-1"],
        );
        assert_eq!(
            two.profiles[0].hue, "cobalt",
            "profile hue stays stable for the profile's lifetime"
        );

        // Pointer activation of the empty space through the production row
        // listener path: chrome focus domain preserved, default's tab stays
        // warm, and the §11.8 empty pane names the owning profile.
        client
            .send_ok(&dsl::click_space(reading))
            .expect("click reading row");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: reading },
            since,
        );
        let activated = client.snapshot().expect("reading activated snapshot");
        assert_eq!(activated.active_space, reading);
        assert!(
            activated.chrome_focused,
            "pointer activation keeps chrome focus"
        );
        assert_eq!(activated.command_bar_mode, CommandBarMode::Inactive);
        assert_eq!(activated.active_web_view, None, "empty space has no pane");
        assert_eq!(activated.engine_sessions, 1, "default tab stays warm");
        assert_eq!(activated.active_profile_title.as_deref(), Some("Default"));
        assert_eq!(
            activated.space_states,
            vec![SpaceVisualState::Default, SpaceVisualState::Selected]
        );
        assert_ids(
            &activated,
            &["bbr-empty-new-tab", "bbr-empty-new-tab-button"],
        );
        assert!(!activated
            .visible_ids
            .iter()
            .any(|id| id.starts_with("bbr-crash")));

        // Warm the reading space with its own tabs; each space remembers its
        // own active tab (the "each space remembers its active tab" contract).
        let _second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let third_view = open_tab(&mut client, &mut since, &page_one, Some(PAGE_ONE_TITLE));
        let warm = client.snapshot().expect("reading warm snapshot");
        assert_eq!(warm.engine_sessions, 3);
        let reading_row = warm.spaces.iter().find(|s| s.space == reading).unwrap();
        assert_eq!(
            reading_row.active_tab,
            Some(1),
            "reading keeps its last tab"
        );
        assert_eq!(reading_row.active_view, Some(third_view));
        assert_eq!(reading_row.tab_count, 2);

        // Switching back restores default's tab with the same WebViewId and
        // no load events for the re-shown view: warm switching only
        // show/hides, it never reloads or recreates.
        let before_back = since;
        client
            .send_ok(&dsl::click_space(default_space))
            .expect("click default row");
        let (seq_back, _) = client
            .wait_for_event_since(
                |event| matches!(event, Event::BrowserChanged { kind } if *kind == BrowserChange::SpaceActivated { space: default_space }),
                since,
                deadline(),
            )
            .expect("default activated");
        let back = client.snapshot().expect("back to default snapshot");
        assert_eq!(back.active_space, default_space);
        assert_eq!(back.active_web_view, Some(first_view));
        assert_eq!(back.engine_sessions, 3, "warm switch keeps sessions");
        assert!(back.chrome_focused);
        let reloads = client
            .attach(Some(before_back))
            .expect("replay switch window")
            .into_iter()
            .filter(|(seq, event)| {
                *seq < seq_back
                    && matches!(
                        event,
                        Event::Web(WebEvent::LoadingChanged(view, true))
                            | Event::Web(WebEvent::UrlChanged(view, _))
                            if *view == first_view
                    )
            })
            .count();
        assert_eq!(reloads, 0, "space switch must not reload the restored tab");
        since = seq_back;

        // And back to reading: its own last-active tab (the second one) is
        // restored, again without reloads or recreation.
        let before_forward = since;
        client
            .send_ok(&dsl::click_space(reading))
            .expect("click reading again");
        let (seq_forward, _) = client
            .wait_for_event_since(
                |event| matches!(event, Event::BrowserChanged { kind } if *kind == BrowserChange::SpaceActivated { space: reading }),
                since,
                deadline(),
            )
            .expect("reading re-activated");
        let forward = client.snapshot().expect("reading restored snapshot");
        assert_eq!(forward.active_space, reading);
        assert_eq!(forward.active_tab_index, Some(1));
        assert_eq!(forward.active_web_view, Some(third_view));
        assert_eq!(forward.engine_sessions, 3);
        assert!(
            forward.tabs[1].active,
            "reading restores its last active tab"
        );
        let reloads = client
            .attach(Some(before_forward))
            .expect("replay forward window")
            .into_iter()
            .filter(|(seq, event)| {
                *seq < seq_forward
                    && matches!(
                        event,
                        Event::Web(WebEvent::LoadingChanged(view, true))
                            | Event::Web(WebEvent::UrlChanged(view, _))
                            if *view == third_view
                    )
            })
            .count();
        assert_eq!(reloads, 0, "restored space tab must not reload");
        since = seq_forward;

        // Default profile cookie state: set alpha in the default store.
        client
            .send_ok(&dsl::click_space(default_space))
            .expect("click default");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        client
            .send_ok(&dsl::navigate(&set_alpha))
            .expect("set alpha");
        since = wait_title(&mut client, first_view, "Cookie Set alpha", since);
        client
            .send_ok(&dsl::navigate(&read_cookie))
            .expect("read alpha");
        since = wait_title(&mut client, first_view, "Cookie alpha", since);

        // A second profile (Violet — the second §12 hue) with its own space.
        client
            .send_ok(&dsl::create_profile("Private", "Private"))
            .expect("create private profile");
        let (seq, created_profile) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = seq;
        let (private_profile, private_space) = match created_profile {
            BrowserChange::ProfileCreated { profile, space } => (profile, space),
            other => panic!("unexpected change: {other:?}"),
        };
        assert_ne!(private_profile, TestProfileId(1));
        let with_private = client.snapshot().expect("with private snapshot");
        assert_eq!(with_private.profiles.len(), 2);
        assert_eq!(with_private.profiles[1].title, "Private");
        assert_eq!(with_private.profiles[1].hue, "violet");
        assert_eq!(
            with_private.active_profile,
            TestProfileId(1),
            "creation does not activate"
        );

        // View-only switch across the profile boundary: no session recreation,
        // the empty pane names the destination profile, and the chip id for
        // the second profile is rendered.
        client
            .send_ok(&dsl::click_space(private_space))
            .expect("click private row");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let private_view = client.snapshot().expect("private activated snapshot");
        assert_eq!(private_view.active_profile, private_profile);
        assert_eq!(
            private_view.active_profile_title.as_deref(),
            Some("Private")
        );
        assert_eq!(private_view.active_profile_hue.as_deref(), Some("violet"));
        assert_eq!(
            private_view.engine_sessions, 3,
            "view-only switch keeps all sessions"
        );
        assert_eq!(private_view.active_web_view, None);
        assert_eq!(
            private_view.space_states,
            vec![
                SpaceVisualState::Default,
                SpaceVisualState::Default,
                SpaceVisualState::Selected,
            ],
            "one selected row per group, in render order"
        );
        assert_ids(&private_view, &["bbr-profile-chip-1", "bbr-empty-new-tab"]);
        assert_eq!(private_view.spaces[2].tab_count, 0);

        // Private store cookie state: beta lives only in the private store.
        let private_view_id = open_tab(&mut client, &mut since, &set_beta, Some("Cookie Set beta"));
        client
            .send_ok(&dsl::navigate(&read_cookie))
            .expect("read beta");
        since = wait_title(&mut client, private_view_id, "Cookie beta", since);
        let isolated = client.snapshot().expect("private isolated snapshot");
        assert_eq!(isolated.engine_sessions, 4);

        // Back to default: the default view keeps its identity and its store
        // still reads alpha — cross-profile switching never leaks cookies and
        // never recreates the warm private session.
        let before_iso = since;
        client
            .send_ok(&dsl::click_space(default_space))
            .expect("click default after private");
        let (seq_iso, _) = client
            .wait_for_event_since(
                |event| matches!(event, Event::BrowserChanged { kind } if *kind == BrowserChange::SpaceActivated { space: default_space }),
                since,
                deadline(),
            )
            .expect("default re-activated");
        let cross = client.snapshot().expect("cross-profile switch snapshot");
        assert_eq!(
            cross.active_web_view,
            Some(first_view),
            "same WebViewId across the switch"
        );
        assert_eq!(
            cross.engine_sessions, 4,
            "no recreation on cross-profile view"
        );
        let private_row = cross
            .spaces
            .iter()
            .find(|s| s.space == private_space)
            .unwrap();
        assert_eq!(
            private_row.active_view,
            Some(private_view_id),
            "private session stays warm"
        );
        let reloads = client
            .attach(Some(before_iso))
            .expect("replay cross-profile window")
            .into_iter()
            .filter(|(seq, event)| {
                *seq < seq_iso
                    && matches!(
                        event,
                        Event::Web(WebEvent::LoadingChanged(view, true))
                            | Event::Web(WebEvent::UrlChanged(view, _))
                            if *view == first_view
                    )
            })
            .count();
        assert_eq!(
            reloads, 0,
            "cross-profile switch must not reload the default tab"
        );
        since = seq_iso;
        client
            .send_ok(&dsl::navigate(&read_cookie))
            .expect("reread default cookie");
        since = wait_title(&mut client, first_view, "Cookie alpha", since);

        // A tab that actually crosses profiles follows the recreation policy:
        // its old session is destroyed and a fresh one is created in the
        // destination store (recreated: true) with a new WebViewId.
        client
            .send_ok(&dsl::click_space(private_space))
            .expect("click private again");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
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
        let after_move = client.snapshot().expect("after cross-profile move");
        assert!(
            after_move.tabs.is_empty(),
            "private space is empty after the move"
        );
        assert_eq!(
            after_move.engine_sessions, 4,
            "destroy + recreate keeps the count"
        );
        let default_row = after_move
            .spaces
            .iter()
            .find(|s| s.space == default_space)
            .unwrap();
        assert_eq!(default_row.tab_count, 2);
        client
            .send_ok(&dsl::click_space(default_space))
            .expect("click default to view the moved tab");
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: default_space,
            },
            since,
        );
        let moved = client.snapshot().expect("moved tab snapshot");
        assert_eq!(moved.tabs.len(), 2);
        assert_eq!(
            moved.active_tab_index,
            Some(1),
            "destination activates the moved tab"
        );
        let moved_view = moved.active_web_view.expect("fresh incarnation");
        assert_ne!(
            moved_view, private_view_id,
            "cross-profile move recreates the session"
        );
        // The fresh session is backed by the default store: it reads the
        // default store's alpha cookie, never the private store's beta. The
        // title event fires while the move happens, so the wait cursor must
        // predate the move.
        wait_title(&mut client, moved_view, "Cookie alpha", before_move);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
