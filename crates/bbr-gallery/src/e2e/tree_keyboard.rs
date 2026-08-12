//! Tree keyboard scenarios: selection/activation, a dedicated keyboard
//! collapse/expand suite (finding 1), and the stable-`StoryId` contract.

use crate::harness::{dispatch, run_scenario, snapshot};

#[test]
#[ignore]
fn tree_keyboard_operates_selection_and_activation() {
    run_scenario(|window, app| {
        let (story_count, group_count, first_story_id, second_story_id) = window.read(|cx| {
            let registry = app.read(cx).registry();
            (
                registry.stories.len(),
                registry.groups.len(),
                registry.stories[0].id.0,
                registry.stories[1].id.0,
            )
        });

        // ↓ moves to the second story; Enter selects it on the canvas.
        // (One ↓ from the group reaches the first story; a second reaches
        // the second.)
        dispatch(window, "down");
        dispatch(window, "down");
        dispatch(window, "enter");
        let snap = snapshot(window, app);
        assert_eq!(
            snap.selected_story, second_story_id,
            "Enter activates the focused story"
        );

        // ↑ returns to the first story; Space also activates.
        dispatch(window, "up");
        dispatch(window, "space");
        let snap = snapshot(window, app);
        assert_eq!(
            snap.selected_story, first_story_id,
            "Space activates the focused story"
        );

        // Home/End jump to the first/last visible node.
        dispatch(window, "end");
        let snap = snapshot(window, app);
        assert_eq!(
            snap.focus_node,
            snap.tree_nodes.last().expect("last node").id
        );
        dispatch(window, "home");
        let snap = snapshot(window, app);
        assert_eq!(snap.focus_node, snap.tree_nodes[0].id);

        // ← on the first group collapses it: story nodes become hidden.
        dispatch(window, "left");
        let snap = snapshot(window, app);
        let visible_stories = snap
            .tree_nodes
            .iter()
            .filter(|n| n.kind == "story" && n.visible)
            .count();
        assert_eq!(
            visible_stories,
            story_count - group_story_count(&snap, 0),
            "collapse hides the group's stories"
        );
        // → re-expands and enters the first child story.
        dispatch(window, "right");
        dispatch(window, "right");
        let snap = snapshot(window, app);
        assert!(
            snap.tree_nodes.iter().all(|n| n.visible),
            "group re-expanded"
        );
        assert_eq!(
            snap.focus_node, snap.tree_nodes[1].id,
            "right on an expanded group enters its first story"
        );

        // The canvas story id stays stable across all tree movement.
        let snap = snapshot(window, app);
        assert_eq!(
            snap.canvas_story_id,
            format!("bbr-gallery-canvas-story-{}", snap.selected_story)
        );
        assert!(group_count >= 1);
    });
}

/// Finding 1: keyboard collapse/expand on group rows — Enter/Space toggle
/// through the one expansion authority, `left`/`right` collapse/expand, and
/// child visibility + the one focus follow every step.
#[test]
#[ignore]
fn tree_keyboard_collapse_expand_groups() {
    run_scenario(|window, app| {
        let (group_id, group_node, first_story_node, story_count_in_group) = window.read(|cx| {
            let registry = app.read(cx).registry();
            let group = &registry.groups[0];
            let story_count_in_group = group.stories.len();
            (group.id.0, 0u64, 1u64, story_count_in_group)
        });
        let total_nodes = snapshot(window, app).tree_nodes.len();

        // Start: first group focused (Home), expanded, all stories visible.
        dispatch(window, "home");
        let snap = snapshot(window, app);
        assert_eq!(snap.focus_node, group_node);
        assert!(group_expanded(&snap, group_id));
        assert_eq!(visible_stories(&snap), total_nodes - group_count(&snap));

        // Space on the focused group toggles it collapsed: the disclosure
        // state flips and the group's stories disappear from the visible
        // tree — the one focus stays on the group.
        dispatch(window, "space");
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
        assert_eq!(snap.focus_node, group_node, "one focus: the group");

        // Enter on the collapsed group re-expands it.
        dispatch(window, "enter");
        let snap = snapshot(window, app);
        assert!(group_expanded(&snap, group_id));
        assert_eq!(visible_stories(&snap), total_nodes - group_count(&snap));
        assert_eq!(snap.focus_node, group_node);

        // `left` collapses; `right` re-expands; `right` again enters the
        // first child story (focus moves into the group, still one focus).
        dispatch(window, "left");
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
        dispatch(window, "right");
        let snap = snapshot(window, app);
        assert!(group_expanded(&snap, group_id));
        dispatch(window, "right");
        let snap = snapshot(window, app);
        assert_eq!(
            snap.focus_node, first_story_node,
            "right on an expanded group enters its first story"
        );
        assert_eq!(visible_stories(&snap), total_nodes - group_count(&snap));
        assert!(story_count_in_group >= 1);

        // `left` from a story returns to the parent group (one focus).
        dispatch(window, "left");
        let snap = snapshot(window, app);
        assert_eq!(snap.focus_node, group_node);
    });
}

#[test]
#[ignore]
fn story_ids_are_stable_across_fixture_label_edits() {
    // The registry unit test pins the first-N assignment; this E2E proves
    // the same contract through the running app: the snapshot's story ids
    // equal the declaration-order prefix, so a label edit can never shift
    // an id (the ids are never derived from labels).
    run_scenario(|window, app| {
        let expected: Vec<u64> = window.read(|cx| {
            app.read(cx)
                .registry()
                .stories
                .iter()
                .map(|s| s.id.0)
                .collect()
        });
        let snap = snapshot(window, app);
        // The snapshot's story nodes expose the stable `StoryId` (the tree
        // node id also counts group nodes, so it is a different space).
        let story_ids: Vec<u64> = snap.tree_nodes.iter().filter_map(|n| n.story_id).collect();
        assert_eq!(
            story_ids, expected,
            "story ids are the declaration-order prefix"
        );
        // The Core Controls stories appended after the pinned prefix are
        // part of the same sequential assignment.
        let labels: Vec<&str> = snap
            .tree_nodes
            .iter()
            .filter_map(|n| n.story_id.map(|_| n.label.as_str()))
            .collect();
        assert!(labels.contains(&"Button"));
        assert!(labels.contains(&"Input"));
        assert!(labels.contains(&"Progress"));
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

/// The number of story nodes in the first group (for the collapse check).
fn group_story_count(snap: &crate::app::GallerySnapshot, group_index: usize) -> usize {
    let mut count = 0;
    let mut in_group = false;
    for node in &snap.tree_nodes {
        if node.kind == "group" {
            if in_group {
                break;
            }
            if group_index == 0 {
                in_group = true;
            }
            continue;
        }
        if in_group {
            count += 1;
        }
    }
    count
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
