//! Core Controls interaction scenarios (live-review finding 2): the
//! stories render the exact vendored gpui-component 0.5.1 production
//! symbols; these scenarios prove **real** interactions — a real pointer
//! click on a Button, real pointer clicks on Checkbox/Switch/Radio, real
//! pointer clicks on the Slider track, and real keyboard focus + typed
//! editing in the Input — and assert the resulting state/callback changes
//! through the app snapshot. Nothing is faked by calling the app methods
//! directly; every interaction is a real platform event at the control's
//! on-screen bounds.

use bbr_ui::theme::{ContrastMode, Density, ThemeMode};
use gpui::{point, px, Entity, VisualTestContext};

use crate::app::GalleryApp;
use crate::harness::{
    bounds_of, click_selector, flush, move_to_story, run_scenario, snapshot, type_text,
};

/// Navigate to a Core Controls story by its label.
fn open_story(window: &mut VisualTestContext, app: &Entity<GalleryApp>, label: &str) {
    let story_id = window.read(|cx| {
        app.read(cx)
            .registry()
            .stories
            .iter()
            .find(|s| s.label == label)
            .expect("core control story registered")
            .id
            .0
    });
    move_to_story(window, app, story_id);
    flush(window);
}

#[test]
#[ignore]
fn real_button_click_reports_through_the_recorder() {
    run_scenario(|window, app| {
        open_story(window, app, "Button");
        let snap = snapshot(window, app);
        assert_eq!(snap.button_clicks, 0);
        // The Button story renders the real gpui-component Button at these
        // bounds; a real pointer click reports through the recorder path.
        let bounds = bounds_of(window, "bbr-gallery-controls-button");
        assert!(bounds.size.width > px(0.0) && bounds.size.height > px(0.0));
        click_selector(window, "bbr-gallery-controls-button");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.button_clicks, 1, "real Button click recorded");
        // A second real click on the ghost variant increments again.
        click_selector(window, "bbr-gallery-controls-button-ghost");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.button_clicks, 2);
        // Mode/state fields are untouched by control interactions.
        assert_eq!(snap.theme, ThemeMode::Dark);
        assert_eq!(snap.contrast, ContrastMode::Standard);
        assert_eq!(snap.density, Density::Normal);
    });
}

#[test]
#[ignore]
fn real_checkbox_and_switch_clicks_flip_state() {
    run_scenario(|window, app| {
        open_story(window, app, "Checkbox");
        let snap = snapshot(window, app);
        assert!(!snap.checkbox_checked);
        click_selector(window, "bbr-gallery-controls-checkbox");
        flush(window);
        let snap = snapshot(window, app);
        assert!(snap.checkbox_checked, "real Checkbox click checks it");
        click_selector(window, "bbr-gallery-controls-checkbox");
        flush(window);
        let snap = snapshot(window, app);
        assert!(!snap.checkbox_checked, "real Checkbox click unchecks it");

        open_story(window, app, "Switch");
        let snap = snapshot(window, app);
        assert!(!snap.switch_checked);
        click_selector(window, "bbr-gallery-controls-switch");
        flush(window);
        let snap = snapshot(window, app);
        assert!(snap.switch_checked, "real Switch click toggles it on");
        click_selector(window, "bbr-gallery-controls-switch");
        flush(window);
        let snap = snapshot(window, app);
        assert!(!snap.switch_checked, "real Switch click toggles it off");
    });
}

#[test]
#[ignore]
fn real_radio_clicks_change_the_selection() {
    run_scenario(|window, app| {
        open_story(window, app, "Radio");
        let snap = snapshot(window, app);
        assert_eq!(snap.radio_index, 0, "first radio selected by default");
        // Click the second radio: the selection moves.
        click_selector(window, "bbr-gallery-controls-radio-1");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.radio_index, 1, "real Radio click selects option 1");
        // Click the third radio: the selection moves again; clicking the
        // already-selected radio does not deselect.
        click_selector(window, "bbr-gallery-controls-radio-2");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.radio_index, 2);
        click_selector(window, "bbr-gallery-controls-radio-2");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.radio_index, 2, "clicking the selected radio keeps it");
    });
}

#[test]
#[ignore]
fn real_input_focus_and_typed_editing() {
    run_scenario(|window, app| {
        open_story(window, app, "Input");
        let snap = snapshot(window, app);
        assert_eq!(snap.input_text, "", "input starts empty");
        // A real pointer click focuses the production Input.
        let bounds = bounds_of(window, "bbr-gallery-controls-input");
        assert!(bounds.size.width > px(0.0) && bounds.size.height > px(0.0));
        click_selector(window, "bbr-gallery-controls-input");
        flush(window);
        // Real keystrokes type into the focused input (the simulated-IME
        // input-handler path — the same path real typing takes).
        type_text(window, "hello");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.input_text, "hello",
            "real keystrokes edit the production InputState"
        );
        // More typed text appends; the value is read from the real entity.
        type_text(window, " world");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(snap.input_text, "hello world");
    });
}

#[test]
#[ignore]
fn real_slider_clicks_change_the_value_and_progress_mirrors_it() {
    run_scenario(|window, app| {
        open_story(window, app, "Slider");
        let bounds = bounds_of(window, "bbr-gallery-controls-slider");
        assert!(bounds.size.width > px(20.0), "slider track is laid out");
        let initial = snapshot(window, app).slider_value;
        assert_eq!(initial, 25.0, "slider starts at its default value");
        // A real pointer click near the left end of the track sets a small
        // value; near the right end sets a large value.
        let left = point(bounds.origin.x + px(4.0), bounds.center().y);
        let right = point(
            bounds.origin.x + bounds.size.width - px(4.0),
            bounds.center().y,
        );
        window.simulate_click(left, gpui::Modifiers::none());
        flush(window);
        let low = snapshot(window, app).slider_value;
        assert!(low < 25.0, "click near the left lowers the value ({low})");
        window.simulate_click(right, gpui::Modifiers::none());
        flush(window);
        let high = snapshot(window, app).slider_value;
        assert!(
            high > 25.0,
            "click near the right raises the value ({high})"
        );
        assert!(low < high, "left click < right click ({low} < {high})");

        // The Progress story mirrors the real SliderState value.
        open_story(window, app, "Progress");
        flush(window);
        let snap = snapshot(window, app);
        assert_eq!(
            snap.slider_value, high,
            "Progress story keeps the same real slider value"
        );
    });
}
