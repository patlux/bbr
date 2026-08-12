use super::*;
use bbr_testkit::protocol::dsl;
/// §11.3/§11.4 footer row: exactly `[Downloads] [Space Strip] [+]` (no
/// fourth control), the permanent space strip always part of the rail with
/// one `bbr-space-chip-<persistent-space-id>` + `bbr-space-identity-<persistent-
/// space-id>` per space (selected exactly on the active space), the empty
/// favorites/folders regions hiding their labels, and the status row +
/// collapse control ids unchanged. The legacy `[Spaces]` reveal toggle is
/// gone: the strip never hides and there is nothing to reveal.
#[test]
#[ignore]
fn footer_row_and_permanent_strip_render_in_the_compact_rail() {
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
            &["bbr-footer", "bbr-downloads", "bbr-space-strip", "bbr-plus"],
        );
        assert!(
            !initial
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-downloads-2")),
            "no fourth footer control"
        );
        // The legacy reveal surface is gone end to end: no `[Spaces]`
        // toggle id and no revealed-list row/identity ids render anywhere
        // in the rail.
        assert!(
            !initial.visible_ids.iter().any(|id| id == "bbr-spaces"
                || id.starts_with("bbr-space-row")
                || id.starts_with("bbr-space-list")),
            "no legacy [Spaces] toggle or revealed-list ids render"
        );
        // Status row + collapse control keep their ids in the reordered rail.
        assert_ids(&initial, &["bbr-status", "bbr-sidebar-toggle"]);

        // §11.3 permanent strip: one chip (and its identity child) per
        // space in canonical display order, ids from the persistent space
        // id, selected exactly on the active space.
        assert!(initial.visible_ids.iter().any(|id| id == "bbr-space-strip"));
        assert_eq!(initial.space_chips.len(), 1);
        assert_eq!(initial.space_chips[0].space, initial.active_space);
        assert!(initial.space_chips[0].selected);
        let chip_id = format!("bbr-space-chip-{}", initial.active_space.0);
        let identity_id = format!("bbr-space-identity-{}", initial.active_space.0);
        assert!(initial.visible_ids.contains(&chip_id));
        assert!(initial.visible_ids.contains(&identity_id));
        // Empty favorites/folders regions hide their labels with them.
        assert!(!initial
            .visible_ids
            .iter()
            .any(|id| id == "bbr-section-favorites" || id == "bbr-section-folders"));
        assert!(initial.favorites.is_empty());
        assert!(initial.folders.is_empty());

        // The strip is always present: a second snapshot shows the same
        // always-rendered chip ids, and activation never touches sessions.
        let always = strip_presence(&mut client);
        assert!(always.space_chips.iter().any(|c| c.selected));
        assert_eq!(always.active_web_view, Some(first_view));
        assert_eq!(
            always.engine_sessions, 1,
            "strip presence never touches sessions"
        );
        assert_ids(
            &always,
            &["bbr-footer", "bbr-downloads", "bbr-space-strip", "bbr-plus"],
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
/// space in the active profile, visible in the permanent strip, without
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
        // §11.3/§11.4: the permanent strip is always part of the rail; the
        // strip presence seam verifies it before the plus-menu steps.
        strip_presence(&mut client);

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
        // space in the active profile and activates it immediately (the
        // standing daily-driver decision), the §11.3 configuration surface
        // opens on top of the usable Space, and the menu closes.
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
        assert_eq!(
            after_space.active_space, created_space,
            "creation activates immediately"
        );
        assert!(after_space.spaces[1].active, "created space is active");
        assert!(!after_space.plus_menu_open, "menu closed after commit");
        assert!(
            after_space
                .visible_ids
                .iter()
                .any(|id| id == &format!("bbr-space-chip-{}", created_space.0)),
            "created space chip id derives from its persistent space id"
        );
        assert_eq!(
            after_space.active_web_view, None,
            "the empty created space has no pane"
        );

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
