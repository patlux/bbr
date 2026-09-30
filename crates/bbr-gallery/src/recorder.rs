//! Gallery-owned observation state for the Core Controls stories (packet
//! `component-gallery-mvp.md`, live-review finding 2).
//!
//! The Core Controls stories render the **exact vendored gpui-component
//! 0.5.1 production symbols** (Button, Input, Checkbox, Switch, Radio,
//! Slider, Progress) with plain fixture state and recorder callbacks —
//! never a gallery-local wrapper, reimplementation, restyle, or copy. The
//! gallery's own contribution is limited to (a) the entities the real
//! controls need (`InputState`, `SliderState`) and (b) this observation
//! state: what the real controls report back so the E2E can assert real
//! interactions and the canvas can reflect them.
//!
//! The recorder is app-owned plain state (not an entity): real control
//! events re-enter the app through the same seam the mode/state controls
//! use, so the app's snapshot can read the observed values without a
//! second authority.

use std::rc::Rc;

use gpui::{App, Entity, MouseDownEvent, Window};
use gpui_component::input::InputState;
use gpui_component::slider::SliderState;

/// What a stateful Core Control reports back to the gallery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlObservation {
    /// A real `Button` click.
    Button,
    /// A real `Checkbox` click with its new checked state.
    Checkbox(bool),
    /// A real `Switch` click with its new checked state.
    Switch(bool),
    /// A real `Radio` click selecting the given option index.
    Radio(usize),
}

/// The shared recorder: plain fixture observation state. Every field is
/// written only through the app's production `observe_control` path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ControlRecorder {
    pub button_clicks: u32,
    pub checkbox_checked: bool,
    pub switch_checked: bool,
    pub radio_index: usize,
}

impl ControlRecorder {
    /// Apply one observed control event.
    pub fn observe(&mut self, event: ControlObservation) {
        match event {
            ControlObservation::Button => self.button_clicks += 1,
            ControlObservation::Checkbox(checked) => self.checkbox_checked = checked,
            ControlObservation::Switch(checked) => self.switch_checked = checked,
            ControlObservation::Radio(index) => self.radio_index = index,
        }
    }
}

/// The real gpui-component state entities the Core Controls stories render:
/// the production `InputState` (typed editing, focus) and `SliderState`
/// (value/thumb). Created lazily by the app on the first render of a Core
/// Controls story (both need the window at creation time, mirroring the
/// production integration pattern).
pub struct CoreControlsEntities {
    pub input: Entity<InputState>,
    pub slider: Entity<SliderState>,
}

/// A plain control-event callback (no payload).
pub type ObserverFn = Rc<dyn Fn(&mut Window, &mut App)>;
/// A control-event callback carrying a checked state.
pub type CheckedObserverFn = Rc<dyn Fn(bool, &mut Window, &mut App)>;
/// A control-event callback carrying a radio option index.
pub type RadioObserverFn = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The callback bundle the Core Controls stories route real control events
/// through. Each callback re-enters the app entity (the same downgraded
/// handle seam the tree and the mode/state controls use), so every
/// interaction resolves through the app's production `observe_control`
/// method and re-renders the canvas.
pub struct ControlObserver {
    pub on_button: ObserverFn,
    pub on_checkbox: CheckedObserverFn,
    pub on_switch: CheckedObserverFn,
    pub on_radio: RadioObserverFn,
}

/// What a Tab Row interaction reports back to the gallery (packet
/// `component-gallery-browser-components.md`): the §11.1 select/close and
/// the §10.9 drag entry (a row-body press). The recorder proves which
/// production callback fired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabRowObservation {
    Select,
    Close,
    BeginDrag,
}

/// The shared Tab Row recorder: plain fixture observation state, written
/// only through the app's production record path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TabRowRecorder {
    pub selects: u32,
    pub closes: u32,
    pub drag_begins: u32,
}

impl TabRowRecorder {
    /// Apply one observed Tab Row interaction.
    pub fn observe(&mut self, event: TabRowObservation) {
        match event {
            TabRowObservation::Select => self.selects += 1,
            TabRowObservation::Close => self.closes += 1,
            TabRowObservation::BeginDrag => self.drag_begins += 1,
        }
    }
}

/// A Tab Row event callback carrying the production `MouseDownEvent` (the
/// exact shape the `bbr-ui` `tab_row` callbacks take), re-entering the app
/// through the production record path.
pub type TabRowObserverFn = Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>;

/// The callback bundle the Tab Row story routes the production
/// `tab_row` callbacks through. Each callback records its interaction;
/// only the close callback stops propagation, exactly like the app's
/// production listener (a close press never also selects or drags the row,
/// §11.1 — while a row-body press fires both `select` and `begin_drag`,
/// the production two-listener path, DESIGN.md §11.1/§10.9).
pub struct TabRowObserver {
    pub on_select: TabRowObserverFn,
    pub on_close: TabRowObserverFn,
    pub on_begin_drag: TabRowObserverFn,
}

/// Build the production `tab_row` observer callbacks from one record
/// action. `record` re-enters the app's production record path (the
/// app-owned entity upgrade); this constructor owns the production
/// **propagation contract**: only the close listener stops propagation,
/// so a close press never also selects or drags the row, while a row-body
/// press fires both `select` and `begin_drag` — the exact production
/// wiring the app's `sidebar_view` uses (DESIGN.md §11.1/§10.9).
pub fn build_tab_row_observer(
    record: impl Fn(TabRowObservation, &mut App) + 'static,
) -> TabRowObserver {
    let record = Rc::new(record);
    let make = |event: TabRowObservation, stop: bool| {
        let record = record.clone();
        Rc::new(
            move |_e: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                if stop {
                    cx.stop_propagation();
                }
                record(event, cx);
            },
        )
    };
    TabRowObserver {
        on_select: make(TabRowObservation::Select, false),
        on_close: make(TabRowObservation::Close, true),
        on_begin_drag: make(TabRowObservation::BeginDrag, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_updates_the_expected_field_only() {
        let mut recorder = ControlRecorder::default();
        recorder.observe(ControlObservation::Button);
        recorder.observe(ControlObservation::Button);
        assert_eq!(recorder.button_clicks, 2);
        assert!(!recorder.checkbox_checked);
        recorder.observe(ControlObservation::Checkbox(true));
        assert!(recorder.checkbox_checked);
        recorder.observe(ControlObservation::Switch(true));
        assert!(recorder.switch_checked);
        recorder.observe(ControlObservation::Radio(2));
        assert_eq!(recorder.radio_index, 2);
        // Observing one control never disturbs the others.
        assert_eq!(recorder.button_clicks, 2);
        assert!(recorder.checkbox_checked);
        assert!(recorder.switch_checked);
    }

    #[test]
    fn recorder_defaults_are_all_rest() {
        let recorder = ControlRecorder::default();
        assert_eq!(recorder.button_clicks, 0);
        assert!(!recorder.checkbox_checked);
        assert!(!recorder.switch_checked);
        assert_eq!(recorder.radio_index, 0);
    }

    #[test]
    fn tab_row_observe_updates_the_expected_field_only() {
        let mut recorder = TabRowRecorder::default();
        recorder.observe(TabRowObservation::Select);
        recorder.observe(TabRowObservation::Select);
        assert_eq!(recorder.selects, 2);
        assert_eq!(recorder.closes, 0);
        assert_eq!(recorder.drag_begins, 0);
        recorder.observe(TabRowObservation::Close);
        recorder.observe(TabRowObservation::BeginDrag);
        assert_eq!(recorder.selects, 2);
        assert_eq!(recorder.closes, 1);
        assert_eq!(recorder.drag_begins, 1);
    }

    #[test]
    fn tab_row_recorder_defaults_are_all_rest() {
        let recorder = TabRowRecorder::default();
        assert_eq!(recorder.selects, 0);
        assert_eq!(recorder.closes, 0);
        assert_eq!(recorder.drag_begins, 0);
    }
}
