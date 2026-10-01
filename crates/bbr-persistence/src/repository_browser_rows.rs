//! The browser-table row writes shared by the complete-snapshot write
//! ([`crate::Repository::replace_snapshot`]) and the focused browser-state
//! write ([`crate::Repository::store_browser_state`]): profiles, spaces,
//! tabs (with their schema-v5 kept places), recently-closed entries,
//! splits, folders with their members, windows and the favorites grid. One
//! INSERT per row with dense per-owner position keys preserving the
//! caller's order; the caller validated the rows and owns the transaction.

use rusqlite::{params, Transaction};

use crate::repository::to_i64;
use crate::{schema, PersistenceError, Profile, SpaceGlyph, WindowState};

/// The browser tables in child-first DELETE order (the write paths enable
/// foreign keys): the organization tables reference tabs and spaces.
pub(crate) const BROWSER_DELETE_TABLES: &[&str] = &[
    "favorites",
    "folder_members",
    "folders",
    "recently_closed",
    "splits",
    "tabs",
    "spaces",
    "profiles",
    "windows",
];

/// Insert every browser row; returns the number of inserted rows.
pub(crate) fn insert_browser_rows(
    transaction: &Transaction<'_>,
    profiles: &[Profile],
    windows: &[WindowState],
    favorites: &[u64],
) -> Result<u64, PersistenceError> {
    let mut inserted_rows = 0_u64;
    let mut insert = |sql: &str, values: &[&dyn rusqlite::ToSql]| {
        transaction.execute(sql, values).map_err(schema::storage)?;
        inserted_rows += 1;
        Ok::<_, PersistenceError>(())
    };
    for (profile_position, profile) in profiles.iter().enumerate() {
        insert(
            "INSERT INTO profiles(id, title, hue, position) VALUES (?, ?, ?, ?)",
            params![
                to_i64(profile.id),
                profile.title,
                profile.hue.as_str(),
                profile_position as i64
            ],
        )?;
        for (space_position, space) in profile.spaces.iter().enumerate() {
            insert(
                "INSERT INTO spaces(id, profile_id, title, tint, glyph_kind, glyph, position, active_tab_id, tint_intensity, grain) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![to_i64(space.id), to_i64(profile.id), space.title, space.tint.as_str(), glyph_kind(&space.glyph), glyph_value(&space.glyph), space_position as i64, space.active_tab.map(to_i64), space.paper.intensity.as_str(), space.paper.grain],
            )?;
            for (tab_position, tab) in space.tabs.iter().enumerate() {
                let page = tab.pinned_page.as_ref();
                insert(
                    "INSERT INTO tabs(id, space_id, title, url, position, pinned, pinned_url, pinned_title, closed) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![to_i64(tab.id), to_i64(space.id), tab.title, tab.url, tab_position as i64, i64::from(tab.pinned), page.map(|page| &page.url), page.map(|page| &page.title), i64::from(tab.closed)],
                )?;
            }
            for (position, closed) in space.recently_closed.iter().enumerate() {
                insert(
                    "INSERT INTO recently_closed(tab_id, space_id, title, url, pinned, former_position, position, in_place) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                    params![to_i64(closed.tab_id), to_i64(space.id), closed.title, closed.url, i64::from(closed.pinned), to_i64(closed.former_position), position as i64, i64::from(closed.in_place)],
                )?;
            }
            if let Some(split) = space.split {
                insert(
                    "INSERT INTO splits(space_id, primary_tab_id, secondary_tab_id) VALUES (?, ?, ?)",
                    params![to_i64(space.id), to_i64(split.primary_tab), to_i64(split.secondary_tab)],
                )?;
            }
            for (folder_position, folder) in space.folders.iter().enumerate() {
                insert(
                    "INSERT INTO folders(id, space_id, title, position) VALUES (?, ?, ?, ?)",
                    params![
                        to_i64(folder.id),
                        to_i64(space.id),
                        folder.title,
                        folder_position as i64
                    ],
                )?;
                for (member_position, tab) in folder.tabs.iter().enumerate() {
                    insert(
                        "INSERT INTO folder_members(tab_id, folder_id, position) VALUES (?, ?, ?)",
                        params![to_i64(*tab), to_i64(folder.id), member_position as i64],
                    )?;
                }
            }
        }
    }
    for (position, window) in windows.iter().enumerate() {
        let (x, y, width, height) = match window.geometry {
            Some(geometry) => (
                Some(geometry.x),
                Some(geometry.y),
                Some(geometry.width),
                Some(geometry.height),
            ),
            None => (None, None, None, None),
        };
        insert(
            "INSERT INTO windows(id, position, x, y, width, height, sidebar_width, sidebar_collapsed, last_expanded_width) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![to_i64(window.id), position as i64, x, y, width, height, window.sidebar.width, i64::from(window.sidebar.collapsed), window.sidebar.last_expanded_width],
        )?;
    }
    for (position, tab) in favorites.iter().enumerate() {
        insert(
            "INSERT INTO favorites(tab_id, position) VALUES (?, ?)",
            params![to_i64(*tab), position as i64],
        )?;
    }
    Ok(inserted_rows)
}

fn glyph_kind(glyph: &SpaceGlyph) -> &'static str {
    match glyph {
        SpaceGlyph::Emoji(_) => "emoji",
        SpaceGlyph::Glyph(_) => "glyph",
    }
}

fn glyph_value(glyph: &SpaceGlyph) -> &str {
    match glyph {
        SpaceGlyph::Emoji(value) => value,
        SpaceGlyph::Glyph(name) => name.as_str(),
    }
}
