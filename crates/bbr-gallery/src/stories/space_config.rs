//! The Space-configuration story (packet
//! `space-creation-appearance-ui.md`, DESIGN.md §11.3): the **exact
//! production `bbr_ui::surfaces::space_config::space_config_surface`**
//! renders every valid matrix family from the one matrix authority
//! [`crate::space_config_matrix`] with recorder callbacks proving which
//! production callback fired (`on_select_theme`, `on_select_identity`,
//! `on_inside_press`, `on_done`, `on_cancel`). The story's state selector
//! switches the matrix family (one surface at a time, so every production
//! id is unique and the E2E can target any element directly). The theme,
//! identity, and name draft come only from the closed curated catalogs —
//! the same closed sets the domain enforces. No gallery-local
//! reimplementation of anatomy, no copied tokens, no fake
//! engine/controller/store.

use std::rc::Rc;

use bbr_ui::surfaces::space_config::space_config_surface;
use gpui::{div, prelude::*, px, AnyElement, SharedString};

use crate::registry::StoryContext;
use crate::space_config_matrix::space_config_cases;
use crate::stories::{section_heading, story_page, value_row};

/// The gallery test-target selector for the rendered surface
/// (`space-config-family` — one family at a time via the story's state
/// selector). Test target only — never a product id.
pub const SPACE_CONFIG_FAMILY_SELECTOR: &str = "space-config-family";

/// The story's state options: the matrix family labels in declaration
/// order (the state selector switches the rendered family). Pinned by a
/// matrix test so the options can never drift from
/// [`crate::space_config_matrix::space_config_cases`].
pub const SPACE_CONFIG_STATE_OPTIONS: &[&str] = &[
    "Fresh defaults",
    "Emoji identity",
    "BBR glyph identity",
    "Renamed draft",
];

/// §11.3 Space configuration: the production `bbr_ui::surfaces::space_config`
/// surface renders the selected matrix family with recorder callbacks
/// proving which production callback fired. The family is drawn on the
/// `surface_overlay` canvas material exactly like the in-rail sidebar
/// panel, with its own §9 focus handle (the E2E focus assertion targets
/// the family container, never the product surface's focus).
pub fn render_space_config(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let story = ctx
        .space_config
        .as_ref()
        .expect("Space configuration story renders with the recorder");
    let observer = story.observer;
    let cases = space_config_cases();
    let index = cases
        .iter()
        .position(|case| case.label == ctx.state)
        .unwrap_or(0);
    let case = &cases[index];

    let on_select_theme = Rc::clone(&observer.on_select_theme);
    let on_select_identity = Rc::clone(&observer.on_select_identity);
    let on_select_intensity = Rc::clone(&observer.on_select_intensity);
    let on_toggle_grain = Rc::clone(&observer.on_toggle_grain);
    let on_inside_press = Rc::clone(&observer.on_inside_press);
    let on_done = Rc::clone(&observer.on_done);
    let on_cancel = Rc::clone(&observer.on_cancel);
    let callbacks = bbr_ui::surfaces::space_config::SpaceConfigCallbacks {
        on_select_theme,
        on_select_identity,
        on_select_intensity,
        on_toggle_grain,
        on_inside_press,
        on_done,
        on_cancel,
    };
    let surface = space_config_surface(theme, &case.model, case.focused, story.focus, &callbacks);

    let meta = format!(
        "{} · theme {} · identity {} · paper {}{} · name {:?}",
        if case.focused { "focused" } else { "unfocused" },
        case.model.theme.name(),
        case.model.identity.name(),
        case.model.paper.intensity.name(),
        if case.model.paper.grain {
            " + grain"
        } else {
            ""
        },
        case.model.name,
    );
    let mut children = vec![
        section_heading(
            theme,
            "§11.3 rename/identity configuration (production bbr_ui::surfaces::space_config)",
        ),
        value_row(
            theme,
            "ids",
            "bbr-space-config · -title · -name · -theme-<n> · -emoji-<n> · -glyph-<n> · -done · -cancel · -actions",
        ),
        value_row(
            theme,
            "callbacks",
            "on_select_theme · on_select_identity · on_select_intensity · on_toggle_grain · on_inside_press · on_done · on_cancel (recorder)",
        ),
        value_row(
            theme,
            "catalogs",
            "SpaceTheme::ALL · ALLOWED_EMOJI · BbrGlyph::ALL (closed curated sets — no free-form values)",
        ),
        value_row(
            theme,
            "contract",
            "creation commits before configuration · cancel keeps the created Space and its defaults · exact SpaceId targeting",
        ),
    ];
    children.push(
        div()
            .debug_selector(move || format!("{SPACE_CONFIG_FAMILY_SELECTOR}{index}"))
            .flex()
            .flex_col()
            .gap_1()
            .w(px(320.0))
            .child(surface)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .child(
                        div()
                            .text_size(theme.type_.label.size)
                            .text_color(theme.palette.ink)
                            .child(SharedString::from(case.label)),
                    )
                    .child(
                        div()
                            .text_size(theme.type_.meta.size)
                            .text_color(theme.palette.ink_faint)
                            .child(SharedString::from(meta)),
                    ),
            )
            .into_any(),
    );
    story_page(ctx, "Space configuration", children)
}
