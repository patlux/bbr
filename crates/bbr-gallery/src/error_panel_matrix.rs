//! The §11.8 glass-pane matrix authority (packet
//! `component-gallery-browser-components.md`, Group 7): the crash panel
//! (Reload / Close tab) and the empty space pane (profile line + New Tab
//! ⌘T) rows of the Error/Crash matrix. Together with
//! [`crate::error_matrix`] (the classified error-surface families) this is
//! the **one declaration source** for the Error/Crash matrix: the gallery
//! stories render exactly the valid pane panels through the production
//! `bbr_ui::components::pane` components, and the fixture builder refuses
//! (`None`) any case the validity authority rejects — so the story, the
//! fixture builder, the matrix table, and the tests cannot drift. The
//! rejected reasons also live here.
//!
//! Matrix rows come straight from the packet's Error/Crash row: the §11.8
//! crash panel (Reload / Close tab) and the §11.8 empty pane (profile line
//! and New Tab ⌘T). The rejected combination is a crash panel on a healthy
//! tab — the crash panel replaces the crashed tab's glass area only, and a
//! healthy tab renders its native slot, never a pane.

/// The §11.8 glass panes (production `bbr_ui::components::pane`): the
/// crash panel (Reload / Close tab) and the empty space pane (profile line
/// + New Tab ⌘T).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PanelKind {
    /// §11.8 crash panel: replaces the crashed tab's glass area.
    Crash,
    /// §11.8 empty space pane: no tabs in this space.
    Empty,
}

/// The gallery test-target selector suffix for a pane kind (`panel-crash`
/// / `panel-empty`). Test targets only — never product ids.
pub const fn panel_suffix(kind: PanelKind) -> &'static str {
    match kind {
        PanelKind::Crash => "crash",
        PanelKind::Empty => "empty",
    }
}

/// The tab state that decides which pane (or no pane) renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneTabState {
    /// No tabs in the space — the §11.8 empty pane.
    NoTabs,
    /// A live, non-crashed tab — the native slot renders, never a pane.
    Healthy,
    /// A crashed tab — the §11.8 crash panel.
    Crashed,
}

/// One panel matrix row: the pane kind and the tab state that decides it,
/// plus the profile line the empty pane names. The tab-state field is the
/// matrix-only dimension — it is never rendered; it decides which pane the
/// fixture returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelCase {
    pub label: &'static str,
    pub panel: PanelKind,
    /// The owning profile line the empty pane names ("in Profile · Work").
    pub profile_line: &'static str,
    pub tab_state: PaneTabState,
}

/// Every valid §11.8 pane panel — the single declaration the story renders.
#[rustfmt::skip]
pub const PANEL_VALID: &[PanelCase] = &[
    PanelCase {
        label: "crash panel — crashed tab",
        panel: PanelKind::Crash,
        profile_line: "",
        tab_state: PaneTabState::Crashed,
    },
    PanelCase {
        label: "empty space pane — no tabs",
        panel: PanelKind::Empty,
        profile_line: "in Profile · Work",
        tab_state: PaneTabState::NoTabs,
    },
];

/// A rejected panel combination with the exact DESIGN reason.
#[derive(Debug, Clone, Copy)]
pub struct RejectedPanel {
    pub label: &'static str,
    pub case: PanelCase,
    pub reason: &'static str,
}

/// The matrix "rejected" column for the panes. Every entry must fail
/// [`panel_case_valid`] and thus refuse fixture construction.
#[rustfmt::skip]
pub const PANEL_REJECTED: &[RejectedPanel] = &[
    RejectedPanel {
        label: "crash panel on a healthy tab",
        case: PanelCase {
            label: "crash panel on a healthy tab",
            panel: PanelKind::Crash,
            profile_line: "",
            tab_state: PaneTabState::Healthy,
        },
        reason: "DESIGN §11.8: the crash panel replaces the crashed tab's glass area only — a healthy tab renders its native slot (or a classified error surface), never the crash panel",
    },
];

/// The one pane validity authority: the crash panel requires the crashed
/// tab, the empty pane requires no tabs in the space; a healthy tab
/// renders neither pane (its native slot).
pub fn panel_case_valid(case: &PanelCase) -> bool {
    matches!(
        (case.panel, case.tab_state),
        (PanelKind::Crash, PaneTabState::Crashed) | (PanelKind::Empty, PaneTabState::NoTabs)
    )
}

/// Build the pane fixture: `Some(panel kind)` when the tab state makes the
/// panel legal; `None` for any rejected combination (a healthy tab never
/// renders a pane).
pub fn build_panel_fixture(case: &PanelCase) -> Option<PanelKind> {
    if !panel_case_valid(case) {
        return None;
    }
    Some(case.panel)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Every declared valid panel builds and every declared rejected
    /// combination fails fixture construction.
    #[test]
    fn every_valid_panel_builds_and_every_rejected_combination_fails() {
        for case in PANEL_VALID {
            assert!(
                panel_case_valid(case),
                "valid panel rejected: {}",
                case.label
            );
            assert_eq!(
                build_panel_fixture(case),
                Some(case.panel),
                "{}",
                case.label
            );
        }
        for rejected in PANEL_REJECTED {
            assert!(
                !panel_case_valid(&rejected.case),
                "rejected panel accepted: {}",
                rejected.label
            );
            assert!(
                build_panel_fixture(&rejected.case).is_none(),
                "rejected panel builds a fixture: {}",
                rejected.label
            );
        }
    }

    /// The panel matrix is the one stable declaration source: the story
    /// renders exactly these panels in this order; reordering or removing
    /// a panel fails here — the no-drift contract.
    #[test]
    fn panel_matrix_is_the_one_stable_declaration_source() {
        let panel_labels: Vec<&str> = PANEL_VALID.iter().map(|c| c.label).collect();
        assert_eq!(
            panel_labels,
            vec!["crash panel — crashed tab", "empty space pane — no tabs"]
        );
        let mut seen = BTreeSet::new();
        for case in PANEL_VALID {
            assert!(seen.insert(case.label), "duplicate label {}", case.label);
        }
    }

    /// The panel matrix covers the required dimensions: both pane panels
    /// and the tab states that decide them.
    #[test]
    fn panel_matrix_covers_the_required_dimensions() {
        let panels: BTreeSet<PanelKind> = PANEL_VALID.iter().map(|c| c.panel).collect();
        assert_eq!(panels, BTreeSet::from([PanelKind::Crash, PanelKind::Empty]));
        assert!(
            PANEL_VALID
                .iter()
                .any(|c| c.panel == PanelKind::Crash && c.tab_state == PaneTabState::Crashed),
            "the crash panel requires the crashed tab"
        );
        assert!(
            PANEL_VALID
                .iter()
                .any(|c| c.panel == PanelKind::Empty && c.tab_state == PaneTabState::NoTabs),
            "the empty pane requires no tabs in the space"
        );
    }

    /// The rejected rules cover the full panel matrix rejected column.
    #[test]
    fn panel_rejected_rules_cover_the_full_matrix_rejected_column() {
        let panel_labels: Vec<&str> = PANEL_REJECTED.iter().map(|r| r.label).collect();
        assert_eq!(panel_labels, vec!["crash panel on a healthy tab"]);
        for rejected in PANEL_REJECTED {
            assert!(!rejected.reason.is_empty());
        }
    }
}
