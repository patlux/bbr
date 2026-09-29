//! Blueprint D23 (board G-10, DESIGN §11.7): a split group that no longer
//! fits the card rests on its focused pane with an "n of N" chip at the
//! card's top-right corner (12 pt in); the chip opens a list of the panes,
//! and picking one focuses it. The app draws both in the Portal (they sit
//! over the page).

use std::rc::Rc;

use gpui::{div, prelude::*, px, App, MouseButton, SharedString, Stateful, Window};

use crate::icons::Icon;
use crate::theme::{Radius, Theme};

pub const SPLIT_CHIP_ID: &str = "bbr-split-chip";
pub const SPLIT_LIST_ID: &str = "bbr-split-list";
/// The chip's inset from the card's top and trailing edges.
pub const SPLIT_CHIP_INSET: f32 = 12.0;
pub const SPLIT_CHIP_HEIGHT: f32 = 26.0;
pub const SPLIT_LIST_WIDTH: f32 = 240.0;

/// The chip and, while open, the list.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitChipModel {
    /// The focused pane's place (1-based) and the pane count.
    pub at: usize,
    pub total: usize,
    /// The panes' titles in reading order while the list is open.
    pub list: Option<Vec<SharedString>>,
}

/// A press on the chip (`None`) or a list row (`Some(index)`).
pub type SplitChipCallback = Rc<dyn Fn(Option<usize>, &mut Window, &mut App)>;

/// The chip (placed by the caller at the card's top-right).
pub fn split_chip(
    theme: &Theme,
    model: &SplitChipModel,
    on_press: SplitChipCallback,
) -> Stateful<gpui::Div> {
    let p = theme.palette;
    let press = on_press.clone();
    div()
        .id(SPLIT_CHIP_ID)
        .h(px(SPLIT_CHIP_HEIGHT))
        .px(px(10.0))
        .flex()
        .items_center()
        .gap(px(6.0))
        .rounded(Radius.full())
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .shadow(crate::theme::elevation::elev2(theme))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
            cx.stop_propagation();
            press(None, window, cx);
        })
        .child(Icon::Split.canvas(theme, px(14.0)))
        .child(
            div()
                .text_size(theme.type_.key.size)
                .text_color(p.ink)
                .child(format!("{} of {}", model.at, model.total)),
        )
}

/// The pane list (placed by the caller under the chip).
pub fn split_list(
    theme: &Theme,
    model: &SplitChipModel,
    on_press: SplitChipCallback,
) -> Option<Stateful<gpui::Div>> {
    let titles = model.list.as_ref()?;
    let p = theme.palette;
    let rows = titles.iter().enumerate().map(|(index, title)| {
        let press = on_press.clone();
        let current = index + 1 == model.at;
        div()
            .id(("bbr-split-list-row", index))
            .h(px(30.0))
            .px(px(10.0))
            .flex()
            .items_center()
            .gap(px(8.0))
            .rounded(Radius.small())
            .when(current, |row| row.bg(p.tint_accent))
            .hover(|row| row.bg(p.wash_hover))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, move |_e, window, cx| {
                cx.stop_propagation();
                press(Some(index), window, cx);
            })
            .child(
                div()
                    .w(px(16.0))
                    .text_size(theme.type_.key.size)
                    .text_color(p.ink_faint)
                    .child(format!("{}", index + 1)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(theme.type_.label.size)
                    .text_color(p.ink)
                    .child(title.clone()),
            )
    });
    Some(
        div()
            .id(SPLIT_LIST_ID)
            .w(px(SPLIT_LIST_WIDTH))
            .p(px(4.0))
            .rounded(Radius.medium())
            .bg(p.surface_overlay)
            .border_1()
            .border_color(p.hairline)
            .shadow(crate::theme::elevation::elev2(theme))
            .flex()
            .flex_col()
            .children(rows),
    )
}
