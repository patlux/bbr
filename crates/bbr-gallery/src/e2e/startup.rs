//! Startup scenario: the tree and the default story present, §15 mode
//! fields at the shipped defaults, the honest scope copy (finding 3), and
//! the window title.

use crate::harness::{flush, run_scenario, snapshot};
use bbr_ui::theme::{ContrastMode, Density, ThemeMode};

#[test]
#[ignore]
fn gallery_startup_presents_tree_and_default_story() {
    run_scenario(|window, app| {
        let snap = snapshot(window, app);
        // The tree and canvas semantic roots exist.
        assert_eq!(snap.tree_id, "bbr-gallery-tree");
        assert_eq!(snap.canvas_id, "bbr-gallery-canvas");
        // Every registered node is present in declaration order.
        let (story_count, group_count) = window.read(|cx| {
            let registry = app.read(cx).registry();
            (registry.stories.len(), registry.groups.len())
        });
        assert_eq!(snap.tree_nodes.len(), story_count + group_count);
        assert!(
            snap.tree_nodes.iter().all(|n| n.visible),
            "all groups start expanded"
        );
        assert!(
            snap.tree_nodes
                .iter()
                .all(|n| n.kind != "group" || n.expanded),
            "all groups start expanded (snapshot field)"
        );
        // The default story is the first declared story, with its stable id.
        let (first_id, first_label) = window.read(|cx| {
            let first = app
                .read(cx)
                .registry()
                .stories
                .first()
                .expect("first story");
            (first.id.0, first.label.to_string())
        });
        assert_eq!(snap.selected_story, first_id);
        assert_eq!(snap.selected_story_label, first_label);
        assert_eq!(
            snap.canvas_story_id,
            format!("bbr-gallery-canvas-story-{first_id}")
        );
        // §15 mode fields at the shipped defaults.
        assert_eq!(snap.theme, ThemeMode::Dark);
        assert_eq!(snap.contrast, ContrastMode::Standard);
        assert_eq!(snap.density, Density::Normal);
        assert!(!snap.reduced_motion);
        // The default story renders without panic.
        flush(window);
    });
}

#[test]
#[ignore]
fn gallery_copy_is_honest_scope_with_g3_continuation() {
    run_scenario(|window, app| {
        let snap = snapshot(window, app);
        // Finding 3: the visible catalog copy names the shipped groups
        // (Foundations / Primitives / Browser Components) and points
        // honestly to the later Gallery phase — no complete-explorer
        // claim.
        assert_eq!(snap.scope, "Foundations / Primitives / Browser Components");
        assert!(snap.continuation.contains("G2"));
        assert!(snap.continuation.contains("G3"));
        // The stable copy ids render.
        flush(window);
        for selector in [
            "bbr-gallery-title",
            "bbr-gallery-scope",
            "bbr-gallery-continuation",
        ] {
            assert!(
                window.debug_bounds(selector).is_some(),
                "copy element {selector} renders"
            );
        }
        // The window title carries the same honest scope.
        let title = window.window_title();
        assert_eq!(
            title.as_deref(),
            Some("BBR Gallery — Foundations / Primitives / Browser Components")
        );
        let _ = app;
    });
}
