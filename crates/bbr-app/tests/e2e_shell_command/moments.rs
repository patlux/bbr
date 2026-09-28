//! §18.5 ⌘T after-commit moments over the real app (task packet
//! `v2-motion-moments`). Commit first: a published result set and a moved
//! cursor are in the typed snapshot (`command_results`, the selected row)
//! in the same snapshot whose motion evidence records the moment starting —
//! full motion: staggered entrance and gliding cursor (`animated`); Reduce
//! Motion: fade-only entrance (`reduced`) and a snapping cursor (`static`).

use super::command_portal;
use super::*;
use bbr_testkit::protocol::{CommandSurface, MotionMomentSnapshot, Snapshot};

/// The newest start of `moment`, if any.
fn last_start(snapshot: &Snapshot, moment: &str) -> Option<MotionMomentSnapshot> {
    snapshot
        .motion
        .started
        .iter()
        .rev()
        .find(|start| start.moment == moment)
        .cloned()
}

/// Type `text` into the open palette and publish its ranking.
fn publish(client: &mut Client, text: &str) -> Snapshot {
    client
        .send_ok(&bbr_testkit::Request::SetCommandBarText { text: text.into() })
        .expect("type in palette");
    client
        .send_ok(&bbr_testkit::Request::FlushCommandRanking)
        .expect("flush palette ranking");
    client.snapshot().expect("published snapshot")
}

fn selected_row(snapshot: &Snapshot) -> usize {
    snapshot
        .command_results
        .iter()
        .position(|row| row.selected)
        .expect("a selected result")
}

#[test]
#[ignore]
fn palette_results_enter_and_the_cursor_glides_after_the_commit() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("cmd-v2-moments", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let mut since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);
        let first = client.snapshot().expect("first snapshot");
        let view = first.active_web_view.expect("first view");
        if first.title.as_deref() != Some(PAGE_ONE_TITLE) {
            since = wait_title(&mut client, view, PAGE_ONE_TITLE, since);
        }
        let _second = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(false)))
            .expect("full motion");
        key(&mut client, "cmd-t");
        let _ = wait_chrome_focus(&mut client, true, since);
        command_portal::wait_command_portal_open(&mut client, CommandSurface::Palette, deadline());

        // Full motion: the published set enters.
        let published = publish(&mut client, "page");
        assert_eq!(published.command_result_state, CommandResultState::Results);
        assert!(published.command_results.len() >= 2, "rows to move between");
        assert_eq!(selected_row(&published), 0, "the set is committed");
        let entrance = last_start(&published, "command-results").expect("entrance started");
        assert_eq!(entrance.target, "bbr-palette");
        assert_eq!(entrance.variant, "animated");

        // The cursor moves: the selection is committed on the key press,
        // then the highlight glides.
        key(&mut client, "down");
        let moved = client.snapshot().expect("moved snapshot");
        assert_eq!(selected_row(&moved), 1, "the selection commits at once");
        let glide = last_start(&moved, "command-cursor").expect("glide started");
        assert_eq!(glide.target, "bbr-palette");
        assert_eq!(glide.variant, "animated");

        // Reduce Motion: fade-only entrance, snapping cursor.
        client
            .send_ok(&dsl::set_accessibility_override(Some(false), Some(true)))
            .expect("reduce motion");
        let reduced = publish(&mut client, "pag");
        assert_eq!(reduced.command_result_state, CommandResultState::Results);
        let entrance = last_start(&reduced, "command-results").expect("reduced entrance");
        assert_eq!(entrance.variant, "reduced");
        let starts = reduced.motion.started.len();
        let before = selected_row(&reduced);
        let (step, expected) = if before > 0 {
            ("up", before - 1)
        } else {
            ("down", 1)
        };
        key(&mut client, step);
        let snapped = client.snapshot().expect("snapped snapshot");
        assert_eq!(
            selected_row(&snapped),
            expected,
            "the selection commits at once"
        );
        assert_eq!(snapped.motion.started.len(), starts + 1);
        let glide = last_start(&snapped, "command-cursor").expect("reduced glide start");
        assert_eq!(
            glide.variant, "static",
            "the cursor snaps under Reduce Motion"
        );
        assert!(
            !snapped
                .motion
                .moments
                .iter()
                .any(|moment| moment.moment == "command-cursor"),
            "no glide runs under Reduce Motion"
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, deadline());
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
