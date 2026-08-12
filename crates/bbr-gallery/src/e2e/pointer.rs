//! Real-pointer disclosure scenarios (live-review finding 1): a genuine
//! mouse click on the `bbr-gallery-disclosure-<group-id>` pointer target
//! collapses/expands the group through the **same** `TreeState` expansion
//! authority the keyboard uses. The click is a real platform mouse
//! down/up at the disclosure's on-screen bounds — never a direct
//! `activate_node` call.

use crate::harness::{bounds_of, click_selector, flush, run_scenario, snapshot};

/// The disclosure pointer target of the first group.
const DISCLOSURE_0: &str = "bbr-gallery-disclosure-0";

#[test]
#[ignore]
fn pointer_click_on_disclosure_collapses_and_reexpands_group() {
    run_scenario(|window, app| {
        let (group_id, group_node, story_count_in_group) = window.read(|cx| {
            let registry = app.read(cx).registry();
            let group = &registry.groups[0];
            (group.id.0, 0u64, group.stories.len())
        });
        let total_nodes = snapshot(window, app).tree_nodes.len();

        // The disclosure target is present and expanded: chevron down, all
        // stories visible.
        flush(window);
        let bounds = bounds_of(window, DISCLOSURE_0);
        assert!(bounds.size.width > gpui::px(0.0) && bounds.size.height > gpui::px(0.0));
        let snap = snapshot(window, app);
        assert!(group_expanded(&snap, group_id));
        assert_eq!(visible_stories(&snap), total_nodes - group_count(&snap));

        // A REAL pointer click on the disclosure: collapse. The expansion
        // authority flips, the stories hide, and the one focus moves to the
        // group (the same focus semantics as keyboard activation).
        click_selector(window, DISCLOSURE_0);
        flush(window);
        let snap = snapshot(window, app);
        assert!(
            !group_expanded(&snap, group_id),
            "pointer click collapses the group"
        );
        let collapsed_visible_stories = (total_nodes - group_count(&snap)) - story_count_in_group;
        assert_eq!(
            visible_stories(&snap),
            collapsed_visible_stories,
            "collapse hides only the first group's stories"
        );
        assert!(
            first_group_stories_hidden(&snap),
            "the collapsed group's story children are hidden"
        );
        assert_eq!(snap.focus_node, group_node, "one focus: the group");
        assert!(
            bounds_of(window, DISCLOSURE_0).size.width > gpui::px(0.0),
            "the disclosure target stays visible while collapsed (chevron right)"
        );
        assert_eq!(
            snap.tree_nodes
                .iter()
                .filter(|n| n.kind == "story" && n.visible)
                .count(),
            collapsed_visible_stories,
            "only non-collapsed story rows remain visible"
        );

        // A second REAL pointer click re-expands: stories reappear.
        click_selector(window, DISCLOSURE_0);
        flush(window);
        let snap = snapshot(window, app);
        assert!(
            group_expanded(&snap, group_id),
            "pointer click re-expands the group"
        );
        assert_eq!(visible_stories(&snap), total_nodes - group_count(&snap));
        assert_eq!(
            snap.focus_node, group_node,
            "focus stays on the toggled group"
        );
        assert!(story_count_in_group >= 1);
    });
}

#[test]
#[ignore]
fn pointer_click_on_group_row_uses_the_same_expansion_authority() {
    run_scenario(|window, app| {
        let (group_id, group_node, story_count_in_group) = window.read(|cx| {
            let registry = app.read(cx).registry();
            (
                registry.groups[0].id.0,
                0u64,
                registry.groups[0].stories.len(),
            )
        });
        let total_nodes = snapshot(window, app).tree_nodes.len();

        // Clicking the group ROW (not just the glyph) collapses through the
        // same authority as the disclosure target and the keyboard.
        flush(window);
        click_selector(window, "bbr-gallery-node-0");
        flush(window);
        let snap = snapshot(window, app);
        assert!(!group_expanded(&snap, group_id));
        assert_eq!(
            visible_stories(&snap),
            (total_nodes - group_count(&snap)) - story_count_in_group,
            "collapse hides only the first group's stories"
        );
        assert!(
            first_group_stories_hidden(&snap),
            "the collapsed group's story children are hidden"
        );
        assert_eq!(snap.focus_node, group_node, "one focus after row click");

        // Re-expand so the story rows render again, then a real pointer
        // click on the first story row selects it (the row target), and the
        // canvas switches to that story.
        click_selector(window, DISCLOSURE_0);
        flush(window);
        let first_story = window.read(|cx| app.read(cx).registry().stories[0].id.0);
        click_selector(window, "bbr-gallery-node-1");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.selected_story, first_story, "story row click selects");
        assert_eq!(total_nodes, snap.tree_nodes.len());
    });
}

fn group_count(snap: &crate::app::GallerySnapshot) -> usize {
    snap.tree_nodes.iter().filter(|n| n.kind == "group").count()
}

fn visible_stories(snap: &crate::app::GallerySnapshot) -> usize {
    snap.tree_nodes
        .iter()
        .filter(|n| n.kind == "story" && n.visible)
        .count()
}

fn group_expanded(snap: &crate::app::GallerySnapshot, group_id: u64) -> bool {
    snap.tree_nodes
        .iter()
        .find(|n| n.kind == "group" && n.id == group_id)
        .map(|n| n.expanded)
        .expect("group node in snapshot")
}

/// True when every story node of the first group is hidden (its children
/// are absent from the visible tree while the group is collapsed).
fn first_group_stories_hidden(snap: &crate::app::GallerySnapshot) -> bool {
    let mut in_first_group = false;
    for node in &snap.tree_nodes {
        match node.kind {
            "group" => {
                if in_first_group {
                    break; // reached the second group: group 0 is done
                }
                in_first_group = true;
            }
            "story" if in_first_group && node.visible => return false,
            _ => {}
        }
    }
    true
}
