//! Blueprint D10 (boards M-6 M-7 PR-22, DESIGN §11.16): the History page
//! and the Clear Browsing Data sheet.
//!
//! History takes the content card like the Saved Pages Library: a side
//! column (title, All history, Recently closed with its count) and the list
//! — a search field, the Today / Week / All range, Clear Browsing Data…,
//! then day groups of rows `[time] [monogram] [title] [url mono]`. The
//! sheet names the profile whose website data it clears, the time range and
//! what goes, with Cancel and the destructive Clear Data.

use std::rc::Rc;

use gpui::{div, prelude::*, px, AnyElement, App, MouseButton, SharedString, Window};

use crate::icons::Icon;
use crate::palette::FaviconSlate;
use crate::theme::{Theme, ThemeMode};

mod ids;
mod sheet;
pub use ids::history_visible_ids;
use sheet::clear_sheet;

/// The §15 ids.
pub const HISTORY_ID: &str = "bbr-history";
pub const HISTORY_SEARCH_ID: &str = "bbr-history-search";
pub const HISTORY_CLEAR_ID: &str = "bbr-history-clear";
pub const CLEAR_DATA_ID: &str = "bbr-clear-data";

/// The stable id of a visit row (`closed` rows use the tab id).
pub fn history_row_id(id: u64, closed: bool) -> String {
    if closed {
        format!("bbr-history-closed-{id}")
    } else {
        format!("bbr-history-row-{id}")
    }
}

/// One visit (or recently closed tab) row.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryRow {
    pub id: u64,
    pub time: Option<SharedString>,
    pub title: SharedString,
    pub url: SharedString,
    pub letter: char,
    pub hue: usize,
}

/// A day group.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryGroup {
    pub title: SharedString,
    pub rows: Vec<HistoryRow>,
}

/// One Clear Browsing Data option.
#[derive(Debug, Clone, PartialEq)]
pub struct ClearOption {
    pub label: SharedString,
    pub checked: bool,
    pub enabled: bool,
}

/// The Clear Browsing Data sheet.
#[derive(Debug, Clone, PartialEq)]
pub struct ClearDataModel {
    pub profile: SharedString,
    pub profile_letter: char,
    pub profile_hue: gpui::Rgba,
    pub range: usize,
    pub ranges: Vec<SharedString>,
    pub options: Vec<ClearOption>,
    pub range_open: bool,
    pub confirm_enabled: bool,
}

/// The whole page.
#[derive(Debug, Clone)]
pub struct HistoryModel {
    pub theme: Theme,
    pub search: SharedString,
    /// 0 Today, 1 Week, 2 All.
    pub range: usize,
    /// 0 All history, 1 Recently closed.
    pub view: usize,
    pub closed_count: usize,
    pub groups: Vec<HistoryGroup>,
    /// The keyboard row (its id).
    pub selected: Option<u64>,
    /// Shown instead of the list when there is nothing to list.
    pub empty: Option<SharedString>,
    /// Shown under a list cut at the row limit.
    pub more: Option<SharedString>,
    pub focused: bool,
    pub clear: Option<ClearDataModel>,
}

/// Every interaction.
#[derive(Debug, Clone, PartialEq)]
pub enum HistoryEvent {
    View(usize),
    Range(usize),
    Open(u64),
    Remove(u64),
    ClearOpen,
    ClearRangeMenu,
    ClearRange(usize),
    ClearToggle(usize),
    ClearCancel,
    ClearConfirm,
}

pub type HistoryCallback = Rc<dyn Fn(HistoryEvent, &mut Window, &mut App)>;

pub(super) fn press(
    events: &HistoryCallback,
    event: HistoryEvent,
) -> impl Fn(&gpui::MouseDownEvent, &mut Window, &mut App) + 'static {
    let events = events.clone();
    move |_e, window, cx| {
        cx.stop_propagation();
        events(event.clone(), window, cx);
    }
}

fn nav(
    model: &HistoryModel,
    view: usize,
    icon: Icon,
    label: &str,
    count: Option<usize>,
    events: &HistoryCallback,
) -> AnyElement {
    let theme = &model.theme;
    let id = if view == 0 {
        "bbr-history-all"
    } else {
        "bbr-history-closed"
    };
    let selected = model.view == view;
    let event = HistoryEvent::View(view);
    let p = theme.palette;
    div()
        .id(SharedString::from(id.to_string()))
        .h(px(32.0))
        .px(px(10.0))
        .rounded(px(8.0))
        .flex()
        .items_center()
        .gap(px(8.0))
        .when(selected, |el| el.bg(p.wash_active))
        .hover(|el| el.bg(p.wash_hover))
        .text_size(theme.type_.label.size)
        .text_color(p.ink)
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, press(events, event))
        .child(icon.canvas(theme, px(14.0)))
        .child(div().flex_1().child(label.to_string()))
        .when_some(count, |el, count| {
            el.child(
                div()
                    .text_size(theme.type_.meta.size)
                    .text_color(p.ink_faint)
                    .child(count.to_string()),
            )
        })
        .into_any_element()
}

fn side_column(model: &HistoryModel, events: &HistoryCallback) -> impl IntoElement {
    let theme = &model.theme;
    let p = theme.palette;
    div()
        .w(px(220.0))
        .h_full()
        .flex_shrink_0()
        .p(px(12.0))
        .border_r_1()
        .border_color(p.hairline)
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .px(px(10.0))
                .pt(px(6.0))
                .pb(px(12.0))
                .child(
                    div()
                        .text_size(theme.type_.title.size)
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(p.ink)
                        .child("History"),
                )
                .child(
                    div()
                        .text_size(theme.type_.meta.size)
                        .text_color(p.ink_faint)
                        .child("stays on this Mac"),
                ),
        )
        .child(nav(model, 0, Icon::Clock, "All history", None, events))
        .child(nav(
            model,
            1,
            Icon::Close,
            "Recently closed",
            Some(model.closed_count),
            events,
        ))
}

fn header(model: &HistoryModel, events: &HistoryCallback) -> impl IntoElement {
    let theme = &model.theme;
    let p = theme.palette;
    let ranges = ["Today", "Week", "All"];
    div()
        .h(px(60.0))
        .flex_shrink_0()
        .px(px(20.0))
        .flex()
        .items_center()
        .gap(px(12.0))
        .border_b_1()
        .border_color(p.hairline)
        .child(
            div()
                .id(HISTORY_SEARCH_ID)
                .w(px(320.0))
                .h(px(32.0))
                .px(px(12.0))
                .rounded(px(16.0))
                .border_1()
                .border_color(if model.focused { p.accent } else { p.hairline })
                .bg(p.surface_field)
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(Icon::Search.canvas(theme, px(14.0)))
                .child(
                    div()
                        .text_size(theme.type_.label.size)
                        .text_color(if model.search.is_empty() {
                            p.ink_faint
                        } else {
                            p.ink
                        })
                        .child(if model.search.is_empty() {
                            SharedString::from("Search history")
                        } else {
                            model.search.clone()
                        }),
                ),
        )
        .child(div().flex_1())
        .when(model.view == 0, |el| {
            el.child(
                div()
                    .p(px(2.0))
                    .rounded(px(8.0))
                    .bg(p.wash_hover)
                    .flex()
                    .children(ranges.iter().enumerate().map(|(index, label)| {
                        let selected = index == model.range;
                        div()
                            .id(SharedString::from(format!("bbr-history-range-{index}")))
                            .px(px(14.0))
                            .py(px(3.0))
                            .rounded(px(6.0))
                            .text_size(theme.type_.label.size)
                            .text_color(if selected { p.ink } else { p.ink_secondary })
                            .when(selected, |el| {
                                el.bg(p.surface_field)
                                    .shadow(crate::theme::elevation::lift(theme))
                            })
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                press(events, HistoryEvent::Range(index)),
                            )
                            .child(label.to_string())
                    })),
            )
        })
        .child(
            div()
                .id(HISTORY_CLEAR_ID)
                .h(px(28.0))
                .px(px(14.0))
                .rounded(px(8.0))
                .border_1()
                .border_color(p.hairline)
                .bg(p.surface_field)
                .flex()
                .items_center()
                .text_size(theme.type_.label.size)
                .text_color(p.ink)
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, press(events, HistoryEvent::ClearOpen))
                .child("Clear Browsing Data…"),
        )
}

fn row(
    theme: &Theme,
    row: &HistoryRow,
    closed: bool,
    selected: bool,
    events: &HistoryCallback,
) -> AnyElement {
    let p = theme.palette;
    let tile = match theme.mode {
        ThemeMode::Dark => FaviconSlate::dark(row.hue),
        ThemeMode::Light => FaviconSlate::light(row.hue),
    };
    div()
        .id(SharedString::from(history_row_id(row.id, closed)))
        .h(px(40.0))
        .flex_shrink_0()
        .px(px(12.0))
        .rounded(px(8.0))
        .flex()
        .items_center()
        .gap(px(12.0))
        .when(selected, |el| el.bg(p.wash_active))
        .hover(|el| el.bg(p.wash_hover))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, press(events, HistoryEvent::Open(row.id)))
        .when(!closed, |el| {
            el.child(
                div()
                    .w(px(40.0))
                    .flex_shrink_0()
                    .text_size(theme.type_.meta.size)
                    .text_color(p.ink_faint)
                    .child(row.time.clone().unwrap_or_default()),
            )
        })
        .child(
            div()
                .size(px(18.0))
                .flex_shrink_0()
                .rounded(px(4.0))
                .bg(tile)
                .flex()
                .items_center()
                .justify_center()
                .text_size(theme.type_.badge.size)
                .text_color(FaviconSlate::letter(theme.mode))
                .child(row.letter.to_string()),
        )
        .child(
            div()
                .max_w(px(420.0))
                .flex_shrink_0()
                .overflow_hidden()
                .text_ellipsis()
                .whitespace_nowrap()
                .text_size(theme.type_.label.size)
                .text_color(p.ink)
                .child(row.title.clone()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .text_ellipsis()
                .whitespace_nowrap()
                .text_size(theme.type_.mono.size)
                .text_color(p.ink_faint)
                .child(row.url.clone()),
        )
        .when(selected && !closed, |el| {
            el.child(
                div()
                    .id(SharedString::from(format!("bbr-history-remove-{}", row.id)))
                    .size(px(22.0))
                    .rounded(px(6.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .hover(|el| el.bg(p.wash_hover))
                    .on_mouse_down(
                        MouseButton::Left,
                        press(events, HistoryEvent::Remove(row.id)),
                    )
                    .child(Icon::Close.canvas(theme, px(12.0))),
            )
        })
        .into_any_element()
}

/// The History page for the content card.
pub fn history_surface(model: &HistoryModel, events: HistoryCallback) -> impl IntoElement {
    let theme = &model.theme;
    let p = theme.palette;
    let closed = model.view == 1;
    let mut items: Vec<AnyElement> = Vec::new();
    if let Some(empty) = &model.empty {
        items.push(
            div()
                .id("bbr-history-empty")
                .pt(px(48.0))
                .flex()
                .justify_center()
                .text_size(theme.type_.label.size)
                .text_color(p.ink_faint)
                .child(empty.clone())
                .into_any_element(),
        );
    }
    for group in &model.groups {
        items.push(
            div()
                .px(px(12.0))
                .pt(px(12.0))
                .pb(px(6.0))
                .text_size(theme.type_.badge.size)
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(p.ink_faint)
                .child(SharedString::from(group.title.to_uppercase()))
                .into_any_element(),
        );
        items.extend(
            group
                .rows
                .iter()
                .map(|r| row(theme, r, closed, model.selected == Some(r.id), &events)),
        );
    }
    if let Some(more) = &model.more {
        items.push(
            div()
                .px(px(12.0))
                .py(px(12.0))
                .text_size(theme.type_.meta.size)
                .text_color(p.ink_faint)
                .child(more.clone())
                .into_any_element(),
        );
    }
    let list = div()
        .id("bbr-history-list")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .px(px(12.0))
        .pb(px(12.0))
        .flex()
        .flex_col()
        .gap(px(2.0))
        .children(items);
    let sheet = model
        .clear
        .as_ref()
        .map(|clear| clear_sheet(theme, clear, &events));
    div()
        .id(HISTORY_ID)
        .relative()
        .size_full()
        .flex()
        .bg(p.surface_canvas)
        .child(side_column(model, &events))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .child(header(model, &events))
                .child(list),
        )
        .children(sheet)
}
