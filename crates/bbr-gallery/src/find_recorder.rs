//! Gallery-owned observation state for the G2 Group 6 Find stories
//! (packet `component-gallery-browser-components.md`).
//!
//! The Find story renders the **exact production `bbr-ui` symbols**
//! (`find_bar`, `FindSurface`, `FindBarRect`) with plain fixture models
//! and recorder callbacks — never a gallery-local wrapper,
//! reimplementation, restyle, or copy. The gallery's own contribution is
//! limited to this observation state: what the production callbacks report
//! back so the E2E can assert real pointer interactions and the canvas can
//! reflect them.
//!
//! The recorder is app-owned plain state (not an entity): real events
//! re-enter the app through the same seam the mode/state controls use, so
//! the app's snapshot can read the observed values without a second
//! authority.
//!
//! Propagation contract: the production find controls never stop
//! propagation themselves (the portal root's card-level listener owns the
//! §13 stop — the card is the surface, never an outside press), so the
//! observer callbacks only record, exactly like the production
//! `find_previous_clicked`/`find_next_clicked`/`find_close_clicked`
//! listeners.

use std::rc::Rc;

use gpui::{App, MouseDownEvent, Window};

/// What a production Find-bar callback reports back to the gallery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindObservation {
    /// The §11.4 previous-control press (the production
    /// `find_previous_clicked` path).
    Previous,
    /// The §11.4 next-control press (the production `find_next_clicked`
    /// path).
    Next,
    /// The §11.4 close-control press (the production `find_close_clicked`
    /// path).
    Close,
}

/// The shared Find-bar recorder: plain fixture observation state, written
/// only through the app's production record path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FindRecorder {
    /// The number of presses that fired the production previous callback.
    pub previous: u32,
    /// The number of presses that fired the production next callback.
    pub next: u32,
    /// The number of presses that fired the production close callback.
    pub closes: u32,
}

impl FindRecorder {
    /// Apply one observed Find-bar interaction.
    pub fn observe(&mut self, event: FindObservation) {
        match event {
            FindObservation::Previous => self.previous += 1,
            FindObservation::Next => self.next += 1,
            FindObservation::Close => self.closes += 1,
        }
    }
}

/// A control callback carrying the production `MouseDownEvent` shape the
/// `bbr-ui` `find_bar` callbacks take, re-entering the app through the
/// production record path.
pub type FindControlFn = Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>;

/// The callback bundle the Find story routes the production `find_bar`
/// callbacks through. Each callback records its interaction; none stops
/// propagation (the production controls never stop — the portal root's
/// card-level §13 listener owns the stop).
pub struct FindObserver {
    pub on_previous: FindControlFn,
    pub on_next: FindControlFn,
    pub on_close: FindControlFn,
}

/// Build the production `find_bar` observer callbacks from one record
/// action. `record` re-enters the app's production record path (the
/// app-owned entity upgrade); the propagation contract matches the
/// production controls (no stop — the card-level §13 listener is the
/// portal root's, absent here).
pub fn build_find_observer(record: impl Fn(FindObservation, &mut App) + 'static) -> FindObserver {
    let record = Rc::new(record);
    let make = |observation: FindObservation| {
        let record = record.clone();
        Rc::new(
            move |_e: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                record(observation, cx);
            },
        )
    };
    FindObserver {
        on_previous: make(FindObservation::Previous),
        on_next: make(FindObservation::Next),
        on_close: make(FindObservation::Close),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_records_control_presses() {
        let mut recorder = FindRecorder::default();
        recorder.observe(FindObservation::Next);
        recorder.observe(FindObservation::Next);
        recorder.observe(FindObservation::Previous);
        recorder.observe(FindObservation::Close);
        assert_eq!(recorder.next, 2);
        assert_eq!(recorder.previous, 1);
        assert_eq!(recorder.closes, 1);
        // Each observation touches exactly one counter.
        recorder.observe(FindObservation::Close);
        assert_eq!(recorder.next, 2);
        assert_eq!(recorder.previous, 1);
        assert_eq!(recorder.closes, 2);
    }

    #[test]
    fn recorder_defaults_are_all_rest() {
        let recorder = FindRecorder::default();
        assert_eq!(recorder.previous, 0);
        assert_eq!(recorder.next, 0);
        assert_eq!(recorder.closes, 0);
    }
}
