//! Gallery-owned observation state for the G2 Group 3 Address/Input stories
//! (packet `component-gallery-browser-components.md`).
//!
//! The Address stories render the **exact production `bbr-ui` symbols**
//! (`address_control`, `staging_surface`) with plain fixture models and
//! recorder callbacks — never a gallery-local wrapper, reimplementation,
//! restyle, or copy. The gallery's own contribution is limited to this
//! observation state: what the production callbacks report back so the E2E
//! can assert real pointer interactions and the canvas can reflect them.
//!
//! The recorder is app-owned plain state (not an entity): real events
//! re-enter the app through the same seam the mode/state controls use, so
//! the app's snapshot can read the observed values without a second
//! authority.

use std::rc::Rc;

use gpui::{App, MouseDownEvent, Window};

/// What a production Address/Staging callback reports back to the gallery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressObservation {
    /// The §11.5 pill hover wash (`on_hover`) with the new hovered state.
    Hover(bool),
    /// The §11.5 pill/staging activation (`on_open` — the pointer press
    /// that would open the §11.4 palette/overlay over the control).
    Open,
}

/// The shared Address/Staging recorder: plain fixture observation state,
/// written only through the app's production record path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AddressRecorder {
    /// The hovered states the production `on_hover` reported, in dispatch
    /// order (a `true` entry proves the pointer entered the pill).
    pub hovers: Vec<bool>,
    /// The number of pointer presses that fired the production `on_open`.
    pub opens: u32,
}

impl AddressRecorder {
    /// Apply one observed Address/Staging interaction.
    pub fn observe(&mut self, event: AddressObservation) {
        match event {
            AddressObservation::Hover(hovered) => self.hovers.push(hovered),
            AddressObservation::Open => self.opens += 1,
        }
    }
}

/// A hover callback carrying the production `&bool` shape the `bbr-ui`
/// `address_control` `on_hover` takes, re-entering the app through the
/// production record path.
pub type AddressHoverFn = Rc<dyn Fn(&bool, &mut Window, &mut App)>;
/// An open callback carrying the production `MouseDownEvent` shape the
/// `bbr-ui` `address_control`/`staging_surface` `on_open` callbacks take,
/// re-entering the app through the production record path.
pub type AddressOpenFn = Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>;

/// The callback bundle the Address stories route the production
/// `address_control`/`staging_surface` callbacks through. Each callback
/// records its interaction. `on_open` stops propagation exactly like the
/// app's production listener (the app's `on_open` opens the §11.4 overlay
/// and stops propagation so the sidebar's click handler never runs after
/// it, DESIGN.md §11.5/§13); `on_hover` never stops — the production
/// hover wash follows the pointer without intercepting it.
pub struct AddressObserver {
    pub on_hover: AddressHoverFn,
    pub on_open: AddressOpenFn,
}

/// Build the production `address_control` observer callbacks from one
/// record action. `record` re-enters the app's production record path (the
/// app-owned entity upgrade); this constructor owns the production
/// **propagation contract**: `on_open` stops propagation (the app opens
/// the §11.4 overlay on press), `on_hover` never does.
pub fn build_address_observer(
    record: impl Fn(AddressObservation, &mut App) + 'static,
) -> AddressObserver {
    let record = Rc::new(record);
    let on_hover = {
        let record = record.clone();
        Rc::new(move |hovered: &bool, _window: &mut Window, cx: &mut App| {
            record(AddressObservation::Hover(*hovered), cx);
        })
    };
    let on_open = {
        let record = record.clone();
        Rc::new(
            move |_e: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                // §11.5/§13: the production on_open listener opens the
                // overlay and stops propagation (one pointer action; the
                // sidebar's click handler must never run after it).
                cx.stop_propagation();
                record(AddressObservation::Open, cx);
            },
        )
    };
    AddressObserver { on_hover, on_open }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_records_hover_states_and_open_counts() {
        let mut recorder = AddressRecorder::default();
        recorder.observe(AddressObservation::Hover(true));
        recorder.observe(AddressObservation::Hover(false));
        recorder.observe(AddressObservation::Open);
        recorder.observe(AddressObservation::Open);
        assert_eq!(recorder.hovers, vec![true, false]);
        assert_eq!(recorder.opens, 2);
        // A hover never touches the open count and vice versa.
        recorder.observe(AddressObservation::Hover(true));
        assert_eq!(recorder.hovers, vec![true, false, true]);
        assert_eq!(recorder.opens, 2);
    }

    #[test]
    fn recorder_defaults_are_all_rest() {
        let recorder = AddressRecorder::default();
        assert!(recorder.hovers.is_empty());
        assert_eq!(recorder.opens, 0);
    }
}
