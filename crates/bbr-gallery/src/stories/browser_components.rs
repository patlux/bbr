//! The G2 browser-component stories (packet
//! `component-gallery-browser-components.md`): real `bbr-ui` components
//! with plain fixture models. The first cut is the §11.4 status line —
//! the production `bbr_ui::components::status_line` with fixture-built
//! `StatusModel` text (the §11.8 derivation branches as plain text, never
//! a controller snapshot). No gallery-local reimplementation of anatomy,
//! no copied tokens, no fake engine/controller/portal.

use bbr_ui::components::{status_line, STATUS_ID};
use gpui::{div, prelude::*, AnyElement};

use crate::fixtures::{status_model, STATUS_VARIANTS};
use crate::registry::StoryContext;
use crate::stories::{section_heading, story_page, value_row};

/// §11.4 status line: the production `bbr-ui` component with fixture
/// models — the §11.8 derivation branches (empty, loading, title, URL
/// fallback, crash) as plain text. The stable §15 id rides the rendered
/// component; the story page shows the contract.
pub fn render_status_line(ctx: &StoryContext) -> AnyElement {
    let theme = ctx.theme;
    let rows: Vec<AnyElement> = STATUS_VARIANTS
        .iter()
        .map(|text| {
            let model = status_model(text);
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(status_line(theme, &model))
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(theme.palette.ink_faint)
                        .child(format!("{text:?}")),
                )
                .into_any()
        })
        .collect();
    let mut children = vec![
        section_heading(theme, "§11.8 derivation branches (fixture text)"),
        value_row(theme, "id", STATUS_ID),
        value_row(theme, "type", "type:meta · ink_faint"),
    ];
    children.extend(rows);
    story_page(theme, "Status line", children)
}
