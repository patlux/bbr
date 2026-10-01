//! Blueprint D19 (several windows, task packet `multi-window.md`): every
//! browser window's active space and per-space selections, kept in
//! `windows.txt` in the profile directory next to the store. The store's
//! window rows keep each window's geometry and rail by position; the
//! model's per-space `active_tab` holds only one window's selections, so
//! the windows' own selections are app state beside the store.
//!
//! The first line is the format version (`1`); every further line is one
//! window, in window order: its active space, a tab, and its selection per
//! space as `space=tab` (`-` for the empty state), comma-separated, e.g.
//! `7\t7=12,9=-`. Unreadable lines are skipped; the controller drops spaces
//! and tabs that are gone.

use std::path::{Path, PathBuf};

use bbr_domain::{SpaceId, TabId};

const VERSION: &str = "1";

/// One window as saved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowLayout {
    pub active_space: SpaceId,
    pub selections: Vec<(SpaceId, Option<TabId>)>,
}

/// The file of a profile directory.
pub fn path_in(profile_dir: &Path) -> PathBuf {
    profile_dir.join("windows.txt")
}

/// Every window as the file body.
pub fn render(windows: &[WindowLayout]) -> String {
    let mut out = format!("{VERSION}\n");
    for window in windows {
        let selections: Vec<String> = window
            .selections
            .iter()
            .map(|(space, tab)| match tab {
                Some(tab) => format!("{}={}", space.0, tab.0),
                None => format!("{}=-", space.0),
            })
            .collect();
        out.push_str(&format!(
            "{}\t{}\n",
            window.active_space.0,
            selections.join(",")
        ));
    }
    out
}

/// The windows in a file body.
pub fn parse(body: &str) -> Vec<WindowLayout> {
    let mut lines = body.lines();
    if lines.next() != Some(VERSION) {
        return Vec::new();
    }
    lines.filter_map(parse_window).collect()
}

fn parse_window(line: &str) -> Option<WindowLayout> {
    let (space, rest) = line.split_once('\t')?;
    let active_space = SpaceId(space.parse().ok()?);
    let mut selections = Vec::new();
    for entry in rest.split(',').filter(|entry| !entry.is_empty()) {
        let (space, tab) = entry.split_once('=')?;
        let space = SpaceId(space.parse().ok()?);
        let tab = match tab {
            "-" => None,
            tab => Some(TabId(tab.parse().ok()?)),
        };
        selections.push((space, tab));
    }
    Some(WindowLayout {
        active_space,
        selections,
    })
}

/// Load the file (nothing when it is missing).
pub fn load(path: &Path) -> Vec<WindowLayout> {
    std::fs::read_to_string(path)
        .map(|body| parse(&body))
        .unwrap_or_default()
}

/// Write the file atomically.
pub fn save(path: &Path, windows: &[WindowLayout]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let temporary = path.with_extension("txt.tmp");
    std::fs::write(&temporary, render(windows))?;
    std::fs::rename(&temporary, path)
}

/// The first window's saved state applied to a restore intent: its active
/// space (when that space is restored) and its selections replace the
/// model's per-space active tabs, so the hydration plans the sessions the
/// first window shows. Returns the active space to hydrate, if the saved one
/// is restored.
///
/// The file is written synchronously and the store by a background writer,
/// so after a force quit the file can name a tab the store never committed:
/// such a selection keeps the store's own active tab (a saved empty state
/// is newer than the store and stays empty).
pub fn apply_first(
    intent: &mut bbr_browser::RestoreIntent,
    first: &WindowLayout,
) -> Option<SpaceId> {
    let mut active = None;
    for space in intent
        .profiles
        .iter_mut()
        .flat_map(|profile| profile.spaces.iter_mut())
    {
        if space.id == first.active_space.0 {
            active = Some(first.active_space);
        }
        if let Some((_, tab)) = first
            .selections
            .iter()
            .find(|(saved, _)| saved.0 == space.id)
        {
            match tab {
                // A closed pinned row is never the selection.
                Some(tab)
                    if space
                        .tabs
                        .iter()
                        .any(|candidate| candidate.id == tab.0 && !candidate.closed) =>
                {
                    space.active_tab = Some(tab.0);
                }
                Some(_) => {}
                None => space.active_tab = None,
            }
        }
    }
    active
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two() -> Vec<WindowLayout> {
        vec![
            WindowLayout {
                active_space: SpaceId(7),
                selections: vec![(SpaceId(7), Some(TabId(12))), (SpaceId(9), None)],
            },
            WindowLayout {
                active_space: SpaceId(9),
                selections: vec![(SpaceId(7), None), (SpaceId(9), Some(TabId(20)))],
            },
        ]
    }

    #[test]
    fn windows_round_trip_as_text() {
        let body = render(&two());
        assert_eq!(body, "1\n7\t7=12,9=-\n9\t7=-,9=20\n");
        assert_eq!(parse(&body), two());
    }

    #[test]
    fn malformed_lines_and_unknown_versions_are_skipped() {
        assert!(parse("2\n7\t7=12\n").is_empty(), "unknown version");
        let parsed = parse("1\nnope\n7\t7=x\n8\t8=1\n9\t\n");
        assert_eq!(
            parsed,
            vec![
                WindowLayout {
                    active_space: SpaceId(8),
                    selections: vec![(SpaceId(8), Some(TabId(1)))],
                },
                WindowLayout {
                    active_space: SpaceId(9),
                    selections: Vec::new(),
                },
            ]
        );
    }

    #[test]
    fn the_first_window_decides_the_hydrated_space_and_selections() {
        use bbr_browser::{IntentProfile, IntentSpace, IntentTab, RestoreIntent};
        let tab = |id| IntentTab {
            id,
            title: String::new(),
            url: "https://example.com/".into(),
            pinned: false,
            closed: false,
            pinned_page: None,
        };
        let space = |id, tabs: Vec<IntentTab>, active| IntentSpace {
            id,
            title: String::new(),
            tabs,
            active_tab: active,
            recently_closed: Vec::new(),
            split: None,
        };
        let mut intent = RestoreIntent {
            profiles: vec![IntentProfile {
                id: 1,
                title: String::new(),
                hue: bbr_domain::ProfileHue::ALL[0],
                spaces: vec![
                    space(7, vec![tab(12), tab(13)], Some(13)),
                    space(9, vec![tab(20)], Some(20)),
                ],
            }],
        };
        let first = WindowLayout {
            active_space: SpaceId(9),
            selections: vec![(SpaceId(7), Some(TabId(12))), (SpaceId(9), Some(TabId(99)))],
        };
        assert_eq!(apply_first(&mut intent, &first), Some(SpaceId(9)));
        let spaces = &intent.profiles[0].spaces;
        assert_eq!(spaces[0].active_tab, Some(12));
        assert_eq!(
            spaces[1].active_tab,
            Some(20),
            "a tab the store never committed keeps the store's selection"
        );
        let empty = WindowLayout {
            active_space: SpaceId(7),
            selections: vec![(SpaceId(7), None)],
        };
        apply_first(&mut intent, &empty);
        assert_eq!(intent.profiles[0].spaces[0].active_tab, None);
        let gone = WindowLayout {
            active_space: SpaceId(42),
            selections: Vec::new(),
        };
        assert_eq!(apply_first(&mut intent, &gone), None);
    }
}
