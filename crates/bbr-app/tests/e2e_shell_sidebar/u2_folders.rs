use super::*;
use bbr_testkit::protocol::dsl;
/// §11.11 folders over the real app: a folder created via the plus menu
/// renders its deterministic row ids; moving a tab into it through the L5
/// command updates the count; the disclosure toggles the nested
/// `bbr-folder-item-<n>-<m>` rows with stable row/name/count/toggle ids and
/// snaps back on the second toggle; a nested member close removes the row.
#[test]
#[ignore]
fn folders_region_disclosure_and_nested_rows() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("sidebar-u2-folders", &page_one);

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
        let _second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));

        // Create a folder through the plus menu (session-local, count 0).
        client
            .send_ok(&dsl::click_footer_plus())
            .expect("open plus menu");
        client
            .send_ok(&dsl::click_plus_menu_item(1))
            .expect("commit New Folder");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after folder creation");
        let created = client.snapshot().expect("folder created snapshot");
        assert_eq!(created.folders.len(), 1);
        let folder_id = created.folders[0].id;
        assert!(
            folder_id > 0,
            "the wire snapshot carries the domain folder id"
        );
        assert_eq!(created.folders[0].tab_count, 0);
        assert!(!created.folders[0].expanded);
        assert_ids(
            &created,
            &[
                "bbr-section-folders",
                "bbr-folders",
                "bbr-folder-row-0",
                "bbr-folder-name-0",
                "bbr-folder-count-0",
                "bbr-folder-toggle-0",
            ],
        );

        // Move tab 0 into the folder through the L5 command path (the
        // folder just created, keyed by its stable domain id): the count
        // updates; the row/name/count/toggle ids stay stable.
        client
            .send_ok(&dsl::move_tab_into_folder(0, folder_id))
            .expect("move tab into folder");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after move into folder");
        let with_member = client.snapshot().expect("folder member snapshot");
        assert_eq!(with_member.folders[0].tab_count, 1);
        assert_eq!(with_member.folders[0].title, "New Folder");
        assert!(!with_member.folders[0].expanded, "starts collapsed");
        assert_ids(
            &with_member,
            &[
                "bbr-folder-row-0",
                "bbr-folder-name-0",
                "bbr-folder-count-0",
                "bbr-folder-toggle-0",
            ],
        );
        assert!(
            !with_member
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-folder-item")),
            "collapsed folders expose no nested rows"
        );
        // The member tab keeps its space membership and session, and is
        // pinned by the grouping (folder member => pinned).
        assert_eq!(with_member.tabs.len(), 2);
        assert_eq!(with_member.engine_sessions, 2);
        assert!(with_member.tabs[0].pinned, "grouping pins the member");
        assert_eq!(with_member.pinned_count, 1);

        // Pointer disclosure toggle: the nested rows render with the
        // deterministic `bbr-folder-item-<n>-<m>` ids and their slots.
        client
            .send_ok(&dsl::click_folder_toggle(folder_id))
            .expect("expand folder");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after expand");
        let expanded = client.snapshot().expect("expanded folder snapshot");
        assert!(expanded.folders[0].expanded, "snap to expanded");
        assert_ids(
            &expanded,
            &[
                "bbr-folder-row-0",
                "bbr-folder-name-0",
                "bbr-folder-count-0",
                "bbr-folder-toggle-0",
                "bbr-folder-item-0-0",
                "bbr-folder-item-0-0-title",
                "bbr-folder-item-0-0-favicon",
                "bbr-folder-item-0-0-close",
            ],
        );
        assert!(
            expanded.chrome_focused,
            "folder pointer entry keeps chrome focus"
        );

        // Nested row selection activates the member tab through the
        // production path.
        client
            .send_ok(&dsl::click_folder_member(folder_id, 0))
            .expect("select folder member");
        let selected = client.snapshot().expect("member selected snapshot");
        assert_eq!(selected.active_tab_index, Some(0), "member tab selected");
        assert_eq!(
            selected.active_web_view,
            Some(first_view),
            "the member's exact session is presented"
        );

        // The second toggle collapses: the nested ids vanish, the row ids
        // stay stable.
        client
            .send_ok(&dsl::click_folder_toggle(folder_id))
            .expect("collapse folder");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after collapse");
        let collapsed = client.snapshot().expect("collapsed folder snapshot");
        assert!(!collapsed.folders[0].expanded, "snap back to collapsed");
        assert_ids(&collapsed, &["bbr-folder-row-0", "bbr-folder-count-0"]);
        assert!(
            !collapsed
                .visible_ids
                .iter()
                .any(|id| id.starts_with("bbr-folder-item")),
            "collapse removes the nested rows"
        );

        // Nested close removes the member row (the folder row survives with
        // the reconciled count).
        client
            .send_ok(&dsl::click_folder_toggle(folder_id))
            .expect("expand again");
        client
            .send_ok(&dsl::click_folder_member_close(folder_id, 0))
            .expect("close folder member");
        client
            .send_ok(&bbr_testkit::Request::FlushFrame)
            .expect("flush after member close");
        let after_close = client.snapshot().expect("member closed snapshot");
        assert_eq!(after_close.folders[0].tab_count, 0, "count reconciled");
        assert_eq!(after_close.tabs.len(), 1, "the member tab closed");
        assert_eq!(after_close.engine_sessions, 1);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
