//! Blueprint D23: split groups in a space.

use crate::tiling::{Axis, Edge, SplitGroup};
use crate::{Browser, DomainError, SpaceId, TabId, TabLifecycle};

fn four() -> (Browser, SpaceId, [TabId; 4]) {
    let mut browser = Browser::new();
    let profile = browser.create_profile("Work");
    let space = browser.create_space(profile, "Main").unwrap();
    let mut ids = [TabId(0); 4];
    for (i, id) in ids.iter_mut().enumerate() {
        *id = browser
            .open_tab(space, format!("T{i}"), format!("https://t{i}.example"))
            .unwrap();
    }
    (browser, space, ids)
}

fn lifecycle(b: &Browser, tab: TabId) -> TabLifecycle {
    b.tab(tab).unwrap().lifecycle
}

#[test]
fn inserting_beside_a_tab_forms_and_grows_a_group() {
    let (mut b, s, [a, c, d, e]) = four();
    b.split_insert(s, a, c, Edge::Right).unwrap();
    assert_eq!(b.split_of(s).unwrap().members(), [a, c]);
    assert_eq!(
        b.space(s).unwrap().active_tab,
        Some(c),
        "the new pane is focused"
    );
    b.split_insert(s, c, d, Edge::Bottom).unwrap();
    assert_eq!(b.split_of(s).unwrap().members(), [a, c, d]);
    // Left of `a`: the new pane leads.
    b.split_insert(s, a, e, Edge::Left).unwrap();
    assert_eq!(b.split_of(s).unwrap().members(), [e, a, c, d]);
    for tab in [a, c, d, e] {
        assert_eq!(
            lifecycle(&b, tab),
            TabLifecycle::Active,
            "every pane is live"
        );
    }
    b.check_invariants().unwrap();
}

#[test]
fn a_tab_outside_keeps_the_group_and_a_member_brings_it_back() {
    let (mut b, s, [a, c, d, _]) = four();
    b.split_insert(s, a, c, Edge::Right).unwrap();
    b.select_tab(s, d).unwrap();
    assert!(b.split_of(s).is_none(), "nothing split is shown");
    assert_eq!(b.space(s).unwrap().splits.len(), 1, "the group stays");
    assert_eq!(lifecycle(&b, a), TabLifecycle::Warm);
    b.select_tab(s, a).unwrap();
    assert_eq!(b.split_of(s).unwrap().members(), [a, c]);
    assert_eq!(lifecycle(&b, c), TabLifecycle::Active);
    b.check_invariants().unwrap();
}

#[test]
fn a_space_holds_several_groups() {
    let (mut b, s, [a, c, d, e]) = four();
    b.split_insert(s, a, c, Edge::Right).unwrap();
    b.split_insert(s, d, e, Edge::Bottom).unwrap();
    assert_eq!(b.space(s).unwrap().splits.len(), 2);
    assert_eq!(b.split_of(s).unwrap().members(), [d, e]);
    // Moving `c` next to `e` takes it out of the first group, which
    // dissolves.
    b.split_insert(s, e, c, Edge::Right).unwrap();
    let space = b.space(s).unwrap();
    assert_eq!(space.splits.len(), 1);
    assert_eq!(space.splits[0].members(), [d, e, c]);
    b.check_invariants().unwrap();
}

#[test]
fn closing_a_pane_keeps_the_rest_and_focuses_its_place() {
    let (mut b, s, [a, c, d, _]) = four();
    b.split_insert(s, a, c, Edge::Right).unwrap();
    b.split_insert(s, c, d, Edge::Right).unwrap();
    b.select_tab(s, c).unwrap();
    b.close_tab(s, c).unwrap();
    assert_eq!(b.split_of(s).unwrap().members(), [a, d]);
    assert_eq!(
        b.space(s).unwrap().active_tab,
        Some(d),
        "the pane at its place"
    );
    b.close_tab(s, d).unwrap();
    assert!(
        b.space(s).unwrap().splits.is_empty(),
        "one pane left: dissolved"
    );
    assert_eq!(b.space(s).unwrap().active_tab, Some(a));
    b.check_invariants().unwrap();
}

#[test]
fn replacing_removing_and_dissolving() {
    let (mut b, s, [a, c, d, e]) = four();
    b.split_insert(s, a, c, Edge::Right).unwrap();
    b.split_insert(s, c, d, Edge::Right).unwrap();
    b.split_replace(s, c, e).unwrap();
    assert_eq!(b.split_of(s).unwrap().members(), [a, e, d]);
    assert_eq!(b.space(s).unwrap().active_tab, Some(e));
    assert_eq!(
        lifecycle(&b, c),
        TabLifecycle::Warm,
        "the replaced tab stays open"
    );
    b.split_remove(s, d).unwrap();
    assert_eq!(b.split_of(s).unwrap().members(), [a, e]);
    let dissolved = b.split_dissolve(s, a).unwrap().unwrap();
    assert_eq!(dissolved.members(), [a, e]);
    assert!(b.space(s).unwrap().splits.is_empty());
    assert_eq!(b.space(s).unwrap().active_tab, Some(e), "the focus stays");
    b.check_invariants().unwrap();
}

#[test]
fn dividers_and_equalize_go_through_the_group() {
    let (mut b, s, [a, c, _, _]) = four();
    b.split_insert(s, a, c, Edge::Right).unwrap();
    b.split_set_divider(s, a, &[], 0, 3_000).unwrap();
    let weights: Vec<u32> = b
        .split_of(s)
        .unwrap()
        .root()
        .children
        .iter()
        .map(|c| c.weight)
        .collect();
    assert_eq!(weights, [3_000, 7_000]);
    b.split_equalize(s, c, &[]).unwrap();
    let weights: Vec<u32> = b
        .split_of(s)
        .unwrap()
        .root()
        .children
        .iter()
        .map(|c| c.weight)
        .collect();
    assert_eq!(weights, [5_000, 5_000]);
}

#[test]
fn restoring_groups_checks_membership_and_overlap() {
    let (mut b, s, [a, c, d, _]) = four();
    let pair = SplitGroup::pair(a, c, Axis::Column).unwrap();
    let overlapping = SplitGroup::pair(c, d, Axis::Row).unwrap();
    assert!(matches!(
        b.restore_splits(s, vec![pair.clone(), overlapping]),
        Err(DomainError::SplitAlreadyExists(_))
    ));
    let stranger = SplitGroup::pair(a, TabId(999), Axis::Row).unwrap();
    assert!(matches!(
        b.restore_splits(s, vec![stranger]),
        Err(DomainError::TabNotInSpace { .. })
    ));
    b.restore_splits(s, vec![pair]).unwrap();
    b.select_tab(s, a).unwrap();
    assert_eq!(b.split_of(s).unwrap().root().axis, Axis::Column);
    b.check_invariants().unwrap();
}
