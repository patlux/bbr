//! The G1b Core Controls stories (packet `component-gallery-mvp.md`,
//! live-review finding 2): the **exact vendored gpui-component 0.5.1
//! production symbols** — `Button`, real `Input` + `InputState`, `Checkbox`,
//! `Switch`, `Radio`, `Slider` + `SliderState`, and `Progress` — rendered
//! with plain fixture state and recorder callbacks.
//!
//! There is deliberately **no** Gallery-local wrapper, reimplementation,
//! restyle, or copy of any control: every control below is a direct
//! `gpui_component` production symbol. The gallery only (a) composes them
//! inside its own story page (the same presentation scaffold every story
//! uses) and (b) attaches recorder callbacks that report real interactions
//! back into the app's production `observe_control` path. The state
//! entities (`InputState`, `SliderState`) are the app-owned production
//! entities created through the integration pattern the controls require.
//!
//! The `debug_selector` container divs are test targets only: they let the
//! real-pointer E2E click the *actual* control at its real on-screen
//! bounds. They never alter the control's anatomy, styling, or behavior.

use gpui::{div, prelude::*, px, AnyElement, ClickEvent, SharedString};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::checkbox::Checkbox;
use gpui_component::input::Input;
use gpui_component::progress::Progress;
use gpui_component::radio::Radio;
use gpui_component::slider::Slider;
use gpui_component::switch::Switch;

use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// The shared no-op body for a Core Controls story rendered without its
/// context (defensive only — the app always supplies the context).
fn missing_context(ctx: &StoryContext, title: &str) -> AnyElement {
    story_page(ctx, title, Vec::new())
}

/// A `self_start` container that hugs its child so its debug bounds equal
/// the real control's bounds — the real-pointer E2E target. Pure layout;
/// the control itself is untouched.
fn control_slot(selector: &str, child: impl IntoElement) -> AnyElement {
    div()
        .self_start()
        .debug_selector(move || selector.to_string())
        .child(child)
        .into_any()
}

/// `gpui-component` Button: real clicks report through the recorder.
pub fn render_button(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let Some(controls) = ctx.controls.as_ref() else {
        return missing_context(ctx, "Button");
    };
    let clicks = controls.recorder.button_clicks;
    let on_button = controls.observer.on_button.clone();
    let on_button_2 = controls.observer.on_button.clone();
    story_page(
        ctx,
        "Button",
        vec![
            section_heading(theme, "gpui-component Button · primary"),
            control_slot(
                "bbr-gallery-controls-button",
                Button::new("bbr-gallery-btn-primary")
                    .primary()
                    .label("Primary action")
                    .on_click(move |_: &ClickEvent, window, cx| on_button(window, cx)),
            ),
            section_heading(theme, "Variants"),
            control_slot(
                "bbr-gallery-controls-button-ghost",
                Button::new("bbr-gallery-btn-ghost")
                    .ghost()
                    .label("Ghost action")
                    .on_click(move |_: &ClickEvent, window, cx| on_button_2(window, cx)),
            ),
            section_heading(theme, "Observed state"),
            value_row(theme, "clicks", &clicks.to_string()),
            value_row(theme, "source", "gpui-component 0.5.1 (vendored)"),
        ],
    )
}

/// `gpui-component` Input + real `InputState`: focus + typed editing through
/// real keystrokes; the typed value is read from the production entity.
pub fn render_input(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let Some(controls) = ctx.controls.as_ref() else {
        return missing_context(ctx, "Input");
    };
    let Some(input) = controls.input else {
        return missing_context(ctx, "Input");
    };
    let input_value = controls.input_value.clone();
    story_page(
        ctx,
        "Input",
        vec![
            section_heading(theme, "gpui-component Input"),
            control_slot(
                "bbr-gallery-controls-input",
                div().w(px(280.0)).child(Input::new(input)),
            ),
            section_heading(theme, "Observed state"),
            value_row(theme, "value", &input_value),
            value_row(theme, "focus", "click the field, then type"),
            value_row(theme, "source", "gpui-component 0.5.1 (vendored)"),
        ],
    )
}

/// `gpui-component` Checkbox: real clicks report the new checked state.
pub fn render_checkbox(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let Some(controls) = ctx.controls.as_ref() else {
        return missing_context(ctx, "Checkbox");
    };
    let checked = controls.recorder.checkbox_checked;
    let on_checkbox = controls.observer.on_checkbox.clone();
    story_page(
        ctx,
        "Checkbox",
        vec![
            section_heading(theme, "gpui-component Checkbox"),
            control_slot(
                "bbr-gallery-controls-checkbox",
                Checkbox::new("bbr-gallery-cb-subscribe")
                    .label("Subscribe to updates")
                    .checked(checked)
                    .on_click(move |new, window, cx| on_checkbox(*new, window, cx)),
            ),
            section_heading(theme, "Observed state"),
            value_row(theme, "checked", if checked { "true" } else { "false" }),
            value_row(theme, "source", "gpui-component 0.5.1 (vendored)"),
        ],
    )
}

/// `gpui-component` Switch: real clicks report the new checked state.
pub fn render_switch(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let Some(controls) = ctx.controls.as_ref() else {
        return missing_context(ctx, "Switch");
    };
    let checked = controls.recorder.switch_checked;
    let on_switch = controls.observer.on_switch.clone();
    story_page(
        ctx,
        "Switch",
        vec![
            section_heading(theme, "gpui-component Switch"),
            control_slot(
                "bbr-gallery-controls-switch",
                Switch::new("bbr-gallery-sw-toggles")
                    .label("Airplane mode")
                    .checked(checked)
                    .on_click(move |new, window, cx| on_switch(*new, window, cx)),
            ),
            section_heading(theme, "Observed state"),
            value_row(theme, "checked", if checked { "true" } else { "false" }),
            value_row(theme, "source", "gpui-component 0.5.1 (vendored)"),
        ],
    )
}

/// `gpui-component` Radio: real clicks report the selected option index.
pub fn render_radio(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let Some(controls) = ctx.controls.as_ref() else {
        return missing_context(ctx, "Radio");
    };
    let selected = controls.recorder.radio_index;
    let options: [&str; 3] = ["Small", "Normal", "Extra large"];
    let radios: Vec<AnyElement> = options
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let on_radio = controls.observer.on_radio.clone();
            control_slot(
                &format!("bbr-gallery-controls-radio-{index}"),
                Radio::new(SharedString::from(format!("bbr-gallery-radio-{index}")))
                    .label(*label)
                    .checked(selected == index)
                    .on_click(move |new, window, cx| {
                        // A radio reports its *new* state; only selecting
                        // (true) changes the choice — never deselecting.
                        if *new {
                            on_radio(index, window, cx);
                        }
                    }),
            )
        })
        .collect();
    story_page(
        ctx,
        "Radio",
        vec![
            section_heading(theme, "gpui-component Radio"),
            div().flex().flex_col().gap_1().children(radios).into_any(),
            section_heading(theme, "Observed state"),
            value_row(theme, "selected", &selected.to_string()),
            value_row(theme, "source", "gpui-component 0.5.1 (vendored)"),
        ],
    )
}

/// `gpui-component` Slider + real `SliderState`: real pointer clicks on the
/// track change the value; the value is read from the production entity.
pub fn render_slider(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let Some(controls) = ctx.controls.as_ref() else {
        return missing_context(ctx, "Slider");
    };
    let Some(slider) = controls.slider else {
        return missing_context(ctx, "Slider");
    };
    let slider_value = controls.slider_value;
    story_page(
        ctx,
        "Slider",
        vec![
            section_heading(theme, "gpui-component Slider · 0–100"),
            control_slot(
                "bbr-gallery-controls-slider",
                div().w(px(280.0)).child(Slider::new(slider).horizontal()),
            ),
            section_heading(theme, "Observed state"),
            value_row(theme, "value", &format!("{slider_value:.0}")),
            value_row(theme, "source", "gpui-component 0.5.1 (vendored)"),
        ],
    )
}

/// `gpui-component` Progress: static read-only bars plus one bar that
/// mirrors the real SliderState (state propagation without any copy).
pub fn render_progress(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let Some(controls) = ctx.controls.as_ref() else {
        return missing_context(ctx, "Progress");
    };
    let slider_value = controls.slider_value;
    let bars: Vec<AnyElement> = [25.0, 50.0, 75.0, 100.0]
        .into_iter()
        .map(|value| {
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(div().w(px(280.0)).child(Progress::new().value(value)))
                .child(
                    div()
                        .w(px(48.0))
                        .text_size(theme.type_.meta.size)
                        .text_color(theme.palette.ink_faint)
                        .child(format!("{value:.0}%")),
                )
                .into_any()
        })
        .collect();
    let mut children = vec![
        section_heading(theme, "gpui-component Progress"),
        section_heading(theme, "Fixed values"),
    ];
    children.extend(bars);
    children.push(section_heading(theme, "Mirrors the Slider story's value"));
    children.push(
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .child(
                div()
                    .w(px(280.0))
                    .child(Progress::new().value(slider_value)),
            )
            .child(
                div()
                    .w(px(48.0))
                    .text_size(theme.type_.meta.size)
                    .text_color(theme.palette.ink_faint)
                    .child(format!("{slider_value:.0}%")),
            )
            .into_any(),
    );
    story_page(ctx, "Progress", children)
}
