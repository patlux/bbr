//! The gallery app's semantic snapshot construction (packet
//! `component-gallery-mvp.md`): a bounded split of `app.rs` so the app
//! entity stays under the ~500-line module budget. The app owns the
//! snapshot construction; the types live in the focused `crate::snapshot`
//! module (re-exported through `crate::app` for the E2E). The submodule
//! sits under `app` so it can read the entity's private state without
//! widening any field visibility.

use gpui::App;

use super::{GalleryApp, GallerySnapshot, TreeNodeSnapshot};
use crate::stories::{GALLERY_CONTINUATION, GALLERY_SCOPE};
use crate::tree::NodeKind;

impl GalleryApp {
    /// The semantic snapshot the E2E asserts.
    pub fn snapshot(&self, cx: &App) -> GallerySnapshot {
        let selected = self.tree.selected_story;
        let story = self
            .registry
            .story(selected)
            .expect("selected story registered");
        let input_text = self
            .core_controls
            .as_ref()
            .map(|entities| entities.input.read(cx).value().to_string())
            .unwrap_or_default();
        let slider_value = self
            .core_controls
            .as_ref()
            .map(|entities| entities.slider.read(cx).value().start())
            .unwrap_or(0.0);
        GallerySnapshot {
            tree_id: "bbr-gallery-tree".into(),
            canvas_id: "bbr-gallery-canvas".into(),
            tree_nodes: self
                .tree
                .nodes
                .iter()
                .map(|node| TreeNodeSnapshot {
                    id: node.id.0,
                    kind: match node.kind {
                        NodeKind::Group(_) => "group",
                        NodeKind::Story(_) => "story",
                    },
                    label: node.label.to_string(),
                    visible: match node.kind {
                        NodeKind::Group(_) => true,
                        NodeKind::Story(_) => self.tree.expanded[node.group.0 as usize],
                    },
                    expanded: match node.kind {
                        NodeKind::Group(group) => self.tree.expanded[group.0 as usize],
                        NodeKind::Story(_) => false,
                    },
                    story_id: match node.kind {
                        NodeKind::Story(id) => Some(id.0),
                        NodeKind::Group(_) => None,
                    },
                })
                .collect(),
            focus_node: self.tree.nodes[self.tree.focused_index()].id.0,
            selected_story: selected.0,
            selected_story_label: story.label.to_string(),
            canvas_story_id: format!("bbr-gallery-canvas-story-{}", selected.0),
            theme: self.mode,
            contrast: self.contrast,
            density: self.density,
            reduced_motion: self.reduced_motion,
            story_state: self.story_state_label(selected).to_string(),
            story_state_index: self.story_states[selected.0 as usize].selected,
            scope: GALLERY_SCOPE.to_string(),
            continuation: GALLERY_CONTINUATION.to_string(),
            button_clicks: self.recorder.button_clicks,
            checkbox_checked: self.recorder.checkbox_checked,
            switch_checked: self.recorder.switch_checked,
            radio_index: self.recorder.radio_index,
            input_text,
            slider_value,
            tab_selects: self.tab_row.counts().0,
            tab_closes: self.tab_row.counts().1,
            tab_pins: self.tab_row.counts().2,
            tab_drag_begins: self.tab_row.counts().3,
            library: self.library_recorder.clone(),
            address: self.address.recorder().clone(),
            group4: self.group4.recorder().clone(),
            downloads: self.downloads.recorder().clone(),
            find: self.find.recorder().clone(),
            error: self.error.recorder().clone(),
            auth: self.auth.recorder().clone(),
            context_menu: self.context_menu.recorder().clone(),
            sidebar: self.sidebar.recorder().clone(),
            command: self.command.recorder().clone(),
        }
    }
}
