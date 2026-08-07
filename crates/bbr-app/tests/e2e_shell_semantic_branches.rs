//! Real-app pointer state and typed semantic-tree E2E (Phase-4 closeout).

#![cfg(target_os = "macos")]

mod e2e_common;
mod e2e_semantics_common;

use bbr_testkit::protocol::{SemanticRole, TabVisualState};
use e2e_common::*;
use e2e_semantics_common::*;

#[test]
#[ignore]
fn shell_profile_crash_empty_and_collapsed_semantics() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("sem-branches", &page_one);

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
        let settled = wait_for_window_settled(&mut client, deadline());
        assert!(
            settled.window_size[0] >= SIDEBAR_DEFAULT + 64.0,
            "settled window must fit the default sidebar",
        );
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush first frame");
        // §11.4 compact rail: the profile-identity assertions below expect
        // the second space's `bbr-space-row-1`/`bbr-profile-chip-1` ids;
        // reveal the §11.3 region through the footer `[Spaces]` production
        // pointer path first (the space rows render only while revealed).
        reveal_spaces_region(&mut client);
        // 6. Profile identity in the tree: every space row keeps its §12 chip
        // with the profile monogram; a second profile adds its own row and
        // chip even before activation.
        client
            .send_ok(&dsl::create_profile("Work", "Work Space"))
            .expect("create work profile");
        let (seq, _) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = seq;
        let profiles = client.snapshot().expect("profiles snapshot");
        assert_node(
            &profiles.semantics,
            "bbr-space-row-1",
            SemanticRole::Row,
            Some("Work Space"),
            Some("0"),
            true,
            false,
            false,
        );
        assert_node(
            &profiles.semantics,
            "bbr-profile-chip-1",
            SemanticRole::Chip,
            Some("Profile · WO"),
            None,
            true,
            false,
            false,
        );
        assert_ids(&profiles, &["bbr-profile-chip-1", "bbr-space-row-1"]);

        // 7. Crash branch: the crashed row keeps its §10.7 error state and
        // the glass shows the §11.8 panel with typed buttons; recovery
        // restores the native slot.
        let active_view = profiles.active_web_view.expect("active view");
        client
            .send_ok(&dsl::inject_web_event(WebEvent::PageCrashed(active_view)))
            .expect("inject crash");
        since = wait_web_event(
            &mut client,
            |event| matches!(event, WebEvent::PageCrashed(view) if *view == active_view),
            since,
        );
        let crashed = client.snapshot().expect("crashed snapshot");
        assert_eq!(crashed.tab_states[0], TabVisualState::Error);
        assert_node(
            &crashed.semantics,
            "bbr-crash-panel",
            SemanticRole::Content,
            Some("This page crashed"),
            None,
            true,
            false,
            false,
        );
        assert_node(
            &crashed.semantics,
            "bbr-crash-reload",
            SemanticRole::Button,
            Some("Reload"),
            None,
            true,
            false,
            false,
        );
        assert_node(
            &crashed.semantics,
            "bbr-crash-close",
            SemanticRole::Button,
            Some("Close tab"),
            None,
            true,
            false,
            false,
        );
        client.send_ok(&dsl::reload()).expect("reload crashed tab");
        let recovered = client.snapshot().expect("recovered snapshot");
        assert!(!recovered.tabs[0].crashed);
        assert_eq!(recovered.tab_states[0], TabVisualState::Selected);
        assert_node(
            &recovered.semantics,
            "bbr-native-web-slot",
            SemanticRole::Content,
            Some("Web content"),
            None,
            true,
            false,
            false,
        );

        // 8. Empty-space branch: an empty space of the active profile renders
        // the §11.8 pane naming the profile, with the primary New Tab button.
        let default_profile = recovered.active_profile;
        client
            .send_ok(&dsl::create_space(default_profile, "Empty"))
            .expect("create empty space");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = seq;
        let empty_space = match created {
            BrowserChange::SpaceCreated { space, .. } => space,
            other => panic!("unexpected change: {other:?}"),
        };
        client
            .send_ok(&dsl::activate_space(empty_space))
            .expect("activate empty space");
        let (seq, _) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceActivated { space } if *space == empty_space),
            since,
        );
        since = seq;
        let empty = client.snapshot().expect("empty space snapshot");
        assert_eq!(empty.tabs.len(), 0);
        assert_node(
            &empty.semantics,
            "bbr-empty-new-tab",
            SemanticRole::Content,
            Some("No tabs in this space"),
            Some("in Profile · Default"),
            true,
            false,
            false,
        );
        assert_node(
            &empty.semantics,
            "bbr-empty-new-tab-button",
            SemanticRole::Button,
            Some("New Tab  ⌘T"),
            None,
            true,
            false,
            false,
        );
        assert!(
            find(&empty.semantics, "bbr-tab-list").is_some(),
            "the empty tab list still renders"
        );
        assert!(
            find(&empty.semantics, "bbr-tab-row-0").is_none(),
            "an empty space renders no tab rows"
        );

        // 9. Collapsed sidebar: the tree keeps only the root and the glass
        // (DESIGN §11.4) — no sidebar nodes exist.
        client
            .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                keystroke: "cmd-b".into(),
            })
            .expect("collapse sidebar");
        let _ = wait_chrome_focus(&mut client, true, since);
        let collapsed = client.snapshot().expect("collapsed snapshot");
        assert!(!collapsed.sidebar_visible);
        assert_eq!(collapsed.semantics.len(), 1);
        assert!(
            find(&collapsed.semantics, "bbr-sidebar").is_none(),
            "a collapsed sidebar renders no nodes"
        );
        assert!(
            find(&collapsed.semantics, "bbr-empty-new-tab").is_some(),
            "the glass pane stays after collapse"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
