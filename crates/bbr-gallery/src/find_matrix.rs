//! The Find-bar matrix authority (packet
//! `component-gallery-browser-components.md`, Group 6): **one declaration
//! source** for every valid §11.4 find-bar family and every rejected
//! combination. The gallery story renders exactly the valid families
//! through the production `bbr_ui::surfaces::find` symbols
//! (`find_bar`, `FindSurface`, `FindBarRect`), and the fixture builder
//! refuses (`None`) any case the validity authority rejects — so the
//! story, the fixture builder, the matrix table, and the tests cannot
//! drift. The rejected reasons also live here.
//!
//! Matrix rows come straight from the packet's Find bar row: open/focused
//! (the §9 chrome-focus ring on the field) vs open/unfocused; query empty
//! (the "Find in page" placeholder) vs typed; count/ordinal ("n of m");
//! 0 of 0; the prev/next/close controls at `density:hit-min(_close)`.
//! Rejected: an ordinal without a count (WebKit reports both or neither —
//! the portable find contract never reports an ordinal without a total),
//! and a "next" enabled at 0 matches (no matches → no step: the portable
//! session rejects navigation at zero matches and the v1 §11.4
//! command-surface controls carry no enabled/disabled state).
//!
//! The packet's "stale flag" is **lifecycle state, not a presentation
//! variant**: the app's `FindState.stale` keeps the bar open with its last
//! report until a fresh query starts; the presentation DTO (the anchor
//! extraction table's five fields — focused, query, match_count,
//! current_match, plus the caller's theme) carries no stale flag and the
//! gallery never fakes a stale visual. The story states this honestly.

use bbr_ui::surfaces::find::{FindBarRect, FindSurface};
use bbr_ui::theme::Theme;

/// The gallery fixture bar width: the §11.4 command-surface compact clamp
/// minimum (the production `FIND_BAR_WIDTH` constant in `bbr-app`; gallery
/// fixtures cannot import the app, so the fixture declares the same
/// settled chrome geometry).
pub const FIND_FAMILY_WIDTH: f32 = 360.0;

/// One matrix row: a family label and the production `FindSurface`
/// fields. The matrix-only flags (`claims_ordinal_without_count`,
/// `claims_next_enabled_at_zero`) exist solely to name the rejected
/// combinations in the same case shape; they are never mapped into
/// `FindSurface`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindCase {
    pub label: &'static str,
    /// §9: whether the find field owns keyboard focus (the chrome-focus
    /// ring on the field).
    pub focused: bool,
    /// The live query text (empty renders the "Find in page" placeholder).
    pub query: &'static str,
    /// Total matches reported by WebKit (`None` until a result lands).
    pub match_count: Option<usize>,
    /// Current 1-based ordinal (`None` for zero matches / no result).
    pub current_match: Option<usize>,
    /// Matrix-only: this case claims an ordinal without a count (WebKit
    /// reports both or neither — rejected).
    pub claims_ordinal_without_count: bool,
    /// Matrix-only: this case claims a "next" control enabled at 0
    /// matches (no matches → no step — rejected).
    pub claims_next_enabled_at_zero: bool,
}

impl FindCase {
    /// The valid-family constructor: the five production fields with the
    /// matrix-only flags defaulting to false. `#[allow]` keeps the dense
    /// const table readable (the fields are the documented matrix row).
    #[allow(clippy::too_many_arguments)]
    const fn new(
        label: &'static str,
        focused: bool,
        query: &'static str,
        match_count: Option<usize>,
        current_match: Option<usize>,
    ) -> Self {
        FindCase {
            label,
            focused,
            query,
            match_count,
            current_match,
            claims_ordinal_without_count: false,
            claims_next_enabled_at_zero: false,
        }
    }
}

/// Every valid find-bar family — the single declaration the story renders.
/// The focused/unfocused, empty/typed query, count/ordinal, and 0-of-0
/// dimensions are covered: the focused open bar with the empty-query
/// placeholder, a typed query with no report yet (the count area renders
/// empty until WebKit reports), the "n of m" count/ordinal, the last
/// ordinal ("m of m"), the zero-match "0 of 0" report, and the unfocused
/// open bar (the §9 chrome-focus ring leaves the field).
#[rustfmt::skip]
pub const FIND_VALID: &[FindCase] = &[
    FindCase::new("open · focused · empty query (placeholder)", true, "", None, None),
    FindCase::new("open · focused · typed query, no result yet", true, "find", None, None),
    FindCase::new("open · focused · count/ordinal (n of m)", true, "find", Some(5), Some(2)),
    FindCase::new("open · focused · last ordinal (m of m)", true, "find", Some(5), Some(5)),
    FindCase::new("open · focused · zero matches (0 of 0)", true, "zzz", Some(0), None),
    FindCase::new("open · unfocused (chrome focus elsewhere)", false, "find", Some(3), Some(1)),
];

/// A rejected combination with the exact DESIGN/packet reason (the matrix
/// "rejected" column). Same `FindCase` shape as the valid rows, so one
/// validity authority decides both sides and the table cannot drift.
#[derive(Debug, Clone, Copy)]
pub struct RejectedFind {
    pub label: &'static str,
    pub case: FindCase,
    pub reason: &'static str,
}

/// The matrix "rejected" column. Every entry must fail
/// [`find_case_valid`] and thus refuse fixture construction.
#[rustfmt::skip]
pub const FIND_REJECTED: &[RejectedFind] = &[
    RejectedFind {
        label: "an ordinal without a count",
        case: FindCase {
            label: "ordinal without count",
            focused: true,
            query: "find",
            match_count: None,
            current_match: Some(2),
            claims_ordinal_without_count: true,
            claims_next_enabled_at_zero: false,
        },
        reason: "WebKit reports both or neither: the portable find contract never reports an ordinal without a total (the adapter echoes count+ordinal together), so a lone ordinal refuses",
    },
    RejectedFind {
        label: "a next enabled at 0 matches",
        case: FindCase {
            label: "next enabled at zero matches",
            focused: true,
            query: "zzz",
            match_count: Some(0),
            current_match: None,
            claims_ordinal_without_count: false,
            claims_next_enabled_at_zero: true,
        },
        reason: "no matches → no step: the portable session rejects navigation at zero matches, and the v1 §11.4 command-surface controls carry no enabled/disabled state — a next-enabled-at-zero claim refuses",
    },
];

/// The one find-bar validity authority: rejects the impossible
/// combinations the matrix forbids. The fixture builder refuses (`None`)
/// any case this rejects, so the story can never render a rejected bar.
pub fn find_case_valid(case: &FindCase) -> bool {
    // WebKit reports both or neither: no ordinal without a count.
    if case.claims_ordinal_without_count {
        return false;
    }
    // No matches → no step: no enabled-next-at-zero claim.
    if case.claims_next_enabled_at_zero {
        return false;
    }
    true
}

/// Build the production `FindSurface` for a valid matrix family; `None`
/// for any rejected combination. The count label derives through the
/// production `bbr_ui::surfaces::find::count_label` the render uses, so
/// the fixture and the production display can never drift.
pub fn build_find_fixture(case: &FindCase) -> Option<FindSurface> {
    if !find_case_valid(case) {
        return None;
    }
    Some(FindSurface {
        focused: case.focused,
        query: case.query.to_string(),
        match_count: case.match_count,
        current_match: case.current_match,
    })
}

/// The production `FindBarRect` for one gallery family: the fixture bar at
/// the top-left of its test-target container, sized to the §11.4 compact
/// clamp width and the settled `density:row` height token.
pub fn find_family_rect(theme: &Theme) -> FindBarRect {
    FindBarRect {
        x: 0.0,
        y: 0.0,
        width: FIND_FAMILY_WIDTH,
        height: f32::from(theme.density.row()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_ui::surfaces::find::count_label;
    use bbr_ui::theme::{ContrastMode, Density, ThemeMode};
    use std::collections::BTreeSet;

    /// The representative fixture theme (the canvas default).
    fn fixture_theme() -> Theme {
        Theme::new(ThemeMode::Dark, ContrastMode::Standard, Density::Normal)
    }

    #[test]
    fn every_valid_family_builds_and_every_rejected_combination_fails() {
        for case in FIND_VALID {
            assert!(
                find_case_valid(case),
                "valid family rejected: {}",
                case.label
            );
            let fixture = build_find_fixture(case)
                .unwrap_or_else(|| panic!("valid family builds: {}", case.label));
            assert_eq!(fixture.focused, case.focused, "{}", case.label);
            assert_eq!(fixture.query, case.query, "{}", case.label);
            assert_eq!(fixture.match_count, case.match_count, "{}", case.label);
            assert_eq!(fixture.current_match, case.current_match, "{}", case.label);
        }
        for rejected in FIND_REJECTED {
            assert!(
                !find_case_valid(&rejected.case),
                "rejected combination accepted: {}",
                rejected.label
            );
            assert!(
                build_find_fixture(&rejected.case).is_none(),
                "rejected combination builds a fixture: {}",
                rejected.label
            );
        }
    }

    #[test]
    fn matrix_is_the_one_stable_declaration_source() {
        // The story renders exactly these families in this order; the E2E
        // pins the same stable ids. Reordering or removing a family fails
        // here — the no-drift contract.
        let labels: Vec<&str> = FIND_VALID.iter().map(|c| c.label).collect();
        assert_eq!(
            labels,
            vec![
                "open · focused · empty query (placeholder)",
                "open · focused · typed query, no result yet",
                "open · focused · count/ordinal (n of m)",
                "open · focused · last ordinal (m of m)",
                "open · focused · zero matches (0 of 0)",
                "open · unfocused (chrome focus elsewhere)",
            ]
        );
        // Labels are unique (ids derive from position).
        let mut seen = BTreeSet::new();
        for case in FIND_VALID {
            assert!(seen.insert(case.label), "duplicate label {}", case.label);
        }
        let rejected_labels: Vec<&str> = FIND_REJECTED.iter().map(|r| r.label).collect();
        assert_eq!(
            rejected_labels,
            vec!["an ordinal without a count", "a next enabled at 0 matches"]
        );
        for rejected in FIND_REJECTED {
            assert!(!rejected.reason.is_empty());
        }
    }

    #[test]
    fn fixtures_derive_the_count_copy_through_production_count_label() {
        // The fixture maps to the production `FindSurface` and the count
        // area renders through the production `count_label` — the same
        // derivation the portal render uses.
        let by_label = |label: &str| {
            FIND_VALID
                .iter()
                .find(|c| c.label == label)
                .unwrap_or_else(|| panic!("family {label}"))
        };
        let empty =
            build_find_fixture(by_label("open · focused · empty query (placeholder)")).unwrap();
        assert!(empty.focused);
        assert!(empty.query.is_empty());
        assert_eq!(count_label(&empty), "", "empty query shows no count");
        let pending =
            build_find_fixture(by_label("open · focused · typed query, no result yet")).unwrap();
        assert_eq!(count_label(&pending), "", "no report yet renders empty");
        let ordinal =
            build_find_fixture(by_label("open · focused · count/ordinal (n of m)")).unwrap();
        assert_eq!(count_label(&ordinal), "2 of 5");
        let last = build_find_fixture(by_label("open · focused · last ordinal (m of m)")).unwrap();
        assert_eq!(count_label(&last), "5 of 5");
        let zero = build_find_fixture(by_label("open · focused · zero matches (0 of 0)")).unwrap();
        assert_eq!(count_label(&zero), "0 of 0");
        let unfocused =
            build_find_fixture(by_label("open · unfocused (chrome focus elsewhere)")).unwrap();
        assert!(!unfocused.focused);
        assert_eq!(count_label(&unfocused), "1 of 3");
    }

    #[test]
    fn family_rect_uses_the_settled_chrome_tokens() {
        let theme = fixture_theme();
        let rect = find_family_rect(&theme);
        assert_eq!(rect.x, 0.0);
        assert_eq!(rect.y, 0.0);
        assert_eq!(rect.width, FIND_FAMILY_WIDTH);
        assert_eq!(rect.height, 36.0, "density:row at Normal");
        // The rect follows the theme density.
        let xl = Theme::new(ThemeMode::Dark, ContrastMode::Standard, Density::ExtraLarge);
        assert_eq!(find_family_rect(&xl).height, 44.0, "density:row at XL");
    }

    #[test]
    fn the_stale_dimension_is_lifecycle_state_only() {
        // The packet matrix's "stale flag" is lifecycle state, not a
        // presentation variant: the app's FindState.stale keeps the bar
        // open with its last report until a fresh query, and the
        // presentation DTO (the anchor extraction table's five fields)
        // carries no stale flag. The story documents this; the matrix
        // covers exactly what the production DTO can express.
        let theme = fixture_theme();
        let _rect = find_family_rect(&theme);
        // Every valid family maps to exactly the five production fields —
        // no invented stale flag (the fixture construction is the proof:
        // FindSurface has no stale field to set).
        for case in FIND_VALID {
            let fixture = build_find_fixture(case).expect("valid family builds");
            assert_eq!(
                (
                    fixture.focused,
                    fixture.query.clone(),
                    fixture.match_count,
                    fixture.current_match,
                ),
                (
                    case.focused,
                    case.query.to_string(),
                    case.match_count,
                    case.current_match
                )
            );
        }
    }
}
