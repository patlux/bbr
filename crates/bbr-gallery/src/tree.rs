//! The gallery tree model — pure focus/selection/expansion logic over the
//! catalog's flattened declaration order (packet `component-gallery-mvp.md`).
//!
//! The tree is a flat list of nodes (groups and stories in declaration
//! order). Groups may be collapsed, hiding their story children. There is
//! exactly **one tree focus**; `Enter`/`Space` on a story selects it on the
//! canvas. The model is GPUI-free so the keyboard contract is unit-tested
//! without a window; the tree surface renders from it with stable ids.

use crate::registry::{CatalogRegistry, GroupId, NodeId, StoryId};

/// What a tree node is: a group or a story.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Group(GroupId),
    Story(StoryId),
}

/// One flattened tree node (declaration order).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub label: &'static str,
    /// The owning group (a group node's own group id).
    pub group: GroupId,
    /// 0 for groups, 1 for stories (single-level tree in v1).
    pub depth: usize,
}

/// The tree state: nodes, per-group expansion, the one focus, and the
/// canvas selection.
#[derive(Debug, Clone)]
pub struct TreeState {
    pub nodes: Vec<TreeNode>,
    /// Per group id (index = group id .0): whether the group is expanded.
    pub expanded: Vec<bool>,
    /// The focused node index (the keyboard cursor).
    pub focus: usize,
    /// The story currently selected on the canvas.
    pub selected_story: StoryId,
}

impl TreeState {
    /// Build the tree from the catalog: groups and stories in declaration
    /// order, all groups expanded, focus on the first node, canvas on the
    /// default story.
    pub fn from_catalog(registry: &CatalogRegistry) -> Self {
        let mut nodes = Vec::new();
        let mut expanded = Vec::new();
        let mut node_id = 0u64;
        for group in &registry.groups {
            nodes.push(TreeNode {
                id: NodeId(node_id),
                kind: NodeKind::Group(group.id),
                label: group.label,
                group: group.id,
                depth: 0,
            });
            node_id += 1;
            expanded.push(true);
            for story in &group.stories {
                // A reserved `StoryId` slot (e.g. the G2-TAB Tab Row at
                // id 22) registers no story and renders no tree node; the
                // owning lane declares the story at this id on merge.
                if story.kind == crate::registry::StoryKind::Reserved {
                    continue;
                }
                nodes.push(TreeNode {
                    id: NodeId(node_id),
                    kind: NodeKind::Story(story.id),
                    label: story.label,
                    group: group.id,
                    depth: 1,
                });
                node_id += 1;
            }
        }
        TreeState {
            nodes,
            expanded,
            focus: 0,
            selected_story: registry.default_story(),
        }
    }

    /// The visible node indices: groups are always visible; a story is
    /// visible iff its group is expanded.
    pub fn visible(&self) -> Vec<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| match node.kind {
                NodeKind::Group(_) => true,
                NodeKind::Story(_) => self.expanded[node.group.0 as usize],
            })
            .map(|(index, _)| index)
            .collect()
    }

    /// The focused node's index, clamped to a visible node.
    pub fn focused_index(&self) -> usize {
        let visible = self.visible();
        let pos = visible.iter().position(|&i| i == self.focus).unwrap_or(0);
        visible[pos]
    }

    /// Move the focus by `delta` visible nodes (`↑`/`↓`).
    pub fn move_focus(&mut self, delta: isize) {
        let visible = self.visible();
        if visible.is_empty() {
            return;
        }
        let pos = visible.iter().position(|&i| i == self.focus).unwrap_or(0);
        let next = (pos as isize + delta).clamp(0, visible.len() as isize - 1) as usize;
        self.focus = visible[next];
    }

    /// `Home`: jump to the first visible node.
    pub fn home(&mut self) {
        if let Some(&first) = self.visible().first() {
            self.focus = first;
        }
    }

    /// `End`: jump to the last visible node.
    pub fn end(&mut self) {
        if let Some(&last) = self.visible().last() {
            self.focus = last;
        }
    }

    /// `←`: collapse a group, or move to the parent group from a story.
    pub fn collapse(&mut self) {
        let node = self.nodes[self.focus];
        match node.kind {
            NodeKind::Group(group) => {
                if self.expanded[group.0 as usize] {
                    self.expanded[group.0 as usize] = false;
                }
            }
            NodeKind::Story(_) => {
                // Move to the parent group (which is visible).
                if let Some(pos) = self
                    .nodes
                    .iter()
                    .position(|n| n.kind == NodeKind::Group(node.group))
                {
                    self.focus = pos;
                }
            }
        }
    }

    /// `→`: expand a group, or enter a story from a group.
    pub fn expand(&mut self) {
        let node = self.nodes[self.focus];
        match node.kind {
            NodeKind::Group(group) => {
                if self.expanded[group.0 as usize] {
                    // Already expanded: enter the first child story.
                    if let Some(child) = self
                        .nodes
                        .iter()
                        .position(|n| matches!(n.kind, NodeKind::Story(_)) && n.group == group)
                    {
                        self.focus = child;
                    }
                } else {
                    self.expanded[group.0 as usize] = true;
                }
            }
            NodeKind::Story(_) => {}
        }
    }

    /// `Enter`/`Space`: activate the focused node. A story is selected on
    /// the canvas; a group toggles expansion.
    pub fn activate(&mut self) {
        let node = self.nodes[self.focus];
        match node.kind {
            NodeKind::Group(group) => {
                self.expanded[group.0 as usize] = !self.expanded[group.0 as usize];
            }
            NodeKind::Story(id) => {
                self.selected_story = id;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stories::catalog;

    fn tree() -> TreeState {
        TreeState::from_catalog(&catalog())
    }

    /// The catalog's first group and first story ids (declaration order).
    fn first_group_id(tree: &TreeState) -> GroupId {
        match tree.nodes[0].kind {
            NodeKind::Group(id) => id,
            _ => panic!("first node is a group"),
        }
    }

    #[test]
    fn initial_state_focuses_first_node_and_default_story() {
        let tree = tree();
        assert_eq!(tree.focus, 0);
        assert_eq!(tree.selected_story, tree.nodes[1].kind.into_story());
        // All groups expanded initially.
        assert!(tree.expanded.iter().all(|&e| e));
    }

    #[test]
    fn up_down_move_through_visible_nodes() {
        let mut tree = tree();
        let visible = tree.visible();
        assert!(
            visible.len() >= 3,
            "catalog has at least one group + stories"
        );
        tree.move_focus(1);
        assert_eq!(tree.focus, visible[1]);
        tree.move_focus(1);
        assert_eq!(tree.focus, visible[2]);
        tree.move_focus(-1);
        assert_eq!(tree.focus, visible[1]);
        // Clamp at the ends (a delta beyond the visible range).
        tree.move_focus(-100);
        assert_eq!(tree.focus, visible[0]);
        tree.move_focus(100);
        assert_eq!(tree.focus, *visible.last().unwrap());
    }

    #[test]
    fn collapse_hides_stories_and_left_returns_to_parent() {
        let mut tree = tree();
        let group = first_group_id(&tree);
        // Focus the first group, collapse it.
        tree.focus = 0;
        tree.collapse();
        assert!(!tree.expanded[group.0 as usize]);
        let visible = tree.visible();
        assert_eq!(
            visible.len(),
            tree.nodes.len() - tree.group_story_count(group)
        );
        // From a story, `←` moves to the parent group.
        tree.expanded[group.0 as usize] = true;
        tree.focus = 1; // first story of the first group
        tree.collapse();
        assert_eq!(tree.focus, 0, "left from a story moves to its parent group");
    }

    #[test]
    fn expand_enters_first_child_from_an_expanded_group() {
        let mut tree = tree();
        let group = first_group_id(&tree);
        tree.focus = 0;
        // Collapse then expand: focus enters the first child story.
        tree.collapse();
        assert!(!tree.expanded[group.0 as usize]);
        tree.expand();
        assert!(tree.expanded[group.0 as usize]);
        tree.expand();
        assert_eq!(
            tree.focus, 1,
            "right on an expanded group enters its first story"
        );
    }

    #[test]
    fn home_end_jump_to_first_and_last_visible() {
        let mut tree = tree();
        tree.move_focus(2);
        tree.home();
        assert_eq!(tree.focus, 0);
        tree.end();
        let visible = tree.visible();
        assert_eq!(tree.focus, *visible.last().unwrap());
    }

    #[test]
    fn activate_selects_story_on_canvas_and_toggles_groups() {
        let mut tree = tree();
        let group = first_group_id(&tree);
        // Activate a story: canvas selection changes.
        tree.focus = 1;
        let story = tree.nodes[1].kind.into_story();
        tree.activate();
        assert_eq!(tree.selected_story, story);
        // Activate a group: expansion toggles.
        tree.focus = 0;
        tree.activate();
        assert!(!tree.expanded[group.0 as usize]);
        tree.activate();
        assert!(tree.expanded[group.0 as usize]);
    }

    #[test]
    fn focus_stays_on_visible_nodes_after_collapse() {
        let mut tree = tree();
        // Focus a story in the first group, then collapse the group: the
        // focused index must resolve to a visible node.
        tree.focus = 1;
        tree.collapse(); // story -> parent group
        assert_eq!(tree.focus, 0);
        let visible = tree.visible();
        assert!(visible.contains(&tree.focused_index()));
    }

    use crate::tree_surface::disclosure_id;

    #[test]
    fn disclosure_ids_are_stable_group_targets() {
        // The pointer target id is derived from the stable GroupId, never
        // from a label or a tree node index, so a catalog reorder or label
        // edit cannot change where the pointer toggles.
        let tree = tree();
        let group = first_group_id(&tree);
        assert_eq!(
            disclosure_id(group),
            format!("bbr-gallery-disclosure-{}", group.0)
        );
        assert!(disclosure_id(group).starts_with("bbr-gallery-disclosure-"));
        // Distinct groups have distinct targets.
        let second_group = match tree
            .nodes
            .iter()
            .find(|n| matches!(n.kind, NodeKind::Group(id) if id != group))
        {
            Some(node) => match node.kind {
                NodeKind::Group(id) => id,
                _ => unreachable!(),
            },
            None => return, // single-group catalogs still keep the contract
        };
        assert_ne!(disclosure_id(group), disclosure_id(second_group));
    }

    #[test]
    fn pointer_and_keyboard_share_the_one_expansion_authority() {
        // The pointer click target and the keyboard both drive `expanded[]`
        // through the same `activate` method: a click on a group toggles
        // exactly what Enter/Space toggles, and `left`/`right` resolve to
        // the same state.
        let mut tree = tree();
        let group = first_group_id(&tree);
        assert!(tree.expanded[group.0 as usize]);

        // Pointer path: focus the group and `activate` (what the disclosure
        // click does) — the group collapses and its stories hide.
        tree.focus = 0;
        tree.activate();
        assert!(!tree.expanded[group.0 as usize]);
        assert_eq!(
            tree.visible().len(),
            tree.nodes.len() - tree.group_story_count(group),
            "collapsed stories are hidden for the pointer path too"
        );

        // Keyboard `right` on the collapsed group re-expands.
        tree.expand();
        assert!(tree.expanded[group.0 as usize]);
        assert_eq!(tree.visible().len(), tree.nodes.len());

        // Keyboard `left` collapses again; the pointer `activate` restores.
        tree.collapse();
        assert!(!tree.expanded[group.0 as usize]);
        tree.activate();
        assert!(tree.expanded[group.0 as usize]);
        assert_eq!(tree.focus, 0, "one focus, unchanged by expansion toggles");
    }

    impl TreeState {
        fn group_story_count(&self, group: GroupId) -> usize {
            self.nodes
                .iter()
                .filter(|n| matches!(n.kind, NodeKind::Story(_)) && n.group == group)
                .count()
        }
    }

    trait IntoStory {
        fn into_story(self) -> StoryId;
    }
    impl IntoStory for NodeKind {
        fn into_story(self) -> StoryId {
            match self {
                NodeKind::Story(id) => id,
                _ => panic!("expected a story node"),
            }
        }
    }
}
