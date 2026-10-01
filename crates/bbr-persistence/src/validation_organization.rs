//! Schema-v5 write validation (task packet `closed-pins-and-favorites`):
//! the kept place of a pinned tab, in-place recently-closed records, and
//! the tab organization (folders and the favorites grid). Split out of
//! `validation.rs`, which calls these from its one snapshot pass. Restore
//! enforces the same rules (`repository_organization`).

use std::collections::{HashMap, HashSet};

use crate::validation::{valid_url, validate_id};
use crate::{InvalidSnapshotIssue, Profile, RecentlyClosedTab, Space, Tab};

/// A pinned page or the closed state belongs to a pinned tab only, and a
/// pinned page carries a valid URL.
pub(crate) fn validate_kept_place(tab: &Tab) -> Result<(), InvalidSnapshotIssue> {
    if (tab.pinned_page.is_some() || tab.closed) && !tab.pinned {
        return Err(InvalidSnapshotIssue::InvalidKeptPlace { tab_id: tab.id });
    }
    if tab
        .pinned_page
        .as_ref()
        .is_some_and(|page| !valid_url(&page.url))
    {
        return Err(InvalidSnapshotIssue::InvalidUrl { tab_id: tab.id });
    }
    Ok(())
}

/// An in-place record names a closed pinned tab of its own space, at most
/// once (the table's key keeps one record per tab id).
pub(crate) fn validate_in_place(
    space: &Space,
    closed: &RecentlyClosedTab,
) -> Result<(), InvalidSnapshotIssue> {
    let waiting = space
        .tabs
        .iter()
        .any(|tab| tab.id == closed.tab_id && tab.pinned && tab.closed);
    let records = space
        .recently_closed
        .iter()
        .filter(|record| record.tab_id == closed.tab_id)
        .count();
    if !waiting || records != 1 {
        return Err(InvalidSnapshotIssue::InvalidKeptPlace {
            tab_id: closed.tab_id,
        });
    }
    if closed.former_position > i64::MAX as u64 {
        return Err(InvalidSnapshotIssue::InvalidFormerPosition {
            tab_id: closed.tab_id,
        });
    }
    Ok(())
}

/// Folder ids are unique among folders; every member is a pinned tab of
/// the folder's space in at most one folder; every favorite is a pinned
/// tab, listed once and in no folder.
pub(crate) fn validate_organization(
    profiles: &[Profile],
    favorites: &[u64],
) -> Result<(), InvalidSnapshotIssue> {
    // Tab id -> (space id, pinned).
    let tabs: HashMap<u64, (u64, bool)> = profiles
        .iter()
        .flat_map(|profile| &profile.spaces)
        .flat_map(|space| {
            space
                .tabs
                .iter()
                .map(|tab| (tab.id, (space.id, tab.pinned)))
        })
        .collect();
    let mut folder_ids = HashSet::new();
    let mut members = HashSet::new();
    for space in profiles.iter().flat_map(|profile| &profile.spaces) {
        for folder in &space.folders {
            validate_id("folder", folder.id)?;
            if !folder_ids.insert(folder.id) {
                return Err(InvalidSnapshotIssue::InvalidFolder {
                    id: folder.id,
                    detail: "duplicate folder id".into(),
                });
            }
            for tab in &folder.tabs {
                if tabs.get(tab) != Some(&(space.id, true)) {
                    return Err(InvalidSnapshotIssue::InvalidFolder {
                        id: folder.id,
                        detail: format!("member {tab} is no pinned tab of space {}", space.id),
                    });
                }
                if !members.insert(*tab) {
                    return Err(InvalidSnapshotIssue::InvalidFolder {
                        id: folder.id,
                        detail: format!("member {tab} belongs to another folder"),
                    });
                }
            }
        }
    }
    let mut seen = HashSet::new();
    for tab in favorites {
        let pinned = tabs.get(tab).is_some_and(|(_, pinned)| *pinned);
        if !pinned || members.contains(tab) || !seen.insert(*tab) {
            return Err(InvalidSnapshotIssue::InvalidFavorite { tab_id: *tab });
        }
    }
    Ok(())
}
