//! Unit tests for the portable domain model (PLAN.md Phase 2 acceptance):
//! invariants, ordering, last-tab behavior, moving across spaces/profiles,
//! pinned close semantics, recently closed tabs, and split teardown.
//! These tests run without macOS frameworks or a display server.

use crate::{Browser, DomainError, ProfileId, SpaceId, TabId, TabLifecycle};

/// A browser with one profile, one space, and three tabs [t1, t2, t3],
/// with `active = t3` (the last opened).
fn populated() -> Browser {
    let mut b = Browser::new();
    let p = b.create_profile("Work");
    let s = b.create_space(p, "Main").unwrap();
    let _t1 = b.open_tab(s, "One", "https://one.example").unwrap();
    let _t2 = b.open_tab(s, "Two", "https://two.example").unwrap();
    let _t3 = b.open_tab(s, "Three", "https://three.example").unwrap();
    b
}

fn ids(b: &Browser, s: SpaceId) -> Vec<TabId> {
    b.space(s).unwrap().tabs.clone()
}

fn profile_of(b: &Browser, s: SpaceId) -> ProfileId {
    b.space(s).unwrap().profile
}

/// Shortcut: select `tab` so the test exercises the active-tab path.
fn select(b: &mut Browser, s: SpaceId, tab: TabId) {
    b.select_tab(s, tab).unwrap();
}

// ---- Creation and ordering ------------------------------------------------

#[test]
fn open_orders_and_activates() {
    let mut b = Browser::new();
    let p = b.create_profile("Work");
    let s = b.create_space(p, "Main").unwrap();
    let t1 = b.open_tab(s, "A", "u1").unwrap();
    let t2 = b.open_tab(s, "B", "u2").unwrap();
    assert_eq!(ids(&b, s), vec![t1, t2]);
    assert_eq!(b.space(s).unwrap().active_tab, Some(t2));
    assert_eq!(b.tab(t2).unwrap().space, s);
    b.check_invariants().unwrap();
}

#[test]
fn open_into_unknown_space_fails() {
    let mut b = Browser::new();
    assert!(matches!(
        b.open_tab(SpaceId(99), "x", "u"),
        Err(DomainError::UnknownSpace(SpaceId(99)))
    ));
    assert!(matches!(
        b.create_space(ProfileId(1), "x"),
        Err(DomainError::UnknownProfile(ProfileId(1)))
    ));
}

#[test]
fn rename_space_updates_only_the_exact_space() {
    let mut b = Browser::new();
    let p = b.create_profile("Work");
    let first = b.create_space(p, "Main").unwrap();
    let second = b.create_space(p, "Other").unwrap();
    let t = b.open_tab(first, "T", "u").unwrap();

    b.rename_space(first, "Renamed").unwrap();
    assert_eq!(b.space(first).unwrap().title, "Renamed");
    assert_eq!(b.space(second).unwrap().title, "Other", "sibling untouched");
    assert_eq!(
        b.space(first).unwrap().tabs,
        vec![t],
        "tab order and selection untouched"
    );
    assert_eq!(b.space(first).unwrap().active_tab, Some(t));
    assert_eq!(b.space(first).unwrap().profile, p, "ownership untouched");
    b.check_invariants().unwrap();
}

#[test]
fn rename_space_rejects_unknown_spaces_whole() {
    let mut b = Browser::new();
    let p = b.create_profile("Work");
    let s = b.create_space(p, "Main").unwrap();
    assert!(matches!(
        b.rename_space(SpaceId(4242), "Ghost"),
        Err(DomainError::UnknownSpace(SpaceId(4242)))
    ));
    assert_eq!(
        b.space(s).unwrap().title,
        "Main",
        "rejection mutates nothing"
    );
}

#[test]
fn ids_never_collide_across_kinds() {
    let mut b = Browser::new();
    let p = b.create_profile("P");
    let s = b.create_space(p, "S").unwrap();
    let t = b.open_tab(s, "T", "u").unwrap();
    assert_ne!(p.0, s.0);
    assert_ne!(p.0, t.0);
    assert_ne!(s.0, t.0);
    // The three tabs of `populated` are distinct too.
    let b = populated();
    let ids = ids(&b, b.spaces().next().unwrap().id);
    assert_ne!(ids[0], ids[1]);
    assert_ne!(ids[1], ids[2]);
}

// ---- Closing ---------------------------------------------------------------

#[test]
fn close_active_selects_next_sibling() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let [t1, t2, _t3] = ids(&b, s)[..] else {
        unreachable!()
    };
    select(&mut b, s, t2);
    b.close_tab(s, t2).unwrap();
    // Next sibling (pre-removal) is t3.
    assert_eq!(b.space(s).unwrap().active_tab, Some(_t3));
    assert_eq!(ids(&b, s), vec![t1, _t3]);
    b.check_invariants().unwrap();
}

#[test]
fn close_active_last_tab_selects_previous() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let [t1, t2, t3] = ids(&b, s)[..] else {
        unreachable!()
    };
    select(&mut b, s, t3);
    b.close_tab(s, t3).unwrap();
    // No next sibling; previous (pre-removal) is t2.
    assert_eq!(b.space(s).unwrap().active_tab, Some(t2));
    assert_eq!(ids(&b, s), vec![t1, t2]);
    b.check_invariants().unwrap();
}

#[test]
fn close_inactive_keeps_active() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let [t1, _t2, t3] = ids(&b, s)[..] else {
        unreachable!()
    };
    select(&mut b, s, t3);
    b.close_tab(s, t1).unwrap();
    assert_eq!(b.space(s).unwrap().active_tab, Some(t3));
    b.check_invariants().unwrap();
}

#[test]
fn close_last_tab_leaves_space_empty() {
    let mut b = Browser::new();
    let p = b.create_profile("P");
    let s = b.create_space(p, "S").unwrap();
    let t = b.open_tab(s, "only", "u").unwrap();
    b.close_tab(s, t).unwrap();
    let sp = b.space(s).unwrap();
    assert!(sp.tabs.is_empty());
    assert_eq!(sp.active_tab, None);
    assert_eq!(sp.recently_closed.len(), 1);
    b.check_invariants().unwrap();
}

#[test]
fn close_pinned_active_focuses_sibling() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let [t1, t2, t3] = ids(&b, s)[..] else {
        unreachable!()
    };
    b.pin_tab(s, t1).unwrap();
    select(&mut b, s, t1);
    b.close_tab(s, t1).unwrap();
    let sp = b.space(s).unwrap();
    assert_eq!(sp.active_tab, Some(t2));
    assert_eq!(sp.tabs, vec![t2, t3]);
    assert_eq!(sp.pinned_count, 0);
    b.check_invariants().unwrap();
}

#[test]
fn closing_unknown_or_wrong_space_fails() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    assert!(matches!(
        b.close_tab(s, TabId(999)),
        Err(DomainError::TabNotInSpace { .. })
    ));
    let p = b.create_profile("Other");
    let s2 = b.create_space(p, "Other-S").unwrap();
    let t = ids(&b, s)[0];
    assert!(matches!(
        b.close_tab(s2, t),
        Err(DomainError::TabNotInSpace { .. })
    ));
    assert!(matches!(
        b.close_tab(SpaceId(4242), t),
        Err(DomainError::UnknownSpace(SpaceId(4242)))
    ));
}

// ---- Reopen ----------------------------------------------------------------

#[test]
fn reopen_restores_most_recent_at_former_position() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let before = ids(&b, s);
    b.close_tab(s, before[1]).unwrap();
    b.close_tab(s, before[0]).unwrap();
    assert_eq!(ids(&b, s), vec![before[2]]);

    let reopened = b.reopen_tab(s).unwrap();
    assert_eq!(reopened, before[0]);
    assert_eq!(ids(&b, s), vec![before[0], before[2]]);
    assert_eq!(b.space(s).unwrap().active_tab, Some(before[0]));
    b.check_invariants().unwrap();
}

#[test]
fn reopen_clamped_when_list_shrank() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let before = ids(&b, s);
    // Order: [A, B, C]. Close C (index 2) then B (index 1): the closed list
    // ends at [{B, 1}, {C, 2}], most recent last. Reopen is LIFO: B first,
    // then C with its former index clamped to the current length.
    b.close_tab(s, before[2]).unwrap();
    b.close_tab(s, before[1]).unwrap();
    let reopened = b.reopen_tab(s).unwrap();
    assert_eq!(reopened, before[1]);
    assert_eq!(ids(&b, s), vec![before[0], before[1]]);
    let reopened = b.reopen_tab(s).unwrap();
    assert_eq!(reopened, before[2]);
    assert_eq!(ids(&b, s), vec![before[0], before[1], before[2]]);
    b.check_invariants().unwrap();
}

#[test]
fn reopen_pinned_tab_lands_in_pinned_region() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let [t1, t2, t3] = ids(&b, s)[..] else {
        unreachable!()
    };
    b.pin_tab(s, t1).unwrap();
    b.pin_tab(s, t3).unwrap();
    // Order: [t1, t3, t2] pinned = 2.
    assert_eq!(ids(&b, s), vec![t1, t3, t2]);
    b.close_tab(s, t3).unwrap();
    // Order: [t1, t2] pinned = 1.
    let reopened = b.reopen_tab(s).unwrap();
    assert_eq!(reopened, t3);
    // Former index 1 is inside the pinned region.
    assert_eq!(ids(&b, s), vec![t1, t3, t2]);
    assert_eq!(b.space(s).unwrap().pinned_count, 2);
    assert!(b.tab(t3).unwrap().pinned);
    b.check_invariants().unwrap();
}

#[test]
fn reopen_closed_tab_restores_that_entry_and_keeps_the_rest() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let before = ids(&b, s);
    b.close_tab(s, before[2]).unwrap();
    b.close_tab(s, before[1]).unwrap();
    // Blueprint D10: History › Recently closed reopens the older entry C.
    let reopened = b.reopen_closed_tab(s, before[2]).unwrap();
    assert_eq!(reopened, before[2]);
    assert_eq!(ids(&b, s), vec![before[0], before[2]]);
    assert_eq!(b.space(s).unwrap().active_tab, Some(before[2]));
    // B stays closed and is still the next ⇧⌘T.
    assert_eq!(b.reopen_tab(s).unwrap(), before[1]);
    assert!(matches!(
        b.reopen_closed_tab(s, before[1]),
        Err(DomainError::NothingToReopen(_))
    ));
    b.check_invariants().unwrap();
}

#[test]
fn reopen_empty_errors() {
    let mut b = Browser::new();
    let p = b.create_profile("P");
    let s = b.create_space(p, "S").unwrap();
    assert!(matches!(
        b.reopen_tab(s),
        Err(DomainError::NothingToReopen(_))
    ));
}

// ---- Reorder ---------------------------------------------------------------

#[test]
fn reorder_within_region_keeps_pinned_first() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let before = ids(&b, s);
    b.pin_tab(s, before[0]).unwrap();
    b.pin_tab(s, before[2]).unwrap();
    assert_eq!(ids(&b, s), vec![before[0], before[2], before[1]]);
    b.reorder_tab(s, 0, 1).unwrap();
    assert_eq!(ids(&b, s), vec![before[2], before[0], before[1]]);
    assert_eq!(b.space(s).unwrap().pinned_count, 2);
    b.check_invariants().unwrap();
}

#[test]
fn reorder_across_pinned_boundary_rejected() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let before = ids(&b, s);
    b.pin_tab(s, before[0]).unwrap();
    assert!(matches!(
        b.reorder_tab(s, 0, 2),
        Err(DomainError::ReorderAcrossPinnedBoundary)
    ));
    assert!(matches!(
        b.reorder_tab(s, 2, 0),
        Err(DomainError::ReorderAcrossPinnedBoundary)
    ));
    b.check_invariants().unwrap();
}

#[test]
fn reorder_out_of_bounds_rejected() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    assert!(matches!(
        b.reorder_tab(s, 3, 0),
        Err(DomainError::IndexOutOfBounds { .. })
    ));
    assert!(matches!(
        b.reorder_tab(s, 0, 3),
        Err(DomainError::IndexOutOfBounds { .. })
    ));
    b.check_invariants().unwrap();
}

// ---- Pin / unpin -----------------------------------------------------------

#[test]
fn pin_unpin_round_trip() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let before = ids(&b, s);
    // With one pinned tab already, pinning C moves it to the end of the
    // pinned region (right after A).
    b.pin_tab(s, before[0]).unwrap();
    b.pin_tab(s, before[2]).unwrap();
    assert_eq!(b.space(s).unwrap().pinned_count, 2);
    assert!(b.tab(before[2]).unwrap().pinned);
    assert_eq!(ids(&b, s), vec![before[0], before[2], before[1]]);
    b.unpin_tab(s, before[2]).unwrap();
    assert_eq!(ids(&b, s), before);
    assert!(!b.tab(before[2]).unwrap().pinned);
    b.check_invariants().unwrap();
}

#[test]
fn pinning_already_pinned_is_ok() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let t = ids(&b, s)[0];
    b.pin_tab(s, t).unwrap();
    b.pin_tab(s, t).unwrap();
    assert_eq!(b.space(s).unwrap().pinned_count, 1);
    b.check_invariants().unwrap();
}

// ---- Moving ----------------------------------------------------------------

#[test]
fn move_tab_across_spaces_same_profile() {
    let mut b = populated();
    let s1 = b.spaces().next().unwrap().id;
    let p = profile_of(&b, s1);
    let s2 = b.create_space(p, "Second").unwrap();
    let [t1, t2, t3] = ids(&b, s1)[..] else {
        unreachable!()
    };
    select(&mut b, s1, t1);
    b.move_tab(t1, s2).unwrap();
    assert_eq!(b.tab(t1).unwrap().space, s2);
    assert_eq!(ids(&b, s2), vec![t1]);
    assert_eq!(b.space(s2).unwrap().active_tab, Some(t1));
    assert_eq!(ids(&b, s1), vec![t2, t3]);
    // Source active tab repaired to its neighbor.
    assert_eq!(b.space(s1).unwrap().active_tab, Some(t2));
    b.check_invariants().unwrap();
}

#[test]
fn move_active_last_tab_repairs_to_previous() {
    let mut b = populated();
    let s1 = b.spaces().next().unwrap().id;
    let p = profile_of(&b, s1);
    let s2 = b.create_space(p, "Second").unwrap();
    let [_t1, t2, t3] = ids(&b, s1)[..] else {
        unreachable!()
    };
    select(&mut b, s1, t3);
    b.move_tab(t3, s2).unwrap();
    assert_eq!(b.space(s1).unwrap().active_tab, Some(t2));
    assert_eq!(ids(&b, s2), vec![t3]);
    b.check_invariants().unwrap();
}

#[test]
fn move_tab_across_profiles_is_allowed() {
    // Moving across profiles is a domain-level no-op; the engine layer
    // recreates the web view in the destination store (PLAN.md §4). The
    // domain must simply allow it and keep invariants intact.
    let mut b = populated();
    let s1 = b.spaces().next().unwrap().id;
    let p2 = b.create_profile("Personal");
    let s2 = b.create_space(p2, "Personal-S").unwrap();
    let t = ids(&b, s1)[0];
    b.move_tab(t, s2).unwrap();
    assert_eq!(b.tab(t).unwrap().space, s2);
    b.check_invariants().unwrap();
}

#[test]
fn move_pinned_tab_keeps_pinned_in_destination() {
    let mut b = populated();
    let s1 = b.spaces().next().unwrap().id;
    let p = profile_of(&b, s1);
    let s2 = b.create_space(p, "Second").unwrap();
    let t = ids(&b, s1)[0];
    b.pin_tab(s1, t).unwrap();
    b.move_tab(t, s2).unwrap();
    assert!(b.tab(t).unwrap().pinned);
    assert_eq!(b.space(s2).unwrap().pinned_count, 1);
    assert_eq!(ids(&b, s2), vec![t]);
    b.check_invariants().unwrap();
}

#[test]
fn move_tab_to_unknown_space_fails() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let t = ids(&b, s)[0];
    assert!(matches!(
        b.move_tab(t, SpaceId(999)),
        Err(DomainError::UnknownSpace(SpaceId(999)))
    ));
    let before = b.clone();
    assert_eq!(b, before); // no mutation happened
}

#[test]
fn move_unknown_tab_fails() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    assert!(matches!(
        b.move_tab(TabId(999), s),
        Err(DomainError::UnknownTab(TabId(999)))
    ));
}

#[test]
fn move_tab_into_same_space_is_noop() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let before = b.clone();
    let t = ids(&b, s)[0];
    b.move_tab(t, s).unwrap();
    assert_eq!(b, before);
}

// ---- Crash discard (WebKit process termination) -----------------------------

#[test]
fn visible_tab_can_be_discarded_and_reactivated() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let [hidden, _t2, visible] = ids(&b, s)[..] else {
        unreachable!()
    };
    // A crash can hit the visible (Active) tab; discard bypasses the
    // visibility guard because it is an external event.
    b.discard_tab(visible).unwrap();
    assert_eq!(b.tab(visible).unwrap().lifecycle, TabLifecycle::Discarded);
    b.check_invariants().unwrap();
    // A crash can also hit a hidden (Warm) tab.
    b.discard_tab(hidden).unwrap();
    assert_eq!(b.tab(hidden).unwrap().lifecycle, TabLifecycle::Discarded);
    b.check_invariants().unwrap();

    // Selection reactivates a discarded tab; the previously-discarded tab
    // that becomes hidden stays recoverable (Discarded), not Warm.
    b.select_tab(s, hidden).unwrap();
    assert_eq!(b.tab(hidden).unwrap().lifecycle, TabLifecycle::Active);
    assert_eq!(b.tab(visible).unwrap().lifecycle, TabLifecycle::Discarded);
    b.check_invariants().unwrap();
}

#[test]
fn discard_unknown_tab_fails() {
    let mut b = populated();
    assert!(matches!(
        b.discard_tab(TabId(999)),
        Err(DomainError::UnknownTab(TabId(999)))
    ));
}

// ---- Navigation application -----------------------------------------------

#[test]
fn apply_navigation_updates_persistent_identity() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let t = ids(&b, s)[0];
    assert_eq!(b.tab(t).unwrap().url, "https://one.example");
    b.apply_navigation(t, "https://two.example", "Two").unwrap();
    assert_eq!(b.tab(t).unwrap().url, "https://two.example");
    assert_eq!(b.tab(t).unwrap().title, "Two");
    b.check_invariants().unwrap();
}

#[test]
fn apply_navigation_ignores_empty_parts() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let t = ids(&b, s)[0];
    b.apply_navigation(t, "https://two.example", "").unwrap();
    assert_eq!(b.tab(t).unwrap().url, "https://two.example");
    assert_eq!(b.tab(t).unwrap().title, "One"); // title untouched
    b.apply_navigation(t, "", "Empty-url-keeps-old").unwrap();
    assert_eq!(b.tab(t).unwrap().url, "https://two.example");
}

#[test]
fn apply_navigation_unknown_tab_fails() {
    let mut b = populated();
    assert!(matches!(
        b.apply_navigation(TabId(999), "https://x", "X"),
        Err(DomainError::UnknownTab(TabId(999)))
    ));
}

#[test]
fn discarded_visible_tab_survives_other_mutations() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let [a, b_id, c] = ids(&b, s)[..] else {
        unreachable!()
    };
    b.discard_tab(c).unwrap(); // visible tab crashed
                               // A split operation must not silently resurrect the crashed tab.
    b.set_split(s, a, b_id).unwrap();
    assert_eq!(b.tab(c).unwrap().lifecycle, TabLifecycle::Discarded);
    b.check_invariants().unwrap();
}

// ---- Profile identity hue (DESIGN.md §12) --------------------------------

#[test]
fn profiles_get_round_robin_hues_stable_for_lifetime() {
    let mut b = Browser::new();
    let hues = [
        "cobalt",
        "violet",
        "fuchsia",
        "amber",
        "terracotta",
        "slate",
    ];
    // Seven profiles wrap the six-hue ramp.
    let mut created = Vec::new();
    for i in 0..7 {
        created.push(b.create_profile(format!("P{i}")));
    }
    for (i, profile) in created.iter().enumerate() {
        let p = b.profile(*profile).expect("profile exists");
        assert_eq!(
            p.hue.name(),
            hues[i % hues.len()],
            "round-robin assignment for profile {i}"
        );
    }
    // The hue is stable: recreating the browser with the same sequence yields
    // the same assignment, and the stored value never changes on mutation.
    let mut b2 = Browser::new();
    let mut second = Vec::new();
    for i in 0..7 {
        second.push(b2.create_profile(format!("P{i}")));
    }
    for (a, id) in created.iter().zip(&second) {
        assert_eq!(
            b.profile(*a).unwrap().hue,
            b2.profile(*id).unwrap().hue,
            "hue assignment is deterministic across identical histories"
        );
    }
    // A tab mutation never touches profile identity.
    let space = b.create_space(created[0], "S").unwrap();
    b.open_tab(space, "T", "https://t").unwrap();
    assert_eq!(
        b.profile(created[0]).unwrap().hue.name(),
        "cobalt",
        "profile identity survives unrelated mutations"
    );
}

// ---- Background-child insertion (background link tabs packet) --------------

#[test]
fn insert_tab_at_position_without_changing_selection() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let before = ids(&b, s);
    let active = b.space(s).unwrap().active_tab;
    // Insert immediately after the opener (index 1) as a regular child.
    let child = b
        .insert_tab(s, 1, "Child", "https://child.example")
        .unwrap();
    assert_eq!(
        ids(&b, s),
        vec![before[0], child, before[1], before[2]],
        "child lands exactly at the requested index"
    );
    assert_eq!(
        b.space(s).unwrap().active_tab,
        active,
        "insertion never changes the selection"
    );
    assert!(b.space(s).unwrap().splits.is_empty(), "no split invented");
    assert!(b.space(s).unwrap().pinned_count == 0);
    b.check_invariants().unwrap();
}

#[test]
fn insert_tab_appends_at_the_end_when_index_is_len() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let len = ids(&b, s).len();
    let child = b
        .insert_tab(s, len, "Child", "https://child.example")
        .unwrap();
    assert_eq!(ids(&b, s).last().copied(), Some(child));
    b.check_invariants().unwrap();
}

#[test]
fn insert_tab_rejects_regular_insertion_into_the_pinned_region() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    b.pin_tab(s, ids(&b, s)[0]).unwrap();
    assert!(matches!(
        b.insert_tab(s, 0, "Child", "https://child.example"),
        Err(DomainError::InsertionAcrossPinnedBoundary {
            index: 0,
            pinned: 1
        })
    ));
    assert!(
        b.insert_tab(s, 1, "Child", "https://child.example").is_ok(),
        "the first regular position is legal"
    );
    assert!(
        matches!(
            b.insert_tab(s, 99, "Child", "https://child.example"),
            Err(DomainError::InsertionAcrossPinnedBoundary { .. })
        ),
        "out-of-range index is rejected"
    );
    b.check_invariants().unwrap();
}

#[test]
fn insert_tab_unknown_space_rejected() {
    let mut b = Browser::new();
    assert!(matches!(
        b.insert_tab(SpaceId(4242), 0, "Child", "https://child.example"),
        Err(DomainError::UnknownSpace(SpaceId(4242)))
    ));
}
