use std::collections::{HashMap, HashSet};

use rusqlite::{Connection, OptionalExtension};

use crate::{
    schema, validation, CorruptionIssue, PersistenceError, Profile, RecentlyClosedTab,
    RecoveryWarning, SidebarState, Split, WindowGeometry, WindowState,
};

pub(crate) fn validate_nested_orders(_profiles: &[Profile]) -> Result<(), PersistenceError> {
    Ok(())
}

pub(crate) fn validate_tab_orders_and_pins(profiles: &[Profile]) -> Result<(), PersistenceError> {
    for profile in profiles {
        for space in &profile.spaces {
            let mut saw_regular = false;
            for tab in &space.tabs {
                if tab.pinned && saw_regular {
                    return Err(PersistenceError::Corruption(
                        CorruptionIssue::InvalidPinnedRegion { space_id: space.id },
                    ));
                }
                saw_regular |= !tab.pinned;
            }
        }
    }
    Ok(())
}

pub(crate) fn load_recently_closed(
    connection: &Connection,
    space_locations: &HashMap<u64, (usize, usize)>,
    profiles: &mut [Profile],
    live_tabs: &HashSet<u64>,
    identities: &mut HashSet<u64>,
) -> Result<(), PersistenceError> {
    let mut statement = connection.prepare(
        "SELECT tab_id, space_id, title, url, pinned, former_position, position FROM recently_closed ORDER BY space_id, position",
    ).map_err(schema::storage)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
            ))
        })
        .map_err(schema::storage)?;
    let mut closed_ids = HashSet::new();
    let mut owner_positions = HashMap::new();
    for row in rows {
        let (tab_id, space_id, title, url, pinned, former_position, position) =
            row.map_err(schema::storage)?;
        let tab_id = persisted_id("recently_closed", tab_id)?;
        let space_id = persisted_id("recently_closed", space_id)?;
        require_owner_order("recently_closed", space_id, position, &mut owner_positions)?;
        let &(profile_index, space_index) = space_locations
            .get(&space_id)
            .ok_or_else(|| invalid_closed(tab_id, "missing owner"))?;
        if identities.contains(&tab_id) || live_tabs.contains(&tab_id) || !closed_ids.insert(tab_id)
        {
            return Err(invalid_closed(
                tab_id,
                "identity collides with live or closed state",
            ));
        }
        identities.insert(tab_id);
        if !validation::valid_url(&url) {
            return Err(invalid_closed(tab_id, "invalid URL"));
        }
        let pinned = require_bool("recently_closed", pinned)
            .map_err(|_| invalid_closed(tab_id, "invalid pinned value"))?;
        if former_position < 0 {
            return Err(invalid_closed(tab_id, "negative former position"));
        }
        profiles[profile_index].spaces[space_index]
            .recently_closed
            .push(RecentlyClosedTab {
                tab_id,
                title: title.unwrap_or_default(),
                url,
                pinned,
                former_position: former_position as u64,
            });
    }
    Ok(())
}

pub(crate) fn project_soft_references(
    connection: &Connection,
    space_locations: &HashMap<u64, (usize, usize)>,
    profiles: &mut [Profile],
    warnings: &mut Vec<RecoveryWarning>,
) -> Result<(), PersistenceError> {
    for &(profile_index, space_index) in space_locations.values() {
        let space = &mut profiles[profile_index].spaces[space_index];
        let members = space.tabs.iter().map(|tab| tab.id).collect::<HashSet<_>>();
        if let Some(tab_id) = space.active_tab {
            if !members.contains(&tab_id) {
                space.active_tab = None;
                warnings.push(RecoveryWarning::ActiveTab {
                    space_id: space.id,
                    persisted_tab_id: tab_id,
                });
            }
        }
        let persisted_split = connection
            .query_row(
                "SELECT primary_tab_id, secondary_tab_id FROM splits WHERE space_id = ?",
                [space.id as i64],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()
            .map_err(schema::storage)?;
        if let Some((primary, secondary)) = persisted_split {
            let primary = persisted_id("splits", primary)?;
            let secondary = persisted_id("splits", secondary)?;
            if primary == secondary || !members.contains(&primary) || !members.contains(&secondary)
            {
                warnings.push(RecoveryWarning::Split {
                    space_id: space.id,
                    primary_tab_id: primary,
                    secondary_tab_id: secondary,
                });
            } else {
                space.split = Some(Split {
                    primary_tab: primary,
                    secondary_tab: secondary,
                });
            }
        }
    }
    let mut statement = connection
        .prepare("SELECT space_id FROM splits ORDER BY space_id")
        .map_err(schema::storage)?;
    let rows = statement
        .query_map([], |row| row.get::<_, i64>(0))
        .map_err(schema::storage)?;
    for row in rows {
        let space_id = persisted_id("splits", row.map_err(schema::storage)?)?;
        if !space_locations.contains_key(&space_id) {
            let (primary, secondary) = connection
                .query_row(
                    "SELECT primary_tab_id, secondary_tab_id FROM splits WHERE space_id = ?",
                    [space_id as i64],
                    |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
                )
                .map_err(schema::storage)?;
            warnings.push(RecoveryWarning::Split {
                space_id,
                primary_tab_id: persisted_id("splits", primary)?,
                secondary_tab_id: persisted_id("splits", secondary)?,
            });
        }
    }
    Ok(())
}

pub(crate) fn load_windows(
    connection: &Connection,
    identities: &mut HashSet<u64>,
    warnings: &mut Vec<RecoveryWarning>,
) -> Result<Vec<WindowState>, PersistenceError> {
    let mut statement = connection.prepare(
        "SELECT id, position, x, y, width, height, sidebar_width, sidebar_collapsed, last_expanded_width FROM windows ORDER BY position",
    ).map_err(schema::storage)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<f64>>(2)?,
                row.get::<_, Option<f64>>(3)?,
                row.get::<_, Option<f64>>(4)?,
                row.get::<_, Option<f64>>(5)?,
                row.get::<_, f64>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, f64>(8)?,
            ))
        })
        .map_err(schema::storage)?;
    let mut result = Vec::new();
    for (expected, row) in rows.enumerate() {
        let (id, position, x, y, width, height, sidebar_width, collapsed, last_width) =
            row.map_err(schema::storage)?;
        require_order("windows", expected, position)?;
        let id = require_id("windows", id, identities)?;
        let geometry = match (x, y, width, height) {
            (None, None, None, None) => None,
            (Some(x), Some(y), Some(width), Some(height)) => {
                let value = WindowGeometry {
                    x,
                    y,
                    width,
                    height,
                };
                if validation::valid_geometry(value) {
                    Some(value)
                } else {
                    warnings.push(RecoveryWarning::WindowGeometry { window_id: id });
                    None
                }
            }
            _ => {
                warnings.push(RecoveryWarning::WindowGeometry { window_id: id });
                None
            }
        };
        let sidebar = if (collapsed == 0 || collapsed == 1)
            && validation::valid_sidebar(sidebar_width, last_width)
        {
            SidebarState {
                width: sidebar_width,
                collapsed: collapsed == 1,
                last_expanded_width: last_width,
            }
        } else {
            warnings.push(RecoveryWarning::Sidebar { window_id: id });
            SidebarState::default()
        };
        result.push(WindowState {
            id,
            geometry,
            sidebar,
        });
    }
    Ok(result)
}

pub(crate) fn require_order(
    owner: &str,
    expected: usize,
    actual: i64,
) -> Result<(), PersistenceError> {
    if actual == expected as i64 {
        Ok(())
    } else {
        Err(PersistenceError::Corruption(
            CorruptionIssue::InvalidOrder {
                owner: owner.to_owned(),
            },
        ))
    }
}

pub(crate) fn require_owner_order(
    table: &str,
    owner: u64,
    actual: i64,
    positions: &mut HashMap<u64, usize>,
) -> Result<(), PersistenceError> {
    let expected = positions.entry(owner).or_default();
    require_order(&format!("{table} owner {owner}"), *expected, actual)?;
    *expected += 1;
    Ok(())
}

pub(crate) fn require_id(
    table: &str,
    value: i64,
    identities: &mut HashSet<u64>,
) -> Result<u64, PersistenceError> {
    let id = persisted_id(table, value)?;
    if identities.insert(id) {
        Ok(id)
    } else {
        Err(PersistenceError::Corruption(
            CorruptionIssue::DuplicateIdentity { id },
        ))
    }
}

pub(crate) fn persisted_id(table: &str, value: i64) -> Result<u64, PersistenceError> {
    if validation::valid_persisted_id(value) {
        Ok(value as u64)
    } else {
        Err(invalid_value(table, format!("invalid identity {value}")))
    }
}

pub(crate) fn require_bool(table: &str, value: i64) -> Result<bool, PersistenceError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(invalid_value(table, format!("invalid boolean {value}"))),
    }
}

pub(crate) fn invalid_value(table: &str, detail: String) -> PersistenceError {
    PersistenceError::Corruption(CorruptionIssue::InvalidRequiredValue {
        table: table.to_owned(),
        detail,
    })
}

fn invalid_closed(tab_id: u64, detail: &str) -> PersistenceError {
    PersistenceError::Corruption(CorruptionIssue::InvalidRecentlyClosed {
        tab_id,
        detail: detail.to_owned(),
    })
}
