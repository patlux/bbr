//! The G3 Sidebar tab region (DESIGN.md §11.1/§11.2/§11.6): the pinned
//! block above the regular region's scroll view, the §11.11 folders list
//! between them, and the bounded dense window of the regular region.
//!
//! Rows are wired to the production pointer path through the [`Callbacks`]
//! slots, and an active drag renders the §10.9 insertion tick at its target
//! slot (muted when the drop would be illegal). The regular region renders
//! only the bounded window around the production scroll anchor
//! ([`TabsModel`]) plus spacers that keep the container's content height
//! equal to the full collection — the native scrollbar and wheel listener
//! behave as if every row were laid out, while a 500-tab list never lays
//! out 500 rows per frame. The pinned region renders all of its rows: it is
//! a small, stable block and must never shift on selection (§11.2).

use gpui::{div, prelude::*, px, Div, MouseButton, Stateful, Styled};

use crate::components::{self, RowVisuals};
use crate::theme::{Radius, Theme};

use super::callbacks;
use super::model::{Callbacks, SidebarModel};
use super::regions::folders_surface;

/// The controller-backed tab surface: the pinned region as a fixed block
/// above the regular region's scroll view (DESIGN.md §11.2/§11.6), with
/// the §11.11 folders list between the pinned block and the Tabs section.
/// Rows are wired to the production pointer path through the callback
/// slots, and an active drag renders the §10.9 insertion tick at its target
/// slot (muted when the drop would be illegal).
///
/// The regular region renders only the bounded window around the
/// production scroll anchor ([`TabsModel`]) plus spacers that keep the
/// container's content height equal to the full collection — the native
/// scrollbar and wheel listener behave as if every row were laid out, while
/// a 500-tab list never lays out 500 rows per frame. The pinned region
/// renders all of its rows: it is a small, stable block and must never
/// shift on selection (§11.2).
pub fn tab_surface(theme: &Theme, model: &SidebarModel, callbacks: &Callbacks) -> Stateful<Div> {
    let tabs = &model.tabs;
    let list_focused = tabs.list_focused;
    let p = theme.palette;
    let pinned_count = tabs.pinned.len();

    let mut children: Vec<gpui::AnyElement> = Vec::new();
    if !tabs.pinned.is_empty() {
        children.push(components::section_label(theme, "Pinned", list_focused).into_any());
        for row in &tabs.pinned {
            if tabs.tick == Some(row.index) {
                children.push(components::drag_tick(theme, row.index, !tabs.tick_valid).into_any());
            }
            children.push(render_row(theme, row, callbacks).into_any());
        }
        if tabs.tick == Some(pinned_count) {
            children.push(components::drag_tick(theme, pinned_count, !tabs.tick_valid).into_any());
        }
        children.push(
            div()
                .h(px(1.0))
                .w_full()
                .bg(theme.palette.hairline)
                .into_any(),
        );
    }
    // The §11.11 folders list sits between the pinned block and the Tabs
    // section (hidden entirely while no folder exists).
    if let Some(folders) = model.folders.as_ref() {
        children.push(folders_surface(theme, model, folders, callbacks).into_any());
    }
    if !tabs.window.is_empty() {
        children.push(components::section_label(theme, "Tabs", list_focused).into_any());
    }

    // The scroll container: spacers keep the total content height equal
    // to the full collection, the rendered window follows the production
    // scroll offset, and the drag tick renders only inside the window.
    let top = tabs.scroll_top;
    let bottom = tabs.scroll_bottom;
    let mut scroll_children: Vec<gpui::AnyElement> = Vec::new();
    if top > 0.0 {
        scroll_children.push(div().h(px(top)).into_any());
    }
    for row in &tabs.window {
        if tabs.tick == Some(row.index) {
            scroll_children
                .push(components::drag_tick(theme, row.index, !tabs.tick_valid).into_any());
        }
        scroll_children.push(render_row(theme, row, callbacks).into_any());
    }
    if bottom > 0.0 {
        scroll_children.push(div().h(px(bottom)).into_any());
    }
    children.push(
        div()
            .id("bbr-tab-scroll")
            .flex_1()
            .min_h_0()
            .min_w_0()
            .overflow_y_scroll()
            .track_scroll(&model.dense_scroll)
            .children(scroll_children)
            .into_any(),
    );

    div()
        .id("bbr-tab-list")
        .track_focus(&model.tab_list_handle)
        .flex_1()
        .flex()
        .flex_col()
        .min_w_0()
        .min_h_0()
        // §9 keyboard-focus ring: 2 pt accent outline inside the focused
        // list's bounds + the brightened section labels — focus is a
        // shape and a text step, never color-only.
        .when(list_focused, |el| {
            el.border_2()
                .border_color(p.accent)
                .rounded(Radius.medium())
        })
        .on_key_down(callbacks::key(&callbacks.on_tab_list_key_down))
        .children(children)
}

/// One §11.1 row: the production `tab_row` component wired to the callback
/// slots — select/close/pin/drag through the row listeners, the §10.2
/// hover wash + close reveal, and the §10.3 pressed wash. The close and
/// pin controls stop propagation in the production component (their
/// handlers run first in the bubble phase), so a press on a control never
/// selects the row or starts a drag.
fn render_row(theme: &Theme, row: &RowVisuals, callbacks: &Callbacks) -> Stateful<Div> {
    let index = row.index;
    components::tab_row(
        theme,
        row,
        callbacks::row(&callbacks.on_select_row, index),
        callbacks::row(&callbacks.on_close_row, index),
        callbacks::row(&callbacks.on_toggle_pin, index),
        callbacks::row(&callbacks.on_begin_drag, index),
    )
    .on_mouse_down(
        MouseButton::Left,
        callbacks::row(&callbacks.on_press_row, index),
    )
    .on_mouse_up(
        MouseButton::Left,
        callbacks::row_up(&callbacks.on_release_row, index),
    )
    .on_hover(callbacks::hover(&callbacks.on_hover_row, index))
}
