//! Unit tests for the session-local folder store (lane L5 foundation):
//! folder creation/ownership, whole-operation rejection, deterministic
//! favorites ordering, reconciliation after browser mutations, and the
//! public invariant checker. Portable: no macOS frameworks or display server.

use crate::folders::{FolderError, FolderStore};
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
fn folders_are_session_local_and_never_touch_the_browser() {
    let (b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let before = b.clone();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.move_tab_into_folder(&b, f, tabs[0]).unwrap();
    store.favorite_toggle(&b, tabs[1]).unwrap();
    // The browser state is byte-for-byte untouched by folder operations.
    assert_eq!(b, before);
    store.check_invariants(&b).unwrap();
}

// ---- Membership -----------------------------------------------------------

#[test]
fn move_tab_into_folder_keeps_space_membership_and_order() {
    let (b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.move_tab_into_folder(&b, f, tabs[0]).unwrap();
    store.move_tab_into_folder(&b, f, tabs[2]).unwrap();
    let folder = store.folder(f).unwrap();
    assert_eq!(folder.tabs, vec![tabs[0], tabs[2]]);
    // Space membership and ordering are untouched (membership is a grouping).
    assert_eq!(b.space(s).unwrap().tabs, tabs);
    assert_eq!(b.tab(tabs[0]).unwrap().space, s);
    store.check_invariants(&b).unwrap();
}

#[test]
fn moving_into_the_same_folder_is_a_noop() {
    let (b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.move_tab_into_folder(&b, f, tabs[0]).unwrap();
    let before = store.clone();
    store.move_tab_into_folder(&b, f, tabs[0]).unwrap();
    assert_eq!(store, before, "no-op must not reorder or duplicate");
    store.check_invariants(&b).unwrap();
}

#[test]
fn moving_between_folders_leaves_the_previous_folder() {
    let (b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let work = store.create_folder(&b, s, "Work").unwrap();
    let docs = store.create_folder(&b, s, "Docs").unwrap();
    store.move_tab_into_folder(&b, work, tabs[0]).unwrap();
    store.move_tab_into_folder(&b, work, tabs[1]).unwrap();
    store.move_tab_into_folder(&b, docs, tabs[0]).unwrap();
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
        store.move_tab_into_folder(&b, f, foreign),
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
        store.move_tab_into_folder(&b, f2, tabs[0]),
        Err(FolderError::TabNotInFolderSpace { .. })
    ));
    assert!(store.folder(f2).unwrap().tabs.is_empty());
    store.check_invariants(&b).unwrap();
}

#[test]
fn unknown_folder_or_tab_are_rejected() {
    let (b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    assert!(matches!(
        store.move_tab_into_folder(&b, FolderId(99), tabs[0]),
        Err(FolderError::UnknownFolder(FolderId(99)))
    ));
    assert!(matches!(
        store.move_tab_into_folder(&b, f, TabId(999)),
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
    let (b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.move_tab_into_folder(&b, f, tabs[0]).unwrap();
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
    let (b, _s, tabs) = populated();
    let mut store = FolderStore::new();
    for tab in &tabs {
        store.favorite_toggle(&b, *tab).unwrap();
    }
    let actual: Vec<TabId> = store.favorites().map(|f| f.tab).collect();
    assert_eq!(actual, tabs, "fav grid renders tab ids in insertion order");
    store.check_invariants(&b).unwrap();
}

#[test]
fn toggling_removes_and_readd_appends_the_same_tab() {
    let (b, _s, tabs) = populated();
    let mut store = FolderStore::new();
    for tab in &tabs {
        store.favorite_toggle(&b, *tab).unwrap();
    }
    let outcome = store.favorite_toggle(&b, tabs[0]).unwrap();
    assert_eq!(
        outcome,
        crate::FavoriteToggle {
            added: false,
            tab: tabs[0]
        }
    );
    assert_eq!(
        store.favorites().map(|f| f.tab).collect::<Vec<_>>(),
        vec![tabs[1], tabs[2]]
    );
    let outcome = store.favorite_toggle(&b, tabs[0]).unwrap();
    assert_eq!(
        outcome,
        crate::FavoriteToggle {
            added: true,
            tab: tabs[0]
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
    store.favorite_toggle(&b, first).unwrap();
    store.favorite_toggle(&b, second).unwrap();
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
    store.favorite_toggle(&b, tabs[0]).unwrap();
    let destination = other_space(&mut b);
    b.move_tab(tabs[0], destination).unwrap();
    assert_eq!(store.reconcile(&b), 0, "moving a tab keeps its favorite");
    assert_eq!(store.favorites().next().unwrap().tab, tabs[0]);
    store.check_invariants(&b).unwrap();
}

#[test]
fn closing_the_concrete_tab_removes_its_favorite() {
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    for tab in &tabs {
        store.favorite_toggle(&b, *tab).unwrap();
    }
    b.close_tab(s, tabs[1]).unwrap();
    assert_eq!(store.reconcile(&b), 1);
    assert_eq!(
        store.favorites().map(|f| f.tab).collect::<Vec<_>>(),
        vec![tabs[0], tabs[2]]
    );
    store.check_invariants(&b).unwrap();
}

// ---- Reconciliation -------------------------------------------------------

#[test]
fn reconcile_drops_memberships_for_closed_tabs() {
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.move_tab_into_folder(&b, f, tabs[0]).unwrap();
    store.move_tab_into_folder(&b, f, tabs[1]).unwrap();
    b.close_tab(s, tabs[0]).unwrap();
    assert_eq!(
        store.folder(f).unwrap().tabs.len(),
        2,
        "the store cannot observe the close"
    );
    let removed = store.reconcile(&b);
    assert_eq!(removed, 1);
    assert_eq!(store.folder(f).unwrap().tabs, vec![tabs[1]]);
    store.check_invariants(&b).unwrap();
}

#[test]
fn reconcile_drops_memberships_for_moved_tabs() {
    let (mut b, s, tabs) = populated();
    let s2 = other_space(&mut b);
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.move_tab_into_folder(&b, f, tabs[0]).unwrap();
    store.move_tab_into_folder(&b, f, tabs[1]).unwrap();
    b.move_tab(tabs[0], s2).unwrap();
    let removed = store.reconcile(&b);
    assert_eq!(removed, 1);
    assert_eq!(store.folder(f).unwrap().tabs, vec![tabs[1]]);
    store.check_invariants(&b).unwrap();
}

#[test]
fn reconcile_never_removes_valid_members() {
    let (b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    for tab in &tabs {
        store.move_tab_into_folder(&b, f, *tab).unwrap();
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
    let (b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f1 = store.create_folder(&b, s, "One").unwrap();
    let f2 = store.create_folder(&b, s, "Two").unwrap();
    store.move_membership(f1, tabs[0], s).unwrap();
    store.move_membership(f2, tabs[0], s).unwrap();
    store.check_invariants(&b).unwrap();
    // Forge: push the same tab into a second folder behind the API's back.
    store.folders[0].tabs.push(tabs[0]);
    assert!(store.check_invariants(&b).is_err());
}

#[test]
fn invariants_detect_unknown_space_and_duplicate_favorite_tabs() {
    let (b, s, tabs) = populated();
    let mut store = FolderStore::new();
    store.create_folder(&b, s, "Work").unwrap();
    store.favorite_toggle(&b, tabs[0]).unwrap();
    // Forge an unknown-space folder and a duplicate favorite tab.
    store.folders[0].space = SpaceId(999);
    store
        .favorites
        .push(crate::folders::Favorite { tab: tabs[0] });
    assert!(store.check_invariants(&b).is_err());
}

#[test]
fn invariants_detect_stale_membership_without_reconcile() {
    let (mut b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.move_tab_into_folder(&b, f, tabs[0]).unwrap();
    b.close_tab(s, tabs[0]).unwrap();
    assert!(
        store.check_invariants(&b).is_err(),
        "checker is a detector: stale memberships must fail until reconciled"
    );
    store.reconcile(&b);
    store.check_invariants(&b).unwrap();
}

#[test]
fn store_is_a_plain_value() {
    let (b, s, tabs) = populated();
    let mut store = FolderStore::new();
    let f = store.create_folder(&b, s, "Work").unwrap();
    store.move_membership(f, tabs[0], s).unwrap();
    store.favorite_toggle(&b, tabs[1]).unwrap();
    assert_eq!(store.clone(), store);
}
