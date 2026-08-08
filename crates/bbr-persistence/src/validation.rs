use std::collections::HashSet;

use crate::{
    CommandHistoryEntry, DownloadRecord, InvalidSnapshotIssue, ProfileHue, SearchEngineId,
    SearchSettings, Snapshot, SpaceGlyph, WindowGeometry,
};

pub(crate) const MAX_SQLITE_ID: u64 = i64::MAX as u64;
pub(crate) const SIDEBAR_MIN: f64 = 200.0;
pub(crate) const SIDEBAR_MAX: f64 = 360.0;

/// Validates a snapshot before any repository write.
///
/// The write side must reject exactly what restore would reject. All
/// persisted identity kinds — profiles, spaces, live tabs, recently closed
/// tabs, and windows — therefore share one global identity set, so a
/// recently closed `tab_id` that collides with a profile, space, live tab,
/// window, or another recently closed entry is rejected here instead of
/// surfacing as corruption on the next restore. Download and command-history
/// identities come from independent counters, so they validate against
/// their own per-table sets only.
pub(crate) fn validate_snapshot(snapshot: &Snapshot) -> Result<(), InvalidSnapshotIssue> {
    let mut identities = HashSet::new();
    let mut live_tabs = HashSet::new();

    // Window identities join the global identity set before recently-closed
    // validation so a closed tab id colliding with a window id is rejected
    // before writing. Restore discovers the same collision in the opposite
    // phase: recently-closed ids enter the shared set before windows load.
    for window in &snapshot.windows {
        validate_id("window", window.id)?;
        insert_identity(&mut identities, window.id)?;
    }

    for profile in &snapshot.profiles {
        validate_id("profile", profile.id)?;
        insert_identity(&mut identities, profile.id)?;
        validate_hue(profile.hue);

        for space in &profile.spaces {
            validate_id("space", space.id)?;
            insert_identity(&mut identities, space.id)?;
            validate_space_appearance(space.id, &space.glyph)?;
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
                // The same global identity set restore uses: a recently closed
                // tab id must not collide with a live tab, another recently
                // closed id, or any profile/space/window identity.
                if live_tabs.contains(&closed.tab_id)
                    || identities.contains(&closed.tab_id)
                    || !closed_tabs.insert(closed.tab_id)
                {
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

    validate_settings(&snapshot.settings)?;
    validate_downloads(&snapshot.downloads)?;
    validate_command_history(&snapshot.command_history)?;

    Ok(())
}

/// The curated tint identity is closed by type; the glyph identity is
/// closed by type for icon names, while an emoji payload must be a member
/// of the curated allowlist (the same rule restore enforces).
fn validate_space_appearance(
    space_id: u64,
    glyph: &SpaceGlyph,
) -> Result<(), InvalidSnapshotIssue> {
    match glyph {
        SpaceGlyph::Emoji(emoji) => {
            if crate::fields::ALLOWED_EMOJI.contains(&emoji.as_str()) {
                Ok(())
            } else {
                Err(InvalidSnapshotIssue::InvalidEmoji { space_id })
            }
        }
        SpaceGlyph::Glyph(_) => Ok(()),
    }
}

pub(crate) fn validate_settings(settings: &SearchSettings) -> Result<(), InvalidSnapshotIssue> {
    match (settings.engine, settings.custom_template.as_deref()) {
        (SearchEngineId::Custom, None) => Err(InvalidSnapshotIssue::InvalidSettings {
            detail: "custom engine requires a search template".into(),
        }),
        (SearchEngineId::Custom, Some(template)) => {
            if valid_custom_template(template) {
                Ok(())
            } else {
                Err(InvalidSnapshotIssue::InvalidSettings {
                    detail: "invalid custom search template".into(),
                })
            }
        }
        (engine, Some(_)) if engine != SearchEngineId::Custom => {
            Err(InvalidSnapshotIssue::InvalidSettings {
                detail: "search template without the custom engine".into(),
            })
        }
        _ => Ok(()),
    }
}

fn validate_downloads(downloads: &[DownloadRecord]) -> Result<(), InvalidSnapshotIssue> {
    let mut ids = HashSet::new();
    for record in downloads {
        validate_id("download", record.id)?;
        if !ids.insert(record.id) {
            return Err(InvalidSnapshotIssue::DuplicateIdentity { id: record.id });
        }
        if record.size > i64::MAX as u64 {
            return Err(InvalidSnapshotIssue::InvalidDownload {
                id: record.id,
                detail: "size exceeds the SQLite integer range".into(),
            });
        }
        if record.terminal_at > i64::MAX as u64 {
            return Err(InvalidSnapshotIssue::InvalidDownload {
                id: record.id,
                detail: "terminal timestamp exceeds the SQLite integer range".into(),
            });
        }
        if let Some(destination) = &record.destination {
            if !valid_path(destination) {
                return Err(InvalidSnapshotIssue::InvalidDownload {
                    id: record.id,
                    detail: "invalid destination path".into(),
                });
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_command_history(
    history: &[CommandHistoryEntry],
) -> Result<(), InvalidSnapshotIssue> {
    let mut ids = HashSet::new();
    for entry in history {
        validate_id("history entry", entry.id)?;
        if !ids.insert(entry.id) {
            return Err(InvalidSnapshotIssue::DuplicateIdentity { id: entry.id });
        }
        if !valid_url(&entry.url) {
            return Err(InvalidSnapshotIssue::InvalidHistory {
                id: entry.id,
                detail: "invalid URL".into(),
            });
        }
        if entry.visit_count > i64::MAX as u64 {
            return Err(InvalidSnapshotIssue::InvalidHistory {
                id: entry.id,
                detail: "visit count exceeds the SQLite integer range".into(),
            });
        }
        if entry.last_visited > i64::MAX as u64 {
            return Err(InvalidSnapshotIssue::InvalidHistory {
                id: entry.id,
                detail: "recency marker exceeds the SQLite integer range".into(),
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

/// Bounded destination-path validation: non-empty and free of NUL and
/// control bytes. Unlike URLs, file-system paths legitimately contain
/// spaces, so whitespace is allowed here.
pub(crate) fn valid_path(path: &str) -> bool {
    !path.is_empty()
        && !path
            .bytes()
            .any(|byte| byte == 0 || byte.is_ascii_control())
}

/// Mirrors the delivered search-settings contract's custom template rules:
/// non-empty after trimming, an absolute `http://`/`https://` URL, exactly
/// one `%s` placeholder, and every other `%` must begin a valid `%XX`
/// percent-encoding. Persistence re-validates on write and on restore so a
/// closed engine/template invariant can never be persisted.
pub(crate) fn valid_custom_template(template: &str) -> bool {
    let trimmed = template.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return false;
    }
    let mut matches = trimmed.match_indices("%s");
    let Some((placeholder, _)) = matches.next() else {
        return false;
    };
    if matches.next().is_some() {
        return false;
    }
    valid_percent_sequences(trimmed.as_bytes(), placeholder)
}

fn valid_percent_sequences(bytes: &[u8], placeholder: usize) -> bool {
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'%' {
            index += 1;
            continue;
        }
        if index == placeholder {
            // The placeholder `%s`; its `s` is the following byte.
            index += 2;
            continue;
        }
        if index + 2 >= bytes.len()
            || !bytes[index + 1].is_ascii_hexdigit()
            || !bytes[index + 2].is_ascii_hexdigit()
        {
            return false;
        }
        index += 3;
    }
    true
}

fn validate_url(tab_id: u64, url: &str) -> Result<(), InvalidSnapshotIssue> {
    if valid_url(url) {
        Ok(())
    } else {
        Err(InvalidSnapshotIssue::InvalidUrl { tab_id })
    }
}

fn validate_hue(hue: ProfileHue) {
    // `ProfileHue` is an exhaustive closed enum whose every variant has a
    // canonical persisted name; constructing a snapshot already guarantees a
    // valid hue, so this stays an identity constraint on the write path.
    let _ = hue.as_str();
}

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
