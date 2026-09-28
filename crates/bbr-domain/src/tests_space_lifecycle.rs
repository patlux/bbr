//! Unit tests for space deletion and display-order moves (blueprint D05).

use crate::{Browser, DomainError, SpaceId, SpaceMove, TabId};

/// Profile "Work" with spaces [a, b, c] and profile "Home" with [h]
/// interleaved in creation order: a, h, b, c.
fn spaces() -> (Browser, [SpaceId; 4]) {
    let mut browser = Browser::new();
    let work = browser.create_profile("Work");
    let home = browser.create_profile("Home");
    let a = browser.create_space(work, "A").unwrap();
    let h = browser.create_space(home, "H").unwrap();
    let b = browser.create_space(work, "B").unwrap();
    let c = browser.create_space(work, "C").unwrap();
    (browser, [a, h, b, c])
}

fn order(browser: &Browser) -> Vec<SpaceId> {
    browser.spaces().map(|space| space.id).collect()
}

fn open(browser: &mut Browser, space: SpaceId, urls: &[&str]) -> Vec<TabId> {
    urls.iter()
        .map(|url| browser.open_tab(space, *url, *url).unwrap())
        .collect()
}

#[test]
fn moves_swap_with_the_nearest_same_profile_neighbor() {
    let (mut browser, [a, h, b, c]) = spaces();
    assert!(!browser.can_move_space(a, SpaceMove::Earlier));
    assert!(browser.can_move_space(a, SpaceMove::Later));
    browser.move_space(a, SpaceMove::Later).unwrap();
    // The Home space keeps its container slot; A and B trade places.
    assert_eq!(order(&browser), vec![b, h, a, c]);
    browser.move_space(c, SpaceMove::Earlier).unwrap();
    assert_eq!(order(&browser), vec![b, h, c, a]);
    assert!(!browser.can_move_space(h, SpaceMove::Earlier));
    assert!(!browser.can_move_space(h, SpaceMove::Later));
    browser.check_invariants().unwrap();
}

#[test]
fn moves_past_the_group_edge_and_unknown_spaces_are_rejected_whole() {
    let (mut browser, [a, _h, _b, c]) = spaces();
    let before = order(&browser);
    assert_eq!(
        browser.move_space(a, SpaceMove::Earlier),
        Err(DomainError::SpaceMoveOutOfRange(a))
    );
    assert_eq!(
        browser.move_space(c, SpaceMove::Later),
        Err(DomainError::SpaceMoveOutOfRange(c))
    );
    assert_eq!(
        browser.move_space(SpaceId(999), SpaceMove::Later),
        Err(DomainError::UnknownSpace(SpaceId(999)))
    );
    assert_eq!(order(&browser), before);
}

#[test]
fn deleting_closes_the_tabs_onto_the_next_same_profile_space() {
    let (mut browser, [a, h, b, c]) = spaces();
    let [x] = open(&mut browser, b, &["x"])[..] else {
        unreachable!()
    };
    let tabs = open(&mut browser, a, &["1", "2", "3"]);
    browser.pin_tab(a, tabs[2]).unwrap();
    let deleted = browser.delete_space(a).unwrap();
    assert_eq!(deleted.space, a);
    assert_eq!(deleted.fallback, b, "the next space of the same profile");
    assert_eq!(
        deleted.closed,
        vec![tabs[2], tabs[0], tabs[1]],
        "space order"
    );
    assert_eq!(order(&browser), vec![h, b, c]);
    assert!(browser.space(a).is_none());
    for tab in &tabs {
        assert!(browser.tab(*tab).is_none(), "closed tabs leave the table");
    }
    browser.check_invariants().unwrap();

    // The former positions are real indices after the fallback's tabs (a
    // persisted closed tab stores them; no sentinel may reach the store).
    let indices: Vec<usize> = browser
        .space(b)
        .unwrap()
        .recently_closed
        .iter()
        .map(|closed| closed.index)
        .collect();
    assert_eq!(indices, vec![3, 2, 1], "most recent (first tab) last");

    // Reopen Closed Tab is the undo: the space's tabs come back first to
    // last, each at the end of its region, owned by the fallback.
    let reopened: Vec<TabId> = (0..3).map(|_| browser.reopen_tab(b).unwrap()).collect();
    assert_eq!(reopened, deleted.closed);
    let space = browser.space(b).unwrap();
    assert_eq!(space.tabs, vec![tabs[2], x, tabs[0], tabs[1]]);
    assert_eq!(space.pinned_count, 1, "the pinned tab reopens pinned");
    browser.check_invariants().unwrap();
}

#[test]
fn deleting_the_last_group_member_falls_back_to_the_previous_space() {
    let (mut browser, [_a, _h, b, c]) = spaces();
    let closed = open(&mut browser, c, &["old"]);
    browser.close_tab(c, closed[0]).unwrap();
    let deleted = browser.delete_space(c).unwrap();
    assert_eq!(deleted.fallback, b);
    assert!(deleted.closed.is_empty());
    // The deleted space's own recently-closed entries move with it.
    assert_eq!(browser.reopen_tab(b).unwrap(), closed[0]);
    browser.check_invariants().unwrap();
}

#[test]
fn the_last_space_of_a_profile_and_unknown_spaces_cannot_be_deleted() {
    let (mut browser, [a, h, _b, _c]) = spaces();
    assert!(browser.can_delete_space(a));
    assert!(!browser.can_delete_space(h));
    assert!(!browser.can_delete_space(SpaceId(999)));
    let before = browser.clone();
    assert_eq!(
        browser.delete_space(h),
        Err(DomainError::LastSpaceOfProfile(h))
    );
    assert_eq!(
        browser.delete_space(SpaceId(999)),
        Err(DomainError::UnknownSpace(SpaceId(999)))
    );
    assert_eq!(browser, before, "rejections change nothing");
}
