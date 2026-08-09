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

use gpui::{App, Entity, Window};
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
}
