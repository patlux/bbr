//! Closed pinned tabs (DESIGN.md §10.12, task packet
//! `closed-pins-and-favorites`): a pinned tab is a kept place. Closing it
//! keeps the identity at its index as `Closed` at its pinned page, the
//! selection never lands on a closed tab, and a selection (at the pinned
//! page) or ⇧⌘T (at the page it closed on) reopens it.

use crate::{Browser, DomainError, PinnedPage, SpaceId, Tab, TabId, TabLifecycle};

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
    assert_eq!(sp.active_tab, Some(p2), "the next open tab is selected");
    // The in-place record remembers the page the row showed.
    assert_eq!(sp.recently_closed.len(), 1);
    let record = &sp.recently_closed[0];
    assert!(record.in_place);
    assert_eq!(record.tab.id, p1);
    assert_eq!(record.tab.url, "https://mail.example/inbox/42");
    assert_eq!(record.index, 0);
    let tab = b.tab(p1).unwrap();
    assert_eq!(tab.lifecycle, TabLifecycle::Closed);
    assert_eq!(tab.url, "https://mail.example", "reset to the pinned page");
    assert_eq!(tab.title, "Mail");
    assert_eq!(b.closed_tab_count(), 1, "a reopenable close happened");
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
fn reopen_brings_back_the_page_a_pinned_tab_closed_on() {
    let (mut b, s, [p1, p2, r1]) = pinned_space();
    b.select_tab(s, p1).unwrap();
    b.close_tab(s, p1).unwrap();
    assert_eq!(b.reopen_tab(s).unwrap(), p1);
    let sp = b.space(s).unwrap();
    assert_eq!(sp.tabs, vec![p1, p2, r1], "the same row reopens");
    assert_eq!(sp.pinned_count, 2);
    assert_eq!(sp.active_tab, Some(p1));
    assert!(sp.recently_closed.is_empty());
    let tab = b.tab(p1).unwrap();
    assert_eq!(tab.lifecycle, TabLifecycle::Active);
    assert_eq!(tab.url, "https://mail.example/inbox/42");
    assert_eq!(tab.title, "Re: lunch");
    assert_eq!(
        tab.pinned_page.as_ref().unwrap().url,
        "https://mail.example",
        "the pinned page stays"
    );
    b.check_invariants().unwrap();
}

#[test]
fn reopen_follows_the_close_order_across_regular_and_pinned_tabs() {
    let (mut b, s, [p1, _, r1]) = pinned_space();
    b.close_tab(s, r1).unwrap();
    b.close_tab(s, p1).unwrap();
    assert_eq!(b.reopen_tab(s).unwrap(), p1, "the latest close first");
    assert_eq!(b.reopen_tab(s).unwrap(), r1);
    assert!(matches!(
        b.reopen_tab(s),
        Err(DomainError::NothingToReopen(_))
    ));
    b.check_invariants().unwrap();
}

#[test]
fn history_reopens_a_closed_pinned_tab_by_identity() {
    let (mut b, s, [p1, p2, r1]) = pinned_space();
    b.close_tab(s, p1).unwrap();
    b.close_tab(s, r1).unwrap();
    assert_eq!(b.reopen_closed_tab(s, p1).unwrap(), p1);
    assert_eq!(b.tab(p1).unwrap().url, "https://mail.example/inbox/42");
    assert_eq!(b.space(s).unwrap().tabs, vec![p1, p2]);
    assert_eq!(b.space(s).unwrap().recently_closed.len(), 1);
    b.check_invariants().unwrap();
}

#[test]
fn opening_the_closed_row_otherwise_drops_its_record() {
    // Selecting the row reopens it at its pinned page.
    let (mut b, s, [p1, _, _]) = pinned_space();
    b.close_tab(s, p1).unwrap();
    b.select_tab(s, p1).unwrap();
    assert!(b.space(s).unwrap().recently_closed.is_empty());
    assert!(matches!(
        b.reopen_tab(s),
        Err(DomainError::NothingToReopen(_))
    ));
    b.check_invariants().unwrap();
    // Unpinning ends the kept place.
    let (mut b, s, [p1, _, _]) = pinned_space();
    b.close_tab(s, p1).unwrap();
    b.unpin_tab(s, p1).unwrap();
    assert!(b.space(s).unwrap().recently_closed.is_empty());
    b.check_invariants().unwrap();
    // Moving the row to another space opens it there.
    let (mut b, s, [p1, _, _]) = pinned_space();
    let profile = b.space(s).unwrap().profile;
    let other = b.create_space(profile, "Other").unwrap();
    b.close_tab(s, p1).unwrap();
    b.move_tab(p1, other).unwrap();
    assert!(b.space(s).unwrap().recently_closed.is_empty());
    b.check_invariants().unwrap();
}

#[test]
fn deleting_the_space_closes_the_row_once() {
    let (mut b, s, [p1, p2, r1]) = pinned_space();
    let profile = b.space(s).unwrap().profile;
    let fallback = b.create_space(profile, "Other").unwrap();
    b.close_tab(s, p1).unwrap();
    b.delete_space(s).unwrap();
    let closed: Vec<TabId> = b
        .space(fallback)
        .unwrap()
        .recently_closed
        .iter()
        .map(|closed| {
            assert!(!closed.in_place);
            closed.tab.id
        })
        .collect();
    assert_eq!(
        closed,
        vec![r1, p2, p1],
        "each tab once; the first row reopens first"
    );
    b.check_invariants().unwrap();
}

#[test]
fn restore_imports_the_kept_place_and_its_record() {
    let mut b = Browser::new();
    let profile = crate::ProfileId(1);
    b.restore_profile(profile, crate::ProfileHue::Cobalt, "Work")
        .unwrap();
    let s = SpaceId(2);
    b.restore_space(s, profile, "Main").unwrap();
    let (p1, r1) = (TabId(3), TabId(4));
    b.restore_tab(p1, s, "Mail", "https://mail.example", true)
        .unwrap();
    b.restore_tab(r1, s, "News", "https://news.example", false)
        .unwrap();
    let page = PinnedPage {
        url: "https://mail.example".into(),
        title: "Mail".into(),
    };
    assert_eq!(
        b.restore_kept_place(r1, page.clone(), true),
        Err(DomainError::InvalidKeptPlace(r1)),
        "only a pinned tab keeps a place"
    );
    // A record for an open row is rejected.
    let last = Tab::new(p1, s, "Re: lunch", "https://mail.example/inbox/42");
    assert_eq!(
        b.restore_in_place_closed(s, last.clone(), 0),
        Err(DomainError::InvalidKeptPlace(p1))
    );
    b.restore_kept_place(p1, page.clone(), true).unwrap();
    b.restore_in_place_closed(s, last.clone(), 0).unwrap();
    assert_eq!(
        b.restore_in_place_closed(s, last, 0),
        Err(DomainError::InvalidKeptPlace(p1)),
        "one record per row"
    );
    b.restore_active_tab(s, Some(r1)).unwrap();
    b.restore_activate_tab(r1).unwrap();
    let tab = b.tab(p1).unwrap();
    assert_eq!(tab.lifecycle, TabLifecycle::Closed);
    assert_eq!(tab.pinned_page, Some(page));
    b.check_invariants().unwrap();
    assert_eq!(b.reopen_tab(s).unwrap(), p1);
    assert_eq!(b.tab(p1).unwrap().url, "https://mail.example/inbox/42");
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
    // Forge an in-place record for an open row.
    let mut forged = b.clone();
    forged.select_tab(s, p1).unwrap();
    let record = b.space(s).unwrap().recently_closed[0].clone();
    forged
        .spaces
        .iter_mut()
        .find(|space| space.id == s)
        .unwrap()
        .recently_closed
        .push(record);
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

#[test]
fn a_stale_split_or_a_crash_never_reopens_a_closed_row() {
    let (mut b, s, [p1, p2, _]) = pinned_space();
    b.close_tab(s, p1).unwrap();
    // A persisted group naming the closed row is stale: it is dropped and
    // the row stays closed with its reopen record.
    let stale = crate::tiling::SplitGroup::pair(p1, p2, crate::tiling::Axis::Row).unwrap();
    b.restore_splits(s, vec![stale]).unwrap();
    assert!(b.space(s).unwrap().splits.is_empty());
    assert_eq!(b.tab(p1).unwrap().lifecycle, TabLifecycle::Closed);
    assert_eq!(b.space(s).unwrap().recently_closed.len(), 1);
    // A crash report cannot reach a session-less closed row either.
    b.discard_tab(p1).unwrap();
    assert_eq!(b.tab(p1).unwrap().lifecycle, TabLifecycle::Closed);
    b.check_invariants().unwrap();
}
