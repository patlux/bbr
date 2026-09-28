//! Repository browser-state row methods (Phase 7, warm-restore slice): the
//! focused structural write that replaces only the browser tables —
//! profiles, spaces, tabs, recently-closed entries, splits, and windows —
//! while leaving the global settings, download, and command-history rows
//! untouched. Focused here so `repository.rs` and `repository_rows.rs` keep
//! their pre-slice size; the `Repository` API surface grows by exactly one
//! method.
//!
//! The write follows the same store disposition as every other write path: a
//! mismatched or corrupt store is discarded and recreated empty on write
//! (`setup::prepare_store`). Validation reuses the complete-snapshot write
//! contract ([`validation::validate_snapshot`] over a synthesized snapshot),
//! so the exact same global-identity uniqueness, pinned-prefix, member
//! active-tab, distinct-member split, bounded-URL, window-geometry, and
//! sidebar invariants that protect `replace_snapshot` protect this write.

use rusqlite::{params, Connection};

use crate::repository::to_i64;
use crate::{
    schema, setup, validation, BrowserState, PersistenceError, Repository, SearchSettings,
    Snapshot, SpaceGlyph, WriteStats,
};

impl Repository {
    /// Transactionally replaces the complete browser structural state
    /// (profiles, spaces, tabs, recently-closed, splits, windows) without
    /// touching the settings, download, or command-history tables.
    ///
    /// The whole browser state is rewritten in one transaction with dense
    /// per-table position keys preserving the caller's order, so a later
    /// restore reads back exactly the persisted browser model. The store
    /// disposition (discard and recreate empty on stamp/shape mismatch) and
    /// the validation contract are identical to
    /// [`Self::replace_snapshot`](crate::Repository::replace_snapshot).
    pub fn store_browser_state(
        &self,
        state: &BrowserState,
    ) -> Result<WriteStats, PersistenceError> {
        // Validate through the complete-snapshot contract: browser tables
        // share the same invariants as the full snapshot write, and the
        // global stores contribute nothing here (defaults/empty collections
        // are trivially valid and never written by this method).
        let full = Snapshot {
            profiles: state.profiles.clone(),
            windows: state.windows.clone(),
            settings: SearchSettings::default(),
            downloads: Vec::new(),
            command_history: Vec::new(),
            // Schema v3: the global Saved Pages tables are never part of a
            // browser-state write (isolation is asserted in the saved-pages
            // repository tests); empty defaults keep the shared snapshot
            // contract valid.
            saved_pages: Vec::new(),
            collections: Vec::new(),
        };
        validation::validate_snapshot(&full).map_err(PersistenceError::InvalidSnapshot)?;
        let mut connection = Connection::open(self.path()).map_err(schema::storage)?;
        setup::prepare_store(&mut connection)?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(schema::storage)?;
        let transaction = connection.transaction().map_err(schema::storage)?;
        let mut deleted_rows = 0_u64;
        // Child-first deletion of the browser tables only: the global
        // settings/download/command-history rows survive this write.
        for table in [
            "recently_closed",
            "splits",
            "tabs",
            "spaces",
            "profiles",
            "windows",
        ] {
            deleted_rows += transaction
                .execute(&format!("DELETE FROM {table}"), [])
                .map_err(schema::storage)? as u64;
        }
        let mut inserted_rows = 0_u64;

        for (profile_position, profile) in state.profiles.iter().enumerate() {
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
                    "INSERT INTO spaces(id, profile_id, title, tint, glyph_kind, glyph, position, active_tab_id, tint_intensity, grain) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![to_i64(space.id), to_i64(profile.id), space.title, space.tint.as_str(), glyph_kind(&space.glyph), glyph_value(&space.glyph), space_position as i64, space.active_tab.map(to_i64), space.paper.intensity.as_str(), space.paper.grain],
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
        for (position, window) in state.windows.iter().enumerate() {
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

fn glyph_kind(glyph: &SpaceGlyph) -> &'static str {
    match glyph {
        crate::SpaceGlyph::Emoji(_) => "emoji",
        crate::SpaceGlyph::Glyph(_) => "glyph",
    }
}

fn glyph_value(glyph: &SpaceGlyph) -> &str {
    match glyph {
        crate::SpaceGlyph::Emoji(value) => value,
        crate::SpaceGlyph::Glyph(name) => name.as_str(),
    }
}
