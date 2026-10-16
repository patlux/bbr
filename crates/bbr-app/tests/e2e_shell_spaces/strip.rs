//! Real-app §11.3/§11.4 strip evidence (F2a strip E2E layer): the two
//! permanent-strip proofs over real WKWebViews that the pointer/identity
//! suite cannot reach.
//!
//! 1. **Grouped-order identity stability:** the canonical grouped display
//!    order (profile groups, creation order within each profile) can differ
//!    from the persistent numeric id order; every `bbr-space-chip-<SpaceId>`
//!    / `bbr-space-identity-<SpaceId>` id stays bound to its persistent
//!    space id, exactly one chip renders selected (the active space), and
//!    activation never reorders or re-derives ids — there is exactly one
//!    ordering authority (the grouped visuals), never a second one.
//! 2. **Overflow + scroll-into-view:** on a compact real window with actual
//!    horizontal overflow (typed `space_strip_scroll.max_offset.x > 0`),
//!    selecting an initially offscreen chip through the production chip
//!    listener (`select_space_via_pointer` → `activate_space`) then a
//!    `FlushFrame` brings the selected chip fully inside the strip viewport
//!    — computed from the typed viewport/offset/chip bounds the production
//!    `space_strip_scroll` handle exposes read-only. No sleeps, no pixel
//!    screenshots, no direct snapshot mutation, no test-only bbr-browser
//!    API; the suite fails loudly when an offscreen chip is not contained.
//!
//! The shared harness lives in `e2e_common`; the pointer/identity, keyboard,
//! and hint suites live in the sibling binaries of this test target.

use super::*;

/// §11.3 id stability when the canonical grouped display order differs from
/// the persistent id order: `Default` (id 2) and `Reading` (id 4) in the
/// default profile, a `Private` profile+space (profile id 5 / space id 6),
/// then `Work` (id 7) back in the default profile. The initial tab consumes
/// allocator id 3, so the spaces never start at id 1. Profile groups stack,
/// so the canonical grouped display order is `[2, 4, 7, 6]` — Work renders
/// before Private — while the persistent numeric id order is `[2, 4, 6, 7]`.
/// Every chip and identity id stays bound to its persistent space id,
/// exactly one chip is selected (the active space), and activating a space
/// never reorders the strip or re-derives ids from positions.
#[test]
#[ignore]
fn chip_ids_stay_bound_to_spaces_across_grouped_order_change() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("spaces-strip-order", &page_one);

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
        assert_eq!(
            default_space.0, 2,
            "launch space is persistent id 2 (the default profile consumed id 1)"
        );

        // §11.3/§11.4: the permanent strip is always part of the rail (the
        // strip presence seam verifies it; there is no reveal step).
        strip_presence(&mut client);

        // Same-profile space, cross-profile profile+space, then a second
        // same-profile space: grouped display order will differ from ids.
        client
            .send_ok(&dsl::create_space(default_profile, "Reading"))
            .expect("create reading");
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
        assert_eq!(
            reading.0, 4,
            "second space is persistent id 4 (the initial tab consumed id 3)"
        );
        client
            .send_ok(&dsl::create_profile("Private", "Private"))
            .expect("create private profile");
        let (seq, created) = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::ProfileCreated { .. }),
            since,
        );
        since = seq;
        let (private_profile, private_space) = match created {
            BrowserChange::ProfileCreated { profile, space } => (profile, space),
            other => panic!("unexpected change: {other:?}"),
        };
        assert_eq!(private_profile.0, 5, "private profile is persistent id 5");
        assert_eq!(private_space.0, 6, "private space is persistent id 6");
        client
            .send_ok(&dsl::create_space(default_profile, "Work"))
            .expect("create work");
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
        assert_eq!(work.0, 7, "work space is persistent id 7");

        // Canonical grouped display order: Default(2), Reading(4), Work(7)
        // under the default profile, then Private(6). Persistent id order
        // [2,4,6,7] differs from display order [2,4,7,6].
        let ordered = client.snapshot().expect("grouped order snapshot");
        let chip_ids: Vec<u64> = ordered.space_chips.iter().map(|c| c.space.0).collect();
        assert_eq!(
            chip_ids,
            vec![2, 4, 7, 6],
            "canonical grouped display order differs from persistent id order"
        );
        assert_eq!(ordered.spaces.len(), 4, "four persistent spaces exist");
        let titles: Vec<&str> = ordered
            .space_chips
            .iter()
            .map(|c| c.title.as_str())
            .collect();
        assert_eq!(titles, vec!["Default", "Reading", "Work", "Private"]);
        // Exactly one selected chip, and it is the active launch space.
        assert_eq!(
            ordered.space_chips.iter().filter(|c| c.selected).count(),
            1,
            "exactly one chip renders selected"
        );
        assert_eq!(
            ordered
                .space_chips
                .iter()
                .find(|c| c.selected)
                .unwrap()
                .space,
            default_space,
            "the selected chip is exactly the active space"
        );
        // Ids derive from the persistent space id, never from position: the
        // chip at display position 2 carries id 7 (Work), position 3 id 6.
        assert_eq!(ordered.space_chips[2].space.0, 7);
        assert_eq!(ordered.space_chips[3].space.0, 6);
        // Every chip + identity id renders, keyed by the persistent id.
        for id in [2u64, 4, 6, 7] {
            assert!(
                ordered
                    .visible_ids
                    .iter()
                    .any(|present| present == &format!("bbr-space-chip-{id}")),
                "chip id bbr-space-chip-{id} renders"
            );
            assert!(
                ordered
                    .visible_ids
                    .iter()
                    .any(|present| present == &format!("bbr-space-identity-{id}")),
                "identity id bbr-space-identity-{id} renders"
            );
        }

        // Activate Work (id 7, display position 2): the selected chip flips
        // to exactly id 7 — the active space — the display order stays
        // [2,4,7,6] (selection never reorders), and the ids stay bound to
        // the same spaces.
        client
            .send_ok(&dsl::click_space(work))
            .expect("activate work");
        since = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: work },
            since,
        );
        let after_work = client.snapshot().expect("work activated snapshot");
        assert_eq!(after_work.active_space, work);
        let chip_ids: Vec<u64> = after_work.space_chips.iter().map(|c| c.space.0).collect();
        assert_eq!(
            chip_ids,
            vec![2, 4, 7, 6],
            "activation never reorders the strip"
        );
        assert_eq!(
            after_work.space_chips.iter().filter(|c| c.selected).count(),
            1,
            "exactly one chip stays selected"
        );
        let selected = after_work.space_chips.iter().find(|c| c.selected).unwrap();
        assert_eq!(
            selected.space.0, 7,
            "the chip at display position 2 (id 7) is selected, not its position"
        );
        assert_eq!(
            selected.title, "Work",
            "id 7 stays bound to Work regardless of position"
        );

        // Activate Private (id 6, display position 3): same binding proof
        // from the other side of the id/display mismatch.
        client
            .send_ok(&dsl::click_space(private_space))
            .expect("activate private");
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated {
                space: private_space,
            },
            since,
        );
        let after_private = client.snapshot().expect("private activated snapshot");
        assert_eq!(after_private.active_space, private_space);
        assert_eq!(
            after_private.active_profile, private_profile,
            "cross-profile activation moves the container"
        );
        let chip_ids: Vec<u64> = after_private
            .space_chips
            .iter()
            .map(|c| c.space.0)
            .collect();
        assert_eq!(chip_ids, vec![2, 4, 7, 6]);
        let selected = after_private
            .space_chips
            .iter()
            .find(|c| c.selected)
            .unwrap();
        assert_eq!(selected.space.0, 6);
        assert_eq!(
            selected.title, "Private",
            "id 6 stays bound to Private regardless of position"
        );
        assert_eq!(
            after_private
                .space_chips
                .iter()
                .filter(|c| c.selected)
                .count(),
            1
        );

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// The visual position of one chip: its raw window-relative layout bounds
/// combined with the strip's signed scroll offset (GPUI paints children
/// shifted by the offset; the wire keeps them separate so the E2E can
/// recompute the exact on-screen rect).
fn chip_visual(rect: [f32; 4], offset: [f32; 2]) -> [f32; 4] {
    [rect[0] + offset[0], rect[1] + offset[1], rect[2], rect[3]]
}

/// Whether `inner` lies fully inside `outer` (half-point tolerance for f32
/// layout arithmetic) — the §11.3 containment rule the strip must satisfy
/// after a selection.
fn fully_inside(inner: [f32; 4], outer: [f32; 4]) -> bool {
    inner[0] >= outer[0] - 0.5
        && inner[1] >= outer[1] - 0.5
        && inner[0] + inner[2] <= outer[0] + outer[2] + 0.5
        && inner[1] + inner[3] <= outer[1] + outer[3] + 0.5
}

/// §11.3 overflow + selected-chip scroll-into-view on a compact real
/// window: seven short-title spaces overflow the strip viewport (typed
/// precondition `max_offset.x > 0`). The last chip (Zeta, id 9) is
/// initially fully outside the viewport; selecting it through the
/// production chip listener and flushing a real frame scrolls it fully
/// inside — computed from the typed viewport/offset/chip bounds the
/// production `space_strip_scroll` handle exposes read-only. The suite
/// fails loudly when an offscreen chip is not brought into view.
#[test]
#[ignore]
fn selected_offscreen_chip_scrolls_fully_into_view_on_compact_window() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("spaces-strip-overflow", &page_one);

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

        // Seven short-title spaces in the default profile (ids 2, 4–9:
        // launch space 2, initial tab 3, then Alpha 4, Beta 5, Gamma 6,
        // Delta 7, Epsilon 8, Zeta 9); the canonical display order equals
        // creation order here, so the last chip is the rightmost one and
        // the offscreen target is deterministic. Seven 28 pt chips (Paper
        // W-1 footer) overflow any compact rail.
        let mut last_space = default_space;
        for title in ["Alpha", "Beta", "Gamma", "Delta", "Epsilon", "Zeta"] {
            client
                .send_ok(&dsl::create_space(default_profile, title))
                .expect("create strip space");
            let (seq, created) = wait_browser_change_matching(
                &mut client,
                |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
                since,
            );
            since = seq;
            last_space = match created {
                BrowserChange::SpaceCreated { space, .. } => space,
                other => panic!("unexpected change: {other:?}"),
            };
        }
        assert_eq!(
            last_space.0, 9,
            "Zeta is persistent id 9 (launch 2, initial tab 3, Alpha 4 … Epsilon 8)"
        );

        // Compact real window: the rail yields toward the window so the
        // footer strip is narrow and the seven chips genuinely overflow.
        client
            .send_ok(&bbr_testkit::Request::SetWindowSize {
                width: 320.0,
                height: 700.0,
            })
            .expect("compact window");
        let settled = wait_for_window_settled(&mut client, deadline());
        assert!(
            settled.window_size[0] <= 340.0,
            "compact window applied: {:?}",
            settled.window_size
        );

        // Precondition: real horizontal overflow (typed max offset > 0),
        // one raw bounds rect per chip, and the last chip (Zeta) fully
        // outside the viewport before any selection.
        let before = client.snapshot().expect("overflow precondition");
        let scroll = before.space_strip_scroll;
        assert_eq!(
            scroll.chip_bounds.len(),
            7,
            "one raw chip rect per space in display order"
        );
        assert!(
            scroll.max_offset[0] > 0.0,
            "precondition: the strip really overflows (max_offset.x = {})",
            scroll.max_offset[0]
        );
        assert!(
            scroll.viewport[2] > 0.0 && scroll.viewport[3] > 0.0,
            "typed viewport is a real rect"
        );
        let delta_index = before
            .space_chips
            .iter()
            .position(|c| c.space == last_space)
            .expect("Zeta chip present");
        assert_eq!(delta_index, 6, "Zeta is the rightmost chip");
        let raw_delta = scroll.chip_bounds[delta_index];
        let visual_delta = chip_visual(raw_delta, scroll.offset);
        assert!(
            !fully_inside(visual_delta, scroll.viewport),
            "precondition: the selected chip is initially outside the viewport"
        );
        assert!(
            before.space_chips.iter().filter(|c| c.selected).count() == 1
                && before
                    .space_chips
                    .iter()
                    .any(|c| c.selected && c.space == default_space),
            "exactly one selected chip before the switch (the active space)"
        );

        // Select the offscreen chip through the production chip listener
        // (select_space_via_pointer → activate_space → the tracked handle's
        // scroll_to_item), then flush a real frame so the strip's prepaint
        // applies the pending scroll.
        client
            .send_ok(&dsl::click_space(last_space))
            .expect("select offscreen chip");
        let _ = wait_browser_change(
            &mut client,
            BrowserChange::SpaceActivated { space: last_space },
            since,
        );

        // The §11.3 outcome: after the flush the selected chip lies fully
        // inside the strip viewport. Frame-driven bounded polling (flush +
        // typed snapshot, no sleeps) so a fast-but-wrong sample can never
        // pass; the deadline failure is the loud proof that scroll-into-
        // view did not happen.
        let until = Instant::now() + Duration::from_secs(30);
        let mut after = client.snapshot().expect("post-select snapshot");
        loop {
            let scroll = &after.space_strip_scroll;
            let raw = scroll.chip_bounds[delta_index];
            let visual = chip_visual(raw, scroll.offset);
            if fully_inside(visual, scroll.viewport) {
                break;
            }
            assert!(
                Instant::now() < until,
                "selected chip never became fully contained: visual {visual:?} in viewport {:?}",
                scroll.viewport
            );
            client
                .send_ok(&bbr_testkit::Request::FlushFrame)
                .expect("flush toward containment");
            after = client.snapshot().expect("containment snapshot");
        }

        let scroll = &after.space_strip_scroll;
        assert_eq!(
            after.active_space, last_space,
            "the offscreen chip was activated"
        );
        assert_eq!(
            after.space_chips.iter().filter(|c| c.selected).count(),
            1,
            "exactly one selected chip after the switch"
        );
        let selected = after.space_chips.iter().find(|c| c.selected).unwrap();
        assert_eq!(
            selected.space, last_space,
            "the selected chip is exactly the activated space"
        );
        assert!(
            scroll.max_offset[0] > 0.0,
            "the strip still overflows after the scroll"
        );
        assert!(
            scroll.offset[0] < 0.0,
            "the strip actually scrolled left to reveal the chip (offset.x = {})",
            scroll.offset[0]
        );
        let visual = chip_visual(scroll.chip_bounds[delta_index], scroll.offset);
        assert!(
            fully_inside(visual, scroll.viewport),
            "selected chip fully inside the strip viewport: visual {visual:?} in viewport {:?}",
            scroll.viewport
        );
        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}

/// §11.4 / Blueprint D01: while the chips fit, they start at the strip's
/// leading edge right after Downloads and follow each other at the strip
/// gap — the free space stays trailing (no centring). Read from the typed
/// viewport/offset/chip bounds of the production `space_strip_scroll`
/// handle on a real window.
#[test]
#[ignore]
fn fitting_chips_start_at_the_strip_leading_edge() {
    let _serial = serial_gui();
    if !allow_gui() {
        return;
    }

    let (_fixtures, page_one, _page_two) = fixtures();
    let (mut harness, socket, token) = spawn_harness("spaces-strip-leading", &page_one);

    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut client = connect(&socket);
        client.hello(&token).expect("hello");
        let replay = client.attach(None).expect("attach");
        let since = replay.iter().map(|(seq, _)| *seq).max().unwrap_or(0);

        let initial = client.snapshot().expect("initial snapshot");
        client
            .send_ok(&dsl::create_space(initial.active_profile, "Reading"))
            .expect("create second space");
        let _ = wait_browser_change_matching(
            &mut client,
            |kind| matches!(kind, BrowserChange::SpaceCreated { .. }),
            since,
        );

        let until = Instant::now() + Duration::from_secs(30);
        let snapshot = loop {
            client
                .send_ok(&bbr_testkit::Request::FlushFrame)
                .expect("flush strip layout");
            let snapshot = client.snapshot().expect("strip snapshot");
            if snapshot.space_strip_scroll.chip_bounds.len() == 2 {
                break snapshot;
            }
            assert!(Instant::now() < until, "two chips never laid out");
        };
        let scroll = &snapshot.space_strip_scroll;
        assert_eq!(scroll.max_offset[0], 0.0, "two chips fit the strip");
        assert_eq!(scroll.offset[0], 0.0, "a fitting strip is not scrolled");
        let first = chip_visual(scroll.chip_bounds[0], scroll.offset);
        let second = chip_visual(scroll.chip_bounds[1], scroll.offset);
        assert!(
            (first[0] - scroll.viewport[0]).abs() < 0.5,
            "the first chip starts at the strip's leading edge: chip {first:?} viewport {:?}",
            scroll.viewport
        );
        let gap = second[0] - (first[0] + first[2]);
        assert!(
            (gap - 4.0).abs() < 0.5,
            "the second chip follows at the Paper S-7b strip gap (4 pt), got {gap}"
        );
        let trailing = scroll.viewport[0] + scroll.viewport[2] - (second[0] + second[2]);
        assert!(trailing > 8.0, "the free space stays trailing ({trailing})");

        client.quit().expect("quit");
        let status = wait_for_exit(&mut harness.child, Instant::now() + Duration::from_secs(10));
        assert!(status.success(), "app exit: {status}");
    }));

    finish(result, &mut harness, &socket, &token);
}
