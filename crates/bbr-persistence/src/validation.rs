use std::collections::HashSet;

use crate::{InvalidSnapshotIssue, ProfileHue, Snapshot, WindowGeometry};

pub(crate) const MAX_SQLITE_ID: u64 = i64::MAX as u64;
pub(crate) const SIDEBAR_MIN: f64 = 200.0;
pub(crate) const SIDEBAR_MAX: f64 = 360.0;

pub(crate) fn validate_snapshot(snapshot: &Snapshot) -> Result<(), InvalidSnapshotIssue> {
    let mut identities = HashSet::new();
    let mut live_tabs = HashSet::new();

    for profile in &snapshot.profiles {
        validate_id("profile", profile.id)?;
        insert_identity(&mut identities, profile.id)?;
        validate_hue(profile.hue);

        for space in &profile.spaces {
            validate_id("space", space.id)?;
            insert_identity(&mut identities, space.id)?;
            let mut members = HashSet::new();
            let mut saw_regular = false;

            for tab in &space.tabs {
                validate_id("tab", tab.id)?;
                insert_identity(&mut identities, tab.id)?;
                members.insert(tab.id);
                live_tabs.insert(tab.id);
                validate_url(tab.id, &tab.url)?;
                if !tab.pinned {
                    saw_regular = true;
                } else if saw_regular {
                    return Err(InvalidSnapshotIssue::InvalidPinnedRegion { space_id: space.id });
                }
            }

            if let Some(tab_id) = space.active_tab {
                if !members.contains(&tab_id) {
                    return Err(InvalidSnapshotIssue::InvalidActiveTab {
                        space_id: space.id,
                        tab_id,
                    });
                }
            }
            if let Some(split) = space.split {
                if split.primary_tab == split.secondary_tab
                    || !members.contains(&split.primary_tab)
                    || !members.contains(&split.secondary_tab)
                {
                    return Err(InvalidSnapshotIssue::InvalidSplit { space_id: space.id });
                }
            }
        }
    }

    let mut closed_tabs = HashSet::new();
    for profile in &snapshot.profiles {
        for space in &profile.spaces {
            for closed in &space.recently_closed {
                validate_id("recently closed tab", closed.tab_id)?;
                if live_tabs.contains(&closed.tab_id) || !closed_tabs.insert(closed.tab_id) {
                    return Err(InvalidSnapshotIssue::RecentlyClosedIdentityCollision {
                        tab_id: closed.tab_id,
                    });
                }
                validate_url(closed.tab_id, &closed.url)?;
                if closed.former_position > i64::MAX as u64 {
                    return Err(InvalidSnapshotIssue::InvalidFormerPosition {
                        tab_id: closed.tab_id,
                    });
                }
            }
        }
    }

    for window in &snapshot.windows {
        validate_id("window", window.id)?;
        insert_identity(&mut identities, window.id)?;
        if let Some(geometry) = window.geometry {
            if !valid_geometry(geometry) {
                return Err(InvalidSnapshotIssue::InvalidGeometry {
                    window_id: window.id,
                });
            }
        }
        if !valid_sidebar(window.sidebar.width, window.sidebar.last_expanded_width) {
            return Err(InvalidSnapshotIssue::InvalidSidebar {
                window_id: window.id,
            });
        }
    }

    Ok(())
}

fn insert_identity(ids: &mut HashSet<u64>, id: u64) -> Result<(), InvalidSnapshotIssue> {
    if ids.insert(id) {
        Ok(())
    } else {
        Err(InvalidSnapshotIssue::DuplicateIdentity { id })
    }
}

pub(crate) fn validate_id(kind: &'static str, id: u64) -> Result<(), InvalidSnapshotIssue> {
    if id == 0 || id > MAX_SQLITE_ID {
        Err(InvalidSnapshotIssue::InvalidId { kind, id })
    } else {
        Ok(())
    }
}

pub(crate) fn valid_persisted_id(id: i64) -> bool {
    id > 0
}

/// This is intentionally bounded validation, not a complete URL parser.
/// Required URLs must be non-empty and contain no ASCII whitespace, control, or NUL bytes.
pub(crate) fn valid_url(url: &str) -> bool {
    !url.is_empty()
        && !url
            .bytes()
            .any(|byte| byte == 0 || byte.is_ascii_whitespace() || byte.is_ascii_control())
}

fn validate_url(tab_id: u64, url: &str) -> Result<(), InvalidSnapshotIssue> {
    if valid_url(url) {
        Ok(())
    } else {
        Err(InvalidSnapshotIssue::InvalidUrl { tab_id })
    }
}

const fn validate_hue(_hue: ProfileHue) {}

pub(crate) fn valid_geometry(geometry: WindowGeometry) -> bool {
    geometry.x.is_finite()
        && geometry.y.is_finite()
        && geometry.width.is_finite()
        && geometry.width > 0.0
        && geometry.height.is_finite()
        && geometry.height > 0.0
}

pub(crate) fn valid_sidebar(width: f64, last_expanded_width: f64) -> bool {
    width.is_finite()
        && (SIDEBAR_MIN..=SIDEBAR_MAX).contains(&width)
        && last_expanded_width.is_finite()
        && (SIDEBAR_MIN..=SIDEBAR_MAX).contains(&last_expanded_width)
}
