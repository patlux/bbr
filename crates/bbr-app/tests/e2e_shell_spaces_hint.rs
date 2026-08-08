//! Real-app cross-context command-field profile hint E2E (DESIGN.md §11.3).
//!
//! Over real WKWebViews: a cross-profile space activation renders the
//! destination profile chip + "in Profile · <title>" hint at the top of the
//! command field for one interaction, same-profile switching never announces
//! a transition, the hint clears at a deterministic user-interaction boundary
//! (a tab mutation or command-bar mode entry), the staged-move hint is a
//! separate surface that never coexists, and activating an empty space in
//! another profile announces the destination while the empty pane names the
//! owning profile. Keyboard counterpart: `e2e_shell_spaces_keyboard.rs`;
//! ordinary spaces surface: `e2e_shell_spaces.rs`. No sleeps; shared
//! harness in `e2e_common`.

#![cfg(target_os = "macos")]

mod e2e_common;
mod e2e_semantics_common;

use bbr_testkit::protocol::{CommandBarMode, SemanticRole, TestProfileId};
use e2e_common::*;
use e2e_semantics_common::find;

/// The §11.3 cross-context rule over real WKWebViews: a cross-profile space
/// activation renders the destination profile chip + "in Profile · <title>"
/// hint at the top of the command field for one interaction, same-profile
/// switching never announces a profile transition, the next chrome context
/// mutation clears the hint (a deterministic user-interaction boundary), and
/// the staged-move hint is a separate surface that never coexists with it.
#[test]
#[ignore]
fn cross_profile_switch_shows_and_clears_the_command_field_profile_hint() {
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

    let (mut harness, socket, token) = spawn_harness("spaces-hint", &page_one);

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

        // §11.4 compact rail: the §11.3 space strip is always part of the
        // footer, so the strip presence seam verifies the chip surface
        // before the cross-context evidence is asserted.
        strip_presence(&mut client);

        // A same-profile space and a cross-profile profile+space, both warm.
        client
            .send_ok(&dsl::create_space(default_profile, "Work"))
            .expect("create work space");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );
        since = seq;
        let work = match created {
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
            .send_ok(&dsl::click_space(private_space))
            .expect("activate private");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let private_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        let sessions = client.snapshot().expect("sessions").engine_sessions;
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

        // Same-profile switching never announces a profile transition: the
        // typed hint is None, no hint id is rendered, and focus/sessions are
        // untouched.
        client
            .send_ok(&dsl::click_space(work))
            .expect("activate work (same profile)");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: work },
            since,
        );
        let work_active = client.snapshot().expect("work snapshot");
        assert_eq!(
            work_active.command_profile_hint, None,
            "same-profile switching must not announce a profile transition"
        );
        assert!(!work_active
            .visible_ids
            .iter()
            .any(|id| id == "bbr-command-profile-hint"));
        assert_eq!(work_active.engine_sessions, sessions);
        assert!(work_active.chrome_focused);
        assert_eq!(work_active.command_bar_mode, CommandBarMode::Inactive);

        // Cross-profile activation sets the hint for one interaction: the
        // typed copy names the destination profile, the chip + hint ids and
        // the §15 semantic node are present, chrome focus stays, and the
        // warm session is neither recreated nor reloaded.
        let before_cross = since;
        client
            .send_ok(&dsl::click_space(private_space))
            .expect("activate private (cross profile)");
        let (seq_cross, _) = client
            .wait_for_event_since(
                |event| matches!(event, Event::BrowserChanged { kind } if *kind == BrowserChange::SpaceActivated { space: private_space }),
                since,
                deadline(),
            )
            .expect("private activated");
        let cross = client.snapshot().expect("cross snapshot");
        assert_eq!(
            cross.command_profile_hint.as_deref(),
            Some("in Profile · Private"),
            "cross-profile destination hint copy"
        );
        assert_ids(&cross, &["bbr-command-profile-hint"]);
        // `visible_ids` carries the hint root; the destination profile chip
        // is the root's semantic child and is intentionally not flattened
        // into `visible_ids` — its role, name, and parent relationship are
        // proven through the typed semantic tree below.
        let hint_node =
            find(&cross.semantics, "bbr-command-profile-hint").expect("hint semantic node");
        assert_eq!(hint_node.role, SemanticRole::Chip);
        assert_eq!(
            hint_node.name.as_deref(),
            Some("in Profile · Private"),
            "semantic hint name"
        );
        let profile_chip = hint_node
            .children
            .iter()
            .find(|c| c.id == "bbr-profile-chip-1")
            .expect("hint chip child in the semantic tree");
        assert_eq!(profile_chip.role, SemanticRole::Chip, "profile chip role");
        assert_eq!(
            profile_chip.name.as_deref(),
            Some("Profile · PR"),
            "hint chip child carries the §12 monogram"
        );
        assert_eq!(cross.active_space, private_space);
        assert_eq!(cross.active_web_view, Some(private_view));
        assert_eq!(
            cross.engine_sessions, sessions,
            "cross-profile view-only switch keeps all sessions"
        );
        assert!(cross.chrome_focused, "activation keeps chrome focus");
        let reloads = client
            .attach(Some(before_cross))
            .expect("replay cross window")
            .into_iter()
            .filter(|(seq, event)| {
                *seq < seq_cross
                    && matches!(
                        event,
                        Event::Web(WebEvent::LoadingChanged(view, true))
                            | Event::Web(WebEvent::UrlChanged(view, _))
                            if *view == private_view
                    )
            })
            .count();
        assert_eq!(reloads, 0, "cross-profile switch must not reload");
        since = seq_cross;

        // The hint clears at a deterministic user-interaction boundary: a
        // command-bar interaction (Cmd-T) ends the one-interaction display.
        client
            .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                keystroke: "cmd-t".into(),
            })
            .expect("cmd-t");
        since = wait_chrome_focus(&mut client, true, since);
        let staged = client.snapshot().expect("new tab staged");
        assert_eq!(staged.command_bar_mode, CommandBarMode::NewTab);
        assert_eq!(
            staged.command_profile_hint, None,
            "a command-bar interaction clears the one-interaction hint"
        );
        assert!(!staged
            .visible_ids
            .iter()
            .any(|id| id == "bbr-command-profile-hint"));
        client
            .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                keystroke: "escape".into(),
            })
            .expect("escape");
        // Escape restores the prior chrome focus domain (the activation left
        // focus in the space strip), so no focus event needs waiting here;
        // the staged mode is simply gone.
        let escaped = client.snapshot().expect("escaped snapshot");
        assert_eq!(escaped.command_bar_mode, CommandBarMode::Inactive);

        // Switching back to the default profile is itself cross-profile: the
        // hint names the new destination, then a tab mutation (pointer
        // selection) clears it.
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
            back.command_profile_hint.as_deref(),
            Some("in Profile · Default"),
            "the reverse direction is cross-profile too"
        );
        assert_eq!(back.active_web_view, Some(first_view));
        client
            .send_ok(&dsl::click_tab(0))
            .expect("select the default tab");
        let seq = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        since = seq;
        let after_tab = client.snapshot().expect("after tab selection");
        assert_eq!(
            after_tab.command_profile_hint, None,
            "a tab mutation clears the one-interaction hint"
        );

        // The staged-move hint is a separate surface: staging a move into the
        // private space shows the MoveTab cross-profile hint and never the
        // space-switch transient (they never coexist).
        client
            .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                keystroke: "cmd-shift-m".into(),
            })
            .expect("cmd-shift-m");
        let _ = wait_chrome_focus(&mut client, true, since);
        client
            .send_ok(&bbr_testkit::Request::SetCommandBarText {
                text: "Private".into(),
            })
            .expect("set move destination");
        let moved_hint = client.snapshot().expect("staged move snapshot");
        assert_eq!(
            moved_hint.command_bar_hint.as_deref(),
            Some("in Profile · Private → Private"),
            "MoveTab keeps its own cross-profile hint copy"
        );
        assert_eq!(
            moved_hint.command_profile_hint, None,
            "the staged move clears the space-switch transient"
        );
        assert!(moved_hint
            .visible_ids
            .iter()
            .any(|id| id == "bbr-command-hint"));
        assert!(!moved_hint
            .visible_ids
            .iter()
            .any(|id| id == "bbr-command-profile-hint"));

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The §11.3 empty-branch over real WKWebViews: activating an empty space of
/// a different profile announces the destination profile chip + hint at the
/// top of the command field for one interaction while the empty pane names
/// the owning profile and no web view renders; the hint stays tied to the
/// active profile (a reverse cross-profile activation re-sets it, never
/// leaves stale copy), and a command-bar mode entry clears it.
#[test]
#[ignore]
fn cross_profile_activation_of_an_empty_space_announces_the_destination_profile() {
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

    let (mut harness, socket, token) = spawn_harness("spaces-hint-empty", &page_one);

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
        let sessions_at_launch = initial.engine_sessions;

        // §11.4 compact rail: the §11.3 space strip is always part of the
        // footer, so the strip presence seam verifies the chip surface
        // before the empty-branch cross-context evidence is asserted.
        strip_presence(&mut client);

        // A second profile whose single space starts empty; creation must
        // not activate it.
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
        assert_eq!(
            with_private.active_profile, default_profile,
            "creation does not activate"
        );
        assert_eq!(
            with_private.profiles[1].title, "Private",
            "the second profile carries its typed identity"
        );
        assert_eq!(with_private.profiles[1].hue, "violet");
        let private_row = with_private
            .spaces
            .iter()
            .find(|s| s.space == private_space)
            .unwrap();
        assert_eq!(private_row.tab_count, 0);
        assert_eq!(private_row.active_tab, None);
        assert_eq!(private_row.active_view, None);

        // Cross-profile activation of the EMPTY space through the production
        // row listener path: the §11.3 hint + §12 chip render with the
        // explicit destination copy, the empty pane names the owning profile
        // and renders no web view, chrome focus stays, and the warm default
        // session is neither recreated nor destroyed (view-only switch).
        client
            .send_ok(&dsl::click_space(private_space))
            .expect("click empty private row");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let empty_cross = client.snapshot().expect("empty cross-profile snapshot");
        assert_eq!(empty_cross.active_space, private_space);
        assert_eq!(empty_cross.active_profile, private_profile);
        assert_eq!(
            empty_cross.active_profile_title.as_deref(),
            Some("Private"),
            "the empty pane names the owning profile"
        );
        assert_eq!(empty_cross.active_profile_hue.as_deref(), Some("violet"));
        assert_eq!(
            empty_cross.active_web_view, None,
            "an empty space renders no web view"
        );
        assert_eq!(
            empty_cross.command_profile_hint.as_deref(),
            Some("in Profile · Private"),
            "the empty cross-profile activation announces the destination"
        );
        assert_ids(
            &empty_cross,
            &[
                "bbr-command-profile-hint",
                "bbr-empty-new-tab",
                "bbr-empty-new-tab-button",
            ],
        );
        // `visible_ids` carries the hint root; the destination profile chip
        // is the root's semantic child and is intentionally not flattened
        // into `visible_ids` — its role, name, and parent relationship are
        // proven through the typed semantic tree below.
        let hint_node = find(&empty_cross.semantics, "bbr-command-profile-hint")
            .expect("hint semantic node on the empty branch");
        assert_eq!(hint_node.role, SemanticRole::Chip);
        assert_eq!(
            hint_node.name.as_deref(),
            Some("in Profile · Private"),
            "semantic hint name on the empty branch"
        );
        let profile_chip = hint_node
            .children
            .iter()
            .find(|c| c.id == "bbr-profile-chip-1")
            .expect("hint chip child in the semantic tree on the empty branch");
        assert_eq!(
            profile_chip.role,
            SemanticRole::Chip,
            "profile chip role on the empty branch"
        );
        assert_eq!(
            profile_chip.name.as_deref(),
            Some("Profile · PR"),
            "hint chip child carries the §12 monogram on the empty branch"
        );
        assert!(
            empty_cross.chrome_focused,
            "activation of an empty space keeps chrome focus"
        );
        assert_eq!(empty_cross.command_bar_mode, CommandBarMode::Inactive);
        assert_eq!(
            empty_cross.engine_sessions, sessions_at_launch,
            "the empty branch is a view-only switch: no recreation, no teardown"
        );
        assert_eq!(
            empty_cross
                .space_chips
                .iter()
                .filter(|c| c.selected)
                .count(),
            1,
            "exactly one chip renders selected"
        );
        assert!(!empty_cross
            .visible_ids
            .iter()
            .any(|id| id.starts_with("bbr-crash")));

        // The hint is present only while it matches the active profile: a
        // reverse cross-profile activation back to default re-sets it for
        // the new destination (never leaves the stale "Private" copy).
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
        let back_default = client.snapshot().expect("back to default snapshot");
        assert_eq!(
            back_default.command_profile_hint.as_deref(),
            Some("in Profile · Default"),
            "the reverse direction re-sets the hint, no stale copy"
        );
        assert_eq!(back_default.active_profile, default_profile);
        assert_eq!(
            back_default.active_web_view,
            Some(first_view),
            "the default tab stayed warm across the empty branch"
        );
        assert_eq!(back_default.engine_sessions, sessions_at_launch);
        assert!(back_default.chrome_focused);

        // Deterministic lifetime boundary on the empty branch: a command-bar
        // mode entry (Cmd-T) ends the one-interaction display.
        client
            .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                keystroke: "cmd-t".into(),
            })
            .expect("cmd-t");
        let _ = wait_chrome_focus(&mut client, true, since);
        let staged = client.snapshot().expect("new tab staged");
        assert_eq!(staged.command_bar_mode, CommandBarMode::NewTab);
        assert_eq!(
            staged.command_profile_hint, None,
            "a command-bar interaction clears the one-interaction hint"
        );
        assert!(!staged
            .visible_ids
            .iter()
            .any(|id| id == "bbr-command-profile-hint"));
        client
            .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                keystroke: "escape".into(),
            })
            .expect("escape");
        let escaped = client.snapshot().expect("escaped snapshot");
        assert_eq!(escaped.command_bar_mode, CommandBarMode::Inactive);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
