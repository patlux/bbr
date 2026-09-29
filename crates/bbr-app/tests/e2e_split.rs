//! Blueprint D23 real-app E2E (DESIGN §11.7, boards G-9/G-10/PR-09): split
//! groups over real WKWebViews.
//!
//! Three tabs tile as a row with a stacked column (every native pane framed
//! at its projected rect, 2 pt dividers between); ⌃⌘+arrow and a press on
//! a pane move the focus; a divider drag moves only its neighbours and a
//! double-click evens them out; selecting a tab outside keeps the group and
//! a member brings it back; a window too narrow for every pane rests the
//! group on its focused pane with an "n of N" chip whose list switches
//! panes; ⇧⌃⌘S dissolves the group and keeps its tabs. ⌃⌘S opens the ⌘T
//! palette in split mode and the committed page becomes the right pane;
//! the group and its shares come back after a restart. Typed snapshots and
//! browser events under deadlines; no sleeps, no pixels.

#![cfg(target_os = "macos")]

mod e2e_common;

use bbr_testkit::protocol::{dsl, CommandBarMode, Snapshot, SplitInput, SplitSnapshot};
use bbr_testkit::Request;
use e2e_common::*;

const TAG: &str = "split";

fn snapshot(client: &mut Client) -> Snapshot {
    client.send_ok(&Request::FlushFrame).expect("flush frame");
    client.snapshot().expect("snapshot")
}

fn wait(client: &mut Client, what: &str, done: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let until = deadline();
    loop {
        let snapshot = snapshot(client);
        if done(&snapshot) {
            return snapshot;
        }
        assert!(
            Instant::now() < until,
            "{what} never happened: {:?} native {:?}",
            snapshot.split,
            snapshot.pane_rects
        );
    }
}

fn split(client: &mut Client, input: SplitInput) {
    client
        .send_ok(&Request::SplitInput { input })
        .expect("split input");
}

fn key(client: &mut Client, keystroke: &str) {
    client
        .send_ok(&Request::DispatchKeystroke {
            keystroke: keystroke.into(),
        })
        .expect("keystroke");
}

fn insert(client: &mut Client, beside: usize, new: usize, edge: &str) {
    split(
        client,
        SplitInput::Insert {
            beside,
            new,
            edge: edge.into(),
        },
    );
}

fn shown(snapshot: &Snapshot) -> &SplitSnapshot {
    snapshot.split.as_ref().expect("a shown split")
}

fn close(a: [f32; 4], b: [f32; 4]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() <= 1.0)
}

/// Every pane's native view is framed at its projected rect.
fn framed(snapshot: &Snapshot) -> bool {
    let Some(split) = snapshot.split.as_ref() else {
        return false;
    };
    split.panes.iter().all(|pane| {
        snapshot
            .pane_rects
            .iter()
            .any(|native| native.view == pane.view && close(native.rect, pane.rect))
    })
}

#[test]
#[ignore]
fn split_groups_tile_focus_resize_collapse_and_dissolve() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, page_one, page_two) = fixtures();
    let page_three = fixtures.url("/long-title.html");
    let (mut harness, socket, token) = spawn_harness(TAG, &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        let settled = wait_for_window_settled(&mut client, deadline());
        let window = settled.window_size;
        client.send_ok(&dsl::open_tab(&page_two)).expect("two");
        client.send_ok(&dsl::open_tab(&page_three)).expect("three");
        let three = wait(&mut client, "three tabs", |s| {
            s.tabs.len() == 3 && s.tabs.iter().all(|t| t.web_view.is_some())
        });
        let views: Vec<_> = three.tabs.iter().map(|t| t.web_view.unwrap()).collect();

        // A row of one and a stacked column of two.
        insert(&mut client, 0, 1, "right");
        insert(&mut client, 1, 2, "bottom");
        let tiled = wait(&mut client, "three panes framed", framed);
        let group = shown(&tiled);
        assert_eq!(
            group.tree,
            format!(
                "R(5000:{},5000:C(5000:{},5000:{}))",
                views[0].0, views[1].0, views[2].0
            )
        );
        assert_eq!(group.panes.len(), 3);
        assert_eq!(group.dividers.len(), 2);
        assert_eq!(group.focused, views[2], "the new pane is focused");
        assert_eq!(tiled.engine_sessions, 3, "every pane is live");
        for id in ["bbr-split", "bbr-pane-2", "bbr-split-divider-1"] {
            assert!(tiled.visible_ids.iter().any(|v| v == id), "{id}");
        }
        let [left, top, _, _] = group.panes[0].rect;
        let divider = group.dividers[0];
        assert!((divider[0] - (left + group.panes[0].rect[2])).abs() < 0.5);

        // ⌃⌘← focuses the left pane; a press on the lower right pane
        // focuses it.
        key(&mut client, "ctrl-cmd-left");
        wait(&mut client, "left focus", |s| {
            s.split.as_ref().is_some_and(|g| g.focused == views[0])
        });
        let lower = shown(&tiled).panes[2].rect;
        split(
            &mut client,
            SplitInput::PointerDown {
                x: lower[0] + lower[2] / 2.0,
                y: lower[1] + lower[3] / 2.0,
            },
        );
        wait(&mut client, "press focus", |s| {
            s.split.as_ref().is_some_and(|g| g.focused == views[2])
        });

        // The row divider follows a drag; its neighbours only; a
        // double-click evens the row out again.
        split(
            &mut client,
            SplitInput::DragDivider {
                index: 0,
                to: left + 400.0,
            },
        );
        let dragged = wait(&mut client, "the drag reframed", |s| {
            framed(s)
                && s.split
                    .as_ref()
                    .is_some_and(|g| !g.tree.starts_with("R(5000"))
        });
        let pane = shown(&dragged).panes[0].rect;
        assert!((pane[2] - 399.0).abs() <= 1.0, "{pane:?}");
        assert_eq!(pane[1], top);
        split(&mut client, SplitInput::DoubleClickDivider { index: 0 });
        wait(&mut client, "even again", |s| {
            framed(s)
                && s.split
                    .as_ref()
                    .is_some_and(|g| g.tree.starts_with("R(5000"))
        });

        // A tab outside keeps the group; a member brings it back.
        client.send_ok(&dsl::open_tab(&page_two)).expect("four");
        let outside = wait(&mut client, "single pane", |s| {
            s.tabs.len() == 4 && s.split.is_none()
        });
        assert!(outside.pane_layout.unwrap().secondary.is_none());
        client.send_ok(&dsl::select_tab(1)).expect("member");
        wait(&mut client, "the group back", |s| {
            framed(s) && s.split.as_ref().is_some_and(|g| g.focused == views[1])
        });

        // Too narrow for every pane: the focused pane with an "n of N" chip
        // whose list switches panes.
        client
            .send_ok(&Request::SetWindowSize {
                width: 820.0,
                height: 700.0,
            })
            .expect("narrow");
        let collapsed = wait(&mut client, "collapsed", |s| {
            s.split
                .as_ref()
                .is_some_and(|g| g.collapsed == Some((2, 3)))
        });
        assert!(collapsed.visible_ids.iter().any(|v| v == "bbr-split-chip"));
        assert!(!collapsed.visible_ids.iter().any(|v| v == "bbr-pane-0"));
        split(&mut client, SplitInput::Chip { row: None });
        let listed = wait(&mut client, "the list", |s| {
            s.split.as_ref().is_some_and(|g| g.list_open)
        });
        assert!(listed.visible_ids.iter().any(|v| v == "bbr-split-list"));
        split(&mut client, SplitInput::Chip { row: Some(0) });
        wait(&mut client, "first pane", |s| {
            s.split
                .as_ref()
                .is_some_and(|g| g.collapsed == Some((1, 3)) && !g.list_open)
        });
        client
            .send_ok(&Request::SetWindowSize {
                width: window[0],
                height: window[1],
            })
            .expect("wide again");
        wait(&mut client, "fits again", |s| {
            framed(s) && s.split.as_ref().is_some_and(|g| g.collapsed.is_none())
        });

        // ⇧⌃⌘S dissolves the group; every tab stays.
        key(&mut client, "ctrl-shift-cmd-s");
        let dissolved = wait(&mut client, "dissolved", |s| s.split.is_none());
        assert_eq!(dissolved.tabs.len(), 4);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn split_right_takes_the_palette_commit_and_survives_a_restart() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("split-restart", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        wait_for_window_settled(&mut client, deadline());

        // ⌃⌘S: the palette in split mode; the committed page opens as the
        // right pane.
        key(&mut client, "ctrl-cmd-s");
        let staged = snapshot(&mut client);
        assert_eq!(staged.command_bar_mode, CommandBarMode::NewTab);
        client
            .send_ok(&Request::SetCommandBarText {
                text: page_two.clone(),
            })
            .expect("type the destination");
        client.send_ok(&Request::FlushCommandRanking).expect("rank");
        key(&mut client, "enter");
        let paired = wait(&mut client, "two panes", |s| {
            framed(s) && s.split.as_ref().is_some_and(|g| g.panes.len() == 2)
        });
        assert_eq!(paired.tabs.len(), 2);
        assert_eq!(paired.tabs[1].url, page_two);
        assert_eq!(shown(&paired).focused, paired.tabs[1].web_view.unwrap());
        // A dragged share persists with the group.
        let divider = shown(&paired).dividers[0];
        split(
            &mut client,
            SplitInput::DragDivider {
                index: 0,
                to: divider[0] - 100.0,
            },
        );
        let tree = wait(&mut client, "resized", |s| {
            s.split
                .as_ref()
                .is_some_and(|g| !g.tree.starts_with("R(5000"))
        })
        .split
        .unwrap()
        .tree;
        client.send_ok(&Request::FlushBrowserState).expect("flush");
        tree
    }));
    let tree = match result {
        Ok(tree) => {
            harness.success = false;
            tree
        }
        Err(panic) => {
            harness.diagnostics(&socket, &token);
            std::panic::resume_unwind(panic);
        }
    };
    drop(harness);

    let (mut harness, socket, token) = spawn_harness("split-restart", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        let restored = wait(&mut client, "the group restored", framed);
        let group = shown(&restored);
        assert_eq!(group.panes.len(), 2);
        let shares = |tree: &str| {
            tree.split(':')
                .next()
                .map(|head| head.trim_start_matches("R(").to_string())
        };
        assert_eq!(shares(&group.tree), shares(&tree), "the share came back");
        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}

#[test]
#[ignore]
fn dropping_rail_tabs_on_the_card_splits_and_the_rail_keeps_the_block() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }
    let (fixtures, page_one, page_two) = fixtures();
    let page_three = fixtures.url("/long-title.html");
    let (mut harness, socket, token) = spawn_harness("split-drop", &page_one);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        client.attach(None).expect("attach");
        wait_for_window_settled(&mut client, deadline());
        client.send_ok(&dsl::open_tab(&page_two)).expect("two");
        client.send_ok(&dsl::open_tab(&page_three)).expect("three");
        wait(&mut client, "three tabs", |s| s.tabs.len() == 3);
        client.send_ok(&dsl::select_tab(0)).expect("one");
        let single = wait(&mut client, "one shown", |s| {
            s.active_tab_index == Some(0) && s.content_rect[2] > 0.0
        });
        let card = single.content_rect;
        let (start_x, start_y) = (130.0, 400.0);
        let drag_to = |client: &mut Client, x: f32, y: f32| {
            client
                .send_ok(&dsl::move_tab_drag(x - start_x, y - start_y))
                .expect("move drag");
        };

        // Tab three over the right edge of the single pane: the right zone
        // is previewed; the drop pairs the two, and three joins one's block.
        client
            .send_ok(&dsl::begin_tab_drag(2, start_x, start_y, 16.0))
            .expect("begin drag");
        drag_to(&mut client, start_x + 20.0, start_y + 5.0);
        drag_to(
            &mut client,
            card[0] + card[2] - 20.0,
            card[1] + card[3] / 2.0,
        );
        let hovering = snapshot(&mut client);
        let zone = hovering.split_drop.expect("a previewed zone");
        assert_eq!(zone.zone, "right");
        assert!((zone.rect[0] - (card[0] + card[2] * 0.76)).abs() < 1.0);
        client.send_ok(&dsl::end_tab_drag()).expect("drop");
        let paired = wait(&mut client, "the pair", |s| {
            framed(s) && s.split.as_ref().is_some_and(|g| g.panes.len() == 2)
        });
        let urls: Vec<&str> = paired.tabs.iter().map(|t| t.url.as_str()).collect();
        assert_eq!(
            urls,
            [page_one.as_str(), page_three.as_str(), page_two.as_str()]
        );
        assert_eq!(paired.split_rows, [0, 1], "one bracketed block");
        assert!(paired.split_drop.is_none());

        // Tab two onto the centre of the right pane replaces it.
        let right = shown(&paired).panes[1].rect;
        client
            .send_ok(&dsl::begin_tab_drag(2, start_x, start_y, 16.0))
            .expect("begin drag");
        drag_to(&mut client, start_x + 20.0, start_y + 5.0);
        drag_to(
            &mut client,
            right[0] + right[2] / 2.0,
            right[1] + right[3] / 2.0,
        );
        assert_eq!(
            snapshot(&mut client).split_drop.map(|d| d.zone).as_deref(),
            Some("center")
        );
        client.send_ok(&dsl::end_tab_drag()).expect("drop");
        let replaced = wait(&mut client, "the replacement", |s| {
            framed(s) && s.split_rows == [0, 1] && s.tabs[1].url == page_two
        });
        assert_eq!(
            shown(&replaced).panes[1].view,
            replaced.tabs[1].web_view.unwrap()
        );
        assert_eq!(
            replaced.tabs[2].url, page_three,
            "the replaced tab stays open"
        );

        // Moving a pane's row out of the block leaves the split.
        key(&mut client, "cmd-shift-down");
        let left = wait(&mut client, "out of the block", |s| s.split.is_none());
        assert!(left.split_rows.is_empty());
        assert_eq!(left.tabs.len(), 3);

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));
    finish(result, &mut harness, &socket, &token);
}
