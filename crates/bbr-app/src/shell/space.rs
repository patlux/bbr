//! Profile-grouped space surface (DESIGN.md §11.3, §12).
//!
//! The space region renders every space grouped under its profile with a
//! per-row chip and a hairline between groups; this module owns the render
//! derivation: grouping, the per-space row visuals, the §11.3 title budget,
//! and the testkit state/title lists the E2E asserts.

use bbr_browser::ControllerSnapshot;
use bbr_ui::components::{ProfileChipSpec, SpaceRowVisuals};
use bbr_ui::theme::{Spacing, Theme};

use bbr_ui::text::truncate_title;

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

/// The active profile's §12 chip identity (creation-order index + hue +
/// monogram), for surfaces that render the active profile's chip outside the
/// space rows — e.g. the §11.3 command-field cross-context hint.
pub fn active_profile_chip(browser: &ControllerSnapshot) -> Option<(usize, ProfileChipSpec)> {
    browser
        .profiles
        .iter()
        .find(|profile| profile.id == browser.active_profile)
        .map(|profile| {
            (
                profile.index,
                ProfileChipSpec {
                    letter: profile_monogram(&profile.title),
                    hue: profile.hue,
                },
            )
        })
}

/// One sidebar space group: the spaces of one profile, in creation order
/// (DESIGN.md §11.3: profile groups stack, no repeated profile header in
/// v1). The canonical order the permanent strip chips, the keyboard
/// traversal, and the §15 id derivation all consume.
pub struct SpaceGroup {
    pub rows: Vec<SpaceRowVisuals>,
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
                title: truncate_title(&s.title, budget).into(),
            })
            .collect();
        groups.push(SpaceGroup { rows });
    }
    groups
}

/// The §11.3 space-row title budget: sidebar width minus the fixed row
/// anatomy (2× padding, the §12 `density:profile-chip`, 2 gaps, the count
/// column). Per-char width is the same conservative 0.45 × `type:title`
/// estimate as the tab row budget.
pub fn space_title_budget_chars(theme: &Theme) -> usize {
    let density = theme.density;
    let padding = f32::from(Spacing::sp2()) * 2.0;
    let chip = f32::from(density.profile_chip());
    let gaps = f32::from(Spacing::sp2()) * 2.0;
    let count = 24.0;
    let area = f32::from(density.sidebar_width()) - padding - chip - gaps - count;
    let per_char = f32::from(theme.type_.title.size) * 0.45;
    (area / per_char).floor().max(1.0) as usize
}

// The `space_groups` derivation is the single canonical order the permanent
// §11.3 strip (chips), the keyboard traversal, and the §15 id derivation
// all consume; the chips carry the per-space title and selected state.

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

    #[test]
    fn active_profile_chip_identifies_the_active_profile() {
        let browser = space_snapshot();
        // Active profile 1 (Default, index 0) carries the §12 chip identity
        // (creation-order index, monogram, hue) for the §11.3 command-field
        // hint; the chip is never keyed to the space index.
        let chip = active_profile_chip(&browser).expect("active profile chip");
        assert_eq!(chip.0, 0);
        assert_eq!(chip.1.letter, "DE");
        assert_eq!(chip.1.hue, bbr_browser::ProfileHue::Cobalt);
        // A snapshot whose active profile is missing from the collection
        // derives no chip (defensive; the controller invariants forbid it).
        let mut missing = space_snapshot();
        missing.active_profile = bbr_web::ProfileId(99);
        assert_eq!(active_profile_chip(&missing), None);
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
            all_tabs: Vec::new(),
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
            lifecycle_counts: bbr_browser::LifecycleCounts::default(),
        }
    }

    #[test]
    fn space_groups_stack_profiles_with_hairlines() {
        let browser = space_snapshot();
        let groups = space_groups(&browser, 30);
        assert_eq!(groups.len(), 2);
        // Default profile first (creation order), then Private.
        assert_eq!(
            groups[0].rows.iter().map(|r| r.index).collect::<Vec<_>>(),
            vec![0, 2],
            "both default-profile spaces render in creation order"
        );
        assert_eq!(
            groups[1].rows.iter().map(|r| r.index).collect::<Vec<_>>(),
            vec![1]
        );
        // The §4 title truncation is applied in the same derivation the
        // strip chips consume.
        assert_eq!(groups[0].rows[0].title, "Default");
        assert_eq!(groups[1].rows[0].title, "Private");
    }

    #[test]
    fn grouped_order_keys_the_active_space_lookup() {
        // The canonical grouped order resolves each row's persistent space
        // id; the strip's display order is exactly this vector order, while
        // the ids derive from the persistent `SpaceId` (never the index).
        let browser = space_snapshot();
        let order: Vec<(usize, bbr_browser::SpaceId)> = space_groups(&browser, usize::MAX)
            .into_iter()
            .flat_map(|group| group.rows.into_iter())
            .map(|row| {
                let space = browser
                    .spaces
                    .iter()
                    .find(|s| s.index == row.index)
                    .expect("space row derives from the snapshot");
                (row.index, space.id)
            })
            .collect();
        // Render order is Default (id 2), Reading (id 5), Private (id 4) —
        // creation order within the Default profile group, then the Private
        // group. The display order (2, 5, 4) differs from the creation-index
        // order (2, 4, 5), proving display order is separate from identity.
        assert_eq!(
            order,
            vec![
                (0, bbr_browser::SpaceId(2)),
                (2, bbr_browser::SpaceId(5)),
                (1, bbr_browser::SpaceId(4))
            ]
        );
    }
}
