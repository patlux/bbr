//! Gallery-owned observation state for the G2 Group 8 Auth sheet story
//! (packet `component-gallery-browser-components.md`).
//!
//! The Auth story renders the **exact production `bbr_ui` symbol**
//! (`bbr_ui::surfaces::auth::auth_sheet`) with plain fixture models and
//! recorder callbacks — never a gallery-local wrapper, reimplementation,
//! restyle, or copy. The gallery's own contribution is limited to this
//! observation state: what the production Log In / Cancel callbacks report
//! back so the E2E can assert real pointer interactions and the canvas can
//! reflect them.
//!
//! The recorder is app-owned plain state (not an entity): real events
//! re-enter the app through the same seam the mode/state controls use, so
//! the app's snapshot can read the observed values without a second
//! authority. The recorder only ever counts **which** production button
//! fired — no credential text exists anywhere in the gallery (the model
//! carries the typed username as sheet-local render data and no password
//! field at all).

use std::rc::Rc;

use gpui::{App, MouseDownEvent, Window};

/// What a production Auth callback reports back to the gallery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthObservation {
    /// The Log In button press (the production `on_submit` path).
    Submit,
    /// The Cancel button press (the production `on_cancel` path).
    Cancel,
}

/// The shared Auth recorder: plain fixture observation state, written only
/// through the app's production record path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthRecorder {
    /// The number of Log In presses that fired the production `on_submit`.
    pub submits: u32,
    /// The number of Cancel presses that fired the production `on_cancel`.
    pub cancels: u32,
}

impl AuthRecorder {
    /// Apply one observed Auth interaction.
    pub fn observe(&mut self, event: AuthObservation) {
        match event {
            AuthObservation::Submit => self.submits += 1,
            AuthObservation::Cancel => self.cancels += 1,
        }
    }
}

/// A submit callback carrying the production `MouseDownEvent` shape the
/// `bbr-ui` `auth_sheet` `on_submit` callback takes, re-entering the app
/// through the production record path.
pub type AuthSubmitFn = Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>;
/// A cancel callback carrying the production `MouseDownEvent` shape the
/// `bbr-ui` `auth_sheet` `on_cancel` callback takes, re-entering the app
/// through the production record path.
pub type AuthCancelFn = Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>;

/// The callback bundle the Auth story routes the production `auth_sheet`
/// callbacks through. Each callback records its interaction. The
/// production portal listeners do not stop propagation inside the card
/// (the card itself is never an outside press; the surface backdrop owns
/// the no-dismissal policy), so the recorder matches that contract and
/// only records.
pub struct AuthObserver {
    pub on_submit: AuthSubmitFn,
    pub on_cancel: AuthCancelFn,
}

/// Build the production `auth_sheet` observer callbacks from one record
/// action. `record` re-enters the app's production record path (the
/// app-owned entity upgrade).
pub fn build_auth_observer(record: impl Fn(AuthObservation, &mut App) + 'static) -> AuthObserver {
    let record = Rc::new(record);
    let on_submit = {
        let record = record.clone();
        Rc::new(
            move |_e: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                record(AuthObservation::Submit, cx);
            },
        )
    };
    let on_cancel = {
        let record = record.clone();
        Rc::new(
            move |_e: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                record(AuthObservation::Cancel, cx);
            },
        )
    };
    AuthObserver {
        on_submit,
        on_cancel,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_records_submit_and_cancel_counts() {
        let mut recorder = AuthRecorder::default();
        recorder.observe(AuthObservation::Submit);
        recorder.observe(AuthObservation::Submit);
        recorder.observe(AuthObservation::Cancel);
        assert_eq!(recorder.submits, 2);
        assert_eq!(recorder.cancels, 1);
        // A submit never touches the cancel count and vice versa.
        recorder.observe(AuthObservation::Cancel);
        assert_eq!(recorder.submits, 2);
        assert_eq!(recorder.cancels, 2);
    }

    #[test]
    fn recorder_defaults_are_all_rest() {
        let recorder = AuthRecorder::default();
        assert_eq!(recorder.submits, 0);
        assert_eq!(recorder.cancels, 0);
    }
}
