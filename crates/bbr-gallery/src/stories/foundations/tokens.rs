//! The G1b token/geometry Foundations pages (packet
//! `component-gallery-mvp.md`): Tokens (materials/elevation), Density
//! (S/N/XL geometry), Elevation (the `elev:2` shadow pair), and Color
//! (per mode+contrast with the derived contrast ratio). Every value derives
//! from the real `bbr-ui` tokens; nothing is copied or reimplemented.

use bbr_ui::theme::{Density, ThemeMode};
use gpui::{div, prelude::*, px, AnyElement, Rgba};

use crate::registry::StoryContext;
use crate::stories::{contrast_ratio, hex_of, section_heading, story_page, swatch, value_row};

/// §2 materials + elevation levels.
pub fn render_tokens(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let p = theme.palette;
    story_page(
        theme,
        "Tokens — surfaces, materials, elevation",
        vec![
            section_heading(theme, "Materials"),
            swatch(theme, "material:canvas", p.surface_canvas),
            swatch(theme, "material:sidebar", p.surface_sidebar),
            swatch(theme, "material:field", p.surface_field),
            swatch(theme, "material:overlay", p.surface_overlay),
            swatch(theme, "material:glass", p.surface_glass),
            section_heading(theme, "Elevation"),
            value_row(theme, "elev:0", "base surface"),
            value_row(theme, "elev:1", "raised control (hairline border)"),
            value_row(theme, "elev:2", "floating surface (shadow pair)"),
        ],
    )
}

/// §8 density geometry table (S/N/XL).
pub fn render_density(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let densities = [Density::Small, Density::Normal, Density::ExtraLarge];
    let header = div()
        .flex()
        .flex_row()
        .child(
            div()
                .w(px(140.0))
                .text_size(theme.type_.badge.size)
                .font_weight(theme.type_.badge.weight)
                .text_color(theme.palette.ink_secondary)
                .child("token"),
        )
        .children(densities.iter().map(|d| {
            div()
                .flex_1()
                .text_size(theme.type_.badge.size)
                .font_weight(theme.type_.badge.weight)
                .text_color(theme.palette.ink_secondary)
                .child(match d {
                    Density::Small => "S",
                    Density::Normal => "N",
                    Density::ExtraLarge => "XL",
                })
        }))
        .into_any();
    let rows: Vec<AnyElement> = [
        "density:row",
        "density:row-tile",
        "icon:",
        "density:sidebar",
        "density:hit-min",
        "density:hit-min.close",
        "density:row-inset",
        "density:section-gap",
        "density:profile-chip",
    ]
    .into_iter()
    .map(|name| {
        div()
            .flex()
            .flex_row()
            .child(
                div()
                    .w(px(140.0))
                    .text_size(theme.type_.label.size)
                    .text_color(theme.palette.ink)
                    .child(name),
            )
            .children(densities.iter().map(|d| {
                div()
                    .flex_1()
                    .text_size(theme.type_.mono.size)
                    .text_color(theme.palette.ink_secondary)
                    .child(density_value(name, *d))
            }))
            .into_any()
    })
    .collect();
    let mut children = vec![header];
    children.extend(rows);
    story_page(theme, "Density — S / N / XL geometry", children)
}

/// The §8 density geometry value for one token at one density.
fn density_value(token: &str, d: Density) -> String {
    let pts = match token {
        "density:row" => d.row(),
        "density:row-tile" => d.row_tile(),
        "icon:" => d.icon(),
        "density:sidebar" => d.sidebar_width(),
        "density:hit-min" => d.hit_min(),
        "density:hit-min.close" => d.hit_min_close(),
        "density:row-inset" => d.row_inset(),
        "density:section-gap" => d.section_gap(),
        "density:profile-chip" => d.profile_chip(),
        _ => return String::new(),
    };
    format!("{}", f32::from(pts).round() as u32)
}

/// §2/§18.2 `elev:2` and `elev:lift` shadows via the one `bbr-ui` helper.
pub fn render_elevation(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let shadows = bbr_ui::theme::elevation::elev2(theme)
        .into_iter()
        .map(|shadow| ("elev:2 shadow", shadow))
        .chain(
            bbr_ui::theme::elevation::lift(theme)
                .into_iter()
                .map(|shadow| ("elev:lift shadow", shadow)),
        )
        .collect::<Vec<_>>();
    let shadow_rows: Vec<AnyElement> = shadows
        .iter()
        .map(|(name, shadow)| {
            value_row(
                theme,
                name,
                &format!(
                    "{}px {}px blur {}px · {}",
                    f32::from(shadow.offset.x).round() as i32,
                    f32::from(shadow.offset.y).round() as i32,
                    f32::from(shadow.blur_radius).round() as i32,
                    hex_of(gpui::Rgba::from(shadow.color))
                ),
            )
        })
        .collect();
    let mut children = vec![
        section_heading(theme, "elev:1 — raised control"),
        value_row(theme, "elev:1", "hairline border, same material family"),
        section_heading(theme, "elev:2 — floating surface"),
    ];
    children.extend(shadow_rows);
    children.push(
        div()
            .mt_2()
            .p_4()
            .rounded(theme.radius.large())
            .bg(theme.palette.surface_overlay)
            .border_1()
            .border_color(theme.palette.hairline)
            .text_size(theme.type_.label.size)
            .text_color(theme.palette.ink)
            .child("Floating surface sample")
            .into_any(),
    );
    story_page(theme, "Elevation", children)
}

/// §3 color tokens per mode+contrast with the derived contrast ratio.
pub fn render_color(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let p = theme.palette;
    let canvas = p.surface_canvas;
    let pairs: Vec<(&str, Rgba, Rgba)> = vec![
        ("color:ink", p.ink, canvas),
        ("color:ink.secondary", p.ink_secondary, canvas),
        ("color:accent", p.accent, canvas),
        ("color:accent.ink", p.accent_ink, canvas),
        ("color:status.error", p.status_error, canvas),
        ("color:status.attention", p.status_attention, canvas),
        ("color:status.ok", p.status_ok, canvas),
        ("color:status.info", p.status_info, canvas),
        // §18.2 veils are shown composited over the rail they paint on.
        (
            "color:tint.selected",
            bbr_ui::palette::over(p.tint_selected, p.surface_sidebar),
            canvas,
        ),
        ("color:tint.accent", p.tint_accent, canvas),
        ("color:hairline", p.hairline, canvas),
        ("color:border.strong", p.border_strong, canvas),
    ];
    let rows: Vec<AnyElement> = pairs
        .into_iter()
        .map(|(name, color, on)| {
            let ratio = contrast_ratio(color, on);
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .size(px(24.0))
                        .rounded(theme.radius.small())
                        .bg(color)
                        .border_1()
                        .border_color(theme.palette.hairline),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_size(theme.type_.label.size)
                                .text_color(theme.palette.ink)
                                .child(name),
                        )
                        .child(
                            div()
                                .text_size(theme.type_.meta.size)
                                .text_color(theme.palette.ink_faint)
                                .child(format!(
                                    "{} · {:.1}:1 on material:canvas",
                                    hex_of(color),
                                    ratio
                                )),
                        ),
                )
                .into_any()
        })
        .collect();
    let mut children = vec![
        section_heading(theme, "Mode / contrast"),
        value_row(
            theme,
            "theme",
            match theme.mode {
                ThemeMode::Light => "light",
                ThemeMode::Dark => "dark",
            },
        ),
        value_row(
            theme,
            "contrast",
            match theme.contrast {
                bbr_ui::theme::ContrastMode::Standard => "standard",
                bbr_ui::theme::ContrastMode::Increased => "increased",
            },
        ),
        section_heading(theme, "Tokens (contrast derived from tokens)"),
    ];
    children.extend(rows);
    children.extend(super::paper::paper_sections(theme));
    story_page(theme, "Color", children)
}
