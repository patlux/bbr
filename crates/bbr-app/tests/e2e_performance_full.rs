//! Full repeated canonical benchmark gate (`just e2e-perf-full`).
//!
//! Materially different from the short gate: it launches the real optimized
//! app `COLD_STARTUP_SAMPLES` times against fresh profiles to sample the cold
//! startup + deterministic-page-ready distributions (contract requires ≥10
//! startup samples per mode), and drives `INTERACTION_SAMPLES` warm
//! interactions once for the percentile-heavy interaction rules. Trend and
//! raw artifacts are persisted. Bounded by the recipe's longer timeout.

#![cfg(target_os = "macos")]

mod perf_common;

use perf_common::scenarios::{
    aggregate, evaluate_aggregate, run_cold_trial, run_persisted_warm_trial, ColdSample,
    TrialConfig, COLD_STARTUP_SAMPLES, INTERACTION_SAMPLES, NEW_TAB_SAMPLES,
    PERSISTED_WARM_STARTUP_SAMPLES,
};
use perf_common::{artifacts_dir, baseline_path, short_config, trend_dir};
use std::panic::{catch_unwind, AssertUnwindSafe};

fn allow_gui() -> bool {
    !std::env::var("BBR_SKIP_GUI_TESTS").is_ok_and(|value| value != "0" && value != "false")
}

#[test]
#[ignore]
fn full_repeated_cold_benchmark_meets_budgets() {
    if !allow_gui() {
        return;
    }

    let config: TrialConfig = short_config();

    let result = catch_unwind(AssertUnwindSafe(|| {
        // Trial 0 drives the interaction matrix; the rest only measure cold
        // startup + page readiness so startup distributions reach the
        // contract-required sample count without repeating warm interactions.
        let mut samples: Vec<ColdSample> = Vec::with_capacity(COLD_STARTUP_SAMPLES);
        for i in 0..COLD_STARTUP_SAMPLES {
            let drive = i == 0;
            samples.push(run_cold_trial(&config, drive, &format!("full-{i}")));
        }

        // Phase-7 persisted warm-startup scenario (warm-restore slice):
        // `PERSISTED_WARM_STARTUP_SAMPLES` warm launches against the dense
        // persisted fixture through the actual repository restore path, each
        // with the semantic fast-but-wrong guards asserted inside the trial.
        let mut agg = aggregate(samples);
        for i in 0..PERSISTED_WARM_STARTUP_SAMPLES {
            let (warm_startup, warm_page_ready) =
                run_persisted_warm_trial(&config, &format!("full-warm-{i}"));
            agg.warm_startup_ms.push(warm_startup);
            agg.warm_page_ready_ms.push(warm_page_ready);
        }

        let report = evaluate_aggregate(
            &agg,
            &config,
            &baseline_path(),
            &artifacts_dir().as_path().join("full"),
            &trend_dir(),
            "cold-fresh",
        );

        // Contract-required repetition counts (startup per mode, interactions).
        assert!(
            agg.startup_ms.len() >= COLD_STARTUP_SAMPLES,
            "cold startup samples {} < {COLD_STARTUP_SAMPLES}",
            agg.startup_ms.len()
        );
        assert!(
            agg.page_ready_ms.len() >= COLD_STARTUP_SAMPLES,
            "page-ready samples {} < {COLD_STARTUP_SAMPLES}",
            agg.page_ready_ms.len()
        );
        let warm = report.results["warm_tab_switch"].samples;
        let new_tab = report.results["new_tab_command_bar"].samples;
        let pointer = report.results["tab_select_pointer"].samples;
        let activate = report.results["tab_activate_keyboard"].samples;
        let reopen = report.results["tab_reopen"].samples;
        let space = report.results["space_switch"].samples;
        let cross_space = report.results["space_switch_cross_profile"].samples;
        let pin = report.results["tab_pin"].samples;
        let reorder = report.results["tab_reorder"].samples;
        let same_move = report.results["tab_move_same_profile"].samples;
        let cross_move = report.results["tab_move_cross_profile"].samples;
        let dense_scroll = report.results["dense_scroll"].samples;
        let dense_select = report.results["dense_select"].samples;
        let hover = report.results["tab_hover"].samples;
        let press = report.results["tab_press"].samples;
        let command_type = report.results["command_type"].samples;
        let command_first_result = report.results["command_bar_first_result"].samples;
        let command_full_search = report.results["command_bar_full_search"].samples;
        let command_activate = report.results["command_activate"].samples;
        let command_new_tab = report.results["command_activate_new_tab"].samples;
        let saved_page_search = report.results["saved_page_search_first_result"].samples;
        let saved_page_open = report.results["saved_page_open"].samples;
        let library_search = report.results["saved_pages_library_search"].samples;
        let library_scroll = report.results["saved_pages_library_scroll"].samples;
        let palette_open = report.results["palette_open"].samples;
        let overlay_open = report.results["overlay_open"].samples;
        let find_open = report.results["find_open"].samples;
        let find_query_result = report.results["find_query_result"].samples;
        let find_step = report.results["find_step"].samples;
        let zoom_step = report.results["zoom_step"].samples;
        let transparency = report.results["chrome_transparency_change"].samples;
        let context_menu_open = report.results["context_menu_open"].samples;
        let tab_menu_open = report.results["tab_context_menu_open"].samples;
        let tab_menu_submenu_commit = report.results["tab_context_menu_submenu_commit"].samples;
        let auth_sheet_open = report.results["auth_sheet_open"].samples;
        let peek_request_to_present = report.results["peek_request_to_present"].samples;
        let peek_dismiss_to_origin = report.results["peek_dismiss_to_origin"].samples;
        let peek_promote_to_tab = report.results["peek_promote_to_tab"].samples;
        let plus_menu = report.results["sidebar_plus_menu"].samples;
        let favorites = report.results["sidebar_favorites"].samples;
        let folder_toggle = report.results["sidebar_folder_toggle"].samples;
        let history_swipe = report.results["web_history_swipe_back"].samples;
        let child_return = report.results["child_tab_swipe_return"].samples;
        let unresponsive_chrome = report.results["unresponsive_chrome"].samples;
        let probe_tick = report.results["probe_tick"].samples;
        assert!(
            warm >= INTERACTION_SAMPLES,
            "warm-tab samples {warm} < {INTERACTION_SAMPLES}"
        );
        assert!(
            pointer >= INTERACTION_SAMPLES,
            "pointer-select samples {pointer} < {INTERACTION_SAMPLES}"
        );
        assert!(
            space >= INTERACTION_SAMPLES,
            "space-switch samples {space} < {INTERACTION_SAMPLES}"
        );
        assert!(
            cross_space >= INTERACTION_SAMPLES,
            "cross-profile space-switch samples {cross_space} < {INTERACTION_SAMPLES}"
        );
        assert!(
            pin >= INTERACTION_SAMPLES,
            "pin samples {pin} < {INTERACTION_SAMPLES}"
        );
        assert!(
            reorder >= INTERACTION_SAMPLES,
            "reorder samples {reorder} < {INTERACTION_SAMPLES}"
        );
        assert!(
            same_move >= INTERACTION_SAMPLES,
            "same-profile move samples {same_move} < {INTERACTION_SAMPLES}"
        );
        assert!(
            cross_move >= INTERACTION_SAMPLES,
            "cross-profile move samples {cross_move} < {INTERACTION_SAMPLES}"
        );
        assert!(
            dense_scroll >= INTERACTION_SAMPLES,
            "dense-scroll samples {dense_scroll} < {INTERACTION_SAMPLES}"
        );
        assert!(
            dense_select >= INTERACTION_SAMPLES,
            "dense-select samples {dense_select} < {INTERACTION_SAMPLES}"
        );
        assert!(
            hover >= INTERACTION_SAMPLES,
            "hover samples {hover} < {INTERACTION_SAMPLES}"
        );
        assert!(
            press >= INTERACTION_SAMPLES,
            "press samples {press} < {INTERACTION_SAMPLES}"
        );
        assert!(
            command_type >= INTERACTION_SAMPLES,
            "command-type samples {command_type} < {INTERACTION_SAMPLES}"
        );
        assert!(
            command_first_result >= INTERACTION_SAMPLES,
            "command-first-result samples {command_first_result} < {INTERACTION_SAMPLES}"
        );
        assert!(
            command_full_search >= INTERACTION_SAMPLES,
            "command-full-search samples {command_full_search} < {INTERACTION_SAMPLES}"
        );
        assert!(
            command_activate >= INTERACTION_SAMPLES,
            "command-activate samples {command_activate} < {INTERACTION_SAMPLES}"
        );
        assert!(
            command_new_tab >= NEW_TAB_SAMPLES,
            "command new-tab samples {command_new_tab} < {NEW_TAB_SAMPLES}"
        );
        assert!(
            saved_page_search >= INTERACTION_SAMPLES,
            "saved-page search samples {saved_page_search} < {INTERACTION_SAMPLES}"
        );
        assert!(
            saved_page_open >= INTERACTION_SAMPLES,
            "saved-page open samples {saved_page_open} < {INTERACTION_SAMPLES}"
        );
        assert!(
            library_search >= INTERACTION_SAMPLES,
            "library-search samples {library_search} < {INTERACTION_SAMPLES}"
        );
        assert!(
            library_scroll >= INTERACTION_SAMPLES,
            "library-scroll samples {library_scroll} < {INTERACTION_SAMPLES}"
        );
        assert!(
            palette_open >= INTERACTION_SAMPLES,
            "palette-open samples {palette_open} < {INTERACTION_SAMPLES}"
        );
        assert!(
            overlay_open >= INTERACTION_SAMPLES,
            "overlay-open samples {overlay_open} < {INTERACTION_SAMPLES}"
        );
        assert!(
            find_open >= INTERACTION_SAMPLES,
            "find-open samples {find_open} < {INTERACTION_SAMPLES}"
        );
        assert!(
            find_query_result >= INTERACTION_SAMPLES,
            "find-query samples {find_query_result} < {INTERACTION_SAMPLES}"
        );
        assert!(
            find_step >= INTERACTION_SAMPLES,
            "find-step samples {find_step} < {INTERACTION_SAMPLES}"
        );
        assert!(
            zoom_step >= INTERACTION_SAMPLES,
            "zoom-step samples {zoom_step} < {INTERACTION_SAMPLES}"
        );
        assert!(
            transparency >= INTERACTION_SAMPLES,
            "transparency samples {transparency} < {INTERACTION_SAMPLES}"
        );
        assert!(
            context_menu_open >= INTERACTION_SAMPLES,
            "context-menu open samples {context_menu_open} < {INTERACTION_SAMPLES}"
        );
        assert!(
            tab_menu_open >= INTERACTION_SAMPLES,
            "tab-menu open samples {tab_menu_open} < {INTERACTION_SAMPLES}"
        );
        assert!(
            tab_menu_submenu_commit >= INTERACTION_SAMPLES,
            "tab-menu submenu-commit samples {tab_menu_submenu_commit} < {INTERACTION_SAMPLES}"
        );
        assert!(
            plus_menu >= INTERACTION_SAMPLES,
            "sidebar plus-menu samples {plus_menu} < {INTERACTION_SAMPLES}"
        );
        assert!(
            favorites >= INTERACTION_SAMPLES,
            "sidebar favorites samples {favorites} < {INTERACTION_SAMPLES}"
        );
        assert!(
            folder_toggle >= INTERACTION_SAMPLES,
            "sidebar folder-toggle samples {folder_toggle} < {INTERACTION_SAMPLES}"
        );
        assert!(
            auth_sheet_open >= INTERACTION_SAMPLES,
            "credential-sheet open samples {auth_sheet_open} < {INTERACTION_SAMPLES}"
        );
        // N1 web-content gesture scenarios: at least 30 accepted optimized
        // samples each (the `web_history_swipe_back` / `child_tab_swipe_return`
        // contract guards). A driver that skipped, cancelled, or rejected
        // samples below the contract floor fails the gate loudly.
        assert!(
            history_swipe >= INTERACTION_SAMPLES,
            "history swipe-back samples {history_swipe} < {INTERACTION_SAMPLES}"
        );
        assert!(
            child_return >= INTERACTION_SAMPLES,
            "child-tab return samples {child_return} < {INTERACTION_SAMPLES}"
        );
        // Renderer-Isolation records two production chrome paths under one
        // key: >=30 neighbor selections plus >=30 Cmd-W closes. The idle
        // cadence scenario independently requires >=30 accepted samples.
        assert!(
            unresponsive_chrome >= INTERACTION_SAMPLES * 2,
            "unresponsive-chrome samples {unresponsive_chrome} < {}",
            INTERACTION_SAMPLES * 2
        );
        assert!(
            probe_tick >= INTERACTION_SAMPLES,
            "probe-tick samples {probe_tick} < {INTERACTION_SAMPLES}"
        );
        // PEEK scenarios likewise require at least 30 accepted optimized
        // samples once performance execution is resumed.
        assert!(
            peek_request_to_present >= INTERACTION_SAMPLES,
            "peek request-to-present samples {peek_request_to_present} < {INTERACTION_SAMPLES}"
        );
        assert!(
            peek_dismiss_to_origin >= INTERACTION_SAMPLES,
            "peek dismiss-to-origin samples {peek_dismiss_to_origin} < {INTERACTION_SAMPLES}"
        );
        assert!(
            peek_promote_to_tab >= INTERACTION_SAMPLES,
            "peek promote-to-tab samples {peek_promote_to_tab} < {INTERACTION_SAMPLES}"
        );
        // Contract-required persisted warm-startup repetition count (the
        // warm page-ready distribution needs >=10 accepted samples too).
        let warm_startup_samples = report.results["persisted_warm_startup"].samples;
        let warm_page_samples = report.results["persisted_warm_page_ready"].samples;
        assert!(
            warm_startup_samples >= PERSISTED_WARM_STARTUP_SAMPLES,
            "persisted warm-startup samples {warm_startup_samples} < {PERSISTED_WARM_STARTUP_SAMPLES}"
        );
        assert!(
            warm_page_samples >= PERSISTED_WARM_STARTUP_SAMPLES,
            "persisted warm page-ready samples {warm_page_samples} < {PERSISTED_WARM_STARTUP_SAMPLES}"
        );
        assert!(
            activate >= NEW_TAB_SAMPLES,
            "keyboard-activate samples {activate} < {NEW_TAB_SAMPLES}"
        );
        assert!(
            new_tab >= NEW_TAB_SAMPLES,
            "new-tab samples {new_tab} < {NEW_TAB_SAMPLES}"
        );
        assert!(
            reopen >= NEW_TAB_SAMPLES,
            "reopen samples {reopen} < {NEW_TAB_SAMPLES}"
        );

        let canonical_note = if report.machine.canonical {
            "CANONICAL MACHINE — relative baseline enforced"
        } else {
            "REPORT-ONLY machine — relative baseline reported, absolute ceilings enforced"
        };
        eprintln!("PERFORMANCE-FULL MACHINE: {canonical_note}");
        eprintln!(
            "PERFORMANCE-FULL RESULTS\n{}",
            serde_json::to_string_pretty(&report).unwrap()
        );
        if !report.regressions.is_empty() {
            for r in &report.regressions {
                eprintln!("REGRESSION {:?} {}: {}", r.kind, r.rule, r.detail);
            }
            panic!(
                "performance regressions: {}",
                serde_json::to_string(&report.regressions).unwrap()
            );
        }
        eprintln!("PERFORMANCE-FULL E2E PASSED");
    }));

    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}
