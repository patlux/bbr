//! The gallery app entity (packet `component-gallery-mvp.md`): owns the
//! catalog registry, the tree state, the mode/contrast/density/reduced-
//! motion controls, the per-story state, and (for the Core Controls
//! stories) the real gpui-component state entities plus the shared
//! recorder. The render composes the tree surface (left), the controls
//! bar, and the canvas (right). The E2E drives the exact production
//! methods (`apply_control`, `on_key_down`, `observe_control`,
//! `activate_node`) and real pointer/keyboard events, and reads the
//! semantic snapshot — never pixels.

use std::rc::Rc;

use bbr_ui::theme::{ContrastMode, Density, Theme, ThemeMode};
use gpui::{div, prelude::*, App, Context, FocusHandle, Focusable, KeyDownEvent, Window};
use gpui_component::input::InputState;
use gpui_component::slider::SliderState;
use gpui_component::WindowExt as _;

use crate::canvas::{canvas_surface, CanvasModel};
use crate::controls::{controls_surface, ControlAction, ControlsModel};
use crate::recorder::{ControlObservation, ControlObserver, ControlRecorder, CoreControlsEntities};
use crate::registry::{CatalogRegistry, ControlsStory, GroupId, NodeId, StoryId};
use crate::stories::{catalog, GALLERY_CONTINUATION, GALLERY_SCOPE};
use crate::tree::{NodeKind, TreeState};
use crate::tree_surface::tree_surface;

/// One story's own State/Variant selector state.
#[derive(Debug, Clone)]
pub struct StoryState {
    pub options: &'static [&'static str],
    pub selected: usize,
}

/// The semantic snapshot the E2E asserts (§15 mode fields + stable ids).
#[derive(Debug, Clone, PartialEq)]
pub struct TreeNodeSnapshot {
    pub id: u64,
    pub kind: &'static str,
    pub label: String,
    pub visible: bool,
    /// Group nodes: whether the group is expanded (drives the disclosure
    /// chevron and child visibility); always `false` for story nodes.
    pub expanded: bool,
    /// The story's stable `StoryId` for story nodes (`None` for groups) —
    /// the E2E asserts the declaration-order prefix on this, never on the
    /// tree node id (which also counts group nodes).
    pub story_id: Option<u64>,
}

/// The semantic snapshot the E2E asserts (§15 mode fields + stable ids).
#[derive(Debug, Clone, PartialEq)]
pub struct GallerySnapshot {
    pub tree_id: String,
    pub canvas_id: String,
    pub tree_nodes: Vec<TreeNodeSnapshot>,
    pub focus_node: u64,
    pub selected_story: u64,
    pub selected_story_label: String,
    pub canvas_story_id: String,
    pub theme: ThemeMode,
    pub contrast: ContrastMode,
    pub density: Density,
    pub reduced_motion: bool,
    pub story_state: String,
    pub story_state_index: usize,
    /// The honest scope copy (finding 3): "Foundations / Primitives /
    /// Browser Components".
    pub scope: String,
    /// The honest continuation note pointing to G2/G3 (finding 3).
    pub continuation: String,
    /// The observed Core Controls state: what the real gpui-component
    /// controls reported through the production recorder path.
    pub button_clicks: u32,
    pub checkbox_checked: bool,
    pub switch_checked: bool,
    pub radio_index: usize,
    /// The real `InputState::value()` (typed text) when the Core Controls
    /// input exists.
    pub input_text: String,
    /// The real `SliderState::value().start()` when the Core Controls
    /// slider exists.
    pub slider_value: f32,
}

pub struct GalleryApp {
    registry: CatalogRegistry,
    tree: TreeState,
    mode: ThemeMode,
    contrast: ContrastMode,
    density: Density,
    reduced_motion: bool,
    story_states: Vec<StoryState>,
    /// The Core Controls group id, found by label at construction.
    core_controls_group: GroupId,
    /// The shared recorder: what real gpui-component control events report
    /// back through the production `observe_control` path.
    recorder: ControlRecorder,
    /// The real gpui-component state entities, created on the first render
    /// of a Core Controls story (both need the window at creation time —
    /// the same production integration pattern the controls require).
    core_controls: Option<CoreControlsEntities>,
    focus_handle: FocusHandle,
}

impl GalleryApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let registry = catalog();
        let core_controls_group = registry
            .groups
            .iter()
            .find(|g| g.label == "Core controls")
            .expect("core controls group registered")
            .id;
        let tree = TreeState::from_catalog(&registry);
        let story_states = registry
            .stories
            .iter()
            .map(|story| StoryState {
                options: story.state_options,
                selected: 0,
            })
            .collect();
        GalleryApp {
            registry,
            tree,
            mode: ThemeMode::Dark,
            contrast: ContrastMode::Standard,
            density: Density::Normal,
            reduced_motion: false,
            story_states,
            core_controls_group,
            recorder: ControlRecorder::default(),
            core_controls: None,
            focus_handle: cx.focus_handle(),
        }
    }

    /// The resolved theme from the current mode/contrast/density.
    pub fn theme(&self) -> Theme {
        Theme::new(self.mode, self.contrast, self.density)
    }

    /// The selected story's current state label.
    pub fn story_state_label(&self, id: StoryId) -> &'static str {
        let state = &self.story_states[id.0 as usize];
        state
            .options
            .get(state.selected)
            .copied()
            .unwrap_or("default")
    }

    /// Whether the story belongs to the Core Controls group.
    fn is_core_controls_story(&self, id: StoryId) -> bool {
        self.registry
            .story(id)
            .is_some_and(|story| story.group == self.core_controls_group)
    }

    /// The production control path: every pointer click and the E2E route
    /// through here. Mode/state changes re-render the canvas.
    pub fn apply_control(&mut self, action: ControlAction, cx: &mut Context<Self>) {
        match action {
            ControlAction::Mode(mode) => self.mode = mode,
            ControlAction::Contrast(contrast) => self.contrast = contrast,
            ControlAction::Density(density) => self.density = density,
            ControlAction::ReducedMotion(on) => self.reduced_motion = on,
            ControlAction::State(index) => {
                let state = &mut self.story_states[self.tree.selected_story.0 as usize];
                if !state.options.is_empty() {
                    state.selected = index.min(state.options.len() - 1);
                }
            }
        }
        cx.notify();
    }

    /// The production recorder path: real Core Controls events (Button
    /// clicks, Checkbox/Switch/Radio state changes) report back here and
    /// re-render the canvas.
    pub fn observe_control(&mut self, event: ControlObservation, cx: &mut Context<Self>) {
        self.recorder.observe(event);
        cx.notify();
    }

    /// The production tree-activation path (pointer click on a group row,
    /// its disclosure target, or a story row): sets the one focus and runs
    /// `TreeState::activate`, the same expansion/selection authority the
    /// keyboard Enter/Space use.
    pub fn activate_node(&mut self, id: NodeId, cx: &mut Context<Self>) {
        if let Some(index) = self.tree.nodes.iter().position(|n| n.id == id) {
            self.tree.focus = index;
            self.tree.activate();
        }
        cx.notify();
    }

    /// The tree keyboard contract: `↑`/`↓` move, `←` collapse/parent,
    /// `→` expand/enter, `Home`/`End` jump, `Enter`/`Space` activate.
    /// While the real Core Controls input owns focus, tree navigation is
    /// suspended so editing keys reach the production input.
    pub fn on_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if window.has_focused_input(cx) {
            return;
        }
        let key = event.keystroke.key.as_str();
        let handled = match key {
            "up" => {
                self.tree.move_focus(-1);
                true
            }
            "down" => {
                self.tree.move_focus(1);
                true
            }
            "left" => {
                self.tree.collapse();
                true
            }
            "right" => {
                self.tree.expand();
                true
            }
            "home" => {
                self.tree.home();
                true
            }
            "end" => {
                self.tree.end();
                true
            }
            "enter" | "space" => {
                self.tree.activate();
                true
            }
            _ => false,
        };
        if handled {
            cx.stop_propagation();
            cx.notify();
        }
    }

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
        }
    }

    /// The registry (read-only access for the E2E story iteration).
    pub fn registry(&self) -> &CatalogRegistry {
        &self.registry
    }
}

impl Focusable for GalleryApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for GalleryApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme();
        let selected = self.tree.selected_story;
        let story_state = self.story_state_label(selected);
        let state_index = self.story_states[selected.0 as usize].selected;
        let state_options = self.story_states[selected.0 as usize].options;
        // The Core Controls stories need the real gpui-component state
        // entities; create them once on the first render of such a story
        // (InputState needs the window, so this cannot happen in `new`).
        let core_selected = self.is_core_controls_story(selected);
        if core_selected && self.core_controls.is_none() {
            let input = cx.new(|cx| InputState::new(window, cx).placeholder("Type here…"));
            let slider = cx.new(|_cx| {
                SliderState::new()
                    .min(0.0)
                    .max(100.0)
                    .step(1.0)
                    .default_value(25.0)
            });
            self.core_controls = Some(CoreControlsEntities { input, slider });
        }
        // Pointer clicks route through the same production methods the
        // keyboard and the E2E use; the closures re-enter the entity through
        // a downgraded handle (the same seam `cx.listener` uses).
        let entity = cx.entity().downgrade();
        let on_activate = move |node_id: NodeId, _window: &mut Window, cx: &mut App| {
            if let Some(entity) = entity.upgrade() {
                entity.update(cx, |app, cx| app.activate_node(node_id, cx));
            }
        };
        let entity = cx.entity().downgrade();
        let on_control = move |action: ControlAction, _window: &mut Window, cx: &mut App| {
            if let Some(entity) = entity.upgrade() {
                entity.update(cx, |app, cx| app.apply_control(action, cx));
            }
        };
        // The Core Controls observer: real control events report back into
        // the app's production `observe_control` path.
        let entity = cx.entity().downgrade();
        let on_button = {
            let entity = entity.clone();
            Rc::new(move |_window: &mut Window, cx: &mut App| {
                if let Some(entity) = entity.upgrade() {
                    entity.update(cx, |app, cx| {
                        app.observe_control(ControlObservation::Button, cx)
                    });
                }
            })
        };
        let on_checkbox = {
            let entity = entity.clone();
            Rc::new(move |checked: bool, _window: &mut Window, cx: &mut App| {
                if let Some(entity) = entity.upgrade() {
                    entity.update(cx, |app, cx| {
                        app.observe_control(ControlObservation::Checkbox(checked), cx)
                    });
                }
            })
        };
        let on_switch = {
            let entity = entity.clone();
            Rc::new(move |checked: bool, _window: &mut Window, cx: &mut App| {
                if let Some(entity) = entity.upgrade() {
                    entity.update(cx, |app, cx| {
                        app.observe_control(ControlObservation::Switch(checked), cx)
                    });
                }
            })
        };
        let on_radio = {
            let entity = entity.clone();
            Rc::new(move |index: usize, _window: &mut Window, cx: &mut App| {
                if let Some(entity) = entity.upgrade() {
                    entity.update(cx, |app, cx| {
                        app.observe_control(ControlObservation::Radio(index), cx)
                    });
                }
            })
        };
        let observer = ControlObserver {
            on_button,
            on_checkbox,
            on_switch,
            on_radio,
        };
        // The Core Controls story context: the real entities, the current
        // recorder snapshot, and the observer callbacks.
        let controls = if core_selected {
            let entities = self.core_controls.as_ref().expect("created above");
            let input_value = entities.input.read(cx).value().to_string();
            let slider_value = entities.slider.read(cx).value().start();
            Some(ControlsStory {
                input: Some(&entities.input),
                slider: Some(&entities.slider),
                recorder: &self.recorder,
                observer: &observer,
                input_value,
                slider_value,
            })
        } else {
            None
        };
        let tree = tree_surface(&self.tree, &theme, on_activate);
        let controls_bar = controls_surface(
            &ControlsModel {
                theme: &theme,
                mode: self.mode,
                contrast: self.contrast,
                density: self.density,
                reduced_motion: self.reduced_motion,
                state_options,
                state_index,
            },
            on_control,
        );
        let canvas = canvas_surface(CanvasModel {
            registry: &self.registry,
            selected,
            theme: &theme,
            reduced_motion: self.reduced_motion,
            story_state,
            controls,
        });
        div()
            .id("bbr-gallery-root")
            .track_focus(&self.focus_handle)
            .key_context("bbr_gallery")
            .flex()
            .flex_row()
            .size_full()
            .bg(theme.palette.surface_canvas)
            .on_key_down(cx.listener(Self::on_key_down))
            .child(
                div()
                    .w(px(280.0))
                    .h_full()
                    .border_r_1()
                    .border_color(theme.palette.hairline)
                    .child(tree),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .size_full()
                    .child(controls_bar)
                    .child(canvas),
            )
    }
}

use gpui::px;
