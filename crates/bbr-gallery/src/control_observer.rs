//! The Core Controls observer builder (packet
//! `component-gallery-mvp.md`): builds the shared `ControlObserver`
//! callback bundle that re-enters the gallery app entity through the
//! production `observe_control` path. Kept in its own focused module so
//! `app.rs` stays within the ~500-line budget (the `tab_row_app.rs` /
//! `library_observer.rs` pattern).

use std::rc::Rc;

use gpui::{App, WeakEntity, Window};

use crate::app::GalleryApp;
use crate::recorder::{ControlObservation, ControlObserver};

/// Build the Core Controls observer: every callback re-enters the app
/// entity (the same downgraded handle seam the tree and the mode/state
/// controls use), so each real control event resolves through the app's
/// production `observe_control` method and re-renders the canvas.
pub fn build_control_observer(entity: WeakEntity<GalleryApp>) -> ControlObserver {
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
    ControlObserver {
        on_button,
        on_checkbox,
        on_switch,
        on_radio,
    }
}
