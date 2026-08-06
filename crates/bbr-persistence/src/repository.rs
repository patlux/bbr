use std::collections::{HashMap, HashSet};

use rusqlite::{params, Connection, OpenFlags};

use crate::repository_load::{
    invalid_value, load_recently_closed, load_windows, persisted_id, project_soft_references,
    require_bool, require_id, require_order, require_owner_order, validate_nested_orders,
    validate_tab_orders_and_pins,
};
use crate::{
    schema, validation, CorruptionIssue, PersistenceError, Profile, ProfileHue, RecoveryWarning,
    Repository, RestoreOutcome, Snapshot, Space, Tab, WriteStats,
};

impl Repository {
    /// Opens or creates the database, applies the sole 0 -> 1 migration, then restores it.
    pub fn initialize_and_restore(&self) -> Result<RestoreOutcome, PersistenceError> {
        let mut connection = Connection::open(self.path()).map_err(schema::storage)?;
        schema::initialize_or_migrate(&mut connection)?;
        restore_from_connection(&connection)
    }

    /// Restores an existing current-schema database without migrating or writing it.
    pub fn restore_read_only(&self) -> Result<RestoreOutcome, PersistenceError> {
        let connection = Connection::open_with_flags(
            self.path(),
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(schema::storage)?;
        connection
            .pragma_update(None, "query_only", true)
            .map_err(schema::storage)?;
        schema::validate_current(&connection)?;
        restore_from_connection(&connection)
    }

    /// Validates and atomically replaces the complete structural snapshot.
    pub fn replace_snapshot(&self, snapshot: &Snapshot) -> Result<WriteStats, PersistenceError> {
        validation::validate_snapshot(snapshot).map_err(PersistenceError::InvalidSnapshot)?;
        let mut connection = Connection::open(self.path()).map_err(schema::storage)?;
        schema::initialize_or_migrate(&mut connection)?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(schema::storage)?;
        let transaction = connection.transaction().map_err(schema::storage)?;
        let deleted_rows = schema::deleted_rows(&transaction)?;
        let mut inserted_rows = 0_u64;

        for (profile_position, profile) in snapshot.profiles.iter().enumerate() {
            transaction
                .execute(
                    "INSERT INTO profiles(id, title, hue, position) VALUES (?, ?, ?, ?)",
                    params![
                        to_i64(profile.id),
                        profile.title,
                        profile.hue.as_str(),
                        profile_position as i64
                    ],
                )
                .map_err(schema::storage)?;
            inserted_rows += 1;
            for (space_position, space) in profile.spaces.iter().enumerate() {
                transaction.execute(
                    "INSERT INTO spaces(id, profile_id, title, position, active_tab_id) VALUES (?, ?, ?, ?, ?)",
                    params![to_i64(space.id), to_i64(profile.id), space.title, space_position as i64, space.active_tab.map(to_i64)],
                ).map_err(schema::storage)?;
                inserted_rows += 1;
                for (tab_position, tab) in space.tabs.iter().enumerate() {
                    transaction.execute(
                        "INSERT INTO tabs(id, space_id, title, url, position, pinned) VALUES (?, ?, ?, ?, ?, ?)",
                        params![to_i64(tab.id), to_i64(space.id), tab.title, tab.url, tab_position as i64, i64::from(tab.pinned)],
                    ).map_err(schema::storage)?;
                    inserted_rows += 1;
                }
                for (position, closed) in space.recently_closed.iter().enumerate() {
                    transaction.execute(
                        "INSERT INTO recently_closed(tab_id, space_id, title, url, pinned, former_position, position) VALUES (?, ?, ?, ?, ?, ?, ?)",
                        params![to_i64(closed.tab_id), to_i64(space.id), closed.title, closed.url, i64::from(closed.pinned), to_i64(closed.former_position), position as i64],
                    ).map_err(schema::storage)?;
                    inserted_rows += 1;
                }
                if let Some(split) = space.split {
                    transaction.execute(
                        "INSERT INTO splits(space_id, primary_tab_id, secondary_tab_id) VALUES (?, ?, ?)",
                        params![to_i64(space.id), to_i64(split.primary_tab), to_i64(split.secondary_tab)],
                    ).map_err(schema::storage)?;
                    inserted_rows += 1;
                }
            }
        }
        for (position, window) in snapshot.windows.iter().enumerate() {
            let (x, y, width, height) = match window.geometry {
                Some(geometry) => (
                    Some(geometry.x),
                    Some(geometry.y),
                    Some(geometry.width),
                    Some(geometry.height),
                ),
                None => (None, None, None, None),
            };
            transaction.execute(
                "INSERT INTO windows(id, position, x, y, width, height, sidebar_width, sidebar_collapsed, last_expanded_width) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![to_i64(window.id), position as i64, x, y, width, height, window.sidebar.width, i64::from(window.sidebar.collapsed), window.sidebar.last_expanded_width],
            ).map_err(schema::storage)?;
            inserted_rows += 1;
        }
        transaction.commit().map_err(schema::storage)?;
        Ok(WriteStats {
            committed_transactions: 1,
            sql_statements: 6 + inserted_rows,
            deleted_rows,
            inserted_rows,
        })
    }
}

const fn to_i64(value: u64) -> i64 {
    value as i64
}

fn restore_from_connection(connection: &Connection) -> Result<RestoreOutcome, PersistenceError> {
    schema::validate_schema(connection)?;
    schema::check_foreign_keys(connection)?;
    let mut warnings = Vec::new();
    let mut identities = HashSet::new();
    let mut profiles = load_profiles(connection, &mut identities)?;
    let profile_indexes = profiles
        .iter()
        .enumerate()
        .map(|(index, profile)| (profile.id, index))
        .collect::<HashMap<_, _>>();
    let spaces = load_spaces(connection, &mut identities)?;
    let mut space_locations = HashMap::new();
    for (space, profile_id) in spaces {
        let profile_index = profile_indexes.get(&profile_id).copied().ok_or_else(|| {
            PersistenceError::Corruption(CorruptionIssue::InvalidRequiredValue {
                table: "spaces".into(),
                detail: format!("space {} has missing profile {profile_id}", space.id),
            })
        })?;
        let space_index = profiles[profile_index].spaces.len();
        space_locations.insert(space.id, (profile_index, space_index));
        profiles[profile_index].spaces.push(space);
    }
    validate_nested_orders(&profiles)?;

    let tabs = load_tabs(connection, &mut identities)?;
    let mut live_tabs = HashSet::new();
    for (tab, space_id) in tabs {
        let &(profile_index, space_index) = space_locations.get(&space_id).ok_or_else(|| {
            PersistenceError::Corruption(CorruptionIssue::InvalidRequiredValue {
                table: "tabs".into(),
                detail: format!("tab {} has missing space {space_id}", tab.id),
            })
        })?;
        live_tabs.insert(tab.id);
        profiles[profile_index].spaces[space_index].tabs.push(tab);
    }
    validate_tab_orders_and_pins(&profiles)?;
    load_recently_closed(
        connection,
        &space_locations,
        &mut profiles,
        &live_tabs,
        &mut identities,
    )?;
    project_soft_references(connection, &space_locations, &mut profiles, &mut warnings)?;
    let windows = load_windows(connection, &mut identities, &mut warnings)?;
    warnings.sort_by_key(RecoveryWarning::sort_key);
    Ok(RestoreOutcome {
        snapshot: Snapshot { profiles, windows },
        warnings,
    })
}

fn load_profiles(
    connection: &Connection,
    identities: &mut HashSet<u64>,
) -> Result<Vec<Profile>, PersistenceError> {
    let mut statement = connection
        .prepare("SELECT id, title, hue, position FROM profiles ORDER BY position")
        .map_err(schema::storage)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(schema::storage)?;
    let mut result = Vec::new();
    for (expected, row) in rows.enumerate() {
        let (id, title, hue, position) = row.map_err(schema::storage)?;
        require_order("profiles", expected, position)?;
        let id = require_id("profiles", id, identities)?;
        let hue = ProfileHue::parse(&hue)
            .ok_or_else(|| invalid_value("profiles", format!("profile {id} has invalid hue")))?;
        result.push(Profile {
            id,
            title,
            hue,
            spaces: Vec::new(),
        });
    }
    Ok(result)
}

fn load_spaces(
    connection: &Connection,
    identities: &mut HashSet<u64>,
) -> Result<Vec<(Space, u64)>, PersistenceError> {
    let mut statement = connection.prepare(
        "SELECT id, profile_id, title, position, active_tab_id FROM spaces ORDER BY profile_id, position",
    ).map_err(schema::storage)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, Option<i64>>(4)?,
            ))
        })
        .map_err(schema::storage)?;
    let mut result = Vec::new();
    let mut owner_positions = HashMap::new();
    for row in rows {
        let (id, profile_id, title, position, active_tab) = row.map_err(schema::storage)?;
        let profile_id = persisted_id("spaces", profile_id)?;
        require_owner_order("spaces", profile_id, position, &mut owner_positions)?;
        let id = require_id("spaces", id, identities)?;
        let active_tab = active_tab
            .map(|value| persisted_id("spaces", value))
            .transpose()?;
        result.push((
            Space {
                id,
                title,
                tabs: Vec::new(),
                active_tab,
                recently_closed: Vec::new(),
                split: None,
            },
            profile_id,
        ));
    }
    Ok(result)
}

fn load_tabs(
    connection: &Connection,
    identities: &mut HashSet<u64>,
) -> Result<Vec<(Tab, u64)>, PersistenceError> {
    let mut statement = connection.prepare(
        "SELECT id, space_id, title, url, position, pinned FROM tabs ORDER BY space_id, position",
    ).map_err(schema::storage)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })
        .map_err(schema::storage)?;
    let mut result = Vec::new();
    let mut owner_positions = HashMap::new();
    for row in rows {
        let (id, space_id, title, url, position, pinned) = row.map_err(schema::storage)?;
        let space_id = persisted_id("tabs", space_id)?;
        require_owner_order("tabs", space_id, position, &mut owner_positions)?;
        let id = require_id("tabs", id, identities)?;
        if !validation::valid_url(&url) {
            return Err(invalid_value("tabs", format!("tab {id} has invalid URL")));
        }
        let pinned = require_bool("tabs", pinned)?;
        result.push((
            Tab {
                id,
                title,
                url,
                pinned,
            },
            space_id,
        ));
    }
    Ok(result)
}
