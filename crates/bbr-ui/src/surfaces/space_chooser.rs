//! Blueprint D13 (board PR-20, answer 6): with "Ask each time", a link from
//! another app opens a small palette that chooses its space — "Open in
//! Default", "Open in Work · Profile Work" — at the palette's place over
//! the card.

use std::rc::Rc;

use gpui::{div, prelude::*, px, App, MouseButton, SharedString, Window};

use crate::icons::Icon;
use crate::theme::Theme;

pub const SPACE_CHOOSER_ID: &str = "bbr-space-chooser";
pub const SPACE_CHOOSER_WIDTH: f32 = 480.0;

/// The chooser.
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceChooserModel {
    /// "Open example.com in…"
    pub title: SharedString,
    /// "Open in Work · Profile Work" per space.
    pub rows: Vec<SharedString>,
    pub selected: usize,
}

/// A press on row `index`.
pub type SpaceChooserCallback = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The chooser card (placed by the caller).
pub fn space_chooser(
    theme: &Theme,
    model: &SpaceChooserModel,
    on_choose: SpaceChooserCallback,
) -> gpui::Stateful<gpui::Div> {
    let p = theme.palette;
    let rows = model.rows.iter().enumerate().map(|(index, label)| {
        let selected = index == model.selected;
        let on_choose = on_choose.clone();
        div()
            .id(SharedString::from(format!("bbr-space-chooser-row-{index}")))
            .h(px(40.0))
            .px(px(12.0))
            .rounded(px(8.0))
            .flex()
            .items_center()
            .gap(px(10.0))
            .when(selected, |el| el.bg(p.tint_accent))
            .hover(|el| el.bg(p.wash_hover))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                cx.stop_propagation();
                on_choose(index, window, cx);
            })
            .child(Icon::Bbr(bbr_domain::BbrGlyph::Globe).canvas(theme, px(16.0)))
            .child(
                div()
                    .flex_1()
                    .text_size(theme.type_.title.size)
                    .text_color(p.ink)
                    .child(label.clone()),
            )
            .child(
                div()
                    .text_size(theme.type_.meta.size)
                    .text_color(p.ink_faint)
                    .child("Space"),
            )
    });
    div()
        .id(SPACE_CHOOSER_ID)
        .w(px(SPACE_CHOOSER_WIDTH))
        .p(px(8.0))
        .rounded(px(14.0))
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .shadow(crate::theme::elevation::elev2(theme))
        .flex()
        .flex_col()
        .gap(px(2.0))
        .on_mouse_down(MouseButton::Left, |_e, _w, cx| cx.stop_propagation())
        .child(
            div()
                .px(px(12.0))
                .pt(px(6.0))
                .pb(px(8.0))
                .text_size(theme.type_.meta.size)
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(p.ink_faint)
                .child(model.title.clone()),
        )
        .children(rows)
}
