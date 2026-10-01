//! Schema-v5 tab organization restore (task packet
//! `closed-pins-and-favorites`): folders per space with their members, and
//! the window-global favorites grid. Enforces the write contract
//! (`validation_organization`): folder ids unique among folders, dense
//! per-owner order keys, members are pinned tabs of the folder's space in
//! at most one folder, favorites are pinned tabs listed once and in no
//! folder. A violation is corruption (the store is discarded whole), never
//! a partial organization.

use std::collections::{HashMap, HashSet};

use rusqlite::Connection;

use crate::load_helpers::{invalid_value, persisted_id, require_order, require_owner_order};
use crate::{schema, Folder, PersistenceError, Profile};

/// Load the folders into their spaces and return the favorites grid.
pub(crate) fn load_organization(
    connection: &Connection,
    space_locations: &HashMap<u64, (usize, usize)>,
    profiles: &mut [Profile],
) -> Result<Vec<u64>, PersistenceError> {
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
    let folder_spaces = load_folders(connection, space_locations, profiles)?;
    let members = load_members(connection, space_locations, profiles, &folder_spaces, &tabs)?;
    load_favorites(connection, &tabs, &members)
}

/// Folders in per-space order; returns folder id -> space id.
fn load_folders(
    connection: &Connection,
    space_locations: &HashMap<u64, (usize, usize)>,
    profiles: &mut [Profile],
) -> Result<HashMap<u64, u64>, PersistenceError> {
    let mut statement = connection
        .prepare("SELECT id, space_id, title, position FROM folders ORDER BY space_id, position")
        .map_err(schema::storage)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(schema::storage)?;
    let mut folder_spaces = HashMap::new();
    let mut owner_positions = HashMap::new();
    for row in rows {
        let (id, space_id, title, position) = row.map_err(schema::storage)?;
        let id = persisted_id("folders", id)?;
        let space_id = persisted_id("folders", space_id)?;
        require_owner_order("folders", space_id, position, &mut owner_positions)?;
        let &(profile_index, space_index) = space_locations
            .get(&space_id)
            .ok_or_else(|| invalid_value("folders", format!("folder {id} has no space")))?;
        folder_spaces.insert(id, space_id);
        profiles[profile_index].spaces[space_index]
            .folders
            .push(Folder {
                id,
                title,
                tabs: Vec::new(),
            });
    }
    Ok(folder_spaces)
}

/// Folder members in per-folder order; returns every member tab id.
fn load_members(
    connection: &Connection,
    space_locations: &HashMap<u64, (usize, usize)>,
    profiles: &mut [Profile],
    folder_spaces: &HashMap<u64, u64>,
    tabs: &HashMap<u64, (u64, bool)>,
) -> Result<HashSet<u64>, PersistenceError> {
    let mut statement = connection
        .prepare(
            "SELECT tab_id, folder_id, position FROM folder_members ORDER BY folder_id, position",
        )
        .map_err(schema::storage)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(schema::storage)?;
    let mut members = HashSet::new();
    let mut owner_positions = HashMap::new();
    for row in rows {
        let (tab_id, folder_id, position) = row.map_err(schema::storage)?;
        let tab_id = persisted_id("folder_members", tab_id)?;
        let folder_id = persisted_id("folder_members", folder_id)?;
        require_owner_order("folder_members", folder_id, position, &mut owner_positions)?;
        let space_id = *folder_spaces.get(&folder_id).ok_or_else(|| {
            invalid_value("folder_members", format!("member {tab_id} has no folder"))
        })?;
        if tabs.get(&tab_id) != Some(&(space_id, true)) || !members.insert(tab_id) {
            return Err(invalid_value(
                "folder_members",
                format!("member {tab_id} is no pinned tab of space {space_id}"),
            ));
        }
        let (profile_index, space_index) = space_locations[&space_id];
        profiles[profile_index].spaces[space_index]
            .folders
            .iter_mut()
            .find(|folder| folder.id == folder_id)
            .expect("the folder was loaded into its space")
            .tabs
            .push(tab_id);
    }
    Ok(members)
}

/// The favorites grid in order: pinned tabs, once, in no folder.
fn load_favorites(
    connection: &Connection,
    tabs: &HashMap<u64, (u64, bool)>,
    members: &HashSet<u64>,
) -> Result<Vec<u64>, PersistenceError> {
    let mut statement = connection
        .prepare("SELECT tab_id, position FROM favorites ORDER BY position")
        .map_err(schema::storage)?;
    let rows = statement
        .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
        .map_err(schema::storage)?;
    let mut favorites = Vec::new();
    for (expected, row) in rows.enumerate() {
        let (tab_id, position) = row.map_err(schema::storage)?;
        require_order("favorites", expected, position)?;
        let tab_id = persisted_id("favorites", tab_id)?;
        let pinned = tabs.get(&tab_id).is_some_and(|(_, pinned)| *pinned);
        if !pinned || members.contains(&tab_id) {
            return Err(invalid_value(
                "favorites",
                format!("favorite {tab_id} is no loose pinned tab"),
            ));
        }
        favorites.push(tab_id);
    }
    Ok(favorites)
}
