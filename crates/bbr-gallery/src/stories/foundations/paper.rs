//! The v2 Color-page sections (DESIGN §18.2, D6/D7): the veils composited
//! over the rail they paint on, and the curated space paper — the three
//! intensity steps of each token and the static grain — resolved through
//! the production `bbr_ui::theme::space_tint` and painted by the production
//! `components::grain`.

use bbr_domain::{SpaceTheme, TintIntensity};
use bbr_ui::palette::over;
use bbr_ui::theme::Theme;
use gpui::{div, prelude::*, px, AnyElement, Rgba};

use crate::stories::{contrast_ratio, hex_of, section_heading, value_row};

/// The v2 veils (`wash`, footer, Favorite tile, address field) over the
/// untinted rail with the text ratio they carry.
fn veil_rows(theme: &Theme) -> Vec<AnyElement> {
    let p = theme.palette;
    let rail = p.surface_sidebar;
    [
        ("wash.hover", p.wash_hover, p.ink),
        ("wash.active", p.wash_active, p.ink),
        ("surface.footer", p.surface_footer, p.ink_secondary),
        ("surface.favorite", p.surface_favorite, p.ink),
        ("address.fill", p.address_fill, p.ink),
    ]
    .into_iter()
    .map(|(name, veil, text)| {
        let composited = over(veil, rail);
        chip_row(
            theme,
            composited,
            &format!(
                "{name} · α {:.0} % · {} on the rail · text {:.1}:1",
                veil.a * 100.0,
                hex_of(composited),
                contrast_ratio(text, composited)
            ),
        )
    })
    .collect()
}

fn chip_row(theme: &Theme, color: Rgba, text: &str) -> AnyElement {
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
                .text_size(theme.type_.meta.size)
                .text_color(theme.palette.ink_secondary)
                .child(text.to_string()),
        )
        .into_any()
}

/// One rail sample of a token at a step, optionally grained.
fn rail_sample(theme: &Theme, token: SpaceTheme, step: TintIntensity, grain: bool) -> AnyElement {
    let chrome = bbr_ui::theme::space_tint(token, step, theme).chrome;
    div()
        .relative()
        .w(px(84.0))
        .h(px(48.0))
        .rounded(theme.radius.medium())
        .overflow_hidden()
        .bg(chrome)
        .border_1()
        .border_color(theme.palette.hairline)
        .when(grain, |el| {
            el.child(bbr_ui::components::grain::grain_layer(theme))
        })
        .child(
            div()
                .relative()
                .p_1()
                .text_size(theme.type_.meta.size)
                .text_color(theme.palette.ink)
                .child(if grain {
                    format!("{} + grain", step.label())
                } else {
                    step.label().to_string()
                }),
        )
        .into_any()
}

/// The Color-page v2 sections.
pub fn paper_sections(theme: &Theme) -> Vec<AnyElement> {
    let mut children = vec![section_heading(theme, "v2 veils over the rail (§18.2)")];
    children.extend(veil_rows(theme));
    children.push(section_heading(
        theme,
        "Space paper (D6/D7) — subtle / standard / rich, static grain",
    ));
    for token in SpaceTheme::ALL {
        let mut row: Vec<AnyElement> = TintIntensity::ALL
            .into_iter()
            .map(|step| rail_sample(theme, token, step, false))
            .collect();
        row.push(rail_sample(theme, token, TintIntensity::Standard, true));
        children.push(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .w(px(72.0))
                        .text_size(theme.type_.label.size)
                        .text_color(theme.palette.ink)
                        .child(token.label()),
                )
                .children(row)
                .into_any(),
        );
    }
    let rich = bbr_ui::theme::space_tint(SpaceTheme::Cobalt, TintIntensity::Rich, theme);
    children.push(value_row(
        theme,
        "rich pins luminance",
        &format!(
            "cobalt standard {} → rich {}",
            hex_of(
                bbr_ui::theme::space_tint(SpaceTheme::Cobalt, TintIntensity::Standard, theme)
                    .chrome
            ),
            hex_of(rich.chrome)
        ),
    ));
    children
}
