//! The G1b type/motion Foundations pages (packet `component-gallery-mvp.md`):
//! Typography (type roles per density), Spacing/Radius (the 4 pt grid and
//! radius tokens); the Motion page lives in the sibling `motion` module.
//! Every value derives from the real `bbr-ui` tokens; nothing is copied or
//! reimplemented.

use gpui::{div, prelude::*, px, AnyElement};

use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page};

/// §4 type roles per density.
pub fn render_typography(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let t = theme.type_;
    let rows: Vec<AnyElement> = [
        ("type:display", t.display, "Display"),
        ("type:title", t.title, "Tab title"),
        ("type:label", t.label, "Button label"),
        ("type:meta", t.meta, "Status · 1,234"),
        ("type:mono", t.mono, "https://example.com"),
        ("type:key", t.key, "⌘T"),
        ("type:badge", t.badge, "PINNED"),
    ]
    .into_iter()
    .map(|(name, step, sample)| {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .text_size(theme.type_.meta.size)
                    .text_color(theme.palette.ink_faint)
                    .child(format!(
                        "{name} · {}pt / {} / leading {}",
                        f32::from(step.size).round() as u32,
                        step.weight.0,
                        step.leading
                    )),
            )
            .child(
                div()
                    .text_size(step.size)
                    .font_weight(step.weight)
                    .text_color(theme.palette.ink)
                    .child(sample),
            )
            .into_any()
    })
    .collect();
    story_page(theme, "Typography — type roles", rows)
}

/// §5 spacing scale + radius tokens.
pub fn render_spacing_radius(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let spacing: Vec<AnyElement> = (1..=8)
        .map(|n| {
            let pts = n as f32 * 4.0;
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .w(px(64.0))
                        .text_size(theme.type_.mono.size)
                        .text_color(theme.palette.ink_secondary)
                        .child(format!("sp:{n}")),
                )
                .child(
                    div()
                        .h(px(8.0))
                        .w(px(pts))
                        .rounded(theme.radius.small())
                        .bg(theme.palette.accent),
                )
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(theme.palette.ink_faint)
                        .child(format!("{pts:.0} pt")),
                )
                .into_any()
        })
        .collect();
    let radii: Vec<AnyElement> = [
        (
            "radius:sm",
            theme.radius.small(),
            "small tile, profile chip",
        ),
        ("radius:md", theme.radius.medium(), "N rows, Favorite tiles"),
        (
            "radius:lg",
            theme.radius.large(),
            "address/palette field, panels",
        ),
        (
            "radius:xl",
            theme.radius.xlarge(),
            "⌘T palette sheet (reserved)",
        ),
        ("radius:full", theme.radius.full(), "dots, avatars"),
    ]
    .into_iter()
    .map(|(name, radius, note)| {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .child(
                div()
                    .size(px(28.0))
                    .rounded(radius)
                    .bg(theme.palette.surface_field)
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
                            .child(note),
                    ),
            )
            .into_any()
    })
    .collect();
    let mut children = vec![section_heading(theme, "Spacing (4 pt grid)")];
    children.extend(spacing);
    children.push(section_heading(theme, "Radius"));
    children.extend(radii);
    story_page(theme, "Spacing / Radius", children)
}
