//! The every-story-renders sweep and the mode/state switching scenario.
//! Both navigate the tree with **real keyboard events** (Home/↓/Enter via
//! the production key path) and assert the semantic snapshot — never
//! pixels. The sweep also records report-only frame-time observations.

use std::time::Instant;

use crate::harness::{flush, move_to_story, run_scenario, snapshot};
use bbr_ui::theme::{ContrastMode, Density, ThemeMode};

/// The perf-sanity report directory (report-only, never a product budget).
fn perf_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("target")
        .join("bbr-gallery-perf")
}

/// Record one report-only frame-time observation.
fn record_perf(report: &mut Vec<(String, f64)>, label: &str, start: Instant) {
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    report.push((label.to_string(), elapsed_ms));
}

#[test]
#[ignore]
fn every_registered_story_renders_with_semantic_root_and_stable_id() {
    run_scenario(|window, app| {
        let (story_ids, story_labels) = window.read(|cx| {
            let registry = app.read(cx).registry();
            let ids: Vec<u64> = registry.stories.iter().map(|s| s.id.0).collect();
            let labels: Vec<String> = registry
                .stories
                .iter()
                .map(|s| s.label.to_string())
                .collect();
            (ids, labels)
        });
        let snap = snapshot(window, app);
        let story_nodes: Vec<u64> = snap
            .tree_nodes
            .iter()
            .filter(|n| n.kind == "story")
            .map(|n| n.id)
            .collect();
        assert_eq!(story_nodes.len(), story_ids.len());
        let mut report = Vec::new();
        for &story_id in &story_ids {
            // Real keyboard navigation to the story, then Enter to select it
            // on the canvas — the exact production key path.
            let start = Instant::now();
            move_to_story(window, app, story_id);
            flush(window); // proves the story renders without panic
            record_perf(&mut report, &format!("story-switch-{story_id}"), start);
            let snap = snapshot(window, app);
            let story_index = story_ids
                .iter()
                .position(|&id| id == snap.selected_story)
                .expect("story in registry");
            assert_eq!(
                snap.canvas_story_id,
                format!("bbr-gallery-canvas-story-{}", story_ids[story_index]),
                "every story exposes its semantic root with its stable id"
            );
            assert_eq!(snap.selected_story_label, story_labels[story_index]);
        }
        write_perf_report("every-story-renders", &report);
    });
}

#[test]
#[ignore]
fn mode_state_switching_changes_canvas_fields_and_keeps_ids_stable() {
    run_scenario(|window, app| {
        let mut report = Vec::new();
        let baseline = snapshot(window, app);
        let baseline_canvas = baseline.canvas_story_id.clone();

        // Mode: light (through the production control path).
        let start = Instant::now();
        window.update(|_window, cx| {
            app.update(cx, |app, cx| {
                app.apply_control(crate::controls::ControlAction::Mode(ThemeMode::Light), cx)
            });
        });
        flush(window);
        record_perf(&mut report, "mode-change-light", start);
        let snap = snapshot(window, app);
        assert_eq!(snap.theme, ThemeMode::Light);
        assert_eq!(
            snap.canvas_story_id, baseline_canvas,
            "ids stay stable across mode change"
        );

        // Contrast: increased.
        window.update(|_window, cx| {
            app.update(cx, |app, cx| {
                app.apply_control(
                    crate::controls::ControlAction::Contrast(ContrastMode::Increased),
                    cx,
                )
            });
        });
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.contrast, ContrastMode::Increased);

        // Density: extralarge.
        window.update(|_window, cx| {
            app.update(cx, |app, cx| {
                app.apply_control(
                    crate::controls::ControlAction::Density(Density::ExtraLarge),
                    cx,
                )
            });
        });
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.density, Density::ExtraLarge);

        // Reduced motion: on.
        window.update(|_window, cx| {
            app.update(cx, |app, cx| {
                app.apply_control(crate::controls::ControlAction::ReducedMotion(true), cx)
            });
        });
        flush(window);
        let snap = snapshot(window, app);
        assert!(snap.reduced_motion);

        // Story state: navigate with real keys to the States story (which
        // registers the §10 vocabulary) and switch its state.
        let states_story = window.read(|cx| {
            app.read(cx)
                .registry()
                .stories
                .iter()
                .position(|s| s.label == "States")
                .expect("States story registered")
        });
        move_to_story(window, app, states_story as u64);
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story, states_story as u64);
        assert_eq!(snap.story_state, "default");
        let start = Instant::now();
        window.update(|_window, cx| {
            app.update(cx, |app, cx| {
                app.apply_control(crate::controls::ControlAction::State(4), cx)
            });
        });
        flush(window);
        record_perf(&mut report, "state-change-loading", start);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.story_state, "loading",
            "state control switches the story state"
        );
        assert_eq!(snap.story_state_index, 4);
        assert_eq!(
            snap.canvas_story_id,
            format!("bbr-gallery-canvas-story-{states_story}"),
            "ids stay stable across state change"
        );

        // Back to dark/standard/normal: the shipped defaults round-trip.
        window.update(|_window, cx| {
            app.update(cx, |app, cx| {
                app.apply_control(crate::controls::ControlAction::Mode(ThemeMode::Dark), cx);
                app.apply_control(
                    crate::controls::ControlAction::Contrast(ContrastMode::Standard),
                    cx,
                );
                app.apply_control(crate::controls::ControlAction::Density(Density::Normal), cx);
                app.apply_control(crate::controls::ControlAction::ReducedMotion(false), cx);
            });
        });
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.theme, ThemeMode::Dark);
        assert_eq!(snap.contrast, ContrastMode::Standard);
        assert_eq!(snap.density, Density::Normal);
        assert!(!snap.reduced_motion);

        write_perf_report("mode-state-switching", &report);
    });
}

/// Write the report-only perf observations (sanity only, never a budget).
fn write_perf_report(suite: &str, report: &[(String, f64)]) {
    let dir = perf_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(format!("{suite}.json"));
    let body = report
        .iter()
        .map(|(label, ms)| format!("    \"{label}\": {ms:.3}"))
        .collect::<Vec<_>>()
        .join(",\n");
    let json = format!(
        "{{\n  \"suite\": \"{suite}\",\n  \"sanity_only\": true,\n  \"note\": \"report-only frame-time observations; never a product budget\",\n  \"observations_ms\": {{\n{body}\n  }}\n}}\n"
    );
    let _ = std::fs::write(&path, json);
    eprintln!("gallery perf sanity (report-only): {}", path.display());
}
