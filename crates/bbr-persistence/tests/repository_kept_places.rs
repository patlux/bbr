//! Schema-v5 contract tests (task packet `closed-pins-and-favorites`): the
//! kept places of pinned tabs (pinned page, closed in place, in-place
//! recently-closed records) and the tab organization (folders, favorites)
//! round-trip through both browser writes, invalid organization or
//! kept-place state rejects whole before any write, and a store that breaks
//! the same rules restores as corruption (or, for a soft reference, with a
//! typed warning).

mod support;

use bbr_persistence::{
    BrowserState, CorruptionIssue, Folder, InvalidSnapshotIssue, PersistenceError, PinnedPage,
    RecentlyClosedTab, RecoveryWarning, Snapshot,
};

use support::{sample_snapshot, TempDb};

/// The sample snapshot (tabs 100/101 pinned, 102 regular, split 100/101,
/// active 100) reshaped: 101 is a closed pinned row (no longer a split
/// pane) with its in-place record, 100 keeps a pinned page it navigated
/// away from, a folder groups 101, and 100 is a favorite.
fn kept_places() -> Snapshot {
    let mut snapshot = sample_snapshot(3);
    let space = &mut snapshot.profiles[0].spaces[0];
    space.split = None;
    space.tabs[0].pinned_page = Some(PinnedPage {
        url: "https://example.test/home".into(),
        title: "Home".into(),
    });
    space.tabs[1].pinned_page = Some(PinnedPage {
        url: "https://example.test/1".into(),
        title: "Tab 1".into(),
    });
    space.tabs[1].closed = true;
    space.recently_closed.push(RecentlyClosedTab {
        tab_id: 101,
        title: "Deep page".into(),
        url: "https://example.test/1/deep".into(),
        pinned: true,
        former_position: 1,
        in_place: true,
    });
    space.folders = vec![Folder {
        id: 7,
        title: "Work".into(),
        tabs: vec![101],
    }];
    snapshot.favorites = vec![100];
    snapshot
}

fn issue(snapshot: &Snapshot) -> InvalidSnapshotIssue {
    let db = TempDb::new("kept-places-invalid");
    match db.repository().replace_snapshot(snapshot) {
        Err(PersistenceError::InvalidSnapshot(issue)) => issue,
        other => panic!("expected an invalid snapshot, got {other:?}"),
    }
}

#[test]
fn kept_places_and_organization_round_trip_through_both_writes() {
    let db = TempDb::new("kept-places-round-trip");
    let snapshot = kept_places();
    db.repository().replace_snapshot(&snapshot).unwrap();
    assert_eq!(
        db.repository().restore_read_only().unwrap().snapshot,
        snapshot
    );
    // The focused browser-state write carries the same rows.
    let state = BrowserState {
        profiles: snapshot.profiles.clone(),
        windows: snapshot.windows.clone(),
        favorites: snapshot.favorites.clone(),
    };
    db.repository().store_browser_state(&state).unwrap();
    let restored = db.repository().restore_read_only().unwrap().snapshot;
    assert_eq!(restored.profiles, snapshot.profiles);
    assert_eq!(restored.favorites, snapshot.favorites);
    // An empty organization replaces the previous one.
    let mut cleared = state.clone();
    cleared.favorites.clear();
    cleared.profiles[0].spaces[0].folders.clear();
    db.repository().store_browser_state(&cleared).unwrap();
    let restored = db.repository().restore_read_only().unwrap().snapshot;
    assert!(restored.favorites.is_empty());
    assert!(restored.profiles[0].spaces[0].folders.is_empty());
}

#[test]
fn invalid_kept_places_reject_whole() {
    // A closed row is never the selection.
    let mut shown = kept_places();
    shown.profiles[0].spaces[0].active_tab = Some(101);
    assert_eq!(
        issue(&shown),
        InvalidSnapshotIssue::InvalidActiveTab {
            space_id: 10,
            tab_id: 101
        }
    );
    // Only a pinned tab keeps a place.
    let mut regular = kept_places();
    regular.profiles[0].spaces[0].tabs[2].closed = true;
    assert_eq!(
        issue(&regular),
        InvalidSnapshotIssue::InvalidKeptPlace { tab_id: 102 }
    );
    // An in-place record names a closed row of its space.
    let mut stray = kept_places();
    stray.profiles[0].spaces[0].recently_closed[1].tab_id = 100;
    assert_eq!(
        issue(&stray),
        InvalidSnapshotIssue::InvalidKeptPlace { tab_id: 100 }
    );
}

#[test]
fn invalid_organization_rejects_whole() {
    // A folder member is a pinned tab of the folder's space.
    let mut regular_member = kept_places();
    regular_member.profiles[0].spaces[0].folders[0].tabs = vec![102];
    assert!(matches!(
        issue(&regular_member),
        InvalidSnapshotIssue::InvalidFolder { id: 7, .. }
    ));
    // Folder ids are unique.
    let mut duplicate = kept_places();
    let folder = duplicate.profiles[0].spaces[0].folders[0].clone();
    duplicate.profiles[0].spaces[0].folders.push(Folder {
        tabs: Vec::new(),
        ..folder
    });
    assert!(matches!(
        issue(&duplicate),
        InvalidSnapshotIssue::InvalidFolder { id: 7, .. }
    ));
    // A favorite is a loose pinned tab, listed once.
    for favorites in [vec![102], vec![101], vec![100, 100], vec![999]] {
        let mut invalid = kept_places();
        invalid.favorites = favorites.clone();
        assert!(
            matches!(
                issue(&invalid),
                InvalidSnapshotIssue::InvalidFavorite { .. }
            ),
            "{favorites:?}"
        );
    }
}

#[test]
fn a_store_breaking_the_organization_restores_as_corruption() {
    let db = TempDb::new("kept-places-corrupt");
    db.repository().replace_snapshot(&kept_places()).unwrap();
    let connection = rusqlite::Connection::open(&db.0).unwrap();
    // A favorite naming the regular tab.
    connection
        .execute("UPDATE favorites SET tab_id = 102", [])
        .unwrap();
    drop(connection);
    assert!(matches!(
        db.repository().restore_read_only(),
        Err(PersistenceError::Corruption(
            CorruptionIssue::InvalidRequiredValue { .. }
        ))
    ));
}

#[test]
fn a_store_selecting_a_closed_row_recovers_the_selection() {
    let db = TempDb::new("kept-places-soft");
    db.repository().replace_snapshot(&kept_places()).unwrap();
    let connection = rusqlite::Connection::open(&db.0).unwrap();
    connection
        .execute("UPDATE spaces SET active_tab_id = 101", [])
        .unwrap();
    drop(connection);
    let outcome = db.repository().restore_read_only().unwrap();
    assert_eq!(outcome.snapshot.profiles[0].spaces[0].active_tab, None);
    assert_eq!(
        outcome.warnings,
        vec![RecoveryWarning::ActiveTab {
            space_id: 10,
            persisted_tab_id: 101
        }]
    );
}
