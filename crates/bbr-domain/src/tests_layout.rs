//! Lifecycle, split, and invariant tests for the portable domain model.

use crate::{Browser, DomainError, ProfileId, SpaceId, TabId, TabLifecycle};

fn populated() -> Browser {
    let mut browser = Browser::new();
    let profile = browser.create_profile("Work");
    let space = browser.create_space(profile, "Main").unwrap();
    browser
        .open_tab(space, "One", "https://one.example")
        .unwrap();
    browser
        .open_tab(space, "Two", "https://two.example")
        .unwrap();
    browser
        .open_tab(space, "Three", "https://three.example")
        .unwrap();
    browser
}

fn ids(browser: &Browser, space: SpaceId) -> Vec<TabId> {
    browser.space(space).unwrap().tabs.clone()
}

fn profile_of(browser: &Browser, space: SpaceId) -> ProfileId {
    browser.space(space).unwrap().profile
}

// ---- Resource lifecycle ----------------------------------------------------

#[test]
fn lifecycle_transition_matrix_is_explicit() {
    use TabLifecycle::{Active, Discarded, Suspended, Warm};

    let states = [Active, Warm, Suspended, Discarded];
    for from in states {
        for to in states {
            let expected = from == to
                || matches!(
                    (from, to),
                    (Active, Warm)
                        | (Warm, Active | Suspended)
                        | (Suspended, Active | Discarded)
                        | (Discarded, Active)
                );
            assert_eq!(
                from.can_transition_to(to),
                expected,
                "unexpected transition verdict for {from:?} -> {to:?}"
            );
        }
    }
}

#[test]
fn selection_and_split_keep_visible_tabs_active() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let [t1, t2, t3] = ids(&b, s)[..] else {
        unreachable!()
    };
    assert_eq!(b.tab(t1).unwrap().lifecycle, TabLifecycle::Warm);
    assert_eq!(b.tab(t2).unwrap().lifecycle, TabLifecycle::Warm);
    assert_eq!(b.tab(t3).unwrap().lifecycle, TabLifecycle::Active);

    b.select_tab(s, t1).unwrap();
    assert_eq!(b.tab(t1).unwrap().lifecycle, TabLifecycle::Active);
    assert_eq!(b.tab(t3).unwrap().lifecycle, TabLifecycle::Warm);

    b.set_split(s, t1, t2).unwrap();
    assert_eq!(b.tab(t1).unwrap().lifecycle, TabLifecycle::Active);
    assert_eq!(b.tab(t2).unwrap().lifecycle, TabLifecycle::Active);
    b.close_split(s).unwrap();
    assert_eq!(b.tab(t1).unwrap().lifecycle, TabLifecycle::Active);
    assert_eq!(b.tab(t2).unwrap().lifecycle, TabLifecycle::Warm);

    b.set_split(s, t1, t2).unwrap();
    b.select_tab(s, t3).unwrap();
    assert!(b.split_of(s).is_none());
    assert_eq!(b.tab(t3).unwrap().lifecycle, TabLifecycle::Active);
    assert_eq!(b.tab(t1).unwrap().lifecycle, TabLifecycle::Warm);
    b.check_invariants().unwrap();
}

#[test]
fn hidden_tabs_suspend_and_discard_without_skipping_states() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let [hidden, _t2, visible] = ids(&b, s)[..] else {
        unreachable!()
    };
    assert_eq!(b.tab(hidden).unwrap().lifecycle, TabLifecycle::Warm);

    assert!(matches!(
        b.transition_tab_lifecycle(hidden, TabLifecycle::Discarded),
        Err(DomainError::IllegalLifecycleTransition { .. })
    ));
    b.transition_tab_lifecycle(hidden, TabLifecycle::Suspended)
        .unwrap();
    b.transition_tab_lifecycle(hidden, TabLifecycle::Discarded)
        .unwrap();
    assert_eq!(b.tab(hidden).unwrap().lifecycle, TabLifecycle::Discarded);
    assert!(matches!(
        b.transition_tab_lifecycle(hidden, TabLifecycle::Active),
        Err(DomainError::LifecycleVisibilityConflict { .. })
    ));
    assert!(matches!(
        b.transition_tab_lifecycle(visible, TabLifecycle::Warm),
        Err(DomainError::LifecycleVisibilityConflict { .. })
    ));

    b.select_tab(s, hidden).unwrap();
    assert_eq!(b.tab(hidden).unwrap().lifecycle, TabLifecycle::Active);
    assert_eq!(b.tab(visible).unwrap().lifecycle, TabLifecycle::Warm);
    b.check_invariants().unwrap();
}

// ---- Splits ----------------------------------------------------------------

#[test]
fn set_then_close_split() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let [a, b_id, _c] = ids(&b, s)[..] else {
        unreachable!()
    };
    b.set_split(s, a, b_id).unwrap();
    assert!(b.split_of(s).is_some());
    let split = b.close_split(s).unwrap().unwrap();
    assert_eq!(split.members(), [a, b_id]);
    assert!(b.split_of(s).is_none());
    b.check_invariants().unwrap();
}

#[test]
fn set_split_requires_member_tabs() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let t = ids(&b, s)[0];
    assert!(matches!(
        b.set_split(s, t, TabId(999)),
        Err(DomainError::TabNotInSpace { .. })
    ));
    assert!(matches!(
        b.set_split(s, t, t),
        Err(DomainError::SplitRequiresDistinctTabs)
    ));
    b.check_invariants().unwrap();
}

#[test]
fn duplicate_split_rejected() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let [a, b_id, c] = ids(&b, s)[..] else {
        unreachable!()
    };
    b.set_split(s, a, b_id).unwrap();
    assert!(matches!(
        b.set_split(s, a, c),
        Err(DomainError::SplitAlreadyExists(_))
    ));
    b.check_invariants().unwrap();
}

#[test]
fn closing_split_pane_dissolves_split() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    let [a, b_id, _c] = ids(&b, s)[..] else {
        unreachable!()
    };
    b.set_split(s, a, b_id).unwrap();
    b.close_tab(s, a).unwrap();
    assert!(
        b.split_of(s).is_none(),
        "split must dissolve when a pane closes"
    );
    // The other pane stays open.
    assert!(ids(&b, s).contains(&b_id));
    b.check_invariants().unwrap();
}

#[test]
fn moving_split_pane_dissolves_split() {
    let mut b = populated();
    let s1 = b.spaces().next().unwrap().id;
    let p = profile_of(&b, s1);
    let s2 = b.create_space(p, "Second").unwrap();
    let [a, b_id, _c] = ids(&b, s1)[..] else {
        unreachable!()
    };
    b.set_split(s1, a, b_id).unwrap();
    b.move_tab(a, s2).unwrap();
    assert!(
        b.split_of(s1).is_none(),
        "split must dissolve when a pane moves away"
    );
    b.check_invariants().unwrap();
}

// ---- Invariants ------------------------------------------------------------

#[test]
fn check_invariants_detects_tampering() {
    let mut b = populated();
    let s = b.spaces().next().unwrap().id;
    b.space_mut(s).unwrap().pinned_count = 1; // disagrees with tab order
    assert!(b.check_invariants().is_err());
    b.space_mut(s).unwrap().pinned_count = 0;
    b.space_mut(s).unwrap().active_tab = Some(TabId(4242)); // not a member
    assert!(b.check_invariants().is_err());
    b.space_mut(s).unwrap().active_tab = None;
    // Split with identical panes.
    let [a, _b, _c] = ids(&b, s)[..] else {
        unreachable!()
    };
    b.set_split(s, a, a).unwrap_err();
    // A duplicate tab in the ordering.
    let dup = ids(&b, s)[0];
    b.space_mut(s).unwrap().tabs.push(dup);
    assert!(b.check_invariants().is_err());
}
