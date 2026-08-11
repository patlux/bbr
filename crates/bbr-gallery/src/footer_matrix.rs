//! The Footer matrix authority (packet
//! `component-gallery-browser-components.md`, Group 4): **one declaration
//! source** for every valid §11.4 footer family and every rejected
//! combination. The gallery story renders exactly the valid families
//! through the production `bbr_ui::components::footer` `footer_surface`
//! with a **fixture-built strip** shown in context, and the fixture
//! builder refuses (`None`) any case the validity authority rejects — so
//! the story, the fixture builder, the matrix table, and the tests cannot
//! drift. The rejected reasons also live here.
//!
//! Matrix rows come straight from the packet's Footer row: the
//! `[Downloads]` control with/without the §11.4 running `tnum` count; the
//! permanent §11.3 space strip in context — one chip, many chips, overflow
//! clip (single line, horizontal scroll); the active chip's selected
//! surface + outline; inactive chips quiet; and the `[+]` control.
//! Rejected: a wrapped/second-line strip (§11.4 single-line clip) and a
//! chip without its accessible name containing space+profile names (§11.3).
//!
//! The strip fixture is the **allowed strip**: a presentation-only,
//! single-line horizontal scroll row of §12 profile chips shown inside the
//! production footer surface "in context". It is not a standalone
//! `space_row` story and carries **no navigation authority** (no space
//! switch routing, no scroll handle, no strip focus) — the real permanent
//! strip composition is G3-owned. The rejected rule "a Group-4 fixture
//! that creates a second strip or navigation authority" therefore also
//! covers any fixture that would duplicate that composition here.

use bbr_domain::ProfileHue;
use bbr_ui::components::footer::FOOTER_STRIP_ID;
use bbr_ui::components::{profile_chip, ProfileChipSpec};
use bbr_ui::theme::{Spacing, Theme};
use gpui::{div, prelude::*, SharedString, Stateful, Styled};

/// One §11.3 strip-chip fixture: the §12 profile identity (creation-order
/// index, 1–2 character monogram, hue tile) plus the space and profile
/// names that make up the chip's accessible name (§11.3 — identity is
/// never color-only), and whether the chip renders the §11.3 active
/// selected surface + outline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StripChip {
    pub profile_index: usize,
    pub monogram: &'static str,
    pub hue: ProfileHue,
    pub space: &'static str,
    pub profile: &'static str,
    pub active: bool,
}

/// The fixture strip chips the footer families declare: distinct profile
/// indices (so the §12 chip ids stay unique within a family), 1–2
/// character monograms, and space+profile accessible names.
const CHIP: StripChip = StripChip {
    profile_index: 0,
    monogram: "Wo",
    hue: ProfileHue::Cobalt,
    space: "Work",
    profile: "Default",
    active: false,
};

/// One matrix row: a family label, the production `footer_surface`
/// dimensions, and the matrix-only claims that name the rejected
/// combinations in the same case shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FooterCase {
    pub label: &'static str,
    /// The §11.4 `[Downloads]` `tnum` count (`None` = no running count —
    /// the count feeds in-flight attempts only).
    pub downloads_count: Option<usize>,
    /// The strip chips this family shows in context (one / many / the
    /// overflow family).
    pub chips: &'static [StripChip],
    /// Matrix-only: this case claims a wrapped/second-line strip (§11.4 —
    /// the strip is a single-line clip, never a wrapping row).
    pub claims_wrapped: bool,
    /// Matrix-only: this case claims a chip without its accessible name
    /// containing space+profile names (§11.3 — identity never relies on
    /// color alone).
    pub claims_unnamed_chip: bool,
}

/// The chips each footer family renders: `(profile_index, monogram, hue,
/// space, profile, active)`.
const fn chip(
    profile_index: usize,
    monogram: &'static str,
    hue: ProfileHue,
    space: &'static str,
    profile: &'static str,
    active: bool,
) -> StripChip {
    StripChip {
        profile_index,
        monogram,
        hue,
        space,
        profile,
        active,
    }
}

/// Every valid footer family — the single declaration the story renders.
/// The `[+]` control is part of every footer anatomy (§11.4: exactly
/// `[Downloads] [strip] [+]`); the "plus control" family pins that
/// contract explicitly.
#[rustfmt::skip]
pub const FOOTER_VALID: &[FooterCase] = &[
    FooterCase {
        label: "downloads — no running count",
        downloads_count: None,
        chips: &[chip(0, "Wo", ProfileHue::Cobalt, "Work", "Default", false), chip(1, "Re", ProfileHue::Violet, "Reading", "Private", false)],
        claims_wrapped: false,
        claims_unnamed_chip: false,
    },
    FooterCase {
        label: "downloads — running count",
        downloads_count: Some(3),
        chips: &[chip(0, "Wo", ProfileHue::Cobalt, "Work", "Default", false), chip(1, "Re", ProfileHue::Violet, "Reading", "Private", false)],
        claims_wrapped: false,
        claims_unnamed_chip: false,
    },
    FooterCase {
        label: "strip — one chip",
        downloads_count: None,
        chips: &[chip(0, "Wo", ProfileHue::Cobalt, "Work", "Default", false)],
        claims_wrapped: false,
        claims_unnamed_chip: false,
    },
    FooterCase {
        label: "strip — many chips",
        downloads_count: None,
        chips: &[chip(0, "Wo", ProfileHue::Cobalt, "Work", "Default", false), chip(1, "Re", ProfileHue::Violet, "Reading", "Private", false), chip(2, "Mu", ProfileHue::Fuchsia, "Music", "Work", false), chip(3, "De", ProfileHue::Amber, "Dev", "Default", false)],
        claims_wrapped: false,
        claims_unnamed_chip: false,
    },
    FooterCase {
        label: "strip — overflow clip (single line)",
        downloads_count: None,
        chips: &[chip(0, "Wo", ProfileHue::Cobalt, "Work", "Default", false), chip(1, "Re", ProfileHue::Violet, "Reading", "Private", false), chip(2, "Mu", ProfileHue::Fuchsia, "Music", "Work", false), chip(3, "De", ProfileHue::Amber, "Dev", "Default", false), chip(4, "Sl", ProfileHue::Terracotta, "Slate", "Private", false), chip(5, "Ar", ProfileHue::Slate, "Archive", "Work", false), chip(6, "Fi", ProfileHue::Cobalt, "Files", "Default", false), chip(7, "In", ProfileHue::Violet, "Inbox", "Private", false), chip(8, "Ca", ProfileHue::Fuchsia, "Calendar", "Work", false), chip(9, "No", ProfileHue::Amber, "Notes", "Default", false), chip(10, "So", ProfileHue::Terracotta, "Social", "Private", false), chip(11, "Tr", ProfileHue::Slate, "Travel", "Work", false)],
        claims_wrapped: false,
        claims_unnamed_chip: false,
    },
    FooterCase {
        label: "active chip — selected surface + outline",
        downloads_count: None,
        chips: &[chip(0, "Wo", ProfileHue::Cobalt, "Work", "Default", true), chip(1, "Re", ProfileHue::Violet, "Reading", "Private", false)],
        claims_wrapped: false,
        claims_unnamed_chip: false,
    },
    FooterCase {
        label: "inactive chips — quiet",
        downloads_count: None,
        chips: &[chip(0, "Wo", ProfileHue::Cobalt, "Work", "Default", false), chip(1, "Re", ProfileHue::Violet, "Reading", "Private", false), chip(2, "Mu", ProfileHue::Fuchsia, "Music", "Work", false)],
        claims_wrapped: false,
        claims_unnamed_chip: false,
    },
    FooterCase {
        label: "plus control",
        downloads_count: None,
        chips: &[chip(0, "Wo", ProfileHue::Cobalt, "Work", "Default", false), chip(1, "Re", ProfileHue::Violet, "Reading", "Private", false)],
        claims_wrapped: false,
        claims_unnamed_chip: false,
    },
];

/// A rejected combination with the exact DESIGN/packet reason (the matrix
/// "rejected" column). Same `FooterCase` shape as the valid rows, so one
/// validity authority decides both sides and the table cannot drift.
#[derive(Debug, Clone, Copy)]
pub struct RejectedFooter {
    pub label: &'static str,
    pub case: FooterCase,
    pub reason: &'static str,
}

/// The matrix "rejected" column. Every entry must fail
/// [`footer_case_valid`] and thus refuse fixture construction.
#[rustfmt::skip]
pub const FOOTER_REJECTED: &[RejectedFooter] = &[
    RejectedFooter {
        label: "wrapped / second-line strip",
        case: FooterCase {
            label: "wrapped / second-line strip",
            downloads_count: None,
            chips: &[CHIP, CHIP],
            claims_wrapped: true,
            claims_unnamed_chip: false,
        },
        reason: "§11.4/§11.3: the strip is a single-line clip that scrolls horizontally — it never wraps to a second line and never changes the vertical region stack",
    },
    RejectedFooter {
        label: "chip without space+profile accessible name",
        case: FooterCase {
            label: "chip without space+profile accessible name",
            downloads_count: None,
            chips: &[StripChip {
                profile_index: 0,
                monogram: "Wo",
                hue: ProfileHue::Cobalt,
                space: "",
                profile: "",
                active: false,
            }],
            claims_wrapped: false,
            claims_unnamed_chip: true,
        },
        reason: "§11.3: every chip has a stable accessible name containing the space and profile names — color or emoji alone never carries that meaning",
    },
];

/// The one footer validity authority: rejects the impossible combinations
/// the matrix forbids. The fixture builder refuses (`None`) any case this
/// rejects, so the story can never render a rejected footer.
pub fn footer_case_valid(case: &FooterCase) -> bool {
    // §11.4/§11.3: the strip is a single-line clip, never a wrapping row.
    if case.claims_wrapped {
        return false;
    }
    // §11.3: identity never relies on color alone — every chip's
    // accessible name contains the space and profile names.
    if case.claims_unnamed_chip {
        return false;
    }
    // A chip with an empty space or profile name is the unnamed-chip
    // impossibility expressed through the data.
    if case
        .chips
        .iter()
        .any(|c| c.space.is_empty() || c.profile.is_empty())
    {
        return false;
    }
    true
}

/// Build the allowed strip fixture for a valid footer family: the §11.4
/// single-line horizontal scroll row (`bbr-space-strip`) of §12 profile
/// chips, shown **in context** inside the production `footer_surface`.
/// `None` for any rejected combination.
///
/// Each chip renders the production `profile_chip` (the §12 hue tile +
/// 1–2 character monogram at `bbr-profile-chip-<profile_index>`) plus a
/// text companion whose label is the chip's accessible name — the space
/// and profile names (§11.3). The active chip (§11.3) adds the persistent
/// selected surface + focus-independent accent outline; inactive chips
/// stay quiet. The fixture is presentation-only: no navigation authority,
/// no scroll handle, no strip focus — the real permanent strip is
/// G3-owned.
pub fn build_footer_strip(theme: &Theme, case: &FooterCase) -> Option<Stateful<gpui::Div>> {
    if !footer_case_valid(case) {
        return None;
    }
    let chips: Vec<gpui::AnyElement> = case
        .chips
        .iter()
        .map(|c| {
            let spec = ProfileChipSpec {
                letter: c.monogram.to_string(),
                hue: c.hue,
            };
            let chip = profile_chip(theme, c.profile_index, &spec);
            let chip: gpui::AnyElement = if c.active {
                // §11.3 active chip: the persistent selected surface plus
                // a focus-independent accent outline.
                chip.border_1()
                    .border_color(theme.palette.accent)
                    .rounded(theme.radius.small())
                    .into_any()
            } else {
                chip.into_any()
            };
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(Spacing::sp2())
                // The chip's accessible name: the space and profile names
                // (§11.3 — identity is never color-only).
                .child(chip)
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .font_weight(theme.type_.meta.weight)
                        .text_color(theme.palette.ink_secondary)
                        .child(SharedString::from(format!("{} · {}", c.space, c.profile))),
                )
                .into_any()
        })
        .collect();
    // §11.4/§11.3: the permanent strip is a single-line horizontal scroll
    // viewport that consumes the footer's middle width (`flex_1`) and
    // never wraps or shrinks the fixed controls.
    Some(
        div()
            .id(FOOTER_STRIP_ID)
            .debug_selector(|| FOOTER_STRIP_ID.to_string())
            .flex_1()
            .min_w_0()
            .h(theme.density.row())
            .gap(Spacing::sp2())
            .items_center()
            .flex()
            .flex_row()
            .overflow_x_scroll()
            .children(chips),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_ui::theme::{ContrastMode, Density, ThemeMode};
    use std::collections::BTreeSet;

    fn fixture_theme() -> Theme {
        Theme::new(ThemeMode::Dark, ContrastMode::Standard, Density::Normal)
    }

    #[test]
    fn every_valid_family_builds_and_every_rejected_combination_fails() {
        let theme = fixture_theme();
        for case in FOOTER_VALID {
            assert!(
                footer_case_valid(case),
                "valid family rejected: {}",
                case.label
            );
            assert!(
                build_footer_strip(&theme, case).is_some(),
                "valid family builds: {}",
                case.label
            );
            for c in case.chips {
                assert!(
                    c.monogram.chars().count() <= 2 && !c.monogram.is_empty(),
                    "1–2 char monogram: {}",
                    case.label
                );
                assert!(
                    !c.space.is_empty() && !c.profile.is_empty(),
                    "{}",
                    case.label
                );
            }
        }
        for rejected in FOOTER_REJECTED {
            assert!(
                !footer_case_valid(&rejected.case),
                "rejected combination accepted: {}",
                rejected.label
            );
            assert!(
                build_footer_strip(&theme, &rejected.case).is_none(),
                "rejected combination builds a fixture: {}",
                rejected.label
            );
        }
    }

    #[test]
    fn matrix_is_the_one_stable_declaration_source() {
        let labels: Vec<&str> = FOOTER_VALID.iter().map(|c| c.label).collect();
        assert_eq!(
            labels,
            vec![
                "downloads — no running count",
                "downloads — running count",
                "strip — one chip",
                "strip — many chips",
                "strip — overflow clip (single line)",
                "active chip — selected surface + outline",
                "inactive chips — quiet",
                "plus control",
            ]
        );
        let mut seen = BTreeSet::new();
        for case in FOOTER_VALID {
            assert!(seen.insert(case.label), "duplicate label {}", case.label);
        }
    }

    #[test]
    fn strip_chip_profile_indices_are_unique_within_each_family() {
        // The §12 chip id `bbr-profile-chip-<profile_index>` stays unique
        // within one strip (the family's chips never collide).
        for case in FOOTER_VALID {
            let mut seen = BTreeSet::new();
            for c in case.chips {
                assert!(
                    seen.insert(c.profile_index),
                    "duplicate profile index {} in {}",
                    c.profile_index,
                    case.label
                );
            }
        }
    }

    #[test]
    fn overflow_family_declares_more_chips_than_one_line_of_quiet() {
        // The overflow family is the many-chips family grown past the
        // viewport width (single line, horizontal scroll) — it declares
        // the most chips and keeps every accessible name.
        let by_label = |label: &str| {
            FOOTER_VALID
                .iter()
                .find(|c| c.label == label)
                .unwrap_or_else(|| panic!("family {label}"))
        };
        let overflow = by_label("strip — overflow clip (single line)");
        assert!(
            overflow.chips.len() > by_label("strip — many chips").chips.len(),
            "the overflow family grows the strip"
        );
        let mut seen = BTreeSet::new();
        for c in overflow.chips {
            assert!(seen.insert(c.profile_index));
            assert!(c.monogram.chars().count() <= 2);
        }
    }

    #[test]
    fn active_chip_family_carries_exactly_one_active_chip() {
        let by_label = |label: &str| {
            FOOTER_VALID
                .iter()
                .find(|c| c.label == label)
                .unwrap_or_else(|| panic!("family {label}"))
        };
        let active = by_label("active chip — selected surface + outline");
        assert_eq!(
            active.chips.iter().filter(|c| c.active).count(),
            1,
            "exactly one active chip (§11.3: at most one identity is loud)"
        );
        let quiet = by_label("inactive chips — quiet");
        assert!(
            quiet.chips.iter().all(|c| !c.active),
            "quiet chips carry no active surface"
        );
    }

    #[test]
    fn rejected_rules_cover_the_full_matrix_rejected_column() {
        let labels: Vec<&str> = FOOTER_REJECTED.iter().map(|r| r.label).collect();
        assert_eq!(
            labels,
            vec![
                "wrapped / second-line strip",
                "chip without space+profile accessible name",
            ]
        );
        for rejected in FOOTER_REJECTED {
            assert!(!rejected.reason.is_empty());
        }
    }
}
