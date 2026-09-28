//! The G1b states/iconography/brand Foundations pages (packet
//! `component-gallery-mvp.md`): the §10 state vocabulary (driven by the
//! story's state selector), the §7 icon catalog × S/N/XL, and the §11.4
//! brand block. Every value derives from the real `bbr-ui` tokens and
//! components; nothing is copied or reimplemented.

use bbr_ui::icons::Icon;
use bbr_ui::state::TabVisualState;
use bbr_ui::theme::Density;
use gpui::{div, prelude::*, px, AnyElement};

use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page};

/// §10 state vocabulary — the selected state drives the sample.
pub fn render_states(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let state = tab_state_from_name(ctx.state);
    let sample = bbr_ui::components::favicon_tile(
        theme,
        0,
        Some(crate::fixtures::favicon_spec("example.com")),
        state,
    );
    let rows: Vec<AnyElement> = crate::fixtures::STATE_VOCABULARY
        .iter()
        .map(|name| {
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .w(px(120.0))
                        .text_size(theme.type_.mono.size)
                        .text_color(if *name == ctx.state {
                            theme.palette.accent_ink
                        } else {
                            theme.palette.ink_secondary
                        })
                        .child(*name),
                )
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(theme.palette.ink_faint)
                        .child(state_note(name)),
                )
                .into_any()
        })
        .collect();
    let mut children = vec![
        section_heading(theme, "Selected state"),
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .child(sample)
            .child(
                div()
                    .text_size(theme.type_.label.size)
                    .text_color(theme.palette.ink)
                    .child(format!("visual_state == {}", ctx.state)),
            )
            .into_any(),
        section_heading(theme, "§10 vocabulary"),
    ];
    children.extend(rows);
    story_page(ctx, "States", children)
}

/// §7 icon catalog × S/N/XL sizes.
pub fn render_iconography(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let sizes = [Density::Small, Density::Normal, Density::ExtraLarge];
    let rows: Vec<AnyElement> = all_icons()
        .into_iter()
        .map(|icon| {
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .w(px(140.0))
                        .text_size(theme.type_.mono.size)
                        .text_color(theme.palette.ink_secondary)
                        .child(icon.name()),
                )
                .children(sizes.iter().map(|d| {
                    div()
                        .size(px(28.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(icon.canvas(theme, d.icon()))
                }))
                .into_any()
        })
        .collect();
    let mut children = vec![section_heading(theme, "Every glyph × S / N / XL")];
    children.extend(rows);
    story_page(ctx, "Iconography", children)
}

/// §11.4 brand block: the chassis mark + product name (gallery-owned
/// presentation of the `bbr-ui` mark glyph).
pub fn render_brand(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    story_page(
        ctx,
        "Brand",
        vec![div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .child(Icon::ChassisMark.canvas(theme, px(28.0)))
            .child(
                div()
                    .text_size(theme.type_.title.size)
                    .font_weight(theme.type_.title.weight)
                    .text_color(theme.palette.ink)
                    .child("BBR"),
            )
            .child(
                div()
                    .text_size(theme.type_.meta.size)
                    .text_color(theme.palette.ink_faint)
                    .child("Chassis & Glass"),
            )
            .into_any()],
    )
}

/// Map a §10 state name to the `bbr-ui` `TabVisualState` the favicon tile
/// renders.
pub fn tab_state_from_name(name: &str) -> TabVisualState {
    match name {
        "hover" => TabVisualState::Hover,
        "active" => TabVisualState::Active,
        "selected" => TabVisualState::Selected,
        "loading" => TabVisualState::Loading,
        "attention" => TabVisualState::Attention,
        "error" => TabVisualState::Error,
        "disabled" => TabVisualState::Disabled,
        "dragging" => TabVisualState::Dragging,
        "drag-target" => TabVisualState::DragTarget,
        "focused" => TabVisualState::Focused,
        _ => TabVisualState::Default,
    }
}

/// The §10 non-color companion note for each state.
fn state_note(name: &str) -> &'static str {
    match name {
        "default" => "row wash transparent; text per role",
        "hover" => "inset rounded wash, close reveals",
        "active" => "wash deepens + 1 px inner hairline",
        "selected" => "tint + 3 pt accent rail at the leading edge",
        "loading" => "accent arc spinner replaces the favicon",
        "attention" => "amber dot + two-beat pulse + reason line",
        "error" => "lifering glyph + border.strong outline",
        "disabled" => "ink.faint, wash suppressed, reason tooltip",
        "dragging" => "92% opacity, elev:2 shadow, strong outline",
        "drag-target" => "2 pt accent tick + target outline + tint",
        "focused" => "accent ring + one text-step brighten",
        _ => "",
    }
}

/// Every §7 glyph in the catalog, in declaration order.
pub fn all_icons() -> Vec<Icon> {
    vec![
        Icon::Close,
        Icon::Pin { pinned: false },
        Icon::Pin { pinned: true },
        Icon::Lifering,
        Icon::ChassisMark,
        Icon::Plus,
        Icon::Downloads,
        Icon::Check,
        Icon::Stopped,
        Icon::Folder { open: false },
        Icon::Folder { open: true },
        Icon::Disclosure { expanded: false },
        Icon::Disclosure { expanded: true },
        Icon::LoadingArc,
        Icon::ChevronLeft,
        Icon::ChevronRight,
        Icon::Clock,
        Icon::Sidebar { expanded: true },
        Icon::Sidebar { expanded: false },
        Icon::NavBack,
        Icon::NavForward,
        Icon::Reload,
        Icon::Bookmark,
        Icon::Pencil,
    ]
}
