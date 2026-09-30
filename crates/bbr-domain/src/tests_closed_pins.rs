//! Closed pinned tabs (DESIGN.md §10.12, task packet
//! `closed-pins-and-favorites`): a pinned tab is a kept place. Closing it
//! keeps the identity at its index as `Closed` at its pinned page, the
//! selection never lands on a closed tab, and only a selection reopens it.

use crate::{Browser, DomainError, SpaceId, TabId, TabLifecycle};

/// One space with [p1, p2 | r1] (two pinned, one regular), `active = r1`.
/// `p1` navigated after it was pinned, so its live page differs from its
/// pinned page.
fn pinned_space() -> (Browser, SpaceId, [TabId; 3]) {
    let mut b = Browser::new();
    let p = b.create_profile("Work");
    let s = b.create_space(p, "Main").unwrap();
    let p1 = b.open_tab(s, "Mail", "https://mail.example").unwrap();
    let p2 = b.open_tab(s, "Chat", "https://chat.example").unwrap();
    let r1 = b.open_tab(s, "News", "https://news.example").unwrap();
    b.pin_tab(s, p1).unwrap();
    b.pin_tab(s, p2).unwrap();
    b.apply_navigation(p1, "https://mail.example/inbox/42", "Re: lunch")
        .unwrap();
    (b, s, [p1, p2, r1])
}

#[test]
fn pinning_captures_the_pinned_page_and_unpinning_forgets_it() {
    let (mut b, s, [p1, _, r1]) = pinned_space();
    let page = b.tab(p1).unwrap().pinned_page.clone().unwrap();
    assert_eq!(page.url, "https://mail.example");
    assert_eq!(page.title, "Mail");
    assert_eq!(b.tab(r1).unwrap().pinned_page, None);
    b.unpin_tab(s, p1).unwrap();
    assert_eq!(b.tab(p1).unwrap().pinned_page, None);
    b.check_invariants().unwrap();
}

#[test]
fn closing_a_pinned_tab_keeps_its_row_at_the_pinned_page() {
    let (mut b, s, [p1, p2, r1]) = pinned_space();
    b.select_tab(s, p1).unwrap();
    b.close_tab(s, p1).unwrap();
    let sp = b.space(s).unwrap();
    assert_eq!(sp.tabs, vec![p1, p2, r1], "the row keeps its place");
    assert_eq!(sp.pinned_count, 2);
    assert!(sp.recently_closed.is_empty(), "the row itself reopens it");
    assert_eq!(sp.active_tab, Some(p2), "the next open tab is selected");
    let tab = b.tab(p1).unwrap();
    assert_eq!(tab.lifecycle, TabLifecycle::Closed);
    assert_eq!(tab.url, "https://mail.example", "reset to the pinned page");
    assert_eq!(tab.title, "Mail");
    assert_eq!(b.closed_tab_count(), 0, "no reopenable close happened");
    b.check_invariants().unwrap();
    // Closing it again changes nothing.
    let before = b.clone();
    b.close_tab(s, p1).unwrap();
    assert_eq!(b, before);
}

#[test]
fn the_selection_never_lands_on_a_closed_tab() {
    let (mut b, s, [p1, p2, r1]) = pinned_space();
    // p2 closes while r1 stays selected.
    b.close_tab(s, p2).unwrap();
    assert_eq!(b.space(s).unwrap().active_tab, Some(r1));
    // Closing r1 (no next sibling) skips the closed p2 and selects p1.
    b.close_tab(s, r1).unwrap();
    assert_eq!(b.space(s).unwrap().active_tab, Some(p1));
    // Closing the last open tab leaves no selection, not a closed tab.
    b.close_tab(s, p1).unwrap();
    let sp = b.space(s).unwrap();
    assert_eq!(sp.active_tab, None);
    assert_eq!(sp.tabs, vec![p1, p2]);
    b.check_invariants().unwrap();
}

#[test]
fn selecting_a_closed_tab_reopens_it_at_the_pinned_page() {
    let (mut b, s, [p1, _, _]) = pinned_space();
    b.close_tab(s, p1).unwrap();
    b.select_tab(s, p1).unwrap();
    let tab = b.tab(p1).unwrap();
    assert_eq!(tab.lifecycle, TabLifecycle::Active);
    assert_eq!(tab.url, "https://mail.example");
    assert_eq!(b.space(s).unwrap().active_tab, Some(p1));
    b.check_invariants().unwrap();
}

#[test]
fn unpinning_a_closed_tab_leaves_an_unloaded_regular_tab() {
    let (mut b, s, [p1, p2, r1]) = pinned_space();
    b.close_tab(s, p1).unwrap();
    b.unpin_tab(s, p1).unwrap();
    let tab = b.tab(p1).unwrap();
    assert!(!tab.pinned);
    assert_eq!(tab.lifecycle, TabLifecycle::Suspended);
    assert_eq!(b.space(s).unwrap().tabs, vec![p2, r1, p1]);
    b.check_invariants().unwrap();
}

#[test]
fn only_closing_enters_the_closed_state() {
    let (mut b, s, [_, p2, _]) = pinned_space();
    assert!(matches!(
        b.transition_tab_lifecycle(p2, TabLifecycle::Closed),
        Err(DomainError::IllegalLifecycleTransition { .. })
    ));
    b.close_tab(s, p2).unwrap();
    assert!(matches!(
        b.transition_tab_lifecycle(p2, TabLifecycle::Suspended),
        Err(DomainError::IllegalLifecycleTransition { .. })
    ));
    assert!(TabLifecycle::Closed.can_transition_to(TabLifecycle::Active));
}

#[test]
fn a_closed_tab_takes_no_session_budget() {
    let (mut b, s, [p1, p2, r1]) = pinned_space();
    b.close_tab(s, p1).unwrap();
    let r2 = b.open_tab(s, "Docs", "https://docs.example").unwrap();
    // Budget 2: the visible r2 and the open pinned p2 stay live; r1 is
    // suspended. The closed p1 owns no session and takes no budget.
    let suspended = b.suspend_least_recently_used(2);
    assert_eq!(suspended, vec![r1]);
    assert_eq!(b.tab(p1).unwrap().lifecycle, TabLifecycle::Closed);
    assert_eq!(b.tab(p2).unwrap().lifecycle, TabLifecycle::Warm);
    assert_eq!(b.tab(r2).unwrap().lifecycle, TabLifecycle::Active);
    b.check_invariants().unwrap();
}

#[test]
fn moving_a_closed_tab_opens_it_in_the_destination() {
    let (mut b, s, [p1, _, _]) = pinned_space();
    let profile = b.space(s).unwrap().profile;
    let other = b.create_space(profile, "Other").unwrap();
    b.close_tab(s, p1).unwrap();
    b.move_tab(p1, other).unwrap();
    let tab = b.tab(p1).unwrap();
    assert!(tab.pinned, "a move keeps the pin");
    assert_eq!(tab.lifecycle, TabLifecycle::Active);
    assert_eq!(b.space(other).unwrap().active_tab, Some(p1));
    b.check_invariants().unwrap();
}

#[test]
fn invariants_reject_a_closed_regular_or_visible_tab() {
    let (mut b, s, [p1, _, r1]) = pinned_space();
    // Forge a closed regular tab.
    let mut forged = b.clone();
    forged
        .tabs
        .iter_mut()
        .find(|tab| tab.id == r1)
        .unwrap()
        .lifecycle = TabLifecycle::Closed;
    assert!(forged.check_invariants().is_err());
    // Forge a closed tab that is still selected.
    b.close_tab(s, p1).unwrap();
    let mut forged = b.clone();
    forged
        .spaces
        .iter_mut()
        .find(|space| space.id == s)
        .unwrap()
        .active_tab = Some(p1);
    assert!(forged.check_invariants().is_err());
    // Forge a pinned tab without its pinned page.
    let mut forged = b.clone();
    forged
        .tabs
        .iter_mut()
        .find(|tab| tab.id == p1)
        .unwrap()
        .pinned_page = None;
    assert!(forged.check_invariants().is_err());
}
