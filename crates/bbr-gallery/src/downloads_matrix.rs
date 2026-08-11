//! The Downloads panel **row** matrix authority (packet
//! `component-gallery-browser-components.md`, Group 5): **one declaration
//! source** for every valid §11.12 download-row family and every rejected
//! combination. The gallery story renders exactly the valid families
//! through the production `bbr_ui::surfaces::downloads` surface
//! (`downloads_panel_surface` + the [`DownloadRowModel`] DTO), and the
//! fixture builder refuses (`None`) any case the validity authority rejects
//! — so the story, the fixture builder, the matrix table, and the tests
//! cannot drift. The rejected reasons also live here. The panel-level
//! families (Active/History groups, empty hint, >6-row scroll window,
//! keyboard cursor focus) live in the sibling [`crate::downloads_panels`]
//! module — the two modules are the one matrix declaration source split by
//! responsibility (row vs panel).
//!
//! Matrix rows come straight from the packet's Downloads panel row:
//! Waiting (unknown total, indeterminate); Waiting (known total); Running
//! (progress fraction); Succeeded (full bar + Reveal eligibility on/off →
//! enabled/disabled); Failed (Retry); Cancelled (post-F2b: Restart +
//! Delete). Rejected: a `Cancelled` row with no action post-F2b (the
//! §17.8 decision replaces the provisional no-action form); a `Retry` on
//! Succeeded/Failed-unknown (retry exists only on `Failed`, §11.12/F2b);
//! progress going backwards (monotonic, §11.12); `NaN` fraction (zero-byte
//! full bar).

use bbr_ui::state::DownloadVisual;
use bbr_ui::surfaces::downloads::{
    DownloadActionKind, DownloadGroupKind, DownloadMachine, DownloadRowModel, PanelLayout,
};
use bbr_ui::theme::Theme;

/// The gallery's §11.12 panel width: the sidebar column's current width −
/// 2 × `density:row-inset` (app geometry). The gallery uses a fixed
/// deterministic fixture width at the N default.
pub const PANEL_WIDTH: f32 = 260.0;

/// The deterministic attempt-id base for the row fixtures: the gallery has
/// no controller, so the fixture ids are plain increasing ids (the
/// `bbr-download-row-<n>` ids come from the deterministic panel index, the
/// §17.8 Delete control from the attempt id).
pub const FIXTURE_ID_BASE: u64 = 1000;

/// The default deterministic group layout for one row family: a single row
/// in its L6 group with the N-default token-derived label height.
pub fn family_layout(theme: &Theme, machine: DownloadMachine) -> PanelLayout {
    let row_h = f32::from(theme.density.row());
    let label_h = f32::from(theme.type_.badge.size) * theme.type_.badge.leading
        + 2.0 * f32::from(bbr_ui::theme::Spacing::sp1());
    let (active_count, history_count) = match machine {
        DownloadMachine::Waiting | DownloadMachine::Running => (1, 0),
        DownloadMachine::Succeeded | DownloadMachine::Failed | DownloadMachine::Cancelled => (0, 1),
    };
    PanelLayout {
        label_h,
        row_h,
        active_count,
        history_count,
        hairline: 1.0,
    }
}

/// The §11.12 machine → panel group derivation (the production row model
/// derives the group from the machine exactly like this — one authority).
pub fn group_of(machine: DownloadMachine) -> DownloadGroupKind {
    match machine {
        DownloadMachine::Waiting | DownloadMachine::Running => DownloadGroupKind::Active,
        DownloadMachine::Succeeded | DownloadMachine::Failed | DownloadMachine::Cancelled => {
            DownloadGroupKind::History
        }
    }
}

/// One row matrix row: the production [`DownloadRowModel`] fields the
/// surface renders. The matrix-only fields (`claims_backwards_progress`,
/// `claims_nan_fraction`) exist solely to name the rejected combinations
/// in the same case shape; they are never mapped into `DownloadRowModel`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DownloadCase {
    pub label: &'static str,
    pub machine: DownloadMachine,
    pub filename: &'static str,
    /// `None` while the total is unknown (indeterminate bar); a known
    /// total fills `fraction()`; a zero-byte success reports `1.0`.
    pub progress_fraction: Option<f64>,
    /// The §10 visual state of the row (default/hover/pressed/focused/
    /// disabled).
    pub visual: DownloadVisual,
    /// The exactly-one contextual primary action (§11.12).
    pub action: DownloadActionKind,
    pub action_enabled: bool,
    pub action_reason: Option<&'static str>,
    /// The secondary-line copy (`type:meta`, `tnum`).
    pub meta: &'static str,
    /// §17.8: only a cancelled row offers the Delete control.
    pub delete_available: bool,
    /// Matrix-only: this case claims the progress moves backwards
    /// (§11.12 rejects it — progress is monotonic).
    pub claims_backwards_progress: bool,
    /// Matrix-only: this case claims a `NaN` fraction (§11.12 rejects it —
    /// a zero-byte download renders a full bar, `fraction()` is never
    /// `NaN`).
    pub claims_nan_fraction: bool,
}

impl DownloadCase {
    /// The valid-family constructor: the production fields with the
    /// matrix-only flags defaulting to false. `#[allow]` keeps the dense
    /// const table readable (the fields are the documented matrix row).
    #[allow(clippy::too_many_arguments)]
    const fn new(
        label: &'static str,
        machine: DownloadMachine,
        filename: &'static str,
        progress_fraction: Option<f64>,
        visual: DownloadVisual,
        action: DownloadActionKind,
        action_enabled: bool,
        action_reason: Option<&'static str>,
        meta: &'static str,
        delete_available: bool,
    ) -> Self {
        DownloadCase {
            label,
            machine,
            filename,
            progress_fraction,
            visual,
            action,
            action_enabled,
            action_reason,
            meta,
            delete_available,
            claims_backwards_progress: false,
            claims_nan_fraction: false,
        }
    }
}

/// Every valid download-row family — the single declaration the story
/// renders (packet matrix row: the six machine families plus the reveal
/// eligibility on/off split of the succeeded row).
#[rustfmt::skip]
pub const DOWNLOAD_VALID: &[DownloadCase] = &[
    DownloadCase::new("Waiting — unknown total (indeterminate)", DownloadMachine::Waiting, "installer.pkg", None, DownloadVisual::Default, DownloadActionKind::Cancel, true, None, "Waiting…", false),
    DownloadCase::new("Waiting — known total", DownloadMachine::Waiting, "report.pdf", Some(0.0), DownloadVisual::Default, DownloadActionKind::Cancel, true, None, "0 B / 24.0 MB", false),
    DownloadCase::new("Running — progress fraction", DownloadMachine::Running, "video.mov", Some(0.4375), DownloadVisual::Default, DownloadActionKind::Cancel, true, None, "14.0 MB / 32.0 MB", false),
    DownloadCase::new("Succeeded — full bar + Reveal enabled", DownloadMachine::Succeeded, "release.dmg", Some(1.0), DownloadVisual::Default, DownloadActionKind::Reveal, true, None, "48.0 MB · Completed", false),
    DownloadCase::new("Succeeded — Reveal disabled (path gone)", DownloadMachine::Succeeded, "archive.zip", Some(1.0), DownloadVisual::Disabled, DownloadActionKind::Reveal, false, Some("File no longer exists"), "12.0 MB · Completed", false),
    DownloadCase::new("Failed — Retry", DownloadMachine::Failed, "update.bin", None, DownloadVisual::Default, DownloadActionKind::Retry, true, None, "Network error", false),
    DownloadCase::new("Cancelled — Restart + Delete", DownloadMachine::Cancelled, "session.log", None, DownloadVisual::Default, DownloadActionKind::Restart, true, None, "2.0 MB · Cancelled", true),
];

/// The §10.2 hover / §10.3 pressed wash row families the story renders
/// over the running row (the pointer-state rows — the production surface's
/// `on_row_hover`/`on_row_press` washes).
#[rustfmt::skip]
pub const DOWNLOAD_WASH_VALID: &[DownloadCase] = &[
    DownloadCase::new("hover wash (Running)", DownloadMachine::Running, "video.mov", Some(0.4375), DownloadVisual::Hover, DownloadActionKind::Cancel, true, None, "14.0 MB / 32.0 MB", false),
    DownloadCase::new("pressed wash (Running)", DownloadMachine::Running, "video.mov", Some(0.4375), DownloadVisual::Pressed, DownloadActionKind::Cancel, true, None, "14.0 MB / 32.0 MB", false),
];

/// A rejected combination with the exact DESIGN/packet reason (the matrix
/// "rejected" column). Same `DownloadCase` shape as the valid rows, so one
/// validity authority decides both sides and the table cannot drift.
#[derive(Debug, Clone, Copy)]
pub struct RejectedDownload {
    pub label: &'static str,
    pub case: DownloadCase,
    pub reason: &'static str,
}

/// The matrix "rejected" column. Every entry must fail
/// [`download_case_valid`] and thus refuse fixture construction.
#[rustfmt::skip]
pub const DOWNLOAD_REJECTED: &[RejectedDownload] = &[
    RejectedDownload {
        label: "a Cancelled row with no action post-F2b",
        case: DownloadCase {
            label: "a Cancelled row with no action",
            machine: DownloadMachine::Cancelled,
            filename: "session.log",
            progress_fraction: None,
            visual: DownloadVisual::Default,
            // The provisional §17.8 form: no Restart and no Delete.
            action: DownloadActionKind::Cancel,
            action_enabled: true,
            action_reason: None,
            meta: "Cancelled",
            delete_available: false,
            claims_backwards_progress: false,
            claims_nan_fraction: false,
        },
        reason: "§17.8 settled: a cancelled row offers Restart **and** Delete — the provisional no-action form is replaced, never rendered",
    },
    RejectedDownload {
        label: "a Retry on Succeeded/Failed-unknown",
        case: DownloadCase {
            label: "a Retry on a succeeded row",
            machine: DownloadMachine::Succeeded,
            filename: "release.dmg",
            progress_fraction: Some(1.0),
            visual: DownloadVisual::Default,
            action: DownloadActionKind::Retry,
            action_enabled: true,
            action_reason: None,
            meta: "48.0 MB · Completed",
            delete_available: false,
            claims_backwards_progress: false,
            claims_nan_fraction: false,
        },
        reason: "§11.12/F2b: retry exists only on Failed — a succeeded row never offers Retry",
    },
    RejectedDownload {
        label: "progress going backwards",
        case: DownloadCase {
            label: "progress going backwards",
            machine: DownloadMachine::Running,
            filename: "video.mov",
            progress_fraction: Some(0.25),
            visual: DownloadVisual::Default,
            action: DownloadActionKind::Cancel,
            action_enabled: true,
            action_reason: None,
            meta: "8.0 MB / 32.0 MB",
            delete_available: false,
            claims_backwards_progress: true,
            claims_nan_fraction: false,
        },
        reason: "§11.12: progress is monotonic — a known total never moves backwards",
    },
    RejectedDownload {
        label: "NaN fraction (zero-byte full bar)",
        case: DownloadCase {
            label: "NaN fraction",
            machine: DownloadMachine::Running,
            filename: "video.mov",
            progress_fraction: Some(f64::NAN),
            visual: DownloadVisual::Default,
            action: DownloadActionKind::Cancel,
            action_enabled: true,
            action_reason: None,
            meta: "14.0 MB / 32.0 MB",
            delete_available: false,
            claims_backwards_progress: false,
            claims_nan_fraction: true,
        },
        reason: "§11.12: `fraction()` is never NaN — a zero-byte download succeeds with a full bar",
    },
];

/// The one row validity authority: rejects the impossible combinations the
/// matrix forbids. The fixture builder refuses (`None`) any case this
/// rejects, so the story can never render a rejected row.
pub fn download_case_valid(case: &DownloadCase) -> bool {
    // §11.12: progress is monotonic — never backwards.
    if case.claims_backwards_progress {
        return false;
    }
    // §11.12: `fraction()` is never NaN.
    if case.claims_nan_fraction {
        return false;
    }
    // The action vocabulary follows the machine (§11.12 + §17.8): active →
    // Cancel, succeeded → Reveal, failed → Retry, cancelled → Restart and
    // Delete.
    match case.machine {
        DownloadMachine::Waiting | DownloadMachine::Running => {
            case.action == DownloadActionKind::Cancel && !case.delete_available
        }
        DownloadMachine::Succeeded => {
            case.action == DownloadActionKind::Reveal && !case.delete_available
        }
        DownloadMachine::Failed => {
            case.action == DownloadActionKind::Retry && !case.delete_available
        }
        // §17.8: a cancelled row offers Restart **and** Delete.
        DownloadMachine::Cancelled => {
            case.action == DownloadActionKind::Restart && case.delete_available
        }
    }
}

/// Build the production [`DownloadRowModel`] for a valid matrix family;
/// `None` for any rejected combination. The panel index is the caller's
/// (the story's panels assign stable per-family indices).
pub fn build_download_fixture(case: &DownloadCase, index: usize) -> Option<DownloadRowModel> {
    if !download_case_valid(case) {
        return None;
    }
    Some(DownloadRowModel {
        index,
        id: FIXTURE_ID_BASE + index as u64,
        filename: case.filename.to_string(),
        machine: case.machine,
        group: group_of(case.machine),
        progress_fraction: case.progress_fraction,
        visual: case.visual,
        action: case.action,
        action_enabled: case.action_enabled,
        action_reason: case.action_reason.map(ToString::to_string),
        meta: case.meta.to_string(),
        resolved_path: if case.machine == DownloadMachine::Succeeded {
            Some(format!("/tmp/{}.dl", case.filename))
        } else {
            None
        },
        retry_of: None,
        attempt: 1,
        delete_available: case.delete_available,
    })
}

/// The stable gallery test-target selectors (test targets only — never
/// product ids): `download-family-<n>` for the row families and
/// `download-wash-family-<n>` for the hover/press wash families, in
/// declaration order. The panel families' selector lives in the sibling
/// [`crate::downloads_panels`] module.
pub const DOWNLOAD_FAMILY_SELECTOR: &str = "download-family-";
pub const DOWNLOAD_WASH_FAMILY_SELECTOR: &str = "download-wash-family-";

/// The stable index of the Cancelled row family (the §17.8 Restart+Delete
/// interaction target for the E2E).
pub const CANCELLED_FAMILY_INDEX: usize = 6;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::downloads_panels::{build_panel_fixture, PANEL_VALID};
    use bbr_ui::theme::{ContrastMode, Density, ThemeMode};
    use std::collections::BTreeSet;

    fn fixture_theme() -> Theme {
        Theme::new(ThemeMode::Dark, ContrastMode::Standard, Density::Normal)
    }

    #[test]
    fn every_valid_family_builds_and_every_rejected_combination_fails() {
        let theme = fixture_theme();
        for (index, case) in DOWNLOAD_VALID.iter().enumerate() {
            assert!(
                download_case_valid(case),
                "valid family rejected: {}",
                case.label
            );
            let fixture = build_download_fixture(case, index)
                .unwrap_or_else(|| panic!("valid family builds: {}", case.label));
            assert_eq!(fixture.machine, case.machine, "{}", case.label);
            assert_eq!(fixture.group, group_of(case.machine), "{}", case.label);
            assert_eq!(fixture.action, case.action, "{}", case.label);
            assert_eq!(
                fixture.delete_available, case.delete_available,
                "{}",
                case.label
            );
            assert_eq!(
                fixture.progress_fraction, case.progress_fraction,
                "{}",
                case.label
            );
            assert!(!fixture.filename.is_empty(), "{}", case.label);
        }
        for (index, case) in DOWNLOAD_WASH_VALID.iter().enumerate() {
            assert!(
                download_case_valid(case),
                "valid wash family rejected: {}",
                case.label
            );
            assert!(build_download_fixture(case, index).is_some());
        }
        for rejected in DOWNLOAD_REJECTED {
            assert!(
                !download_case_valid(&rejected.case),
                "rejected combination accepted: {}",
                rejected.label
            );
            assert!(
                build_download_fixture(&rejected.case, 0).is_none(),
                "rejected combination builds a fixture: {}",
                rejected.label
            );
        }
        // Every panel family builds its fixture (the story renders them).
        for case in PANEL_VALID {
            let fixture = build_panel_fixture(case, &theme);
            assert_eq!(fixture.rows.len(), case.rows.len(), "{}", case.label);
            assert_eq!(fixture.layout.total(), case.rows.len(), "{}", case.label);
            assert!(
                fixture.window.end <= fixture.layout.total(),
                "{}",
                case.label
            );
        }
    }

    #[test]
    fn matrix_is_the_one_stable_declaration_source() {
        let labels: Vec<&str> = DOWNLOAD_VALID.iter().map(|c| c.label).collect();
        assert_eq!(
            labels,
            vec![
                "Waiting — unknown total (indeterminate)",
                "Waiting — known total",
                "Running — progress fraction",
                "Succeeded — full bar + Reveal enabled",
                "Succeeded — Reveal disabled (path gone)",
                "Failed — Retry",
                "Cancelled — Restart + Delete",
            ]
        );
        let wash_labels: Vec<&str> = DOWNLOAD_WASH_VALID.iter().map(|c| c.label).collect();
        assert_eq!(
            wash_labels,
            vec!["hover wash (Running)", "pressed wash (Running)"]
        );
        let panel_labels: Vec<&str> = PANEL_VALID.iter().map(|c| c.label).collect();
        assert_eq!(
            panel_labels,
            vec![
                "Active + History groups",
                "History-only (empty Active hides its label)",
                "Empty hint",
                ">6-row scroll window (bounded)",
                "Keyboard cursor focus",
            ]
        );
        let mut seen = BTreeSet::new();
        for case in DOWNLOAD_VALID {
            assert!(seen.insert(case.label), "duplicate label {}", case.label);
        }
        for case in PANEL_VALID {
            assert!(seen.insert(case.label), "duplicate label {}", case.label);
        }
    }

    #[test]
    fn row_families_cover_the_exact_packet_machine_contract() {
        // Every L6 machine state is covered (Waiting ×2: unknown/known
        // total; Running; Succeeded ×2: reveal on/off; Failed; Cancelled).
        let mut machines: Vec<DownloadMachine> = DOWNLOAD_VALID.iter().map(|c| c.machine).collect();
        machines.sort_by_key(|m| format!("{m:?}"));
        machines.dedup();
        assert_eq!(machines.len(), 5);
        // The indeterminate family carries no fraction; a known total and
        // a running row carry fractions; a succeeded row always presents
        // the full bar.
        let by_label = |label: &str| {
            DOWNLOAD_VALID
                .iter()
                .find(|c| c.label == label)
                .unwrap_or_else(|| panic!("family {label}"))
        };
        assert_eq!(
            by_label("Waiting — unknown total (indeterminate)").progress_fraction,
            None
        );
        assert_eq!(
            by_label("Waiting — known total").progress_fraction,
            Some(0.0)
        );
        assert_eq!(
            by_label("Succeeded — full bar + Reveal enabled").progress_fraction,
            Some(1.0)
        );
        // The reveal eligibility split: enabled vs disabled with a reason.
        assert!(by_label("Succeeded — full bar + Reveal enabled").action_enabled);
        let disabled = by_label("Succeeded — Reveal disabled (path gone)");
        assert!(!disabled.action_enabled);
        assert_eq!(disabled.action_reason, Some("File no longer exists"));
        assert_eq!(disabled.visual, DownloadVisual::Disabled);
    }

    #[test]
    fn cancelled_rows_offer_restart_and_delete() {
        let cancelled = DOWNLOAD_VALID
            .iter()
            .find(|c| c.machine == DownloadMachine::Cancelled)
            .expect("cancelled family");
        assert_eq!(cancelled.action, DownloadActionKind::Restart);
        assert!(cancelled.delete_available);
        let fixture = build_download_fixture(cancelled, CANCELLED_FAMILY_INDEX).unwrap();
        assert!(fixture.delete_available);
        assert_eq!(fixture.action, DownloadActionKind::Restart);
    }

    #[test]
    fn rejected_rules_cover_the_full_matrix_rejected_column() {
        let labels: Vec<&str> = DOWNLOAD_REJECTED.iter().map(|r| r.label).collect();
        assert_eq!(
            labels,
            vec![
                "a Cancelled row with no action post-F2b",
                "a Retry on Succeeded/Failed-unknown",
                "progress going backwards",
                "NaN fraction (zero-byte full bar)",
            ]
        );
        for rejected in DOWNLOAD_REJECTED {
            assert!(!rejected.reason.is_empty());
        }
    }
}
