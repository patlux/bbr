use std::collections::{HashMap, HashSet};

use rusqlite::{Connection, OptionalExtension};

use crate::load_helpers::{
    invalid_closed, invalid_value, persisted_id, require_bool, require_id, require_order,
    require_owner_order,
};
use crate::{
    schema, validation, CommandHistoryEntry, CorruptionIssue, DownloadOutcome, DownloadRecord,
    PersistenceError, Profile, RecentlyClosedTab, RecoveryWarning, SearchEngineId, SearchSettings,
    SidebarState, Split, WindowGeometry, WindowState,
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

/// Loads the single global settings row, or falls back to defaults with a
/// typed warning when the persisted record is incoherent.
///
/// The row itself is required structural data (exactly one row with id 1);
/// an unknown engine id is fatal. The custom search template is optional
/// user text: an invalid template under the custom engine, or a template
/// without the custom engine, recovers to defaults rather than corrupting
/// the load.
pub(crate) fn load_settings(
    connection: &Connection,
    warnings: &mut Vec<RecoveryWarning>,
) -> Result<SearchSettings, PersistenceError> {
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM settings", [], |row| row.get(0))
        .map_err(schema::storage)?;
    if count != 1 {
        return Err(invalid_value(
            "settings",
            format!("expected exactly one settings row, found {count}"),
        ));
    }
    let row = connection
        .query_row(
            "SELECT id, search_engine, custom_search_template FROM settings ORDER BY id",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .map_err(schema::storage)?;
    let (id, engine, template) = row;
    if id != 1 {
        return Err(invalid_value(
            "settings",
            format!("settings row id must be 1, found {id}"),
        ));
    }
    let engine = SearchEngineId::parse(&engine)
        .ok_or_else(|| invalid_value("settings", format!("unknown search engine `{engine}`")))?;
    let mut settings = SearchSettings {
        engine,
        custom_template: template,
    };
    match (settings.engine, settings.custom_template.as_deref()) {
        (SearchEngineId::Custom, None) => {
            warnings.push(RecoveryWarning::Settings {
                detail: "custom engine without a search template".into(),
            });
            settings = SearchSettings::default();
        }
        (SearchEngineId::Custom, Some(template))
            if !validation::valid_custom_template(template) =>
        {
            warnings.push(RecoveryWarning::Settings {
                detail: "invalid custom search template".into(),
            });
            settings = SearchSettings::default();
        }
        (engine, Some(_template)) if engine != SearchEngineId::Custom => {
            warnings.push(RecoveryWarning::Settings {
                detail: "search template without the custom engine".into(),
            });
            settings.custom_template = None;
        }
        _ => {}
    }
    Ok(settings)
}

/// Loads terminal download history in persisted order.
pub(crate) fn load_downloads(
    connection: &Connection,
    identities: &mut HashSet<u64>,
) -> Result<Vec<DownloadRecord>, PersistenceError> {
    let mut statement = connection.prepare(
        "SELECT id, outcome, size, destination, terminal_at, position FROM downloads ORDER BY position",
    ).map_err(schema::storage)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })
        .map_err(schema::storage)?;
    let mut result = Vec::new();
    for (expected, row) in rows.enumerate() {
        let (id, outcome, size, destination, terminal_at, position) =
            row.map_err(schema::storage)?;
        require_order("downloads", expected, position)?;
        let id = require_id("downloads", id, identities)?;
        let outcome = DownloadOutcome::parse(&outcome).ok_or_else(|| {
            invalid_value(
                "downloads",
                format!("attempt {id} has unknown outcome `{outcome}`"),
            )
        })?;
        if size < 0 {
            return Err(invalid_value(
                "downloads",
                format!("attempt {id} has negative size"),
            ));
        }
        if terminal_at < 0 {
            return Err(invalid_value(
                "downloads",
                format!("attempt {id} has negative terminal timestamp"),
            ));
        }
        if let Some(destination) = &destination {
            if !validation::valid_path(destination) {
                return Err(invalid_value(
                    "downloads",
                    format!("attempt {id} has an invalid destination path"),
                ));
            }
        }
        result.push(DownloadRecord {
            id,
            outcome,
            size: size as u64,
            destination,
            terminal_at: terminal_at as u64,
        });
    }
    Ok(result)
}

/// Loads visited command-history entries in persisted corpus order.
pub(crate) fn load_command_history(
    connection: &Connection,
    identities: &mut HashSet<u64>,
) -> Result<Vec<CommandHistoryEntry>, PersistenceError> {
    let mut statement = connection.prepare(
        "SELECT id, title, url, visit_count, last_visited, position FROM command_history ORDER BY position",
    ).map_err(schema::storage)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })
        .map_err(schema::storage)?;
    let mut result = Vec::new();
    for (expected, row) in rows.enumerate() {
        let (id, title, url, visit_count, last_visited, position) = row.map_err(schema::storage)?;
        require_order("command_history", expected, position)?;
        let id = require_id("command_history", id, identities)?;
        if !validation::valid_url(&url) {
            return Err(invalid_value(
                "command_history",
                format!("entry {id} has an invalid URL"),
            ));
        }
        if visit_count < 0 {
            return Err(invalid_value(
                "command_history",
                format!("entry {id} has a negative visit count"),
            ));
        }
        if last_visited < 0 {
            return Err(invalid_value(
                "command_history",
                format!("entry {id} has a negative recency marker"),
            ));
        }
        result.push(CommandHistoryEntry {
            id,
            title,
            url,
            visit_count: visit_count as u64,
            last_visited: last_visited as u64,
        });
    }
    Ok(result)
}
