//! Gallery-owned observation state for the G2 Group 7 Error/Crash stories
//! (packet `component-gallery-browser-components.md`).
//!
//! The Error/Crash stories render the **exact production `bbr-ui`
//! symbols** — the §11.8 error surface (`bbr_ui::surfaces::error::error_surface`)
//! and the §11.8 glass panes (`bbr_ui::components::pane::crash_panel` /
//! `empty_space_pane`) — with plain fixture models and recorder callbacks —
//! never a gallery-local wrapper, reimplementation, restyle, or copy. The
//! gallery's own contribution is limited to this observation state: what
//! the production callbacks report back so the E2E can assert real pointer
//! presses and the canvas can reflect them.
//!
//! The recorder is app-owned plain state (not an entity): real events
//! re-enter the app through the same seam the mode/state controls use, so
//! the app's snapshot can read the observed values without a second
//! authority.

use std::rc::Rc;

use gpui::{App, MouseDownEvent, Window};

/// What a production Error/Crash callback reports back to the gallery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorObservation {
    /// The error surface's primary recovery action (`bbr-error-action`).
    Action,
    /// The error surface's dismiss (`bbr-error-dismiss`).
    Dismiss,
    /// The crash panel Reload (`bbr-crash-reload`).
    Reload,
    /// The crash panel Close tab (`bbr-crash-close`).
    Close,
    /// The empty pane New Tab ⌘T (`bbr-empty-new-tab-button`).
    NewTab,
}

/// The shared Error/Crash recorder: plain fixture observation state,
/// written only through the app's production record path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ErrorRecorder {
    /// The number of pointer presses that fired the production
    /// `on_action` (the error surface's primary recovery action).
    pub actions: u32,
    /// The number of pointer presses that fired the production
    /// `on_dismiss` (the error surface's dismiss).
    pub dismissals: u32,
    /// The number of pointer presses that fired the crash panel `on_reload`.
    pub reloads: u32,
    /// The number of pointer presses that fired the crash panel `on_close`.
    pub closes: u32,
    /// The number of pointer presses that fired the empty pane
    /// `on_new_tab` (New Tab ⌘T).
    pub new_tabs: u32,
}

impl ErrorRecorder {
    /// Apply one observed Error/Crash interaction.
    pub fn observe(&mut self, event: ErrorObservation) {
        match event {
            ErrorObservation::Action => self.actions += 1,
            ErrorObservation::Dismiss => self.dismissals += 1,
            ErrorObservation::Reload => self.reloads += 1,
            ErrorObservation::Close => self.closes += 1,
            ErrorObservation::NewTab => self.new_tabs += 1,
        }
    }
}

/// A pointer callback carrying the production `MouseDownEvent` shape the
/// `bbr-ui` error surface and pane callbacks take, re-entering the app
/// through the production record path.
pub type ErrorPointerFn = Rc<dyn Fn(&MouseDownEvent, &mut Window, &mut App)>;

/// The callback bundle the Error/Crash stories route the production
/// `error_surface` / `crash_panel` / `empty_space_pane` callbacks through.
/// Each callback records its interaction. None of the production listeners
/// stop propagation (the surface fills the content area; the pane buttons
/// are plain mouse-down controls) — the recorder mirrors that contract and
/// never stops propagation either.
pub struct ErrorObserver {
    pub on_action: ErrorPointerFn,
    pub on_dismiss: ErrorPointerFn,
    pub on_reload: ErrorPointerFn,
    pub on_close: ErrorPointerFn,
    pub on_new_tab: ErrorPointerFn,
}

/// Build the production Error/Crash observer callbacks from one record
/// action. `record` re-enters the app's production record path (the
/// app-owned entity upgrade); this constructor owns the production
/// **propagation contract**: none of the production error-surface or pane
/// listeners stop propagation, so the recorder callbacks do not either.
pub fn build_error_observer(
    record: impl Fn(ErrorObservation, &mut App) + 'static,
) -> ErrorObserver {
    let record = Rc::new(record);
    let on_action = {
        let record = record.clone();
        Rc::new(
            move |_e: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                record(ErrorObservation::Action, cx);
            },
        )
    };
    let on_dismiss = {
        let record = record.clone();
        Rc::new(
            move |_e: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                record(ErrorObservation::Dismiss, cx);
            },
        )
    };
    let on_reload = {
        let record = record.clone();
        Rc::new(
            move |_e: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                record(ErrorObservation::Reload, cx);
            },
        )
    };
    let on_close = {
        let record = record.clone();
        Rc::new(
            move |_e: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                record(ErrorObservation::Close, cx);
            },
        )
    };
    let on_new_tab = {
        let record = record.clone();
        Rc::new(
            move |_e: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                record(ErrorObservation::NewTab, cx);
            },
        )
    };
    ErrorObserver {
        on_action,
        on_dismiss,
        on_reload,
        on_close,
        on_new_tab,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_records_each_interaction_independently() {
        let mut recorder = ErrorRecorder::default();
        recorder.observe(ErrorObservation::Action);
        recorder.observe(ErrorObservation::Action);
        recorder.observe(ErrorObservation::Dismiss);
        recorder.observe(ErrorObservation::Reload);
        recorder.observe(ErrorObservation::Close);
        recorder.observe(ErrorObservation::NewTab);
        assert_eq!(recorder.actions, 2);
        assert_eq!(recorder.dismissals, 1);
        assert_eq!(recorder.reloads, 1);
        assert_eq!(recorder.closes, 1);
        assert_eq!(recorder.new_tabs, 1);
        // An action never touches the other counters.
        recorder.observe(ErrorObservation::Action);
        assert_eq!(recorder.actions, 3);
        assert_eq!(recorder.dismissals, 1);
        assert_eq!(recorder.reloads, 1);
        assert_eq!(recorder.closes, 1);
        assert_eq!(recorder.new_tabs, 1);
    }

    #[test]
    fn recorder_defaults_are_all_rest() {
        let recorder = ErrorRecorder::default();
        assert_eq!(recorder.actions, 0);
        assert_eq!(recorder.dismissals, 0);
        assert_eq!(recorder.reloads, 0);
        assert_eq!(recorder.closes, 0);
        assert_eq!(recorder.new_tabs, 0);
        assert_eq!(recorder, ErrorRecorder::default());
    }
}
