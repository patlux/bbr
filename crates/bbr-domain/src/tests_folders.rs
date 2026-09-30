//! Unit tests for the session-local folder store (lane L5 foundation):
//! folder creation/ownership, whole-operation rejection, deterministic
//! favorites ordering, reconciliation after browser mutations, and the
//! public invariant checker. Portable: no macOS frameworks or display server.

use crate::folders::{FolderError, FolderStore, TabFacts};
use crate::{Browser, FolderId, SpaceId, TabId};

/// A browser with one profile, one space, and three tabs [t1, t2, t3].
fn populated() -> (Browser, SpaceId, Vec<TabId>) {
    let mut b = Browser::new();
    let p = b.create_profile("Work");
    let s = b.create_space(p, "Main").unwrap();
    let tabs = ["a", "b", "c"]
        .into_iter()
        .map(|u| b.open_tab(s, u, format!("https://{u}.example")).unwrap())
        .collect();
    (b, s, tabs)
}

/// A second profile with its own empty space (for cross-space rejections).
fn other_space(b: &mut Browser) -> SpaceId {
    let p = b.create_profile("Personal");
    b.create_space(p, "Personal-S").unwrap()
}

// ---- Folder creation ------------------------------------------------------

#[test]
fn create_folder_assigns_distinct_sequential_ids() {
    let (b, s, _) = populated();
    let mut store = FolderStore::new();
    let f1 = store.create_folder(&b, s, "Work").unwrap();
    let f2 = store.create_folder(&b, s, "Docs").unwrap();
    assert_ne!(f1, f2);
    assert_eq!(store.folder(f1).unwrap().title, "Work");
    assert_eq!(store.folder(f2).unwrap().title, "Docs");
    assert_eq!(store.folders().count(), 2);
    store.check_invariants(&b).unwrap();
}

#[test]
fn create_folder_in_unknown_space_is_rejected_whole() {
    let (b, _, _) = populated();
    let mut store = FolderStore::new();
    assert!(matches!(
        store.create_folder(&b, SpaceId(999), "Ghost"),
        Err(FolderError::UnknownSpace(SpaceId(999)))
    ));
    assert!(
        store.folders().next().is_none(),
        "rejected creation must not mutate the store"
    );
    store.check_invariants(&b).unwrap();
}

#[test]
fn folders_touch_the_browser_only_by_pinning() {
    let (mut b, s, tabs) = populated();
    b.pin_tab(s, tabs[0]).unwrap();
    b.pin_tab(s, tabs[1]).unwrap();
    let mut store = FolderStore::new();
    let before = b.clone();
    let f = store.create_folder(&b, s, "Work").unwrap();
    // Grouping or favoriting an already pinned tab needs no browser change.
    assert_eq!(store.move_tab_into_folder(&mut b, f, tabs[0]), Ok(false));
    assert!(!store.favorite_toggle(&mut b, tabs[1]).unwrap().newly_pinned);
    // The browser state is byte-for-byte untouched by folder operations.
    assert_eq!(b, before);
    store.check_invariants(&b).unwrap();
}

// ---- Membership -----------------------------------------------------------

#[test]
fn grouping_pins_atomically_and_keeps_space_membership() {
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    assert_eq!(store.move_tab_into_folder(&mut b, f, tabs[0]), Ok(true));
    assert_eq!(store.move_tab_into_folder(&mut b, f, tabs[2]), Ok(true));
    let folder = store.folder(f).unwrap();
    assert_eq!(folder.tabs, vec![tabs[0], tabs[2]]);
    // Both members are pinned: the grouped tab moved to the end of the
    // pinned region; identity and space membership are untouched.
    let space = b.space(s).unwrap();
    assert_eq!(space.tabs, vec![tabs[0], tabs[2], tabs[1]]);
    assert_eq!(space.pinned_count, 2);
    assert!(b.tab(tabs[0]).unwrap().pinned && b.tab(tabs[2]).unwrap().pinned);
    assert!(!b.tab(tabs[1]).unwrap().pinned, "non-members stay regular");
    assert_eq!(b.tab(tabs[2]).unwrap().space, s);
    store.check_invariants(&b).unwrap();
}

#[test]
fn a_membership_primitive_never_creates_an_unpinned_member() {
    let (b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    let before = store.clone();
    assert_eq!(
        store.move_membership(
            f,
            tabs[0],
            TabFacts {
                space: s,
                pinned: false
            }
        ),
        Err(FolderError::TabNotPinned(tabs[0]))
    );
    assert_eq!(store, before, "rejected whole");
}

#[test]
fn rejected_grouping_leaves_the_browser_and_store_unchanged() {
    let (mut b, _s, tabs) = populated();
    let s2 = other_space(&mut b);
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s2, "Other").unwrap();
    let (browser_before, store_before) = (b.clone(), store.clone());
    assert!(matches!(
        store.move_tab_into_folder(&mut b, f, tabs[0]),
        Err(FolderError::TabNotInFolderSpace { .. })
    ));
    assert!(matches!(
        store.move_tab_into_folder(&mut b, FolderId(99), tabs[0]),
        Err(FolderError::UnknownFolder(_))
    ));
    assert_eq!(b, browser_before, "the tab was not pinned");
    assert_eq!(store, store_before);
}

#[test]
fn ungrouping_leaves_a_loose_pinned_tab() {
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.move_tab_into_folder(&mut b, f, tabs[1]).unwrap();
    store.remove_tab_from_folder(f, tabs[1]).unwrap();
    assert!(store.folder(f).unwrap().tabs.is_empty());
    assert!(b.tab(tabs[1]).unwrap().pinned, "ungroup keeps the pin");
    assert_eq!(store.reconcile(&b), 0);
    store.check_invariants(&b).unwrap();
}

#[test]
fn unpinning_a_member_ends_its_membership() {
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.move_tab_into_folder(&mut b, f, tabs[0]).unwrap();
    store.move_tab_into_folder(&mut b, f, tabs[1]).unwrap();
    b.unpin_tab(s, tabs[0]).unwrap();
    assert!(
        store.check_invariants(&b).is_err(),
        "an unpinned member is a detected violation until reconciled"
    );
    assert_eq!(store.reconcile(&b), 1);
    assert_eq!(store.folder(f).unwrap().tabs, vec![tabs[1]]);
    assert!(!b.tab(tabs[0]).unwrap().pinned, "the unpin stands");
    store.check_invariants(&b).unwrap();
}

#[test]
fn moving_into_the_same_folder_is_a_noop() {
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.move_tab_into_folder(&mut b, f, tabs[0]).unwrap();
    let before = store.clone();
    store.move_tab_into_folder(&mut b, f, tabs[0]).unwrap();
    assert_eq!(store, before, "no-op must not reorder or duplicate");
    store.check_invariants(&b).unwrap();
}

#[test]
fn moving_between_folders_leaves_the_previous_folder() {
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let work = store.create_folder(&b, s, "Work").unwrap();
    let docs = store.create_folder(&b, s, "Docs").unwrap();
    store.move_tab_into_folder(&mut b, work, tabs[0]).unwrap();
    store.move_tab_into_folder(&mut b, work, tabs[1]).unwrap();
    store.move_tab_into_folder(&mut b, docs, tabs[0]).unwrap();
    assert_eq!(store.folder(work).unwrap().tabs, vec![tabs[1]]);
    assert_eq!(store.folder(docs).unwrap().tabs, vec![tabs[0]]);
    store.check_invariants(&b).unwrap();
}

#[test]
fn tab_of_another_space_is_rejected_whole() {
    let (mut b, s, tabs) = populated();
    let s2 = other_space(&mut b);
    let foreign = b
        .open_tab(s2, "Foreign", "https://foreign.example")
        .unwrap();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    assert!(matches!(
        store.move_tab_into_folder(&mut b, f, foreign),
        Err(FolderError::TabNotInFolderSpace { .. })
    ));
    assert!(
        store.folder(f).unwrap().tabs.is_empty(),
        "rejected move must not partially mutate"
    );
    store.check_invariants(&b).unwrap();
    // A folder of the other space accepts only that space's tabs.
    let f2 = store.create_folder(&b, s2, "Other").unwrap();
    assert!(matches!(
        store.move_tab_into_folder(&mut b, f2, tabs[0]),
        Err(FolderError::TabNotInFolderSpace { .. })
    ));
    assert!(store.folder(f2).unwrap().tabs.is_empty());
    store.check_invariants(&b).unwrap();
}

#[test]
fn unknown_folder_or_tab_are_rejected() {
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    assert!(matches!(
        store.move_tab_into_folder(&mut b, FolderId(99), tabs[0]),
        Err(FolderError::UnknownFolder(FolderId(99)))
    ));
    assert!(matches!(
        store.move_tab_into_folder(&mut b, f, TabId(999)),
        Err(FolderError::UnknownTab(TabId(999)))
    ));
    assert!(matches!(
        store.remove_tab_from_folder(FolderId(99), tabs[0]),
        Err(FolderError::UnknownFolder(FolderId(99)))
    ));
    store.check_invariants(&b).unwrap();
}

#[test]
fn remove_tab_from_folder_is_idempotent() {
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.move_tab_into_folder(&mut b, f, tabs[0]).unwrap();
    store.remove_tab_from_folder(f, tabs[0]).unwrap();
    assert!(store.folder(f).unwrap().tabs.is_empty());
    // Removing a non-member is a no-op, not an error.
    store.remove_tab_from_folder(f, tabs[0]).unwrap();
    store.remove_tab_from_folder(f, tabs[1]).unwrap();
    store.check_invariants(&b).unwrap();
}

// ---- Favorites ------------------------------------------------------------

#[test]
fn favorites_keep_deterministic_tab_insertion_order() {
    let (mut b, _s, tabs) = populated();
    let mut store = FolderStore::new();
    for tab in &tabs {
        store.favorite_toggle(&mut b, *tab).unwrap();
    }
    let actual: Vec<TabId> = store.favorites().map(|f| f.tab).collect();
    assert_eq!(actual, tabs, "fav grid renders tab ids in insertion order");
    store.check_invariants(&b).unwrap();
}

#[test]
fn toggling_removes_and_readd_appends_the_same_tab() {
    let (mut b, _s, tabs) = populated();
    let mut store = FolderStore::new();
    for tab in &tabs {
        assert!(store.favorite_toggle(&mut b, *tab).unwrap().newly_pinned);
    }
    let outcome = store.favorite_toggle(&mut b, tabs[0]).unwrap();
    assert_eq!(
        outcome,
        crate::FavoriteToggle {
            added: false,
            tab: tabs[0],
            newly_pinned: false,
        }
    );
    assert!(
        b.tab(tabs[0]).unwrap().pinned,
        "Remove from Favorites leaves a loose pinned tab"
    );
    assert_eq!(
        store.favorites().map(|f| f.tab).collect::<Vec<_>>(),
        vec![tabs[1], tabs[2]]
    );
    let outcome = store.favorite_toggle(&mut b, tabs[0]).unwrap();
    assert_eq!(
        outcome,
        crate::FavoriteToggle {
            added: true,
            tab: tabs[0],
            newly_pinned: false,
        }
    );
    assert_eq!(
        store.favorites().map(|f| f.tab).collect::<Vec<_>>(),
        vec![tabs[1], tabs[2], tabs[0]]
    );
    store.check_invariants(&b).unwrap();
}

#[test]
fn favorite_identity_is_the_tab_not_its_url() {
    let (mut b, s, _) = populated();
    let mut store = FolderStore::new();
    let first = b.open_tab(s, "A", "https://same.example").unwrap();
    let second = b.open_tab(s, "A copy", "https://same.example").unwrap();
    store.favorite_toggle(&mut b, first).unwrap();
    store.favorite_toggle(&mut b, second).unwrap();
    assert_eq!(
        store.favorites().map(|f| f.tab).collect::<Vec<_>>(),
        vec![first, second],
        "two concrete tabs with the same URL are distinct favorites"
    );
    b.apply_navigation(first, "https://elsewhere.example", "Moved on")
        .unwrap();
    assert_eq!(store.favorites().next().unwrap().tab, first);
    store.check_invariants(&b).unwrap();
}

#[test]
fn favorite_follows_its_tab_across_spaces_and_profiles() {
    let (mut b, _s, tabs) = populated();
    let mut store = FolderStore::new();
    store.favorite_toggle(&mut b, tabs[0]).unwrap();
    let destination = other_space(&mut b);
    b.move_tab(tabs[0], destination).unwrap();
    assert_eq!(store.reconcile(&b), 0, "moving a tab keeps its favorite");
    assert_eq!(store.favorites().next().unwrap().tab, tabs[0]);
    store.check_invariants(&b).unwrap();
}

#[test]
fn closing_a_favorite_keeps_its_tile_closed() {
    // DESIGN.md §10.12: a favorite is a kept place; closing its tab closes
    // it in place and the tile stays (greyed by the presentation).
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    for tab in &tabs {
        store.favorite_toggle(&mut b, *tab).unwrap();
    }
    b.close_tab(s, tabs[1]).unwrap();
    assert_eq!(store.reconcile(&b), 0);
    assert_eq!(
        store.favorites().map(|f| f.tab).collect::<Vec<_>>(),
        tabs.to_vec()
    );
    assert_eq!(
        b.tab(tabs[1]).unwrap().lifecycle,
        crate::TabLifecycle::Closed
    );
    store.check_invariants(&b).unwrap();
}

#[test]
fn favoriting_pins_a_regular_tab_and_leaves_its_folder() {
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.move_tab_into_folder(&mut b, f, tabs[0]).unwrap();
    // A folder member becomes a favorite: it lists in the grid only.
    let grouped = store.favorite_toggle(&mut b, tabs[0]).unwrap();
    assert!(grouped.added && !grouped.newly_pinned);
    assert!(store.folder(f).unwrap().tabs.is_empty());
    // A regular tab is pinned by the same transition, at the end of the
    // pinned region, with its current page as its pinned page.
    let regular = store.favorite_toggle(&mut b, tabs[2]).unwrap();
    assert!(regular.added && regular.newly_pinned);
    let space = b.space(s).unwrap();
    assert_eq!(space.tabs, vec![tabs[0], tabs[2], tabs[1]]);
    assert_eq!(space.pinned_count, 2);
    let pinned_page = b.tab(tabs[2]).unwrap().pinned_page.clone().unwrap();
    assert_eq!(pinned_page.url, b.tab(tabs[2]).unwrap().url);
    store.check_invariants(&b).unwrap();
}

#[test]
fn grouping_or_unpinning_a_favorite_ends_it() {
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.favorite_toggle(&mut b, tabs[0]).unwrap();
    store.favorite_toggle(&mut b, tabs[1]).unwrap();
    // Grouping lists the tab in its folder, so it leaves the grid.
    store.move_tab_into_folder(&mut b, f, tabs[0]).unwrap();
    assert_eq!(
        store.favorites().map(|f| f.tab).collect::<Vec<_>>(),
        vec![tabs[1]]
    );
    store.check_invariants(&b).unwrap();
    // Unpinning ends the favorite at the next reconcile.
    b.unpin_tab(s, tabs[1]).unwrap();
    assert!(
        store.check_invariants(&b).is_err(),
        "detected until reconciled"
    );
    assert_eq!(store.reconcile(&b), 1);
    assert!(store.favorites().next().is_none());
    store.check_invariants(&b).unwrap();
}

// ---- Reconciliation -------------------------------------------------------

#[test]
fn closing_a_member_keeps_it_closed_in_its_folder() {
    // A member is pinned, so closing it closes it in place (DESIGN.md
    // §10.12): it stays in the folder as a closed row.
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.move_tab_into_folder(&mut b, f, tabs[0]).unwrap();
    store.move_tab_into_folder(&mut b, f, tabs[1]).unwrap();
    b.close_tab(s, tabs[0]).unwrap();
    assert_eq!(store.reconcile(&b), 0);
    assert_eq!(store.folder(f).unwrap().tabs, vec![tabs[0], tabs[1]]);
    assert_eq!(
        b.tab(tabs[0]).unwrap().lifecycle,
        crate::TabLifecycle::Closed
    );
    store.check_invariants(&b).unwrap();
}

#[test]
fn reconcile_drops_memberships_for_moved_tabs() {
    let (mut b, s, tabs) = populated();
    let s2 = other_space(&mut b);
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.move_tab_into_folder(&mut b, f, tabs[0]).unwrap();
    store.move_tab_into_folder(&mut b, f, tabs[1]).unwrap();
    b.move_tab(tabs[0], s2).unwrap();
    let removed = store.reconcile(&b);
    assert_eq!(removed, 1);
    assert_eq!(store.folder(f).unwrap().tabs, vec![tabs[1]]);
    store.check_invariants(&b).unwrap();
}

#[test]
fn reconcile_never_removes_valid_members() {
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    for tab in &tabs {
        store.move_tab_into_folder(&mut b, f, *tab).unwrap();
    }
    assert_eq!(store.reconcile(&b), 0, "all members still valid");
    assert_eq!(store.folder(f).unwrap().tabs.len(), 3);
    store.check_invariants(&b).unwrap();
}

// ---- Invariants -----------------------------------------------------------

#[test]
fn invariants_reject_duplicate_membership_across_folders() {
    // Mutations forbid a tab in two folders; the checker must still detect a
    // forged violation (the fields are pub(crate) exactly like Browser's).
    let (mut b, s, tabs) = populated();
    b.pin_tab(s, tabs[0]).unwrap();
    let facts = TabFacts {
        space: s,
        pinned: true,
    };
    let mut store = FolderStore::new();
    let f1 = store.create_folder(&b, s, "One").unwrap();
    let f2 = store.create_folder(&b, s, "Two").unwrap();
    store.move_membership(f1, tabs[0], facts).unwrap();
    store.move_membership(f2, tabs[0], facts).unwrap();
    store.check_invariants(&b).unwrap();
    // Forge: push the same tab into a second folder behind the API's back.
    store.folders[0].tabs.push(tabs[0]);
    assert!(store.check_invariants(&b).is_err());
}

#[test]
fn invariants_detect_unknown_space_and_duplicate_favorite_tabs() {
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    store.create_folder(&b, s, "Work").unwrap();
    store.favorite_toggle(&mut b, tabs[0]).unwrap();
    // Forge an unknown-space folder and a duplicate favorite tab.
    store.folders[0].space = SpaceId(999);
    store
        .favorites
        .push(crate::folders::Favorite { tab: tabs[0] });
    assert!(store.check_invariants(&b).is_err());
}

#[test]
fn invariants_detect_a_grouped_or_unpinned_favorite() {
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.favorite_toggle(&mut b, tabs[0]).unwrap();
    store.check_invariants(&b).unwrap();
    // Forge: the favorite is also a folder member.
    let mut grouped = store.clone();
    grouped.folders[0].tabs.push(tabs[0]);
    assert_eq!(grouped.folders[0].id, f);
    assert!(grouped.check_invariants(&b).is_err());
    // Forge: a regular tab in the grid.
    let mut unpinned = store.clone();
    unpinned
        .favorites
        .push(crate::folders::Favorite { tab: tabs[1] });
    assert!(unpinned.check_invariants(&b).is_err());
}

#[test]
fn invariants_detect_stale_membership_without_reconcile() {
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.move_tab_into_folder(&mut b, f, tabs[0]).unwrap();
    let s2 = other_space(&mut b);
    b.move_tab(tabs[0], s2).unwrap();
    assert!(
        store.check_invariants(&b).is_err(),
        "checker is a detector: stale memberships must fail until reconciled"
    );
    store.reconcile(&b);
    store.check_invariants(&b).unwrap();
}

#[test]
fn store_is_a_plain_value() {
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store
        .move_membership(
            f,
            tabs[0],
            TabFacts {
                space: s,
                pinned: true,
            },
        )
        .unwrap();
    store.favorite_toggle(&mut b, tabs[1]).unwrap();
    assert_eq!(store.clone(), store);
}

#[test]
fn rename_folder_trims_and_rejects_empty_titles_whole() {
    let (b, s, _) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "New Folder").unwrap();
    assert_eq!(store.rename_folder(f, "  Reading  ").unwrap(), "Reading");
    assert_eq!(store.folder(f).unwrap().title, "Reading");
    assert_eq!(
        store.rename_folder(f, "   "),
        Err(FolderError::EmptyTitle(f))
    );
    assert_eq!(store.folder(f).unwrap().title, "Reading", "rejected whole");
    assert_eq!(
        store.rename_folder(FolderId(99), "Ghost"),
        Err(FolderError::UnknownFolder(FolderId(99)))
    );
}
