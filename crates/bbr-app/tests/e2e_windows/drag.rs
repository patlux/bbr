//! A tab dragged onto another window's rail or card.

use super::*;

/// Two windows apart: the first on the default space with `page_one` and
/// the long-title page, the second moved 1400 pt to the right on space
/// `space_title` (created in profile `profile`, or a new profile when
/// `None`) showing `page_two`. Returns the second window's key.
fn two_windows_apart(
    client: &mut Client,
    since: &mut u64,
    long: &str,
    page_two: &str,
    new_profile: bool,
) -> u64 {
    open_tab(client, since, long, None);
    let first = settle(client, "the long page", |s| s.tabs.len() == 2);
    let profile = first.active_profile;
    let (second, _) = new_window(client);
    if new_profile {
        client
            .send_ok(&bbr_testkit::protocol::dsl::create_profile(
                "Private",
                "Elsewhere",
            ))
            .expect("create a profile");
    } else {
        client
            .send_ok(&bbr_testkit::protocol::dsl::create_space(
                profile,
                "Elsewhere",
            ))
            .expect("create a space");
    }
    let target = settle(client, "the other space", |s| {
        s.spaces.iter().any(|space| space.title == "Elsewhere")
    })
    .spaces
    .iter()
    .find(|space| space.title == "Elsewhere")
    .unwrap()
    .space;
    client
        .send_ok(&bbr_testkit::protocol::dsl::activate_space(target))
        .expect("activate it in the second window");
    settle(client, "the second window on it", |s| {
        s.active_space == target
    });
    open_tab(client, since, page_two, None);
    client
        .send_ok(&bbr_testkit::Request::MoveWindowBy {
            window: second,
            dx: 1400.0,
            dy: 0.0,
        })
        .expect("move the second window apart");
    second
}

/// The source-window pointer offset that reaches window point `local` of
/// window `target`, for a drag pressed at `press` in window `source`.
fn offset_to(
    snapshot: &Snapshot,
    source: u64,
    target: u64,
    press: (f32, f32),
    local: (f32, f32),
) -> (f32, f32) {
    let from = entry(snapshot, source).frame.expect("source frame");
    let to = entry(snapshot, target).frame.expect("target frame");
    let x = to[0] + local.0 - from[0];
    let y = to[1] + local.1 - from[1];
    (x - press.0, y - press.1)
}

/// Press tab row `index` of the current window at its center and activate
/// the drag; returns the press point.
fn press_row(client: &mut Client, index: usize) -> (f32, f32) {
    let snapshot = settle(client, "the rows drawn", |s| {
        s.tab_row_rects.iter().any(|(row, _)| *row == index)
    });
    let rect = snapshot
        .tab_row_rects
        .iter()
        .find(|(row, _)| *row == index)
        .unwrap()
        .1;
    let press = (rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0);
    client
        .send_ok(&bbr_testkit::protocol::dsl::begin_tab_drag(
            index,
            press.0,
            press.1,
            rect[3] / 2.0,
        ))
        .expect("press the row");
    client
        .send_ok(&bbr_testkit::protocol::dsl::move_tab_drag(0.0, 6.0))
        .expect("activate the drag");
    press
}

#[test]
#[ignore]
fn a_tab_dragged_onto_another_windows_rail_moves_there() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, _page_one, page_two, mut harness, socket, token) = start("windows-drag-rail");
    let long = fixtures.url("/long-title.html");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let mut since = ready(&mut client, &token);
        let second = two_windows_apart(&mut client, &mut since, &long, &page_two, false);
        let target = select_window(&mut client, second);
        let elsewhere = target.active_space;
        let row = target.tab_row_rects[0].1;
        let source = select_window(&mut client, 0);
        let home = source.active_space;
        let dragged = source.tabs[1].web_view.expect("the long page is live");

        // Out over the second window's rail, just below its only row.
        let press = press_row(&mut client, 1);
        let snapshot = client.snapshot().expect("frames");
        let (dx, dy) = offset_to(
            &snapshot,
            0,
            second,
            press,
            (row[0] + 40.0, row[1] + row[3] + 2.0),
        );
        client
            .send_ok(&bbr_testkit::protocol::dsl::move_tab_drag(dx, dy))
            .expect("over the other rail");
        let over = settle(&mut client, "the preview there", |s| {
            entry(s, second).incoming_drop.as_deref() == Some("rail:1")
        });
        assert_eq!(entry(&over, 0).incoming_drop, None);

        // Released there: the page lands in that window's space after its
        // row, shows there, and keeps its session.
        client
            .send_ok(&bbr_testkit::protocol::dsl::end_tab_drag())
            .expect("drop");
        let landed = settle(&mut client, "the page moved", |s| {
            current(s) == second && s.tabs.len() == 2
        });
        assert_eq!(landed.active_space, elsewhere);
        assert_eq!(landed.tabs[1].url, long);
        assert_eq!(landed.active_tab_index, Some(1), "shown there");
        assert_eq!(
            landed.active_web_view,
            Some(dragged),
            "same profile: same session"
        );
        assert_eq!(entry(&landed, second).selected_root, Some(second));
        assert_eq!(entry(&landed, second).incoming_drop, None);
        let first = select_window(&mut client, 0);
        assert_eq!(first.active_space, home);
        assert_eq!(first.tabs.len(), 1, "it left the first window's space");

        // A drop outside every window does nothing.
        let tabs_before = first.tabs.len();
        let press = press_row(&mut client, 0);
        client
            .send_ok(&bbr_testkit::protocol::dsl::move_tab_drag(
                5000.0 - press.0,
                5000.0 - press.1,
            ))
            .expect("out of every window");
        client
            .send_ok(&bbr_testkit::protocol::dsl::end_tab_drag())
            .expect("release outside");
        let after = settle(&mut client, "nothing moved", |s| s.windows.len() == 2);
        assert_eq!(after.tabs.len(), tabs_before);
        assert_eq!(current(&after), 0);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn a_tab_dragged_onto_another_windows_card_splits_there() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, page_one, page_two, mut harness, socket, token) = start("windows-drag-card");
    let long = fixtures.url("/long-title.html");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let mut since = ready(&mut client, &token);
        let second = two_windows_apart(&mut client, &mut since, &long, &page_two, false);
        let target = select_window(&mut client, second);
        let card = target.content_rect;
        select_window(&mut client, 0);

        // Onto the left edge of the second window's page.
        let press = press_row(&mut client, 0);
        let snapshot = client.snapshot().expect("frames");
        let local = (card[0] + 12.0, card[1] + card[3] / 2.0);
        let (dx, dy) = offset_to(&snapshot, 0, second, press, local);
        client
            .send_ok(&bbr_testkit::protocol::dsl::move_tab_drag(dx, dy))
            .expect("over the other card");
        settle(&mut client, "the split preview there", |s| {
            entry(s, second).incoming_drop.as_deref() == Some("card:left")
        });
        client
            .send_ok(&bbr_testkit::protocol::dsl::end_tab_drag())
            .expect("drop");
        let split = settle(&mut client, "the split there", |s| {
            current(s) == second && s.split.as_ref().is_some_and(|g| g.panes.len() == 2)
        });
        let urls: Vec<&str> = split.tabs.iter().map(|tab| tab.url.as_str()).collect();
        assert!(urls.contains(&page_one.as_str()) && urls.contains(&page_two.as_str()));
        let panes = &split.split.as_ref().unwrap().panes;
        let left = panes
            .iter()
            .min_by(|a, b| a.rect[0].total_cmp(&b.rect[0]))
            .unwrap()
            .view;
        let page_one_view = split
            .tabs
            .iter()
            .find(|tab| tab.url == page_one)
            .and_then(|tab| tab.web_view);
        assert_eq!(
            Some(left),
            page_one_view,
            "the dropped page is the left pane"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn a_tab_dragged_into_another_profiles_window_is_recreated_there() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, _page_one, page_two, mut harness, socket, token) = start("windows-drag-profile");
    let long = fixtures.url("/long-title.html");
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        let mut since = ready(&mut client, &token);
        let second = two_windows_apart(&mut client, &mut since, &long, &page_two, true);
        let target = select_window(&mut client, second);
        let row = target.tab_row_rects[0].1;
        let private = target.active_profile;
        let source = select_window(&mut client, 0);
        assert_ne!(source.active_profile, private);
        let old = source.tabs[1].web_view.expect("live");

        let press = press_row(&mut client, 1);
        let snapshot = client.snapshot().expect("frames");
        let (dx, dy) = offset_to(&snapshot, 0, second, press, (row[0] + 40.0, row[1] + 2.0));
        client
            .send_ok(&bbr_testkit::protocol::dsl::move_tab_drag(dx, dy))
            .expect("over the other rail");
        settle(&mut client, "the preview there", |s| {
            entry(s, second).incoming_drop.as_deref() == Some("rail:0")
        });
        client
            .send_ok(&bbr_testkit::protocol::dsl::end_tab_drag())
            .expect("drop");
        let landed = settle(&mut client, "recreated there", |s| {
            current(s) == second && s.tabs.len() == 2
        });
        assert_eq!(landed.tabs[0].url, long, "at the slot above the row");
        assert_eq!(landed.active_tab_index, Some(0));
        let new = landed.active_web_view.expect("a live session");
        assert_ne!(new, old, "a fresh session in the destination store");
        assert_eq!(landed.active_profile, private);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
