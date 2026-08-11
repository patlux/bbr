//! The Profile-chip matrix authority (packet
//! `component-gallery-browser-components.md`, Group 4): **one declaration
//! source** for every valid §12 profile-chip family and every rejected
//! combination. The gallery story renders exactly the valid families
//! through the production `bbr_ui::components::space` `profile_chip`
//! symbol, and the fixture builder refuses (`None`) any case the validity
//! authority rejects — so the story, the fixture builder, the matrix
//! table, and the tests cannot drift. The rejected reasons also live here.
//!
//! Matrix rows come straight from the packet's Profile chip /
//! Space-strip row: profile chip S/N/XL density (20 / 24 / 28 pt §12), a
//! 1–2 character monogram, the hue tile, and the profile-keyed id
//! `bbr-profile-chip-<profile_index>` (§12 — never the space index).
//! Rejected: the removed legacy `space_row` / `space_new_row` list
//! anatomy, any chip keyed to a space index, and (shared with the footer
//! matrix) any wrapped/second-line strip or fixture that creates a second
//! strip or navigation authority — the strip-level rejections live in
//! [`crate::footer_matrix`] where the strip fixture is built.

use bbr_domain::ProfileHue;
use bbr_ui::components::ProfileChipSpec;
use bbr_ui::theme::Density;

/// One matrix row: a family label, the production `profile_chip`
/// dimensions (`profile_index`, 1–2 character monogram, hue), the §8
/// density the family renders at, and the matrix-only claims that name
/// the rejected combinations in the same case shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileChipCase {
    pub label: &'static str,
    /// The §8 density (drives the §12 chip size: S 20 / N 24 / XL 28).
    pub density: Density,
    /// The creation-order profile index — the `bbr-profile-chip-<n>` key
    /// (§12; never the space index).
    pub profile_index: usize,
    /// The §12 1–2 character monogram (the non-color identity companion).
    pub monogram: &'static str,
    /// The §12 round-robin hue the domain assigned to the profile.
    pub hue: ProfileHue,
    /// Matrix-only: this case claims the chip is keyed to a *space* index
    /// (§12 rejects it — the chip key is the profile index).
    pub claims_space_keyed: bool,
    /// Matrix-only: this case claims the removed legacy
    /// `space_row` / `space_new_row` list anatomy (§11.4 — that list is
    /// gone; the permanent strip renders identity chips instead).
    pub claims_legacy_row: bool,
}

impl ProfileChipCase {
    /// The valid-family constructor.
    #[allow(clippy::too_many_arguments)]
    const fn new(
        label: &'static str,
        density: Density,
        profile_index: usize,
        monogram: &'static str,
        hue: ProfileHue,
    ) -> Self {
        ProfileChipCase {
            label,
            density,
            profile_index,
            monogram,
            hue,
            claims_space_keyed: false,
            claims_legacy_row: false,
        }
    }
}

/// Every valid profile-chip family — the single declaration the story
/// renders. The density families cover the §12 chip sizes (S 20 / N 24 /
/// XL 28), the monogram families cover the 1- and 2-character companions,
/// the hue families cover the six §12 round-robin tiles, and the
/// profile-keyed-id family pins the `bbr-profile-chip-<profile_index>`
/// keying at a distinct index.
#[rustfmt::skip]
pub const PROFILE_CHIP_VALID: &[ProfileChipCase] = &[
    ProfileChipCase::new("S density — 20 pt", Density::Small, 0, "Co", ProfileHue::Cobalt),
    ProfileChipCase::new("N density — 24 pt", Density::Normal, 1, "Vi", ProfileHue::Violet),
    ProfileChipCase::new("XL density — 28 pt", Density::ExtraLarge, 2, "Fu", ProfileHue::Fuchsia),
    ProfileChipCase::new("1-character monogram", Density::Normal, 3, "W", ProfileHue::Amber),
    ProfileChipCase::new("2-character monogram", Density::Normal, 4, "Wo", ProfileHue::Amber),
    ProfileChipCase::new("hue tile — Cobalt", Density::Normal, 0, "Co", ProfileHue::Cobalt),
    ProfileChipCase::new("hue tile — Violet", Density::Normal, 1, "Vi", ProfileHue::Violet),
    ProfileChipCase::new("hue tile — Fuchsia", Density::Normal, 2, "Fu", ProfileHue::Fuchsia),
    ProfileChipCase::new("hue tile — Amber", Density::Normal, 3, "Am", ProfileHue::Amber),
    ProfileChipCase::new("hue tile — Terracotta", Density::Normal, 4, "Te", ProfileHue::Terracotta),
    ProfileChipCase::new("hue tile — Slate", Density::Normal, 5, "Sl", ProfileHue::Slate),
    ProfileChipCase::new("profile-keyed id", Density::Normal, 8, "Pr", ProfileHue::Slate),
];

/// A rejected combination with the exact DESIGN/packet reason (the matrix
/// "rejected" column). Same `ProfileChipCase` shape as the valid rows, so
/// one validity authority decides both sides and the table cannot drift.
#[derive(Debug, Clone, Copy)]
pub struct RejectedProfileChip {
    pub label: &'static str,
    pub case: ProfileChipCase,
    pub reason: &'static str,
}

/// The matrix "rejected" column. Every entry must fail
/// [`profile_chip_case_valid`] and thus refuse fixture construction.
#[rustfmt::skip]
pub const PROFILE_CHIP_REJECTED: &[RejectedProfileChip] = &[
    RejectedProfileChip {
        label: "a chip keyed to a space index",
        case: ProfileChipCase {
            label: "a chip keyed to a space index",
            density: Density::Normal,
            profile_index: 0,
            monogram: "Wo",
            hue: ProfileHue::Cobalt,
            claims_space_keyed: true,
            claims_legacy_row: false,
        },
        reason: "§12: the chip key is the profile's creation-order index (bbr-profile-chip-<profile_index>) — never the space index; the space strip's own ids are G3-owned",
    },
    RejectedProfileChip {
        label: "legacy space_row / space_new_row list anatomy",
        case: ProfileChipCase {
            label: "legacy space_row / space_new_row list anatomy",
            density: Density::Normal,
            profile_index: 0,
            monogram: "Wo",
            hue: ProfileHue::Cobalt,
            claims_space_keyed: false,
            claims_legacy_row: true,
        },
        reason: "§11.4: the legacy space-row list and its [Spaces] toggle are removed — the permanent strip renders identity chips instead",
    },
];

/// The one profile-chip validity authority: rejects the impossible
/// combinations the matrix forbids. The fixture builder refuses (`None`)
/// any case this rejects, so the story can never render a rejected chip.
pub fn profile_chip_case_valid(case: &ProfileChipCase) -> bool {
    // §12: the chip key is the profile index, never the space index.
    if case.claims_space_keyed {
        return false;
    }
    // §11.4: the legacy space-row list anatomy is removed.
    if case.claims_legacy_row {
        return false;
    }
    // §12: the monogram is 1–2 characters (never a full title).
    let chars = case.monogram.chars().count();
    if chars == 0 || chars > 2 {
        return false;
    }
    true
}

/// Build the production `ProfileChipSpec` (+ the profile index that keys
/// the `bbr-profile-chip-<n>` id) for a valid matrix family; `None` for
/// any rejected combination.
pub fn build_profile_chip_fixture(case: &ProfileChipCase) -> Option<(usize, ProfileChipSpec)> {
    if !profile_chip_case_valid(case) {
        return None;
    }
    Some((
        case.profile_index,
        ProfileChipSpec {
            letter: case.monogram.to_string(),
            hue: case.hue,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_ui::components::space::profile_chip_id;
    use std::collections::BTreeSet;

    #[test]
    fn every_valid_family_builds_and_every_rejected_combination_fails() {
        for case in PROFILE_CHIP_VALID {
            assert!(
                profile_chip_case_valid(case),
                "valid family rejected: {}",
                case.label
            );
            let (index, spec) = build_profile_chip_fixture(case)
                .unwrap_or_else(|| panic!("valid family builds: {}", case.label));
            assert_eq!(index, case.profile_index, "{}", case.label);
            assert_eq!(spec.hue, case.hue, "{}", case.label);
            assert_eq!(spec.letter, case.monogram, "{}", case.label);
            // The §12 keying: bbr-profile-chip-<profile_index>.
            assert_eq!(
                profile_chip_id(index).to_string(),
                format!("bbr-profile-chip-{}", case.profile_index),
                "{}",
                case.label
            );
        }
        for rejected in PROFILE_CHIP_REJECTED {
            assert!(
                !profile_chip_case_valid(&rejected.case),
                "rejected combination accepted: {}",
                rejected.label
            );
            assert!(
                build_profile_chip_fixture(&rejected.case).is_none(),
                "rejected combination builds a fixture: {}",
                rejected.label
            );
        }
    }

    #[test]
    fn matrix_is_the_one_stable_declaration_source() {
        let labels: Vec<&str> = PROFILE_CHIP_VALID.iter().map(|c| c.label).collect();
        assert_eq!(
            labels,
            vec![
                "S density — 20 pt",
                "N density — 24 pt",
                "XL density — 28 pt",
                "1-character monogram",
                "2-character monogram",
                "hue tile — Cobalt",
                "hue tile — Violet",
                "hue tile — Fuchsia",
                "hue tile — Amber",
                "hue tile — Terracotta",
                "hue tile — Slate",
                "profile-keyed id",
            ]
        );
        let mut seen = BTreeSet::new();
        for case in PROFILE_CHIP_VALID {
            assert!(seen.insert(case.label), "duplicate label {}", case.label);
        }
    }

    #[test]
    fn density_families_cover_the_section_12_chip_sizes() {
        // §12: 24 pt square (20 at S, 28 at XL).
        let by_label = |label: &str| {
            PROFILE_CHIP_VALID
                .iter()
                .find(|c| c.label == label)
                .unwrap_or_else(|| panic!("family {label}"))
        };
        assert_eq!(by_label("S density — 20 pt").density, Density::Small);
        assert_eq!(by_label("N density — 24 pt").density, Density::Normal);
        assert_eq!(by_label("XL density — 28 pt").density, Density::ExtraLarge);
    }

    #[test]
    fn monograms_are_1_to_2_characters_and_hues_cover_the_round_robin() {
        // §12: the chip always carries the 1–2 character monogram (the
        // non-color identity companion); hues come round-robin from the
        // fixed six-token set.
        let hues: std::collections::HashSet<ProfileHue> = PROFILE_CHIP_VALID
            .iter()
            .filter(|c| c.label.starts_with("hue tile"))
            .map(|c| c.hue)
            .collect();
        assert_eq!(hues.len(), 6, "all six §12 hues are storyable");
        for case in PROFILE_CHIP_VALID {
            assert!(
                (1..=2).contains(&case.monogram.chars().count()),
                "1–2 char monogram: {}",
                case.label
            );
        }
    }

    #[test]
    fn profile_keyed_id_family_uses_a_distinct_profile_index() {
        // The §12 profile-keyed id family pins the keying at a distinct
        // creation-order index — `bbr-profile-chip-8` is unique across the
        // story (the E2E target).
        let case = PROFILE_CHIP_VALID
            .iter()
            .find(|c| c.label == "profile-keyed id")
            .expect("profile-keyed id family declared");
        assert_eq!(case.profile_index, 8);
        let (index, _) = build_profile_chip_fixture(case).unwrap();
        assert_eq!(profile_chip_id(index).to_string(), "bbr-profile-chip-8");
    }

    #[test]
    fn rejected_rules_cover_the_full_matrix_rejected_column() {
        let labels: Vec<&str> = PROFILE_CHIP_REJECTED.iter().map(|r| r.label).collect();
        assert_eq!(
            labels,
            vec![
                "a chip keyed to a space index",
                "legacy space_row / space_new_row list anatomy",
            ]
        );
        for rejected in PROFILE_CHIP_REJECTED {
            assert!(!rejected.reason.is_empty());
        }
    }
}
