//! Checked-in accepted-baseline schema and regression policy for real-app
#![allow(dead_code)]
//! performance gates.
//!
//! The relative (>&nbsp;15% p95) regression rule is enforced **only** on the
//! explicitly identifiable canonical machine (`canonical_host_model` in the
//! checked-in baseline). Non-canonical machines stay report-only for the
//! relative rule but always enforce the absolute ceilings. A baseline entry
//! whose `source` is `contract-budget` is a seed (not yet measured on the
//! canonical runner) and is excluded from relative enforcement.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Percentage threshold for the canonical relative-regression rule.
pub const DEFAULT_RELATIVE_REGRESSION_PCT: f64 = 15.0;

/// Absolute ceilings from the product performance contract, used to seed the
/// checked-in baseline and as the always-enforced safety floor.
pub fn contract_budget(kind: &str) -> Option<Budget> {
    if let Some(budget) = u2::contract_budget(kind) {
        return Some(budget);
    }
    let b = match kind {
        "startup" => Budget {
            p95_ms: 500.0,
            max_ms: Some(750.0),
        },
        "page_ready" => Budget {
            p95_ms: 1500.0,
            max_ms: None,
        },
        "warm_tab_switch" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        // Performance contract: "Switch space → its already-warm selected tab
        // presented" — p95 ≤ 50 ms, max ≤ 100 ms. Same warm-presentation class
        // as warm tab switching.
        "space_switch" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        // Cross-profile view-only space activation with the §11.3 profile
        // hint presented: still a warm view-only switch (sessions are never
        // recreated), so it shares the 50/100 ms warm budget; the hint is
        // part of the presentation the sample must show.
        "space_switch_cross_profile" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "tab_select_pointer" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "tab_activate_keyboard" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "tab_reopen" => Budget {
            p95_ms: 100.0,
            max_ms: Some(150.0),
        },
        // Warm tab organization: pin/unpin and region-bounded reorder commit
        // the new row order on the next frame without touching the session
        // (the warm-interaction class, same as tab selection).
        "tab_pin" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "tab_reorder" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "tab_move_same_profile" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        // Cross-profile moves destroy the old session and create a fresh one
        // in the destination profile's store — the session-recreation class
        // with its own budget, like reopen.
        "tab_move_cross_profile" => Budget {
            p95_ms: 100.0,
            max_ms: Some(150.0),
        },
        "sidebar_toggle" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        // Contract: "Open, close, or toggle the sidebar → chrome and native
        // content geometry committed — p95 ≤ 50 ms, max ≤ 100 ms". Resize
        // commits the same next-frame geometry class; restoration (expand to
        // the last expanded width) is the same toggle class.
        "sidebar_resize" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "sidebar_restore" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "new_tab_command_bar" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "new_tab_open" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "active_tab_close" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        // Contract covers presentation commits; default to the interaction
        // ceiling for any operation without an explicit separate budget.
        // Dense-list scrolling and dense selection are warm-interaction
        // class (the performance contract lists "dense-list scrolling" as a
        // must-measure interaction; selection of a warm dense row is the same
        // next-frame presentation class as any warm tab selection), so they
        // carry the explicit warm budgets below rather than relying on the
        // fallback.
        "dense_scroll" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "dense_select" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        // Pointer-state transitions (hover/press) commit the §10.2/§10.3
        // wash on the next frame — transient indication in the warm
        // next-frame presentation class, same as any tab-row mutation.
        "tab_hover" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "tab_press" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        // Command-surface scenarios (DESIGN.md §11.5): `command_type` spans
        // Cmd-L + replace-selection typing + the real ranking debounce
        // publishing the correct first result — the warm interaction class.
        // `command_activate` spans Enter routing the selected/typed
        // destination through the app/controller seams to the presented web
        // focus — the warm activation class (50/100 from the packet).
        "command_type" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "command_activate" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        // Cmd-Enter opens the destination in exactly one fresh tab/session
        // and focuses it — the session-recreation class with its own budget,
        // like reopen and cross-profile moves.
        "command_activate_new_tab" => Budget {
            p95_ms: 100.0,
            max_ms: Some(150.0),
        },
        // §11.4 palette-open: Cmd-T presents the centered command palette
        // (`bbr-palette`, elev:2) on the next frame — the warm-interaction
        // class with the same 50/100 ms budget as the other command-surface
        // scenarios. Staged semantics: no tab or session is created before
        // Enter.
        "palette_open" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        // §11.4 overlay-open: Cmd-L / address-pill click presents the
        // field-anchored overlay (`bbr-command-overlay`, elev:2) **over**
        // the address control on the next frame with the full URL
        // preselected — the same warm-presentation class as `palette_open`
        // (50/100 ms); staged semantics, no tab or session is created
        // before Enter.
        "overlay_open" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        // Find-in-page (task packet `find-in-page-portal-bar`): Cmd-F
        // presents the find bar in the one `NativeAboveWeb` portal window
        // with the field focused — the warm-presentation class. Query
        // results and warm next/previous steps span the production entry,
        // WebKit's content-process computation, and the re-rendered
        // WebKit-reported report — the command-surface warm budget (packet:
        // "Opening and warm result navigation meet the existing
        // command-surface warm budget").
        "find_open" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "find_query_result" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "find_step" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        // Downloads sidebar panel (task packet `downloads-sidebar-panel`,
        // DESIGN.md §11.12): the footer `[Downloads]` open presents the
        // anchored panel with the keyboard cursor — the warm-presentation
        // class. The L6 progress commit (a deterministic event applied
        // through the production pump re-renders the running row at the
        // armed fraction) and the retry row action (the fresh attempt
        // presented in the Active group) are the same warm next-frame
        // presentation class, so all three carry the standard 50/100 ms
        // warm budget.
        "downloads_panel_open" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "download_progress_commit" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        "download_row_retry" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        // Web-error surface (task packet `error-surface-contract`): the
        // production incident-presentation (`on_incident` → the contract
        // `present` lifecycle, surface drawn over the unchanged active
        // view) is ordinary GPUI chrome in the warm next-frame
        // presentation class — the same 50/100 ms budget as the other
        // warm presentation commits. Fixture/network time is excluded by
        // construction (the classified incident is applied in-process,
        // never through a network transfer).
        "error_surface_present" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        // Per-site zoom (queue item 11, part A; task packet
        // `site-settings-zoom-contract`): `zoom_step` spans the production
        // `Cmd-Plus` path — the lifecycle change, the persistence write, and
        // the native `pageZoom` application on the active view — a warm
        // main-thread interaction in the same class as the other
        // command-surface scenarios, so it reuses the existing 50/100 ms
        // warm interaction budget (no new class invented).
        "zoom_step" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        // BBR context menu over web content (task packet
        // `bbr-context-menus-over-web`): `context_menu_open` spans the
        // production event-driven open path — the row-model build, the
        // anchored placement, the portal presentation, and the focus
        // handoff — and commits only from a render of the one
        // `NativeAboveWeb` portal window that drew the menu over the active
        // view. The same warm-presentation class as the find bar open
        // (50/100 ms), no new budget class invented.
        "context_menu_open" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        // Credential sheet (queue item 10): the sheet open presents the
        // awaiting challenge in the one `NativeAboveWeb` portal window — the
        // warm-presentation class (same 50/100 ms budget as the other
        // command-surface/portal presentations, `find_open`/`palette_open`).
        // No tab or session is created or touched.
        "auth_sheet_open" => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
        _ => Budget {
            p95_ms: 50.0,
            max_ms: Some(100.0),
        },
    };
    Some(b)
}

#[derive(Debug, Clone)]
pub struct Budget {
    pub p95_ms: f64,
    pub max_ms: Option<f64>,
}

/// One measured distribution for a benchmark kind.
#[derive(Debug, Clone)]
pub struct Measured {
    pub samples: usize,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub max_ms: f64,
}

/// A failed gate with the exact rule that tripped it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Regression {
    pub kind: String,
    pub measured_p95_ms: f64,
    pub measured_max_ms: f64,
    pub budget_p95_ms: f64,
    pub budget_max_ms: Option<f64>,
    /// `abs_max`, `abs_p95`, or `relative_p95` (canonical-only).
    pub rule: String,
    /// Mean for the measurement when the rule is not on the hot path.
    pub detail: String,
}

fn regression(kind: &str, m: &Measured, budget: &Budget, rule: &str, detail: String) -> Regression {
    Regression {
        kind: kind.to_string(),
        measured_p95_ms: m.p95_ms,
        measured_max_ms: m.max_ms,
        budget_p95_ms: budget.p95_ms,
        budget_max_ms: budget.max_ms,
        rule: rule.to_string(),
        detail,
    }
}

/// Evaluate the measured distributions against the absolute ceilings and,
/// when canonical, the checked-in accepted baseline. Returns every triggered
/// rule; an empty vector means green.
pub fn evaluate(
    measured: &BTreeMap<String, Measured>,
    baseline: &Baseline,
    is_canonical: bool,
) -> Vec<Regression> {
    let mut regressions = Vec::new();
    for (kind, m) in measured {
        let Some(budget_entry) = baseline.entries.get(kind) else {
            continue;
        };
        // Absolute ceilings: always enforced.
        let budget_abs = Budget {
            p95_ms: budget_entry.budget_p95_ms,
            max_ms: budget_entry.budget_max_ms,
        };
        if let Some(max_ms) = budget_abs.max_ms {
            if m.max_ms > max_ms {
                regressions.push(regression(
                    kind,
                    m,
                    &budget_abs,
                    "abs_max",
                    format!("max {:.2}ms exceeded the {:.0}ms ceiling", m.max_ms, max_ms),
                ));
            }
        }
        if m.p95_ms > budget_abs.p95_ms {
            regressions.push(regression(
                kind,
                m,
                &budget_abs,
                "abs_p95",
                format!(
                    "p95 {:.2}ms exceeded the {:.0}ms ceiling",
                    m.p95_ms, budget_abs.p95_ms
                ),
            ));
        }
        // Relative > 15% p95 regression: canonical machine + established baseline only.
        if is_canonical && budget_entry.source == "accepted" {
            let threshold = budget_entry.p95_ms * (1.0 + baseline.relative_regression_pct / 100.0);
            if m.p95_ms > threshold {
                regressions.push(regression(
                    kind,
                    m,
                    &budget_abs,
                    "relative_p95",
                    format!(
                        "p95 {:.2}ms regressed past the accepted {:.2}ms +{:.0}% threshold",
                        m.p95_ms, budget_entry.p95_ms, baseline.relative_regression_pct
                    ),
                ));
            }
        }
    }
    regressions
}

/// The checked-in accepted-baseline document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub schema_version: u32,
    pub canonical_host_model: String,
    pub generated_revision: String,
    #[serde(default = "default_relative_pct")]
    pub relative_regression_pct: f64,
    pub entries: BTreeMap<String, BaselineEntry>,
}

fn default_relative_pct() -> f64 {
    DEFAULT_RELATIVE_REGRESSION_PCT
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineEntry {
    pub samples: usize,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub max_ms: f64,
    pub budget_p95_ms: f64,
    pub budget_max_ms: Option<f64>,
    /// `accepted` = measured on the canonical machine (relative rule applies),
    /// `contract-budget` = seed, not yet measured.
    pub source: String,
}

impl Baseline {
    /// Load from a JSON path; falls back to a contract-budget seed when the
    /// file is absent or unreadable so local runs stay functional.
    pub fn load_or_seed(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_else(Self::seed)
    }

    /// A baseline whose entries are all contract-budget seeds. Used before the
    /// first canonical run establishes a measured accepted baseline.
    pub fn seed() -> Self {
        let mut entries = BTreeMap::new();
        for kind in [
            "startup",
            "page_ready",
            "warm_tab_switch",
            "space_switch",
            "space_switch_cross_profile",
            "tab_select_pointer",
            "tab_activate_keyboard",
            "tab_reopen",
            "tab_pin",
            "tab_reorder",
            "tab_move_same_profile",
            "tab_move_cross_profile",
            "sidebar_toggle",
            "sidebar_resize",
            "sidebar_restore",
            "new_tab_command_bar",
            "new_tab_open",
            "active_tab_close",
            "dense_scroll",
            "dense_select",
            "tab_hover",
            "tab_press",
            "command_type",
            "command_activate",
            "command_activate_new_tab",
            "palette_open",
            "overlay_open",
            "find_open",
            "find_query_result",
            "find_step",
            "downloads_panel_open",
            "download_progress_commit",
            "download_row_retry",
            "error_surface_present",
            "zoom_step",
            "context_menu_open",
            "auth_sheet_open",
        ]
        .into_iter()
        .chain(u2::seed_kinds())
        {
            let b = contract_budget(kind).expect("known kind");
            entries.insert(
                kind.to_string(),
                BaselineEntry {
                    samples: 0,
                    p50_ms: 0.0,
                    p95_ms: b.p95_ms,
                    max_ms: b.max_ms.unwrap_or(b.p95_ms),
                    budget_p95_ms: b.p95_ms,
                    budget_max_ms: b.max_ms,
                    source: "contract-budget".to_string(),
                },
            );
        }
        Self {
            schema_version: 1,
            canonical_host_model: String::new(),
            generated_revision: "seed".to_string(),
            relative_regression_pct: DEFAULT_RELATIVE_REGRESSION_PCT,
            entries,
        }
    }
}

#[cfg(test)]
mod u2;

#[cfg(test)]
mod tests {
    use super::*;

    mod u2;

    fn measured(p95: f64, max: f64) -> Measured {
        Measured {
            samples: 30,
            p50_ms: p95 * 0.5,
            p95_ms: p95,
            max_ms: max,
        }
    }

    fn baseline_accepted() -> Baseline {
        let mut b = Baseline::seed();
        b.canonical_host_model = "Mac16,5".to_string();
        for e in b.entries.values_mut() {
            e.source = "accepted".to_string();
            e.p95_ms = e.budget_p95_ms; // accepted == budget ceiling for the test
        }
        b
    }

    #[test]
    fn absolute_max_always_enforced_even_non_canonical() {
        let b = baseline_accepted();
        let mut m = BTreeMap::new();
        m.insert("warm_tab_switch".into(), measured(30.0, 350.0)); // max ceiling 100
        let r = evaluate(&m, &b, false);
        assert!(r.iter().any(|x| x.rule == "abs_max"));
        assert!(!r.iter().any(|x| x.rule == "relative_p95"));
    }

    #[test]
    fn relative_p95_only_enforced_when_canonical_and_accepted() {
        let b = baseline_accepted();
        let mut m = BTreeMap::new();
        // p95 60 > 50*1.15=57.5 → relative regression on canonical.
        m.insert("warm_tab_switch".into(), measured(60.0, 90.0));
        let on_canonical = evaluate(&m, &b, true);
        assert!(on_canonical.iter().any(|x| x.rule == "relative_p95"));
        let off_canonical = evaluate(&m, &b, false);
        assert!(!off_canonical.iter().any(|x| x.rule == "relative_p95"));
    }

    #[test]
    fn seed_baseline_never_trips_relative_rule() {
        let b = Baseline::seed(); // source == contract-budget everywhere
        let mut m = BTreeMap::new();
        m.insert("warm_tab_switch".into(), measured(2000.0, 2500.0));
        // Absolute still fails, but no relative rule because source != accepted.
        let r = evaluate(&m, &b, true);
        assert!(r.iter().any(|x| x.rule == "abs_p95"));
        assert!(!r.iter().any(|x| x.rule == "relative_p95"));
    }

    #[test]
    fn space_switch_budget_is_enforced_by_the_gate() {
        // Regression: the warm space-switch scenario was measured and reported
        // but never seeded into the baseline, so a future 100× regression would
        // fail no gate. The seed must carry the contract budget and `evaluate`
        // must trip on it.
        let b = Baseline::seed();
        assert!(
            b.entries.contains_key("space_switch"),
            "seed baseline must carry the space_switch contract budget"
        );
        let entry = &b.entries["space_switch"];
        assert_eq!(entry.budget_p95_ms, 50.0);
        assert_eq!(entry.budget_max_ms, Some(100.0));
        let mut bad = BTreeMap::new();
        bad.insert("space_switch".into(), measured(120.0, 160.0));
        let r = evaluate(&bad, &b, false);
        assert!(r.iter().any(|x| x.rule == "abs_p95"));
        assert!(r.iter().any(|x| x.rule == "abs_max"));
        let mut good = BTreeMap::new();
        good.insert("space_switch".into(), measured(45.0, 90.0));
        assert!(evaluate(&good, &b, false).is_empty());
    }

    #[test]
    fn cross_profile_space_switch_budget_is_seeded_and_enforced() {
        // Regression guard: the cross-profile space-switch scenario (with the
        // §11.3 profile hint presented) must carry its warm contract budget
        // in the seed so a future hint-render regression trips the gate
        // instead of being silently unreported.
        let b = Baseline::seed();
        let entry = b
            .entries
            .get("space_switch_cross_profile")
            .expect("seed baseline must carry the space_switch_cross_profile budget");
        assert_eq!(entry.budget_p95_ms, 50.0);
        assert_eq!(entry.budget_max_ms, Some(100.0));
        let mut bad = BTreeMap::new();
        bad.insert("space_switch_cross_profile".into(), measured(120.0, 160.0));
        let r = evaluate(&bad, &b, false);
        assert!(r.iter().any(|x| x.rule == "abs_p95"));
        assert!(r.iter().any(|x| x.rule == "abs_max"));
        let mut good = BTreeMap::new();
        good.insert("space_switch_cross_profile".into(), measured(45.0, 90.0));
        assert!(evaluate(&good, &b, false).is_empty());
    }

    #[test]
    fn dense_budgets_are_seeded_and_enforced() {
        // Regression guard (HIGH-1): the dense scroll/select scenarios must
        // carry their contract budgets in the seed so a future dense-frame
        // regression trips the gate instead of being silently unreported
        // dead code. The performance contract names "dense-list scrolling"
        // as a must-measure interaction.
        let b = Baseline::seed();
        for kind in ["dense_scroll", "dense_select"] {
            let entry = b
                .entries
                .get(kind)
                .unwrap_or_else(|| panic!("seed baseline must carry the {kind} contract budget"));
            assert_eq!(entry.budget_p95_ms, 50.0, "{kind} warm p95 ceiling");
            assert_eq!(entry.budget_max_ms, Some(100.0), "{kind} warm max ceiling");
            let mut slow = BTreeMap::new();
            slow.insert(kind.into(), measured(120.0, 160.0));
            let r = evaluate(&slow, &b, false);
            assert!(
                r.iter().any(|x| x.rule == "abs_p95"),
                "{kind} absolute p95 ceiling enforced"
            );
            assert!(
                r.iter().any(|x| x.rule == "abs_max"),
                "{kind} absolute max ceiling enforced"
            );
        }
    }

    #[test]
    fn pointer_state_budgets_are_seeded_and_enforced() {
        // Regression guard: the §10.2 hover / §10.3 press scenarios must
        // carry their contract budgets in the seed so a future pointer-state
        // render regression trips the gate instead of being silently
        // unreported dead code. Both are warm next-frame presentation class.
        let b = Baseline::seed();
        for kind in ["tab_hover", "tab_press"] {
            let entry = b
                .entries
                .get(kind)
                .unwrap_or_else(|| panic!("seed baseline must carry the {kind} contract budget"));
            assert_eq!(entry.budget_p95_ms, 50.0, "{kind} warm p95 ceiling");
            assert_eq!(entry.budget_max_ms, Some(100.0), "{kind} warm max ceiling");
            let mut slow = BTreeMap::new();
            slow.insert(kind.into(), measured(120.0, 160.0));
            let r = evaluate(&slow, &b, false);
            assert!(
                r.iter().any(|x| x.rule == "abs_p95"),
                "{kind} absolute p95 ceiling enforced"
            );
            assert!(
                r.iter().any(|x| x.rule == "abs_max"),
                "{kind} absolute max ceiling enforced"
            );
        }
    }

    #[test]
    fn command_budgets_are_seeded_and_enforced() {
        // Regression guard: the command-surface scenarios must carry their
        // contract budgets in the seed so a future command-surface regression
        // trips the gate instead of being silently unreported. `command_type`
        // and `command_activate` are warm (50/100); `command_activate_new_tab`
        // opens a fresh tab/session — the session-recreation class (100/150);
        // `palette_open` and `overlay_open` present the two §11.4 surfaces on
        // the next frame — the warm class (50/100).
        let b = Baseline::seed();
        for (kind, p95, max) in [
            ("command_type", 50.0, 100.0),
            ("command_activate", 50.0, 100.0),
            ("command_activate_new_tab", 100.0, 150.0),
            ("palette_open", 50.0, 100.0),
            ("overlay_open", 50.0, 100.0),
        ] {
            let entry = b
                .entries
                .get(kind)
                .unwrap_or_else(|| panic!("seed baseline must carry the {kind} contract budget"));
            assert_eq!(entry.budget_p95_ms, p95);
            assert_eq!(entry.budget_max_ms, Some(max));
            let mut slow = BTreeMap::new();
            slow.insert(kind.into(), measured(p95 + 20.0, max + 20.0));
            let r = evaluate(&slow, &b, false);
            assert!(
                r.iter().any(|x| x.rule == "abs_p95"),
                "{kind} absolute p95 ceiling enforced"
            );
        }
    }

    #[test]
    fn within_budget_is_green() {
        let b = baseline_accepted();
        let mut m = BTreeMap::new();
        m.insert("warm_tab_switch".into(), measured(45.0, 90.0));
        m.insert("sidebar_toggle".into(), measured(40.0, 75.0));
        assert!(evaluate(&m, &b, true).is_empty());
    }

    #[test]
    fn downloads_budgets_are_seeded_and_enforced() {
        // Regression guard: the downloads-panel scenarios must carry their
        // contract budgets in the seed so a future panel-open, progress
        // commit, or retry-row render regression trips the gate instead of
        // being silently unreported. All three are the warm presentation
        // class (50/100): the footer `[Downloads]` open, the L6 progress
        // commit, and the retry row action.
        let b = Baseline::seed();
        for (kind, p95, max) in [
            ("downloads_panel_open", 50.0, 100.0),
            ("download_progress_commit", 50.0, 100.0),
            ("download_row_retry", 50.0, 100.0),
        ] {
            let entry = b
                .entries
                .get(kind)
                .unwrap_or_else(|| panic!("seed baseline must carry the {kind} contract budget"));
            assert_eq!(entry.budget_p95_ms, p95);
            assert_eq!(entry.budget_max_ms, Some(max));
            let mut slow = BTreeMap::new();
            slow.insert(kind.into(), measured(p95 + 20.0, max + 20.0));
            let r = evaluate(&slow, &b, false);
            assert!(
                r.iter().any(|x| x.rule == "abs_p95"),
                "{kind} absolute p95 ceiling enforced"
            );
            assert!(
                r.iter().any(|x| x.rule == "abs_max"),
                "{kind} absolute max ceiling enforced"
            );
            let mut good = BTreeMap::new();
            good.insert(kind.into(), measured(45.0, 90.0));
            assert!(
                evaluate(&good, &b, false).is_empty(),
                "{kind} warm samples within budget stay green"
            );
        }
    }

    #[test]
    fn find_budgets_are_seeded_and_enforced() {
        // Regression guard: the find-in-page scenarios must carry their
        // contract budgets in the seed so a future find-presentation or
        // WebKit-report regression trips the gate instead of being silently
        // unreported. All three are the warm command-surface class
        // (50/100): Cmd-F open, query result, and warm next/previous step.
        let b = Baseline::seed();
        for (kind, p95, max) in [
            ("find_open", 50.0, 100.0),
            ("find_query_result", 50.0, 100.0),
            ("find_step", 50.0, 100.0),
        ] {
            let entry = b
                .entries
                .get(kind)
                .unwrap_or_else(|| panic!("seed baseline must carry the {kind} contract budget"));
            assert_eq!(entry.budget_p95_ms, p95);
            assert_eq!(entry.budget_max_ms, Some(max));
            let mut slow = BTreeMap::new();
            slow.insert(kind.into(), measured(p95 + 20.0, max + 20.0));
            let r = evaluate(&slow, &b, false);
            assert!(
                r.iter().any(|x| x.rule == "abs_p95"),
                "{kind} absolute p95 ceiling enforced"
            );
            assert!(
                r.iter().any(|x| x.rule == "abs_max"),
                "{kind} absolute max ceiling enforced"
            );
            let mut good = BTreeMap::new();
            good.insert(kind.into(), measured(45.0, 90.0));
            assert!(
                evaluate(&good, &b, false).is_empty(),
                "{kind} warm samples within budget stay green"
            );
        }
    }

    #[test]
    fn zoom_budgets_are_seeded_and_enforced() {
        // Regression guard (queue item 11, part A): the zoom-step scenario
        // must carry its contract budget in the seed so a future zoom
        // regression trips the gate instead of being silently unreported.
        // `zoom_step` reuses the existing warm command-surface class
        // (50/100 ms) — the lifecycle change, persistence write, and native
        // `pageZoom` application are a warm main-thread interaction, no new
        // budget class.
        let b = Baseline::seed();
        let entry = b
            .entries
            .get("zoom_step")
            .expect("seed baseline must carry the zoom_step contract budget");
        assert_eq!(entry.budget_p95_ms, 50.0);
        assert_eq!(entry.budget_max_ms, Some(100.0));
        let mut slow = BTreeMap::new();
        slow.insert("zoom_step".into(), measured(120.0, 160.0));
        let r = evaluate(&slow, &b, false);
        assert!(r.iter().any(|x| x.rule == "abs_p95"));
        assert!(r.iter().any(|x| x.rule == "abs_max"));
        let mut good = BTreeMap::new();
        good.insert("zoom_step".into(), measured(45.0, 90.0));
        assert!(evaluate(&good, &b, false).is_empty());
    }

    #[test]
    fn context_menu_budgets_are_seeded_and_enforced() {
        // Regression guard (task packet `bbr-context-menus-over-web`): the
        // context-menu open scenario must carry its contract budget in the
        // seed so a future menu-presentation regression trips the gate
        // instead of being silently skipped (an unseeded kind is skipped by
        // `evaluate` with no signal). `context_menu_open` reuses the warm
        // portal-presentation class (50/100 ms) of the find bar open — the
        // row-model build, anchored placement, portal presentation, and
        // focus handoff, committed from the portal render — no new budget
        // class.
        let b = Baseline::seed();
        let entry = b
            .entries
            .get("context_menu_open")
            .expect("seed baseline must carry the context_menu_open contract budget");
        assert_eq!(entry.budget_p95_ms, 50.0);
        assert_eq!(entry.budget_max_ms, Some(100.0));
        let mut slow = BTreeMap::new();
        slow.insert("context_menu_open".into(), measured(120.0, 160.0));
        let r = evaluate(&slow, &b, false);
        assert!(r.iter().any(|x| x.rule == "abs_p95"));
        assert!(r.iter().any(|x| x.rule == "abs_max"));
        let mut good = BTreeMap::new();
        good.insert("context_menu_open".into(), measured(45.0, 90.0));
        assert!(evaluate(&good, &b, false).is_empty());
    }

    #[test]
    fn session_recreation_budget_holds_observed_p95_and_trips_on_doubling() {
        // Reopen presents a freshly created WKWebView (measured 24–60 ms on
        // the benchmark host), a distinct class from warm presentation
        // commits: the 100 ms p95 ceiling keeps ≥ 2.5× headroom while a 2×
        // real regression still trips.
        let b = baseline_accepted();
        let mut m = BTreeMap::new();
        m.insert("tab_reopen".into(), measured(60.0, 90.0)); // worst observed
        assert!(
            evaluate(&m, &b, true).is_empty(),
            "observed reopen p95 stays green"
        );
        let mut doubled = BTreeMap::new();
        doubled.insert("tab_reopen".into(), measured(120.0, 160.0));
        let r = evaluate(&doubled, &b, true);
        assert!(r.iter().any(|x| x.rule == "abs_p95"));
    }

    #[test]
    fn organize_budgets_are_seeded_and_enforced() {
        // Regression guard: the tab-organization scenarios must carry their
        // contract budgets in the seed so a future regression trips the gate.
        let b = Baseline::seed();
        for (kind, p95, max) in [
            ("tab_pin", 50.0, 100.0),
            ("tab_reorder", 50.0, 100.0),
            ("tab_move_same_profile", 50.0, 100.0),
            ("tab_move_cross_profile", 100.0, 150.0),
        ] {
            let entry = b
                .entries
                .get(kind)
                .unwrap_or_else(|| panic!("seed baseline must carry the {kind} contract budget"));
            assert_eq!(entry.budget_p95_ms, p95);
            assert_eq!(entry.budget_max_ms, Some(max));
            let mut slow = BTreeMap::new();
            slow.insert(kind.into(), measured(p95 + 20.0, max + 20.0));
            let r = evaluate(&slow, &b, false);
            assert!(
                r.iter().any(|x| x.rule == "abs_p95"),
                "{kind} absolute p95 ceiling enforced"
            );
        }
    }
}
