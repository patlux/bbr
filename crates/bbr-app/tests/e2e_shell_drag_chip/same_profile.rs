//! Same-profile drag-to-chip evidence (DESIGN.md §11.3/§10.9): the concrete
//! pressed tab is moved into the exact persistent destination chip, the
//! during-drag target + §12 companion are typed on the snapshot and in the
//! semantic tree, and the moved tab keeps its live session/WebViewId.

use super::*;
use super::{assert_footer_invariant, chip_center, chip_index, companion_id, drag_onto};
use bbr_testkit::protocol::SemanticRole;
use bbr_testkit::protocol::{dsl, BrowserChange, Event};
use e2e_semantics_common::find;

/// Drag the concrete non-active tab onto a chip of the same profile: the
/// pressed row (index 1) — not the active row (index 0) — is the moved tab,
/// the destination is the exact persistent chip under the pointer, the
/// during-drag wire/semantics report the typed chip target and its §12
/// destination-profile companion, and the commit keeps the live session and
/// repairs the source selection. The footer composition is untouched.
#[test]
#[ignore]
fn drag_pressed_tab_onto_exact_chip_keeps_session_and_repairs_selection() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("drag-chip-same", &page_one);

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

        // Two warm tabs; the active tab is row 0, so the dragged row (1) is
        // explicitly not the active tab — the pressed identity is the
        // evidence target.
        let second_view = open_tab(&mut client, &mut since, &page_two, Some(PAGE_TWO_TITLE));
        client.send_ok(&dsl::click_tab(0)).expect("select row 0");
        since = wait_browser_change(&mut client, BrowserChange::TabSelected { index: 0 }, since);
        let two = client.snapshot().expect("two-tab snapshot");
        assert_eq!(two.active_tab_index, Some(0));
        assert_eq!(two.engine_sessions, 2);

        // A second space in the same profile: the strip now renders two
        // chips in canonical order; the second chip is the exact persistent
        // destination.
        let default_profile = two.active_profile;
        client
            .send_ok(&dsl::create_space(default_profile, "Reading"))
            .expect("create reading space");
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
        // One flushed frame so the strip has laid out the new chip (the
        // scroll-handle geometry is render-derived).
        let seeded = strip_presence(&mut client);
        assert_eq!(seeded.space_chips.len(), 2, "two chips render");
        assert_footer_invariant(&seeded);

        // Enter the Reading chip through its real rendered span (the same
        // layout the app hit-tests): the during-drag snapshot reports the
        // exact persistent chip, the typed drag-target flag on that chip,
        // and the §12 destination-profile companion.
        let target_center = chip_center(&seeded.space_strip_scroll, chip_index(&seeded, reading));
        let during = drag_onto(&mut client, 1, target_center.0, target_center.1);
        assert_eq!(
            during.drag_source,
            Some(1),
            "the pressed row — not the active row 0 — is the dragged tab"
        );
        assert_eq!(during.drag_slot, None, "chip drag carries no slot tick");
        assert_eq!(
            during.drag_chip,
            Some(reading),
            "the pointer targets the exact persistent chip"
        );
        assert!(during.drag_valid, "the chip is a legal drop");
        let target_chip = &during.space_chips[chip_index(&during, reading)];
        assert!(
            target_chip.drag_target,
            "the targeted chip reports the flag"
        );
        let companion = target_chip
            .companion
            .as_ref()
            .expect("targeted chip carries the §12 companion");
        let dest_profile = during
            .profiles
            .iter()
            .find(|p| {
                p.id == during
                    .spaces
                    .iter()
                    .find(|s| s.space == reading)
                    .unwrap()
                    .profile
            })
            .expect("destination profile");
        assert_eq!(
            companion.profile, dest_profile.index,
            "companion index is the profile's"
        );
        // The §12 monogram is the profile's non-color identity (the first
        // two title chars uppercased — the same canonical derivation the
        // render uses).
        let expected_monogram = dest_profile
            .title
            .chars()
            .take(2)
            .map(|ch| ch.to_ascii_uppercase())
            .collect::<String>();
        assert_eq!(
            companion.monogram, expected_monogram,
            "the §12 monogram identity"
        );
        assert!(
            !companion.hue.is_empty(),
            "the companion carries a typed hue"
        );
        // The companion id is part of the rendered surface exactly during
        // the drag (visible_ids) and typed in the semantic tree.
        let companion_id = companion_id(&during, reading);
        assert!(
            during.visible_ids.iter().any(|id| id == &companion_id),
            "the companion chip id renders during the drag"
        );
        let chip_node = find(&during.semantics, &format!("bbr-space-chip-{}", reading.0))
            .expect("the targeted chip is in the semantic tree");
        assert!(
            chip_node.drag_target,
            "the semantic node reports the drag target"
        );
        assert!(
            chip_node
                .children
                .iter()
                .any(|child| child.id == companion_id && child.role == SemanticRole::Chip),
            "the §12 companion is a typed child of the targeted chip"
        );

        // Release: the commit routes through the existing move authority —
        // same profile keeps the live session, the source selection repairs
        // to row 0, and the footer composition is untouched.
        client.send_ok(&dsl::end_tab_drag()).expect("end drag");
        since = wait_browser_change(
            &mut client,
            BrowserChange::TabMoved {
                to_space: reading,
                recreated: false,
            },
            since,
        );
        let moved = client.snapshot().expect("moved snapshot");
        assert_eq!(moved.tabs.len(), 1, "the source space keeps row 0");
        assert_eq!(moved.tabs[0].web_view, Some(first_view));
        assert_eq!(moved.active_tab_index, Some(0), "source selection repaired");
        assert_eq!(
            moved.active_web_view,
            Some(first_view),
            "the source's live session is untouched"
        );
        assert_eq!(
            moved.engine_sessions, 2,
            "same-profile move recreates nothing"
        );
        assert!(
            moved.chrome_focused,
            "the drop keeps the chrome focus domain"
        );
        let reading_row = moved
            .spaces
            .iter()
            .find(|s| s.space == reading)
            .expect("destination space");
        assert_eq!(reading_row.tab_count, 1);
        assert_eq!(
            reading_row.active_view,
            Some(second_view),
            "the moved tab keeps its live WebViewId"
        );
        assert_eq!(moved.drag_source, None, "the drag state is cleared");
        assert_eq!(moved.drag_chip, None);
        assert_footer_invariant(&moved);
        // No spurious reorder event: the only mutation was the move.
        let events = client
            .attach(Some(since))
            .expect("post-move replay")
            .into_iter()
            .filter(|(_, event)| {
                matches!(
                    event,
                    Event::BrowserChanged {
                        kind: BrowserChange::TabReordered { .. }
                    }
                )
            })
            .count();
        assert_eq!(events, 0, "the chip drop emits no reorder event");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
