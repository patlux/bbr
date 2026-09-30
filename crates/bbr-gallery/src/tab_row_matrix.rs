//! The Tab Row matrix authority (packet
//! `component-gallery-browser-components.md`): **one declaration source**
//! for every valid §11.1/§11.2/§10/§8 family and every rejected
//! combination. The gallery story renders exactly the valid families
//! through the production `bbr_ui::components::tab_row`, and the fixture
//! builder refuses (`None`) any case the validity authority rejects — so
//! the story, the fixture builder, the matrix table, and the tests cannot
//! drift. The drag-tick families (legal vs clamped-invalid, §10.9) and the
//! rejected reasons also live here.
//!
//! Matrix rows come straight from the packet's Tab Row row: states
//! default/hover/active/selected/loading/attention/error/disabled/
//! dragging/drag-target/focused; pinned + regular; S (icon-only pinned) /
//! N / XL; long-title truncation; the favicon slot (monogram tile /
//! missing-favicon host-scheme fallback / loading-arc replacement —
//! consumed inline from the embedded G1b favicon component, never
//! re-storied); reveal-close on hover/selected; and drag-tick legal vs
//! clamped-invalid. Rejected: attention on a crashed row (error wins, §10
//! composition order), selected+dragging without the drag state machine,
//! S-density pinned with a title (icon-only, §8/§11.2), and a real favicon
//! bitmap (the tile is monogram-or-arc today, §7.2).

use bbr_ui::components::{FaviconSpec, RowVisuals};
use bbr_ui::state::TabVisualState;
use bbr_ui::text::{display_title, truncate_title};
use bbr_ui::theme::Density;
use gpui::SharedString;

/// §4 conservative fixture title budget (chars) the long-title family is
/// truncated against. The gallery owns plain fixture builders; the
/// pixel-exact cut stays in the component (`text_ellipsis`).
pub const FIXTURE_TITLE_BUDGET: usize = 32;

/// A deterministic monogram spec for the matrix's regular families.
const MONOGRAM: FaviconSpec = FaviconSpec {
    image: None,
    letter: 'E',
    hue: 2,
};

/// One matrix row: a family label, the production `RowVisuals` fields, and
/// the density the story renders the family at. The matrix-only fields
/// (`crashed`, `bitmap_favicon`, `claims_selected`) exist solely to name
/// the rejected combinations in the same case shape; they are never mapped
/// into `RowVisuals`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabRowCase {
    pub label: &'static str,
    pub state: TabVisualState,
    pub pinned: bool,
    /// The raw display title; the fixture applies the §11.9 fallback and
    /// §4 truncation through `bbr_ui::text` (the shared authority).
    pub title: &'static str,
    /// The tab URL used for the §11.9 display-title fallback.
    pub url: &'static str,
    pub favicon: Option<FaviconSpec>,
    pub reveal_close: bool,
    pub density: Density,
    /// Matrix-only: this case claims a crashed row (§10: a crashed row
    /// derives `Error`; attention/loading/… never ride a crash).
    pub crashed: bool,
    /// Matrix-only: this case claims a real bitmap favicon (§7.2: the tile
    /// is monogram-or-arc only).
    pub bitmap_favicon: bool,
    /// Matrix-only: this case claims the selected rail while dragging
    /// (§10.9: `Dragging` comes only from the drag state machine).
    pub claims_selected: bool,
}

impl TabRowCase {
    /// The valid-family constructor: the eight production fields with the
    /// matrix-only flags defaulting to false. `#[allow]` keeps the dense
    /// const table readable (the fields are the documented matrix row).
    #[allow(clippy::too_many_arguments)]
    const fn new(
        label: &'static str,
        state: TabVisualState,
        pinned: bool,
        title: &'static str,
        url: &'static str,
        favicon: Option<FaviconSpec>,
        reveal_close: bool,
        density: Density,
    ) -> Self {
        TabRowCase {
            label,
            state,
            pinned,
            title,
            url,
            favicon,
            reveal_close,
            density,
            crashed: false,
            bitmap_favicon: false,
            claims_selected: false,
        }
    }
}

/// Every valid Tab Row family — the single declaration the story renders.
#[rustfmt::skip]
pub const TAB_ROW_VALID: &[TabRowCase] = &[
    // §10 state families (N density, regular rows, monogram favicon).
    TabRowCase::new("default", TabVisualState::Default, false, "Example", "https://example.com/", Some(MONOGRAM), false, Density::Normal),
    TabRowCase::new("hover", TabVisualState::Hover, false, "Example", "https://example.com/", Some(MONOGRAM), false, Density::Normal),
    TabRowCase::new("active", TabVisualState::Active, false, "Example", "https://example.com/", Some(MONOGRAM), false, Density::Normal),
    TabRowCase::new("selected", TabVisualState::Selected, false, "Example", "https://example.com/", Some(MONOGRAM), true, Density::Normal),
    TabRowCase::new("loading (arc replaces favicon)", TabVisualState::Loading, false, "Example", "https://example.com/", None, false, Density::Normal),
    TabRowCase::new("attention", TabVisualState::Attention, false, "Example", "https://example.com/", Some(MONOGRAM), false, Density::Normal),
    TabRowCase::new("error", TabVisualState::Error, false, "Example", "https://example.com/", Some(MONOGRAM), false, Density::Normal),
    TabRowCase::new("unresponsive", TabVisualState::Unresponsive, false, "Example", "https://example.com/", Some(MONOGRAM), false, Density::Normal),
    TabRowCase::new("disabled", TabVisualState::Disabled, false, "Example", "https://example.com/", Some(MONOGRAM), false, Density::Normal),
    TabRowCase::new("dragging", TabVisualState::Dragging, false, "Example", "https://example.com/", Some(MONOGRAM), false, Density::Normal),
    TabRowCase::new("drag-target", TabVisualState::DragTarget, false, "Example", "https://example.com/", Some(MONOGRAM), false, Density::Normal),
    TabRowCase::new("focused", TabVisualState::Focused, false, "Example", "https://example.com/", Some(MONOGRAM), false, Density::Normal),
    // §8/§11.2 pinned + regular; §8 densities (S pinned icon-only).
    TabRowCase::new("pinned (N anatomy)", TabVisualState::Selected, true, "Pinned", "https://pinned.example/", Some(MONOGRAM), true, Density::Normal),
    // The S icon-only family carries an empty display title (url and title
    // both empty, so the §11.9 fallback yields nothing): the row renders
    // icon-only at S (§8/§11.2).
    TabRowCase::new("pinned icon-only (S)", TabVisualState::Selected, true, "", "", Some(MONOGRAM), false, Density::Small),
    TabRowCase::new("regular (XL)", TabVisualState::Default, false, "Example", "https://example.com/", Some(MONOGRAM), false, Density::ExtraLarge),
    // §4 long-title truncation through the shared truncation authority.
    TabRowCase::new("long title truncation", TabVisualState::Default, false, "Example – a deliberately long page title that cannot fit the row budget", "https://example.com/very/long/path", Some(MONOGRAM), false, Density::Normal),
    // §7.2 favicon slot variants consumed inline (monogram / missing /
    // loading-arc — the loading family above is the arc replacement).
    TabRowCase::new("missing favicon (host-scheme fallback)", TabVisualState::Default, false, "", "https://example.com/", None, false, Density::Normal),
    // §10.2 reveal-close on hover/selected (the selected family above
    // reveals on selection; this family on hover).
    TabRowCase::new("reveal-close (hover/selected)", TabVisualState::Hover, false, "Example", "https://example.com/", Some(MONOGRAM), true, Density::Normal),
];

/// A rejected combination with the exact DESIGN/packet reason (the matrix
/// "rejected" column). Same `TabRowCase` shape as the valid rows, so one
/// validity authority decides both sides and the table cannot drift.
#[derive(Debug, Clone)]
pub struct RejectedTabRow {
    pub label: &'static str,
    pub case: TabRowCase,
    pub reason: &'static str,
}

/// The matrix "rejected" column. Every entry must fail
/// [`tab_row_case_valid`] and thus refuse fixture construction.
#[rustfmt::skip]
pub const TAB_ROW_REJECTED: &[RejectedTabRow] = &[
    RejectedTabRow {
        label: "attention on a crashed row",
        case: TabRowCase {
            label: "attention on a crashed row",
            state: TabVisualState::Attention,
            pinned: false,
            title: "Example",
            url: "https://example.com/",
            favicon: Some(MONOGRAM),
            reveal_close: false,
            density: Density::Normal,
            crashed: true,
            bitmap_favicon: false,
            claims_selected: false,
        },
        reason: "§10 composition order: a crashed row derives Error (the app's row_state), so Attention never rides a crashed row",
    },
    RejectedTabRow {
        label: "selected+dragging without the drag state machine",
        case: TabRowCase {
            label: "selected+dragging",
            state: TabVisualState::Dragging,
            pinned: false,
            title: "Example",
            url: "https://example.com/",
            favicon: Some(MONOGRAM),
            reveal_close: false,
            density: Density::Normal,
            crashed: false,
            bitmap_favicon: false,
            claims_selected: true,
        },
        reason: "§10.9: Dragging comes only from the app's drag state machine (drag_source replaces the composed state); the row model carries exactly one §10 state",
    },
    RejectedTabRow {
        label: "S-density pinned with a title",
        case: TabRowCase::new("S-density pinned with a title", TabVisualState::Selected, true, "Pinned", "https://pinned.example/", Some(MONOGRAM), false, Density::Small),
        reason: "§8/§11.2: at S density pinned rows are icon-only 32×32 tiles with no title",
    },
    RejectedTabRow {
        label: "a real favicon bitmap",
        case: TabRowCase {
            label: "a real favicon bitmap",
            state: TabVisualState::Default,
            pinned: false,
            title: "Example",
            url: "https://example.com/",
            favicon: None,
            reveal_close: false,
            density: Density::Normal,
            crashed: false,
            bitmap_favicon: true,
            claims_selected: false,
        },
        reason: "§7.2: the tile is monogram-or-arc today; FaviconSpec carries only {letter, hue} and no bitmap",
    },
];

/// The one validity authority: rejects the impossible combinations the
/// matrix forbids. The fixture builder refuses (`None`) any case this
/// rejects, so the story can never render a rejected row.
pub fn tab_row_case_valid(case: &TabRowCase) -> bool {
    // §8/§11.2: at S density pinned rows are icon-only (no title).
    if case.density == Density::Small && case.pinned && !case.title.is_empty() {
        return false;
    }
    // §10 composition order: a crashed row derives Error; any other state
    // on a crashed row (attention/loading/…) is an impossible combination.
    if case.crashed && case.state != TabVisualState::Error {
        return false;
    }
    // §7.2: the tile is monogram-or-arc; no bitmap favicon exists.
    if case.bitmap_favicon {
        return false;
    }
    // §10.9: Dragging comes only from the drag state machine; claiming a
    // selected rail while dragging is rejected.
    if case.state == TabVisualState::Dragging && case.claims_selected {
        return false;
    }
    true
}

/// Build the production `RowVisuals` for a valid matrix family; `None` for
/// any rejected combination. The row index is the case's declaration
/// position in [`TAB_ROW_VALID`], so the story's ids and the matrix table
/// share one source (stable `bbr-tab-row-<n>` ids per family).
pub fn build_tab_row_fixture(case: &TabRowCase) -> Option<RowVisuals> {
    if !tab_row_case_valid(case) {
        return None;
    }
    let index = TAB_ROW_VALID
        .iter()
        .position(|c| c.label == case.label)
        .unwrap_or(0);
    Some(RowVisuals {
        index,
        title: SharedString::from(truncate_title(
            &display_title(case.url, case.title),
            FIXTURE_TITLE_BUDGET,
        )),
        state: case.state,
        favicon: case.favicon.clone(),
        pinned: case.pinned,
        closed: false,
        show_close: true,
        reveal_close: case.reveal_close,
        split: None,
    })
}

/// The §10.9 drag-tick families rendered by the story: the legal accent
/// tick and the clamped-invalid muted hairline tick, at stable ids after
/// the row families (one declaration, shared with the E2E).
pub const DRAG_TICK_LEGAL_INDEX: usize = 100;
pub const DRAG_TICK_INVALID_INDEX: usize = 101;
/// The stable `bbr-drag-tick-<n>` ids of the two tick families.
pub const DRAG_TICK_LEGAL_ID: &str = "bbr-drag-tick-100";
pub const DRAG_TICK_INVALID_ID: &str = "bbr-drag-tick-101";

/// The stable §15 ids of the interaction-target families the E2E clicks
/// (the close of the selected family — visible, §10.2 — and the icon-only
/// pinned-S row). Pinned by the matrix test at the declared positions
/// (3/13) and rendered by the story's id contract rows, so the E2E targets
/// never drift from the declaration.
pub const REVEAL_CLOSE_CLOSE_ID: &str = "bbr-tab-close-3";
pub const PINNED_S_ROW_ID: &str = "bbr-tab-row-13";

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn every_valid_family_builds_and_every_rejected_combination_fails() {
        for case in TAB_ROW_VALID {
            assert!(
                tab_row_case_valid(case),
                "valid family rejected: {}",
                case.label
            );
            let fixture = build_tab_row_fixture(case)
                .unwrap_or_else(|| panic!("valid family builds: {}", case.label));
            assert_eq!(fixture.state, case.state, "{}", case.label);
            assert_eq!(fixture.pinned, case.pinned, "{}", case.label);
        }
        for rejected in TAB_ROW_REJECTED {
            assert!(
                !tab_row_case_valid(&rejected.case),
                "rejected combination accepted: {}",
                rejected.label
            );
            assert!(
                build_tab_row_fixture(&rejected.case).is_none(),
                "rejected combination builds a fixture: {}",
                rejected.label
            );
        }
    }

    #[test]
    fn matrix_is_the_one_stable_declaration_source() {
        // The story renders exactly these families at these ids; the E2E
        // pins the same stable ids. Reordering or removing a family shifts
        // `bbr-tab-row-<n>` and fails here — the no-drift contract.
        let labels: Vec<&str> = TAB_ROW_VALID.iter().map(|c| c.label).collect();
        assert_eq!(
            labels,
            vec![
                "default",
                "hover",
                "active",
                "selected",
                "loading (arc replaces favicon)",
                "attention",
                "error",
                "unresponsive",
                "disabled",
                "dragging",
                "drag-target",
                "focused",
                "pinned (N anatomy)",
                "pinned icon-only (S)",
                "regular (XL)",
                "long title truncation",
                "missing favicon (host-scheme fallback)",
                "reveal-close (hover/selected)",
            ]
        );
        // Labels are unique (ids derive from position).
        let mut seen = BTreeSet::new();
        for case in TAB_ROW_VALID {
            assert!(seen.insert(case.label), "duplicate label {}", case.label);
        }
        // The drag-tick ids are derived from the index consts.
        assert_eq!(
            DRAG_TICK_LEGAL_ID,
            format!("bbr-drag-tick-{DRAG_TICK_LEGAL_INDEX}")
        );
        assert_eq!(
            DRAG_TICK_INVALID_ID,
            format!("bbr-drag-tick-{DRAG_TICK_INVALID_INDEX}")
        );
        // The interaction-target positions point at the declared families
        // and their stable ids (the E2E clicks these targets).
        assert_eq!(TAB_ROW_VALID[3].label, "selected");
        assert_eq!(TAB_ROW_VALID[12].label, "pinned (N anatomy)");
        assert_eq!(TAB_ROW_VALID[13].label, "pinned icon-only (S)");
        assert_eq!(REVEAL_CLOSE_CLOSE_ID, "bbr-tab-close-3");
        assert_eq!(PINNED_S_ROW_ID, "bbr-tab-row-13");
    }

    #[test]
    fn favicon_slot_variants_are_monogram_missing_and_loading_arc() {
        // §7.2 consumed inline: the row slot is monogram-or-arc; the
        // matrix's loading family is the arc replacement, the missing
        // family the host-scheme fallback, every other family the
        // monogram tile. No real bitmap exists (the model cannot carry one).
        let by_label = |label: &str| {
            TAB_ROW_VALID
                .iter()
                .find(|c| c.label == label)
                .unwrap_or_else(|| panic!("family {label}"))
        };
        let monogram = build_tab_row_fixture(by_label("default")).unwrap();
        assert!(monogram.favicon.is_some(), "monogram tile has a spec");
        let missing =
            build_tab_row_fixture(by_label("missing favicon (host-scheme fallback)")).unwrap();
        assert!(missing.favicon.is_none(), "missing favicon has no spec");
        // The missing family's title falls back to the hostname (§11.9).
        assert_eq!(missing.title, "example.com");
        let loading = build_tab_row_fixture(by_label("loading (arc replaces favicon)")).unwrap();
        assert_eq!(loading.state, TabVisualState::Loading);
    }

    #[test]
    fn long_title_uses_the_section_4_truncation_authority() {
        let case = TAB_ROW_VALID
            .iter()
            .find(|c| c.label == "long title truncation")
            .expect("long title family");
        let fixture = build_tab_row_fixture(case).unwrap();
        assert!(
            fixture.title.chars().count() <= FIXTURE_TITLE_BUDGET,
            "long title truncated to the §4 budget, got {:?}",
            fixture.title
        );
        // The separator-aware cut keeps the site name (the §4 preference).
        assert!(fixture.title.starts_with("Example"), "{}", fixture.title);
        // Short titles are never truncated.
        let short = build_tab_row_fixture(by_label("default")).unwrap();
        assert_eq!(short.title, "Example");
    }

    fn by_label(label: &str) -> &TabRowCase {
        TAB_ROW_VALID
            .iter()
            .find(|c| c.label == label)
            .unwrap_or_else(|| panic!("family {label}"))
    }

    #[test]
    fn pinned_families_follow_the_density_contract() {
        // §8/§11.2: S pinned rows are icon-only (empty title), N/XL pinned
        // rows keep the full anatomy.
        let s = build_tab_row_fixture(by_label("pinned icon-only (S)")).unwrap();
        assert!(s.pinned);
        assert!(s.title.is_empty(), "S pinned rows are icon-only");
        let n = build_tab_row_fixture(by_label("pinned (N anatomy)")).unwrap();
        assert!(n.pinned);
        assert_eq!(n.title, "Pinned");
        // A pinned row with a title at S is the expressible rejected rule.
        let rejected = TAB_ROW_REJECTED
            .iter()
            .find(|r| r.label == "S-density pinned with a title")
            .expect("rejected rule registered");
        assert!(!tab_row_case_valid(&rejected.case));
    }

    #[test]
    fn rejected_rules_cover_the_full_matrix_rejected_column() {
        let labels: Vec<&str> = TAB_ROW_REJECTED.iter().map(|r| r.label).collect();
        assert_eq!(
            labels,
            vec![
                "attention on a crashed row",
                "selected+dragging without the drag state machine",
                "S-density pinned with a title",
                "a real favicon bitmap",
            ]
        );
        for rejected in TAB_ROW_REJECTED {
            assert!(!rejected.reason.is_empty());
        }
    }
}
