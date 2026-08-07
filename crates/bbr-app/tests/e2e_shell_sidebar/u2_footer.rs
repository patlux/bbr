use super::*;
use bbr_testkit::protocol::dsl;
/// §11.4 footer row + compact spaces region: exactly `[Downloads] [Spaces]
/// [+]` (no fourth control), the spaces region hidden at launch with no
/// space/chip ids in the §9 traversal, the `[Spaces]` control revealing and
/// hiding the §11.3 rows with unchanged `bbr-space-row-<n>` ids through the
/// production pointer path, the empty favorites/folders regions hiding their
/// labels, and the status row + collapse control ids unchanged.
#[test]
#[ignore]
fn footer_row_and_spaces_toggle_reveal_and_hide_the_compact_rail() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("sidebar-u2-footer", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let _since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        let first_view = initial.active_web_view.expect("first native view");

        // §11.4 footer: exactly the three controls with their §15 ids; no
        // fourth control id is exposed.
        assert_ids(
            &initial,
            &["bbr-footer", "bbr-downloads", "bbr-spaces", "bbr-plus"],
        );
        assert!(
            !initial
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-downloads-2")),
            "no fourth footer control"
        );
        // Status row + collapse control keep their ids in the reordered rail.
        assert_ids(&initial, &["bbr-status", "bbr-sidebar-toggle"]);

        // §11.4 compact default: the spaces region is hidden — no space-row,
        // space-list, or chip id is part of the traversal.
        assert!(!initial.spaces_visible, "spaces region hidden at launch");
        for id in &initial.visible_ids {
            assert!(
                !(id.starts_with("bbr-space-row")
                    || id == "bbr-space-list"
                    || id == "bbr-space-new"
                    || id.starts_with("bbr-profile-chip")),
                "hidden region exposes {id}"
            );
        }
        // Empty favorites/folders regions hide their labels with them.
        assert!(!initial
            .visible_ids
            .iter()
            .any(|id| id == "bbr-section-favorites" || id == "bbr-section-folders"));
        assert!(initial.favorites.is_empty());
        assert!(initial.folders.is_empty());

        // The `[Spaces]` control reveals the §11.3 rows through the
        // production pointer path: unchanged row ids, chips, and the active
        // space selected.
        let revealed = reveal_spaces_region(&mut client);
        assert!(revealed.spaces_visible);
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
        assert_eq!(revealed.active_web_view, Some(first_view));
        assert_eq!(revealed.engine_sessions, 1, "toggle never touches sessions");

        // A second activation hides the region again (snap back).
        client
            .send_ok(&dsl::click_footer_spaces())
            .expect("hide spaces region");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after hide");
        let hidden = client.snapshot().expect("hidden snapshot");
        assert!(!hidden.spaces_visible);
        assert!(!hidden
            .visible_ids
            .iter()
            .any(|id| id == "bbr-space-list" || id.starts_with("bbr-space-row")));
        assert_ids(
            &hidden,
            &["bbr-footer", "bbr-downloads", "bbr-spaces", "bbr-plus"],
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// §11.4 plus menu over the real app: `[+]` opens `bbr-plus-menu` with the
/// two v1 items; keyboard `↑`/`↓` wraps the selection, Enter commits, Escape
/// dismisses without action; the pointer path commits New Space (a real
/// space in the active profile, visible in the revealed region, without
/// switching) and New Folder (a session-local folder with count 0).
#[test]
#[ignore]
fn plus_menu_pointer_and_keyboard_create_real_space_and_folder() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("sidebar-u2-plusmenu", &page_one);

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
        let default_profile = initial.active_profile;
        reveal_spaces_region(&mut client);

        // `[+]` opens the menu: surface id, two item ids, selection 0.
        client
            .send_ok(&dsl::click_footer_plus())
            .expect("open plus menu");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after menu open");
        let open = client.snapshot().expect("plus menu open snapshot");
        assert!(open.plus_menu_open);
        assert_eq!(open.plus_menu_selection, 0);
        assert_ids(
            &open,
            &["bbr-plus-menu", "bbr-plus-new-space", "bbr-plus-new-folder"],
        );

        // Keyboard: down → New Folder, down wraps to New Space, up back to
        // New Folder; Escape dismisses without creating anything.
        let keystroke = |client: &mut Client, key: &str| {
            client
                .send_ok(&bbr_testkit::Request::DispatchKeystroke {
                    keystroke: key.into(),
                })
                .expect("keystroke")
        };
        keystroke(&mut client, "down");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after down");
        let down = client.snapshot().expect("menu down snapshot");
        assert_eq!(down.plus_menu_selection, 1, "down selects New Folder");
        keystroke(&mut client, "down");
        let wrapped = client.snapshot().expect("menu wrap snapshot");
        assert_eq!(wrapped.plus_menu_selection, 0, "down wraps to New Space");
        keystroke(&mut client, "up");
        let up = client.snapshot().expect("menu up snapshot");
        assert_eq!(up.plus_menu_selection, 1, "up returns to New Folder");
        keystroke(&mut client, "escape");
        let dismissed = client.snapshot().expect("dismissed snapshot");
        assert!(!dismissed.plus_menu_open, "Escape dismisses without action");
        assert_eq!(dismissed.spaces.len(), 1, "no space was created");
        assert!(dismissed.folders.is_empty(), "no folder was created");
        assert_eq!(dismissed.engine_sessions, 1);

        // Pointer New Space: the menu opens again, the item commits a real
        // space in the active profile (visible in the revealed region, not
        // activated), and the menu closes.
        client
            .send_ok(&dsl::click_footer_plus())
            .expect("reopen plus menu");
        let (_seq, created) = {
            client
                .send_ok(&dsl::click_plus_menu_item(0))
                .expect("commit New Space");
            wait_browser_change_matching(
                &mut client,
                |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
                since,
            )
        };
        let (created_profile, created_space) = match created {
            BrowserChange::SpaceCreated { profile, space } => (profile, space),
            other => panic!("unexpected change: {other:?}"),
        };
        let after_space = client.snapshot().expect("after New Space snapshot");
        assert_eq!(
            created_profile, default_profile,
            "created in the active profile"
        );
        assert_eq!(after_space.spaces.len(), 2);
        assert_eq!(after_space.spaces[1].space, created_space);
        assert_eq!(after_space.spaces[1].title, "New Space");
        assert_eq!(after_space.spaces[1].tab_count, 0);
        assert!(!after_space.spaces[1].active, "creation does not switch");
        assert!(!after_space.plus_menu_open, "menu closed after commit");
        assert_ids(&after_space, &["bbr-space-row-1", "bbr-space-title-1"]);
        assert_eq!(after_space.active_web_view, Some(first_view));

        // Pointer New Folder: a session-local folder with count 0 renders its
        // §11.11 row ids; the region label appears.
        client
            .send_ok(&dsl::click_footer_plus())
            .expect("open plus menu for folder");
        client
            .send_ok(&dsl::click_plus_menu_item(1))
            .expect("commit New Folder");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after folder");
        let after_folder = client.snapshot().expect("after New Folder snapshot");
        assert_eq!(after_folder.folders.len(), 1);
        assert_eq!(after_folder.folders[0].title, "New Folder");
        assert_eq!(after_folder.folders[0].tab_count, 0);
        assert!(!after_folder.folders[0].expanded);
        assert!(!after_folder.plus_menu_open);
        assert_ids(
            &after_folder,
            &[
                "bbr-section-folders",
                "bbr-folders",
                "bbr-folder-row-0",
                "bbr-folder-name-0",
                "bbr-folder-count-0",
                "bbr-folder-toggle-0",
            ],
        );
        assert_eq!(
            after_folder.engine_sessions, 1,
            "menu actions never touch sessions"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
