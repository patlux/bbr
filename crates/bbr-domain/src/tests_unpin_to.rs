//! Unpinning to a position (task packet `drag-back-out-of-pins`).

use crate::{Browser, DomainError, SpaceId, TabId, TabLifecycle};

/// One space with [p1, p2 | r1, r2] (two pinned, two regular).
fn space() -> (Browser, SpaceId, [TabId; 4]) {
    let mut b = Browser::new();
    let p = b.create_profile("Work");
    let s = b.create_space(p, "Main").unwrap();
    let p1 = b.open_tab(s, "Mail", "https://mail.example").unwrap();
    let p2 = b.open_tab(s, "Chat", "https://chat.example").unwrap();
    let r1 = b.open_tab(s, "News", "https://news.example").unwrap();
    let r2 = b.open_tab(s, "Docs", "https://docs.example").unwrap();
    b.pin_tab(s, p1).unwrap();
    b.pin_tab(s, p2).unwrap();
    (b, s, [p1, p2, r1, r2])
}

fn order(b: &Browser, s: SpaceId) -> (Vec<TabId>, usize) {
    let sp = b.space(s).unwrap();
    (sp.tabs.clone(), sp.pinned_count)
}

#[test]
fn unpinning_to_a_position_lands_exactly_there() {
    for (position, expected) in [(0, [1, 0, 2, 3]), (1, [1, 2, 0, 3]), (2, [1, 2, 3, 0])] {
        let (mut b, s, tabs) = space();
        b.unpin_tab_to(s, tabs[0], position).unwrap();
        let (order, pinned) = order(&b, s);
        assert_eq!(pinned, 1, "position {position}");
        assert_eq!(
            order,
            expected.map(|i| tabs[i]).to_vec(),
            "position {position}"
        );
        let tab = b.tab(tabs[0]).unwrap();
        assert!(!tab.pinned);
        assert_eq!(tab.pinned_page, None, "unpinning forgets the pinned page");
        b.check_invariants().unwrap();
    }
}

#[test]
fn unpinning_into_an_empty_regular_region() {
    let mut b = Browser::new();
    let p = b.create_profile("Work");
    let s = b.create_space(p, "Main").unwrap();
    let p1 = b.open_tab(s, "Mail", "https://mail.example").unwrap();
    b.pin_tab(s, p1).unwrap();
    b.unpin_tab_to(s, p1, 0).unwrap();
    assert_eq!(order(&b, s), (vec![p1], 0));
    b.check_invariants().unwrap();
}

#[test]
fn a_closed_pinned_tab_unpins_as_an_unloaded_regular_tab() {
    let (mut b, s, [p1, p2, r1, r2]) = space();
    b.select_tab(s, p1).unwrap();
    b.close_tab(s, p1).unwrap();
    assert_eq!(b.tab(p1).unwrap().lifecycle, TabLifecycle::Closed);
    b.unpin_tab_to(s, p1, 1).unwrap();
    assert_eq!(order(&b, s), (vec![p2, r1, p1, r2], 1));
    assert_eq!(b.tab(p1).unwrap().lifecycle, TabLifecycle::Suspended);
    assert!(
        !b.space(s)
            .unwrap()
            .recently_closed
            .iter()
            .any(|closed| closed.in_place),
        "the in-place reopen record goes with the pin"
    );
    b.check_invariants().unwrap();
}

#[test]
fn an_unpinned_tab_or_a_position_past_the_end_is_rejected_whole() {
    let (mut b, s, [p1, _, r1, _]) = space();
    let before = order(&b, s);
    assert_eq!(b.unpin_tab_to(s, r1, 0), Err(DomainError::NotPinned(r1)));
    assert_eq!(
        b.unpin_tab_to(s, p1, 3),
        Err(DomainError::IndexOutOfBounds { index: 3, len: 3 })
    );
    assert_eq!(order(&b, s), before, "nothing changed");
    assert!(b.tab(p1).unwrap().pinned);
}
