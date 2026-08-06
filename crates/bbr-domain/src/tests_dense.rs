//! Dense-list resource-policy tests (the session-budget slice).
//!
//! Covers the deterministic "recently used" recency stamps and the
//! `suspend_least_recently_used` budget policy: visible and pinned tabs stay
//! live, the most recently used tabs fill the remaining budget, least
//! recently used warm tabs are suspended first, and every ordering is
//! deterministic (ties break on the stable tab id, the suspend list is
//! emitted in id order).

use crate::{Browser, TabId, TabLifecycle};

/// Open `count` tabs into the default space, returning their ids in order.
fn open_many(browser: &mut Browser, count: usize) -> Vec<TabId> {
    let profile = browser.create_profile("Default");
    let space = browser.create_space(profile, "Dense").expect("fresh space");
    (0..count)
        .map(|i| {
            browser
                .open_tab(space, format!("tab {i}"), format!("https://tab{i}.example"))
                .expect("open tab")
        })
        .collect()
}

/// The recency stamp of `id` (helper keeps the closure borrow out of the
/// mutation calls).
fn stamp(browser: &Browser, id: TabId) -> u64 {
    browser.tab(id).unwrap().last_used
}

#[test]
fn open_select_and_reopen_bump_recency_monotonically() {
    let mut browser = Browser::new();
    let ids = open_many(&mut browser, 3);
    // Opening touches each new tab in order.
    assert!(
        stamp(&browser, ids[0]) < stamp(&browser, ids[1])
            && stamp(&browser, ids[1]) < stamp(&browser, ids[2])
    );
    // Selecting an older tab makes it the most recent.
    let space = browser.spaces().next().unwrap().id;
    browser.select_tab(space, ids[0]).expect("select");
    let after = stamp(&browser, ids[0]);
    assert!(after > stamp(&browser, ids[2]), "selection is most recent");
    // Reopen of a closed tab also touches the restored identity.
    browser.close_tab(space, ids[2]).expect("close");
    let restored = browser.reopen_tab(space).expect("reopen");
    assert_eq!(restored, ids[2]);
    assert!(stamp(&browser, restored) > after);
}

#[test]
fn budget_keeps_visible_then_pinned_then_recent() {
    let mut browser = Browser::new();
    let ids = open_many(&mut browser, 10);
    let space = browser.spaces().next().unwrap().id;
    // Pin the first two tabs; select tab 5 (recency order changes).
    browser.pin_tab(space, ids[0]).expect("pin");
    browser.pin_tab(space, ids[1]).expect("pin");
    browser.select_tab(space, ids[5]).expect("select");

    // Cap 4: kept = visible (5) + pinned (0, 1) = 3, then 1 most recent
    // remaining (ids[9], opened last).
    let suspended = browser.suspend_least_recently_used(4);
    for kept in [ids[0], ids[1], ids[5], ids[9]] {
        assert!(
            !matches!(
                browser.tab(kept).unwrap().lifecycle,
                TabLifecycle::Suspended
            ),
            "tab {kept:?} must stay live"
        );
    }
    assert_eq!(
        suspended.len(),
        6,
        "10 tabs − 4 kept (1 visible + 2 pinned + 1 recent)"
    );
}

#[test]
fn budget_is_deterministic_and_tie_breaks_on_stable_id() {
    let mut browser = Browser::new();
    let ids = open_many(&mut browser, 6);
    let space = browser.spaces().next().unwrap().id;
    // No selection after open: recency order equals id order (0 oldest).
    browser.pin_tab(space, ids[0]).expect("pin");
    let suspended = browser.suspend_least_recently_used(2);
    // Kept: pinned 0 + most recent (5) → suspended are 1..=4 in id order.
    assert_eq!(suspended, vec![ids[1], ids[2], ids[3], ids[4]]);
    // Re-running the policy on the same state is a stable no-op.
    let again = browser.suspend_least_recently_used(2);
    assert!(again.is_empty(), "already at budget");
}

#[test]
fn zero_budget_suspends_every_regular_tab() {
    let mut browser = Browser::new();
    let ids = open_many(&mut browser, 5);
    let space = browser.spaces().next().unwrap().id;
    browser.pin_tab(space, ids[2]).expect("pin");
    let suspended = browser.suspend_least_recently_used(0);
    // Visible tab (the last opened, id 4) and the pinned tab survive.
    assert!(!suspended.contains(&ids[2]));
    assert_eq!(suspended.len(), 3);
    for id in &suspended {
        assert_eq!(browser.tab(*id).unwrap().lifecycle, TabLifecycle::Suspended);
    }
    for kept in [ids[2], ids[4]] {
        assert!(matches!(
            browser.tab(kept).unwrap().lifecycle,
            TabLifecycle::Active | TabLifecycle::Warm
        ));
    }
}

#[test]
fn budget_never_touches_discarded_tabs() {
    let mut browser = Browser::new();
    let ids = open_many(&mut browser, 4);
    browser.discard_tab(ids[0]).expect("discard");
    let suspended = browser.suspend_least_recently_used(1);
    // Discarded owns no session and is not part of the suspend set.
    assert!(!suspended.contains(&ids[0]));
    assert_eq!(
        browser.tab(ids[0]).unwrap().lifecycle,
        TabLifecycle::Discarded
    );
}
