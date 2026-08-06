//! Profile-grouped space surface (DESIGN.md §11.3, §12).
//!
//! The space region renders every space grouped under its profile with a
//! per-row chip and a hairline between groups; this module owns the render
//! derivation: grouping, the per-space row visuals, the §11.3 title budget,
//! and the testkit state/title lists the E2E asserts.

use crate::components::{ProfileChipSpec, SpaceRowVisuals, SpaceVisualState};
use crate::theme::{Spacing, Theme};
use bbr_browser::ControllerSnapshot;

use crate::shell::presentation::truncate_title;

/// DESIGN.md §12 profile chip monogram: the first 1–2 title characters,
/// uppercased. Identity is readable per row without a header.
pub fn profile_monogram(profile_title: &str) -> String {
    let mut chars = profile_title.chars();
    let first = chars.next().unwrap_or('?');
    let second = chars.next();
    let mut monogram = String::new();
    monogram.push(first.to_ascii_uppercase());
    if let Some(ch) = second {
        monogram.push(ch.to_ascii_uppercase());
    }
    monogram
}

/// One sidebar space group: the spaces of one profile, in creation order,
/// preceded by a hairline when it is not the first group (DESIGN.md §11.3:
/// profile groups stack under a hairline, no repeated profile header in v1).
pub struct SpaceGroup {
    pub rows: Vec<SpaceRowVisuals>,
    pub hairline_before: bool,
}

/// Render-order space rows grouped by profile, derived from the controller
/// snapshot — the same source the sidebar renders (DESIGN.md §11.3).
pub fn space_groups(browser: &ControllerSnapshot, budget: usize) -> Vec<SpaceGroup> {
    let mut groups: Vec<SpaceGroup> = Vec::new();
    for profile in &browser.profiles {
        let spaces: Vec<&bbr_browser::SpaceSnapshot> = browser
            .spaces
            .iter()
            .filter(|s| s.profile == profile.id)
            .collect();
        if spaces.is_empty() {
            continue;
        }
        let rows = spaces
            .iter()
            .map(|s| SpaceRowVisuals {
                index: s.index,
                profile_index: profile.index,
                title: truncate_title(&s.title, budget).into(),
                count: s.tab_count,
                state: if s.active {
                    SpaceVisualState::Selected
                } else {
                    SpaceVisualState::Default
                },
                chip: ProfileChipSpec {
                    letter: profile_monogram(&profile.title),
                    hue: profile.hue,
                },
            })
            .collect();
        groups.push(SpaceGroup {
            rows,
            hairline_before: !groups.is_empty(),
        });
    }
    groups
}

/// The §11.3 space-row title budget: sidebar width minus the fixed row
/// anatomy (2× padding, the 20 pt chip, 2 gaps, the count column). Per-char
/// width is the same conservative 0.45 × `type:title` estimate as the tab
/// row budget.
pub fn space_title_budget_chars(theme: &Theme) -> usize {
    let density = theme.density;
    let padding = f32::from(Spacing::sp2()) * 2.0;
    let chip = 20.0;
    let gaps = f32::from(Spacing::sp2()) * 2.0;
    let count = 24.0;
    let area = f32::from(density.sidebar_width()) - padding - chip - gaps - count;
    let per_char = f32::from(theme.type_.title.size) * 0.45;
    (area / per_char).floor().max(1.0) as usize
}

/// §11.3 space visual states in render order (grouped), mirroring the rows
/// the sidebar renders — `selected` for the active space, `drag-target` for
/// the space a tab drag currently targets, else `default`.
#[cfg(any(feature = "testkit", test))]
pub fn space_states(
    browser: &ControllerSnapshot,
    drag_target: Option<bbr_browser::SpaceId>,
) -> Vec<SpaceVisualState> {
    space_groups(browser, usize::MAX)
        .into_iter()
        .flat_map(|group| group.rows.into_iter())
        .map(|mut row| {
            let space = browser
                .spaces
                .iter()
                .find(|s| s.index == row.index)
                .map(|s| s.id);
            if space.is_some() && space == drag_target {
                row.state = SpaceVisualState::DragTarget;
            }
            row.state
        })
        .collect()
}

/// §11.3 space row titles in render order, after §4 truncation — the same
/// `space_groups` source the sidebar renders.
#[cfg(any(feature = "testkit", test))]
pub fn space_titles(browser: &ControllerSnapshot, budget: usize) -> Vec<String> {
    space_groups(browser, budget)
        .into_iter()
        .flat_map(|group| group.rows.into_iter().map(|row| row.title.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_monogram_uses_first_two_title_chars() {
        assert_eq!(profile_monogram("Default"), "DE");
        assert_eq!(profile_monogram("Private"), "PR");
        assert_eq!(profile_monogram("w"), "W");
        assert_eq!(profile_monogram(""), "?");
    }

    /// Build a snapshot with two profiles and three spaces (two in the first
    /// profile, one in the second) for the grouping derivation tests.
    fn space_snapshot() -> ControllerSnapshot {
        let space =
            |index: usize, id: u64, profile: u64, title: &str, active: bool, tabs: usize| {
                bbr_browser::SpaceSnapshot {
                    index,
                    id: bbr_browser::SpaceId(id),
                    profile: bbr_web::ProfileId(profile),
                    title: title.into(),
                    active_tab: Some(0),
                    active_view: None,
                    active,
                    tab_count: tabs,
                }
            };
        ControllerSnapshot {
            active_profile: bbr_web::ProfileId(1),
            active_profile_title: Some("Default".into()),
            active_profile_hue: Some(bbr_browser::ProfileHue::Cobalt),
            active_space: bbr_browser::SpaceId(2),
            tabs: Vec::new(),
            profiles: vec![
                bbr_browser::ProfileSnapshot {
                    index: 0,
                    id: bbr_web::ProfileId(1),
                    title: "Default".into(),
                    hue: bbr_browser::ProfileHue::Cobalt,
                },
                bbr_browser::ProfileSnapshot {
                    index: 1,
                    id: bbr_web::ProfileId(3),
                    title: "Private".into(),
                    hue: bbr_browser::ProfileHue::Violet,
                },
            ],
            spaces: vec![
                space(0, 2, 1, "Default", true, 3),
                space(1, 4, 3, "Private", false, 1),
                space(2, 5, 1, "Reading", false, 0),
            ],
            active_index: Some(0),
            nav: Default::default(),
            web_view_hosted: false,
            engine_sessions: 0,
            active_view: None,
        }
    }

    #[test]
    fn space_groups_stack_profiles_with_hairlines() {
        let browser = space_snapshot();
        let groups = space_groups(&browser, 30);
        assert_eq!(groups.len(), 2);
        // Default profile first (creation order), then Private; a hairline
        // separates the groups; every row carries its profile chip.
        assert!(!groups[0].hairline_before);
        assert!(groups[1].hairline_before);
        assert_eq!(
            groups[0].rows.iter().map(|r| r.index).collect::<Vec<_>>(),
            vec![0, 2],
            "both default-profile spaces render in creation order"
        );
        assert_eq!(
            groups[1].rows.iter().map(|r| r.index).collect::<Vec<_>>(),
            vec![1]
        );
        // Chip identity: hue + monogram per row, never a repeated header.
        assert_eq!(groups[0].rows[0].chip.hue, bbr_browser::ProfileHue::Cobalt);
        assert_eq!(groups[0].rows[0].chip.letter, "DE");
        assert_eq!(groups[1].rows[0].chip.hue, bbr_browser::ProfileHue::Violet);
        assert_eq!(groups[1].rows[0].chip.letter, "PR");
        // The §11.3 count column mirrors each space's tab count.
        assert_eq!(groups[0].rows[0].count, 3);
        assert_eq!(groups[1].rows[0].count, 1);
    }

    #[test]
    fn space_states_mark_only_the_active_space() {
        let browser = space_snapshot();
        assert_eq!(
            space_states(&browser, None),
            vec![
                SpaceVisualState::Selected,
                SpaceVisualState::Default,
                SpaceVisualState::Default,
            ]
        );
        // A targeted space renders `drag-target` instead of its rest state.
        // Render order is Default, Reading, Private — Reading (id 5) is the
        // second rendered row.
        assert_eq!(
            space_states(&browser, Some(bbr_browser::SpaceId(5))),
            vec![
                SpaceVisualState::Selected,
                SpaceVisualState::DragTarget,
                SpaceVisualState::Default,
            ]
        );
        assert_eq!(
            space_titles(&browser, 30),
            vec![
                "Default".to_string(),
                "Reading".to_string(),
                "Private".to_string()
            ]
        );
    }

    #[test]
    fn space_titles_follow_the_truncation_contract() {
        let browser = space_snapshot();
        // A long name with an early separator cuts so the head survives (§4);
        // names without a separator keep the full title (GPUI's text_ellipsis
        // is the pixel-exact backstop).
        let mut long = space_snapshot();
        long.spaces[0].title =
            "Reading List – a long suffix that surely exceeds the row budget".into();
        let titles = space_titles(&long, 20);
        assert_eq!(titles[0], "Reading List…");
        assert_eq!(
            space_titles(&browser, 6)[0],
            "Default",
            "no-separator names stay whole"
        );
    }
}
