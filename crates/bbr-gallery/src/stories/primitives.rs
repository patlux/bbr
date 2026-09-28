//! The G1b primitive stories (packet `component-gallery-mvp.md`): real
//! `bbr-ui` components with plain fixture models — `favicon_tile`,
//! `profile_chip`, `section_label`, and the §10.5 `Icon::LoadingArc`.
//! No gallery-local reimplementation of anatomy, no copied tokens.

use bbr_ui::components::{favicon_tile, profile_chip, section_label};
use bbr_ui::icons::Icon;
use bbr_ui::state::TabVisualState;
use bbr_ui::theme::Density;
use gpui::{div, prelude::*, AnyElement};

use crate::fixtures::{favicon_spec, profile_chip_spec, ALL_HUES, FAVICON_HOSTS};
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// §7.2 favicon tile: the deterministic monogram ramp + the loading arc.
/// The story's state selector switches the tile's `TabVisualState`.
pub fn render_favicon_tile(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let state = tab_state(ctx.state);
    let tiles: Vec<AnyElement> = FAVICON_HOSTS
        .iter()
        .enumerate()
        .map(|(index, host)| {
            let spec = favicon_spec(host);
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(favicon_tile(theme, index, Some(spec.clone()), state))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_size(theme.type_.label.size)
                                .text_color(theme.palette.ink)
                                .child(*host),
                        )
                        .child(
                            div()
                                .text_size(theme.type_.meta.size)
                                .text_color(theme.palette.ink_faint)
                                .child(format!("monogram {} · slate {}", spec.letter, spec.hue)),
                        ),
                )
                .into_any()
        })
        .collect();
    let mut children = vec![
        section_heading(theme, "Monogram tiles (missing-favicon rule)"),
        value_row(theme, "state", ctx.state),
    ];
    children.extend(tiles);
    children.push(section_heading(theme, "Missing favicon"));
    children.push(
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .child(favicon_tile(theme, 99, None, TabVisualState::Default))
            .child(
                div()
                    .text_size(theme.type_.meta.size)
                    .text_color(theme.palette.ink_faint)
                    .child("no spec → '?' on surface:field"),
            )
            .into_any(),
    );
    story_page(ctx, "Favicon tile", children)
}

/// §12 profile chip: one chip per hue with its deterministic monogram.
pub fn render_profile_chip(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let chips: Vec<AnyElement> = ALL_HUES
        .iter()
        .enumerate()
        .map(|(index, hue)| {
            let spec = profile_chip_spec(*hue);
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(profile_chip(theme, index, &spec))
                .child(
                    div()
                        .text_size(theme.type_.label.size)
                        .text_color(theme.palette.ink)
                        .child(format!("{hue:?}")),
                )
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(theme.palette.ink_faint)
                        .child(format!(
                            "monogram {} · bbr-profile-chip-{index}",
                            spec.letter
                        )),
                )
                .into_any()
        })
        .collect();
    let mut children = vec![section_heading(theme, "§12 identity chips")];
    children.extend(chips);
    story_page(ctx, "Profile chip", children)
}

/// §11.2 section label: uppercase `type:badge`, focused brightens one step.
pub fn render_section_label(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    story_page(
        ctx,
        "Section label",
        vec![
            section_heading(theme, "At rest"),
            section_label(theme, "Pinned", false).into_any(),
            section_label(theme, "Tabs", false).into_any(),
            section_label(theme, "Favorites", false).into_any(),
            section_heading(theme, "Focused (one text-step brighten)"),
            section_label(theme, "Pinned", true).into_any(),
            section_heading(theme, "Contract"),
            value_row(theme, "type", "type:badge · 11 pt / 600 · uppercase"),
            value_row(theme, "id", "bbr-section-<name>"),
        ],
    )
}

/// §10.5 loading arc: the accent spinner glyph at S/N/XL sizes.
pub fn render_loading_arc(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let sizes = [Density::Small, Density::Normal, Density::ExtraLarge];
    let rows: Vec<AnyElement> = sizes
        .iter()
        .map(|d| {
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(Icon::LoadingArc.canvas(theme, d.icon()))
                .child(
                    div()
                        .text_size(theme.type_.label.size)
                        .text_color(theme.palette.ink)
                        .child(match d {
                            Density::Small => "S · 16 pt",
                            Density::Normal => "N · 18 pt",
                            Density::ExtraLarge => "XL · 20 pt",
                        }),
                )
                .into_any()
        })
        .collect();
    let mut children = vec![
        section_heading(theme, "The §10.5 accent arc (static)"),
        value_row(theme, "color", "color:accent"),
        value_row(theme, "motion", "rotation is a later polish pass"),
    ];
    children.extend(rows);
    story_page(ctx, "Loading arc", children)
}

/// Map the favicon story's state option to a `TabVisualState`.
fn tab_state(name: &str) -> TabVisualState {
    match name {
        "loading" => TabVisualState::Loading,
        "error" => TabVisualState::Error,
        _ => TabVisualState::Default,
    }
}
