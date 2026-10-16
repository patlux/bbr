//! Downloads panel surface render (DESIGN.md §11.12, Paper M-4): the
//! anchored popover composition — the `bbr-downloads-panel` surface div,
//! the `type:badge` group labels, the bounded scroll window (visible range +
//! §11.6 overscan; the full history is never laid out), and the per-row
//! anatomy `[status glyph] [name / progress / secondary] [action]`. The
//! panel is a flex child of the sidebar column placed directly above the
//! footer row, so it never overlaps the native web view and never uses the
//! NativeAboveWeb Portal host. Every height comes from
//! [`super::model::DownloadsMetrics`] and [`super::model::PanelLayout`], the
//! same authority the scroll math uses.
//!
//! This module owns the anatomy only: the machine state and the offered
//! actions arrive as [`super::model::DownloadRowModel`] DTOs built by
//! `bbr-app`'s controller, and every interactive path routes through the
//! [`super::callbacks::DownloadsCallbacks`] bundle.

use gpui::{div, prelude::*, px, Div, MouseButton, MouseDownEvent, Stateful, Styled, Window};

use crate::icons::Icon;
use crate::state::DownloadVisual;
use crate::theme::elevation::elev2;
use crate::theme::Theme;

use super::callbacks::DownloadsCallbacks;
use super::ids::{
    download_row_id, download_slot_id, DOWNLOADS_ACTIVE_ID, DOWNLOADS_EMPTY_ID,
    DOWNLOADS_HISTORY_ID, DOWNLOADS_PANEL_ID,
};
use super::model::{
    DownloadGroupKind, DownloadMachine, DownloadRowModel, DownloadsMetrics, DownloadsPanelModel,
    PanelItem, PanelLayout,
};
use super::row_detail::{
    downloads_action, downloads_clear_control, downloads_delete_control, downloads_row_text,
};

/// Paper M-4: the empty hint's band height inside the panel padding.
const EMPTY_HINT_H: f32 = 120.0;
/// Paper M-4 row insets: content-inset leading, 8 pt trailing, 10 pt gap.
const ROW_PAD_LEADING: f32 = 10.0;
const ROW_PAD_TRAILING: f32 = 8.0;
const ROW_GAP: f32 = 10.0;
/// Paper M-4 status glyphs inside the 18 pt slot: the spinner and the
/// neutral cancelled mark at 14, the check and the error mark at 16.
const STATUS_GLYPH_SMALL: f32 = 14.0;
const STATUS_GLYPH: f32 = 16.0;

/// The §11.12 anchored popover surface. `None` while closed and the
/// `dur:fine` fade-out has completed (the app owns the transition
/// lifecycle and passes the evaluated `alpha` in the model). The panel
/// owns the keyboard cursor while open (the app's `on_key_down` callback),
/// marks pointer presses inside so the root outside-press handler never
/// dismisses it (§13 one pointer action), and carries the §9 focus ring
/// while it owns chrome focus.
pub fn downloads_panel_surface(
    theme: &Theme,
    model: &DownloadsPanelModel,
    callbacks: &DownloadsCallbacks,
) -> Option<Stateful<Div>> {
    // §11.12 open/close: opacity fade `dur:fine` only — never translate or
    // scale. While the fade-out runs the panel stays mounted; at zero it
    // unmounts. The transition itself is app lifecycle (it needs the
    // window state at creation); the model carries the evaluated value.
    if !model.open && model.alpha <= 0.001 {
        return None;
    }

    let p = theme.palette;
    let layout = &model.layout;
    let metrics = DownloadsMetrics::for_theme(theme);
    let total = model.rows.len();

    // The §11.12 6-row bound: at most `DOWNLOADS_MAX_VISIBLE` rows are
    // visible; beyond that the viewport is six History rows tall (labels
    // scroll with the content) and the full history is never laid out (the
    // render window is the visible range plus the §11.6 8-row overscan).
    let viewport_h = layout.viewport_height();

    // The scroll container's flow children in paint order (the layout
    // authority): the top spacer (everything above the rendered window),
    // the window's labels/hairline/rows, and the bottom spacer. The content
    // height equals the full content, so the native scrollbar and wheel
    // behave as if every row were laid out while only the bounded window is.
    let mut scroll_children: Vec<gpui::AnyElement> = Vec::new();
    for item in layout.items(&model.window) {
        let element = match item {
            PanelItem::Spacer(height) => div().h(px(height)).into_any(),
            PanelItem::ActiveLabel => downloads_group_label(theme, layout, true).into_any(),
            PanelItem::HistoryLabel => downloads_group_label(theme, layout, false).into_any(),
            PanelItem::Hairline => downloads_group_hairline(theme, layout).into_any(),
            PanelItem::Row(index) => {
                let Some(visual) = model.rows.iter().find(|row| row.index == index) else {
                    continue;
                };
                let check = model
                    .check
                    .filter(|(id, _)| *id == visual.id)
                    .map(|(_, key)| key);
                downloads_row(theme, layout, visual, check, callbacks).into_any()
            }
        };
        scroll_children.push(element);
    }

    let on_key_down = callbacks.on_key_down.clone();
    let on_inside_press = callbacks.on_inside_press.clone();
    let panel = div()
        .id(DOWNLOADS_PANEL_ID)
        .debug_selector(|| DOWNLOADS_PANEL_ID.to_string())
        .track_focus(model.focus)
        // Paper M-4: inset `density:row-inset` from the rail's leading edge,
        // so the rail − 2 × inset width sits centred on the visible rail.
        .ml(theme.density.row_inset())
        .w(px(model.width))
        .opacity(model.alpha)
        .rounded(theme.radius.large())
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .shadow(elev2(theme))
        .overflow_hidden()
        .p(px(metrics.panel_pad))
        .flex()
        .flex_col()
        // §9 ring while the panel owns chrome focus (the same structural
        // cue as the tab/space lists; resting selected rows stay quiet).
        // The padding gives back the ring's extra point so focus never
        // moves the content.
        .when(model.focused, |el| {
            el.border_2()
                .border_color(p.accent)
                .rounded(theme.radius.large())
                .p(px(metrics.panel_pad - 1.0))
        })
        .on_key_down(
            move |event: &gpui::KeyDownEvent, window: &mut Window, cx: &mut gpui::App| {
                on_key_down(event, window, cx)
            },
        )
        // §13: the inside-press marker runs deepest in the bubble phase
        // and stops propagation, so a press inside the panel never
        // dismisses it and the sidebar chrome-focus handler never steals
        // focus (§11.12: the panel is chrome focus while open).
        .on_mouse_down(
            MouseButton::Left,
            move |e: &MouseDownEvent, window: &mut Window, cx: &mut gpui::App| {
                on_inside_press(e, window, cx)
            },
        )
        .children(crate::rect_probe::rect_probe(DOWNLOADS_PANEL_ID));

    // The panel content: the empty hint or the bounded scroll window.
    let panel = if total == 0 {
        panel.child(
            div()
                .id(DOWNLOADS_EMPTY_ID)
                .debug_selector(|| DOWNLOADS_EMPTY_ID.to_string())
                .h(px(EMPTY_HINT_H))
                .flex()
                .items_center()
                .justify_center()
                .text_size(theme.type_.label.size)
                .font_weight(theme.type_.label.weight)
                .line_height(px(metrics.line))
                .text_color(p.ink_secondary)
                .child("No downloads yet"),
        )
    } else {
        panel.child(
            div()
                .id("bbr-downloads-scroll")
                .h(px(viewport_h))
                .w_full()
                .overflow_y_scroll()
                .track_scroll(model.scroll)
                .children(scroll_children),
        )
    };
    // The settled Clear panel action (§17.7): while any history exists,
    // the explicit `bbr-downloads-clear` control renders below the scroll
    // area and empties the controller history + repository durably. It is
    // the same production pointer path the testkit `ClickDownloadsClear`
    // request drives.
    let panel = if layout.has_history() {
        panel.child(downloads_clear_control(theme, callbacks))
    } else {
        panel
    };
    Some(panel)
}

/// One §11.12 `type:badge` group label ("Active"/"History"), hidden by
/// the caller when the group is empty. Its height is the layout's
/// `label_h`, the exact value the scroll math uses.
pub fn downloads_group_label(theme: &Theme, layout: &PanelLayout, active: bool) -> Stateful<Div> {
    let p = theme.palette;
    let metrics = DownloadsMetrics::for_theme(theme);
    let id = if active {
        DOWNLOADS_ACTIVE_ID
    } else {
        DOWNLOADS_HISTORY_ID
    };
    div()
        .id(id)
        .debug_selector(move || id.to_string())
        .h(px(layout.label_h))
        .px(px(8.0))
        .pt(px(4.0))
        .flex()
        .items_start()
        .text_size(theme.type_.badge.size)
        .font_weight(theme.type_.badge.weight)
        .line_height(px(metrics.label_line))
        .text_color(p.ink_faint)
        .child(if active { "ACTIVE" } else { "HISTORY" })
}

/// The §11.12 group separator: a 1 px hairline centred in its band and
/// inset like the labels.
pub fn downloads_group_hairline(theme: &Theme, layout: &PanelLayout) -> Div {
    div()
        .h(px(layout.hairline))
        .w_full()
        .px(px(8.0))
        .flex()
        .items_center()
        .child(div().h(px(1.0)).flex_1().bg(theme.palette.hairline))
}

/// One §11.12 download row: `[status glyph] [name / progress / secondary]
/// [action]` at the layout's row height, the §10.2/§10.3 inset hover/press
/// washes, the deterministic `bbr-download-*` ids, and the production
/// pointer paths (hover/press state, action activation) routed through the
/// callbacks bundle.
pub fn downloads_row(
    theme: &Theme,
    layout: &PanelLayout,
    visual: &DownloadRowModel,
    check: Option<u64>,
    callbacks: &DownloadsCallbacks,
) -> Stateful<Div> {
    let p = theme.palette;
    let density = theme.density;
    let index = visual.index;
    let status = match visual.machine {
        DownloadMachine::Waiting | DownloadMachine::Running => {
            // §10.5 spinner arc (accent, transient indication — allowed).
            Icon::LoadingArc
                .canvas(theme, px(STATUS_GLYPH_SMALL))
                .into_any()
        }
        DownloadMachine::Succeeded => {
            // §3.3 `status.ok` check with its non-color companion (the
            // status glyph is always next to the name line); right after
            // the success commits it draws itself once (§18.5).
            super::check_moment::succeeded_glyph(theme, px(STATUS_GLYPH), check)
        }
        DownloadMachine::Failed => Icon::Lifering.canvas(theme, px(STATUS_GLYPH)).into_any(),
        DownloadMachine::Cancelled => {
            // Neutral marker at `ink.secondary`.
            Icon::Stopped
                .canvas(theme, px(STATUS_GLYPH_SMALL))
                .into_any()
        }
    };
    let on_row_action = callbacks.on_row_action.clone();
    let activate = move |_e: &MouseDownEvent, window: &mut Window, cx: &mut gpui::App| {
        on_row_action(index, window, cx);
        cx.stop_propagation();
    };
    let on_row_press = callbacks.on_row_press.clone();
    let on_row_press_release = on_row_press.clone();
    let press = move |_e: &MouseDownEvent, window: &mut Window, cx: &mut gpui::App| {
        on_row_press(index, true, window, cx);
    };
    let release = move |_e: &gpui::MouseUpEvent, window: &mut Window, cx: &mut gpui::App| {
        on_row_press_release(index, false, window, cx);
    };
    let on_row_hover = callbacks.on_row_hover.clone();
    let hover = move |hovered: &bool, window: &mut Window, cx: &mut gpui::App| {
        on_row_hover(index, *hovered, window, cx);
    };

    // The row's group decides its height (Active rows carry the bar); in
    // the production list this equals `layout.row_h(index)`.
    let row_h = match visual.group {
        DownloadGroupKind::Active => layout.active_row_h,
        DownloadGroupKind::History => layout.history_row_h,
    };
    div()
        .id(download_row_id(index))
        .debug_selector(move || download_row_id(index))
        .h(px(row_h))
        .w_full()
        .rounded(density.density_radius())
        .pl(px(ROW_PAD_LEADING))
        .pr(px(ROW_PAD_TRAILING))
        .gap(px(ROW_GAP))
        .items_center()
        .flex()
        .flex_row()
        .min_w_0()
        .when(visual.visual == DownloadVisual::Hover, |el| {
            el.bg(p.wash_hover)
        })
        .when(visual.visual == DownloadVisual::Pressed, |el| {
            el.bg(p.wash_active)
        })
        .when(visual.visual == DownloadVisual::Focused, |el| {
            // The keyboard cursor ring takes its 2 pt from the row insets,
            // so the row content never shifts under focus.
            el.border_2()
                .border_color(p.accent)
                .rounded(density.density_radius())
                .pl(px(ROW_PAD_LEADING - 2.0))
                .pr(px(ROW_PAD_TRAILING - 2.0))
        })
        .child(
            div()
                .id(download_slot_id("status", index))
                .debug_selector(move || download_slot_id("status", index))
                .flex_shrink_0()
                .size(density.icon())
                .flex()
                .items_center()
                .justify_center()
                .child(status),
        )
        .child(downloads_row_text(theme, visual))
        .child(downloads_action(theme, visual, index, activate))
        .children(downloads_delete_control(theme, visual, callbacks))
        .on_mouse_down(MouseButton::Left, press)
        .on_mouse_up(MouseButton::Left, release)
        .on_hover(hover)
}
