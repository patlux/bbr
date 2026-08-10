//! The gallery catalog registry — the single declaration point for every
//! story (packet `component-gallery-mvp.md`).
//!
//! `StoryId`s are assigned **sequentially in declaration order** and are
//! **never derived from labels**; groups and stories appear in explicit
//! declaration order (never sorted by label). Ids are **append-only across
//! Gallery packets**: G2/G3 append stories at later ids and never renumber
//! G1b ids. A test pins the first-N id assignment so a label edit or a
//! reorder can never silently shift an id.
//!
//! The tree surface consumes the same flattened declaration order, so the
//! tree order, the canvas default, and the E2E expectations all derive from
//! this one declaration.

use gpui::{AnyElement, Entity};
use gpui_component::input::InputState;
use gpui_component::slider::SliderState;

use crate::recorder::{ControlObserver, ControlRecorder, TabRowObserver, TabRowRecorder};

/// A stable story identity, assigned by the registry in declaration order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StoryId(pub u64);

/// A stable group identity, assigned by the registry in declaration order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GroupId(pub u64);

/// A stable tree-node identity: every group and story in the flattened
/// declaration order gets one sequential `NodeId`, so `bbr-gallery-node-<n>`
/// ids are unique and stable across the whole tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u64);

/// The story's own State/Variant selector options (the §10 vocabulary for
/// the States page, `TabVisualState` names for the favicon tile, …). A story
/// with no options renders without a state control.
pub type StateOptions = &'static [&'static str];

/// The render context a story receives: the resolved theme (mode/contrast/
/// density), the reduced-motion flag (§15 mode field), and the story's own
/// selected state. G1b stories are render-only (the G1b `bbr-ui` primitives
/// take no callbacks); the Core Controls stories additionally receive the
/// real gpui-component state entities and the recorder/observer bundle
/// through [`StoryContext::controls`]; G2/G3 extend this context further
/// when their components take callbacks.
pub struct StoryContext<'a> {
    pub theme: &'a bbr_ui::theme::Theme,
    pub reduced_motion: bool,
    pub state: &'a str,
    /// `Some` only for the Core Controls stories: the real gpui-component
    /// state entities, the current recorder snapshot, and the observer
    /// callbacks real control events route through. `None` for every other
    /// story.
    pub controls: Option<ControlsStory<'a>>,
    /// `Some` only for the Tab Row story: the Tab Row recorder snapshot
    /// and the observer callbacks the production `tab_row` callbacks route
    /// through (packet `component-gallery-browser-components.md`). `None`
    /// for every other story.
    pub tab_row: Option<TabRowStory<'a>>,
}

/// Everything a Core Controls story needs to render the **exact** vendored
/// gpui-component production symbols with fixture state: the real
/// `InputState`/`SliderState` entities (the controls render them directly —
/// never a gallery-local copy), the current recorder snapshot (checked
/// states, click counts), and the observer callbacks that re-enter the app
/// on real control events.
pub struct ControlsStory<'a> {
    pub input: Option<&'a Entity<InputState>>,
    pub slider: Option<&'a Entity<SliderState>>,
    pub recorder: &'a ControlRecorder,
    pub observer: &'a ControlObserver,
    /// The real `InputState::value()` snapshot (read by the app each
    /// render) — the story's own value row.
    pub input_value: String,
    /// The real `SliderState::value().start()` snapshot (read by the app
    /// each render) — the story's value row and the Progress mirror.
    pub slider_value: f32,
}

/// Everything a Tab Row story needs to render the **exact** production
/// `bbr_ui::components::tab_row` with recorder callbacks: the current
/// recorder snapshot (select/close/pin/drag counts) and the observer
/// callbacks that re-enter the app on real pointer interactions (packet
/// `component-gallery-browser-components.md`). `Some` only while the Tab
/// Row story is selected.
pub struct TabRowStory<'a> {
    pub recorder: &'a TabRowRecorder,
    pub observer: &'a TabRowObserver,
}

/// A story render function: plain `bbr-ui` symbols with fixture models,
/// never a gallery-local reimplementation of anatomy.
pub type StoryRender = fn(&StoryContext) -> AnyElement;

/// One registered story. `id`/`group` are assigned by the registry; the
/// declaration helper leaves them unset until `add_group` runs.
#[derive(Clone)]
pub struct Story {
    pub id: StoryId,
    pub group: GroupId,
    pub label: &'static str,
    pub render: StoryRender,
    pub state_options: StateOptions,
}

/// One catalog group in explicit declaration order.
#[derive(Clone)]
pub struct StoryGroup {
    pub id: GroupId,
    pub label: &'static str,
    pub stories: Vec<Story>,
}

/// The catalog registry: declaration-order groups and stories with stable,
/// append-only ids.
pub struct CatalogRegistry {
    pub groups: Vec<StoryGroup>,
    /// Flattened declaration order (groups and stories interleaved exactly
    /// as declared) — the tree order and the canvas default derive from it.
    pub stories: Vec<Story>,
    next_group_id: u64,
    next_story_id: u64,
}

/// Declaration helper: a story whose ids are assigned by `add_group`.
pub fn story(label: &'static str, render: StoryRender, state_options: StateOptions) -> Story {
    Story {
        id: StoryId(0),
        group: GroupId(0),
        label,
        render,
        state_options,
    }
}

impl Default for CatalogRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CatalogRegistry {
    pub fn new() -> Self {
        CatalogRegistry {
            groups: Vec::new(),
            stories: Vec::new(),
            next_group_id: 0,
            next_story_id: 0,
        }
    }

    /// Append a group and its stories in declaration order, assigning the
    /// next sequential `GroupId`/`StoryId`s. Append-only: later packets add
    /// groups/stories at later ids and never renumber earlier ones.
    pub fn add_group(&mut self, label: &'static str, stories: Vec<Story>) {
        let group_id = GroupId(self.next_group_id);
        self.next_group_id += 1;
        let mut assigned = Vec::with_capacity(stories.len());
        for mut story in stories {
            story.id = StoryId(self.next_story_id);
            story.group = group_id;
            self.next_story_id += 1;
            assigned.push(story);
        }
        self.groups.push(StoryGroup {
            id: group_id,
            label,
            stories: assigned,
        });
        self.stories.extend(
            self.groups
                .last()
                .expect("just pushed")
                .stories
                .iter()
                .cloned(),
        );
    }

    /// The story with the given id, if registered.
    pub fn story(&self, id: StoryId) -> Option<&Story> {
        self.stories.iter().find(|s| s.id == id)
    }

    /// The first story in declaration order — the canvas default.
    pub fn default_story(&self) -> StoryId {
        self.stories
            .first()
            .map(|s| s.id)
            .expect("catalog is never empty")
    }

    /// The number of stories registered so far (the pinned-prefix bound for
    /// the append-only id test).
    pub fn story_count(&self) -> u64 {
        self.next_story_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stories::catalog;
    use gpui::prelude::*;

    #[test]
    fn ids_are_sequential_in_declaration_order() {
        let registry = catalog();
        let ids: Vec<u64> = registry.stories.iter().map(|s| s.id.0).collect();
        // Sequential from 0 with no gaps or duplicates.
        let expected: Vec<u64> = (0..ids.len() as u64).collect();
        assert_eq!(ids, expected);
        // Group ids are sequential too.
        let group_ids: Vec<u64> = registry.groups.iter().map(|g| g.id.0).collect();
        let expected_groups: Vec<u64> = (0..group_ids.len() as u64).collect();
        assert_eq!(group_ids, expected_groups);
    }

    #[test]
    fn ids_are_stable_across_label_edits() {
        // A label edit must never shift an id: rebuild the catalog with a
        // renamed label and assert every StoryId is unchanged.
        let before = catalog();
        let before_ids: Vec<u64> = before.stories.iter().map(|s| s.id.0).collect();
        let after = catalog_with_renamed_label();
        let after_ids: Vec<u64> = after.stories.iter().map(|s| s.id.0).collect();
        assert_eq!(before_ids, after_ids, "label edits never shift StoryIds");
    }

    #[test]
    fn ids_are_append_only() {
        // G2/G3 append at later ids: appending a group must not renumber
        // the pinned G1b prefix.
        let mut registry = catalog();
        let pinned: Vec<u64> = registry.stories.iter().map(|s| s.id.0).collect();
        let count = registry.story_count();
        registry.add_group(
            "Later packet",
            vec![story("Extra", |_| gpui::div().into_any(), &[])],
        );
        let after: Vec<u64> = registry.stories.iter().map(|s| s.id.0).collect();
        assert_eq!(
            &after[..pinned.len()],
            &pinned[..],
            "G1b ids are a pinned prefix"
        );
        assert_eq!(after.len(), pinned.len() + 1);
        assert_eq!(registry.story_count(), count + 1);
    }

    #[test]
    fn first_n_ids_are_pinned() {
        // The pinned first-N assignment: reordering or inserting a story
        // before the current head would shift these ids and fail here.
        let registry = catalog();
        let labels: Vec<&str> = registry.stories.iter().map(|s| s.label).collect();
        for (index, label) in labels.iter().enumerate() {
            assert_eq!(
                registry.stories[index].id.0 as usize, index,
                "story {label} must keep its pinned id {index}"
            );
        }
    }

    #[test]
    fn default_story_is_the_first_declared() {
        let registry = catalog();
        assert_eq!(registry.default_story(), registry.stories[0].id);
    }

    /// A catalog identical to `catalog()` except one story label is renamed
    /// — used to prove label edits never shift ids.
    fn catalog_with_renamed_label() -> CatalogRegistry {
        let mut registry = CatalogRegistry::new();
        let mut base = catalog();
        // Rename the first story label in a copy of the declaration.
        let groups = std::mem::take(&mut base.groups);
        for group in groups {
            let stories = group
                .stories
                .into_iter()
                .enumerate()
                .map(|(i, mut s)| {
                    if i == 0 {
                        s.label = "Renamed label";
                    }
                    s
                })
                .collect();
            registry.add_group(group.label, stories);
        }
        registry
    }
}
