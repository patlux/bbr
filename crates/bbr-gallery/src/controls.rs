//! The gallery mode/state controls (packet `component-gallery-mvp.md`):
//! `ThemeMode`, `ContrastMode`, `Density`, Reduced Motion, and the selected
//! story's own State/Variant selector. Every control resolves through the
//! app's production methods (`ControlAction` → `GalleryApp::apply_control`),
//! so the E2E drives the exact same path a pointer click takes. The canvas
//! re-renders on every change and the §15 mode fields are asserted through
//! the app snapshot.

use bbr_ui::theme::{ContrastMode, Density, Theme, ThemeMode};
use gpui::{div, prelude::*, AnyElement, App, SharedString, Window};
use std::rc::Rc;

/// One control action, resolved by the app's production method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlAction {
    Mode(ThemeMode),
    Contrast(ContrastMode),
    Density(Density),
    ReducedMotion(bool),
    State(usize),
}

/// The control callback the surface routes pointer clicks through (the same
/// seam `cx.listener` uses).
type ControlCallback = Rc<dyn Fn(ControlAction, &mut Window, &mut App)>;

/// The current control values the surface renders.
pub struct ControlsModel<'a> {
    pub theme: &'a Theme,
    pub mode: ThemeMode,
    pub contrast: ContrastMode,
    pub density: Density,
    pub reduced_motion: bool,
    pub state_options: &'a [&'a str],
    pub state_index: usize,
}

/// The controls bar: one labeled group per control, buttons with stable
/// `bbr-gallery-controls-*` ids.
pub fn controls_surface(
    model: &ControlsModel,
    on_control: impl Fn(ControlAction, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let theme = model.theme;
    let on_control: ControlCallback = Rc::new(on_control);
    let mut groups: Vec<(&str, &str, Vec<&str>, usize)> = vec![
        (
            "mode",
            "Mode",
            vec!["light", "dark"],
            match model.mode {
                ThemeMode::Light => 0,
                ThemeMode::Dark => 1,
            },
        ),
        (
            "contrast",
            "Contrast",
            vec!["standard", "increased"],
            match model.contrast {
                ContrastMode::Standard => 0,
                ContrastMode::Increased => 1,
            },
        ),
        (
            "density",
            "Density",
            vec!["small", "normal", "extralarge"],
            match model.density {
                Density::Small => 0,
                Density::Normal => 1,
                Density::ExtraLarge => 2,
            },
        ),
        (
            "motion",
            "Reduced Motion",
            vec!["off", "on"],
            if model.reduced_motion { 1 } else { 0 },
        ),
    ];
    if !model.state_options.is_empty() {
        groups.push((
            "state",
            "State",
            model.state_options.to_vec(),
            model.state_index,
        ));
    }
    div()
        .id("bbr-gallery-controls")
        .flex()
        .flex_row()
        .items_center()
        .gap_3()
        .px_3()
        .py_2()
        .bg(theme.palette.surface_sidebar)
        .border_b_1()
        .border_color(theme.palette.hairline)
        .children(groups.into_iter().map(|(name, label, options, active)| {
            control_group(theme, name, label, options, active, on_control.clone())
        }))
        .into_any()
}

/// One labeled control group: a `type:badge` label and a row of buttons.
fn control_group(
    theme: &Theme,
    name: &str,
    label: &str,
    options: Vec<&str>,
    active: usize,
    on_control: ControlCallback,
) -> AnyElement {
    let name = SharedString::from(name);
    let label = SharedString::from(label);
    let group_id = SharedString::from(format!("bbr-gallery-controls-{name}"));
    div()
        .id(group_id)
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .text_size(theme.type_.badge.size)
                .font_weight(theme.type_.badge.weight)
                .text_color(theme.palette.ink_faint)
                .child(label),
        )
        .children(options.into_iter().enumerate().map(|(index, option)| {
            let selected = index == active;
            let action = action_for(&name, index, option);
            let option = SharedString::from(option);
            let on_control = on_control.clone();
            let button_id = SharedString::from(format!("bbr-gallery-controls-{name}-{option}"));
            div()
                .id(button_id)
                .px_2()
                .py_1()
                .rounded(theme.radius.small())
                .cursor_pointer()
                .bg(if selected {
                    theme.palette.tint_selected
                } else {
                    theme.palette.surface_field
                })
                .border_1()
                .border_color(if selected {
                    theme.palette.accent
                } else {
                    theme.palette.hairline
                })
                .text_size(theme.type_.label.size)
                .text_color(if selected {
                    theme.palette.ink
                } else {
                    theme.palette.ink_secondary
                })
                .on_click(move |_event, window, cx| {
                    if let Some(action) = action {
                        on_control(action, window, cx);
                    }
                    cx.stop_propagation();
                })
                .child(option)
                .into_any()
        }))
        .into_any()
}

/// Map a control group + option to its action.
fn action_for(name: &str, index: usize, option: &str) -> Option<ControlAction> {
    match name {
        "mode" => Some(ControlAction::Mode(match option {
            "dark" => ThemeMode::Dark,
            _ => ThemeMode::Light,
        })),
        "contrast" => Some(ControlAction::Contrast(match option {
            "increased" => ContrastMode::Increased,
            _ => ContrastMode::Standard,
        })),
        "density" => Some(ControlAction::Density(match option {
            "small" => Density::Small,
            "extralarge" => Density::ExtraLarge,
            _ => Density::Normal,
        })),
        "motion" => Some(ControlAction::ReducedMotion(option == "on")),
        "state" => Some(ControlAction::State(index)),
        _ => None,
    }
}
