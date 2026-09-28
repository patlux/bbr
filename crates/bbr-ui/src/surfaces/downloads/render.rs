//! Downloads panel surface render (DESIGN.md §11.12, G2 packet
//! `component-gallery-browser-components.md`, Group 5): the anchored
//! popover composition — the `bbr-downloads-panel` surface div, the
//! `type:badge` group labels, the bounded scroll window (visible range +
//! §11.6 overscan; the full history is never laid out), and the per-row
//! anatomy `[status glyph] [name + secondary] [action]`. The panel is a
//! flex child of the sidebar column placed directly above the footer row,
//! so it never overlaps the native web view and never uses the
//! NativeAboveWeb Portal host. All geometry derives from the settled S1
//! tokens — no invented sizes.
//!
//! This module owns the anatomy only: the machine state and the offered
//! actions arrive as [`super::model::DownloadRowModel`] DTOs built by
//! `bbr-app`'s controller, and every interactive path routes through the
//! [`super::callbacks::DownloadsCallbacks`] bundle.

use gpui::{div, prelude::*, px, Div, MouseButton, MouseDownEvent, Stateful, Styled, Window};

use crate::icons::Icon;
use crate::state::DownloadVisual;
use crate::theme::elevation::elev2;
use crate::theme::{Spacing, Theme};

use super::callbacks::DownloadsCallbacks;
use super::ids::{
    download_row_id, download_slot_id, DOWNLOADS_ACTIVE_ID, DOWNLOADS_EMPTY_ID,
    DOWNLOADS_HISTORY_ID, DOWNLOADS_PANEL_ID,
};
use super::model::{DownloadMachine, DownloadRowModel, DownloadsPanelModel, DOWNLOADS_MAX_VISIBLE};
use super::row_detail::{
    downloads_action, downloads_clear_control, downloads_delete_control, downloads_secondary_line,
};

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
    let total = model.rows.len();

    // The §11.12 6-row bound: at most `DOWNLOADS_MAX_VISIBLE` rows are
    // visible; beyond that the scroll viewport is exactly 6 rows tall
    // (labels scroll with the content) and the full history is never laid
    // out (the render window is the visible range plus the §11.6 8-row
    // overscan).
    let content_h = layout.content_height();
    let viewport_h = if total > DOWNLOADS_MAX_VISIBLE {
        DOWNLOADS_MAX_VISIBLE as f32 * layout.row_h
    } else {
        content_h
    };

    // The scroll container's flow children: the top spacer (everything
    // above the rendered window), the window's own label/hairline/row
    // items, and the bottom spacer. The content height equals the full
    // content, so the native scrollbar and wheel behave as if every row
    // were laid out while only the bounded window is.
    let window_items = layout.items_in_window(&model.window);
    let top = layout.top_spacer(&model.window);
    let bottom = layout.bottom_spacer(&model.window);
    let mut scroll_children: Vec<gpui::AnyElement> = Vec::new();
    if top > 0.0 {
        scroll_children.push(div().h(px(top)).into_any());
    }
    if window_items.active_label {
        scroll_children.push(downloads_group_label(theme, true).into_any());
    }
    for visual in model
        .rows
        .iter()
        .filter(|row| model.window.contains(&row.index))
    {
        let check = model
            .check
            .filter(|(id, _)| *id == visual.id)
            .map(|(_, key)| key);
        scroll_children.push(downloads_row(theme, visual, check, callbacks).into_any());
    }
    if window_items.hairline {
        scroll_children.push(
            div()
                .h(px(layout.hairline))
                .w_full()
                .bg(p.hairline)
                .into_any(),
        );
    }
    if window_items.history_label {
        scroll_children.push(downloads_group_label(theme, false).into_any());
    }
    if bottom > 0.0 {
        scroll_children.push(div().h(px(bottom)).into_any());
    }

    let on_key_down = callbacks.on_key_down.clone();
    let on_inside_press = callbacks.on_inside_press.clone();
    let panel = div()
        .id(DOWNLOADS_PANEL_ID)
        .debug_selector(|| DOWNLOADS_PANEL_ID.to_string())
        .track_focus(model.focus)
        .w(px(model.width))
        .opacity(model.alpha)
        .rounded(theme.radius.large())
        .bg(p.surface_overlay)
        .border_1()
        .border_color(p.hairline)
        .shadow(elev2(theme))
        .overflow_hidden()
        // §9 ring while the panel owns chrome focus (the same structural
        // cue as the tab/space lists; resting selected rows stay quiet).
        .when(model.focused, |el| {
            el.border_2()
                .border_color(p.accent)
                .rounded(theme.radius.large())
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
        );

    // The panel content: the empty hint or the bounded scroll window.
    let panel = if total == 0 {
        panel.child(
            div()
                .id(DOWNLOADS_EMPTY_ID)
                .debug_selector(|| DOWNLOADS_EMPTY_ID.to_string())
                .h(px(3.0 * layout.row_h))
                .flex()
                .items_center()
                .justify_center()
                .px(Spacing::sp3())
                .text_size(theme.type_.meta.size)
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
/// the caller when the group is empty. The label height is the
/// token-derived constant the scroll math uses.
pub fn downloads_group_label(theme: &Theme, active: bool) -> Stateful<Div> {
    let p = theme.palette;
    let label_h = f32::from(theme.type_.badge.size) * theme.type_.badge.leading
        + 2.0 * f32::from(Spacing::sp1());
    div()
        .id(if active {
            DOWNLOADS_ACTIVE_ID
        } else {
            DOWNLOADS_HISTORY_ID
        })
        .debug_selector(|| {
            if active {
                DOWNLOADS_ACTIVE_ID.to_string()
            } else {
                DOWNLOADS_HISTORY_ID.to_string()
            }
        })
        .h(px(label_h))
        .px(Spacing::sp3())
        .flex()
        .items_center()
        .text_size(theme.type_.badge.size)
        .font_weight(theme.type_.badge.weight)
        .text_color(p.ink_faint)
        .child(if active { "ACTIVE" } else { "HISTORY" })
}

/// One §11.12 download row: `[status glyph] [name line + secondary
/// line] [action]` at `density:row` tall with `density:row-padding-x`,
/// the §10.2/§10.3 inset hover/press washes (`density:row-inset`,
/// `density:radius`), the deterministic `bbr-download-*` ids, and the
/// production pointer paths (hover/press state, action activation) routed
/// through the callbacks bundle.
pub fn downloads_row(
    theme: &Theme,
    visual: &DownloadRowModel,
    check: Option<u64>,
    callbacks: &DownloadsCallbacks,
) -> Stateful<Div> {
    let p = theme.palette;
    let density = theme.density;
    let index = visual.index;
    let icon_size = density.icon();
    let status = match visual.machine {
        DownloadMachine::Waiting | DownloadMachine::Running => {
            // §10.5 spinner arc (accent, transient indication — allowed).
            Icon::LoadingArc.canvas(theme, icon_size).into_any()
        }
        DownloadMachine::Succeeded => {
            // §3.3 `status.ok` check with its non-color companion (the
            // status glyph is always next to the name line); right after
            // the success commits it draws itself once (§18.5).
            super::check_moment::succeeded_glyph(theme, icon_size, check)
        }
        DownloadMachine::Failed => Icon::Lifering.canvas(theme, icon_size).into_any(),
        DownloadMachine::Cancelled => {
            // Neutral marker at `ink.secondary`.
            Icon::Stopped.canvas(theme, icon_size).into_any()
        }
    };
    let secondary = downloads_secondary_line(theme, visual);
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

    div()
        .id(download_row_id(index))
        .debug_selector(move || download_row_id(index))
        .h(density.row())
        .mx(density.row_inset())
        .rounded(density.density_radius())
        .px(density.row_padding_x())
        .gap(Spacing::sp2())
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
            el.border_2()
                .border_color(p.accent)
                .rounded(density.density_radius())
        })
        .child(
            div()
                .id(download_slot_id("status", index))
                .debug_selector(move || download_slot_id("status", index))
                .size(icon_size)
                .flex()
                .items_center()
                .justify_center()
                .child(status),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .justify_center()
                .gap(px(1.0))
                .child(
                    div()
                        .id(download_slot_id("name", index))
                        .debug_selector(move || download_slot_id("name", index))
                        .overflow_hidden()
                        .text_ellipsis()
                        .text_size(theme.type_.title.size)
                        .font_weight(theme.type_.title.weight)
                        .text_color(p.ink)
                        .child(visual.filename.clone()),
                )
                .child(secondary),
        )
        .child(downloads_action(theme, visual, index, activate))
        .child(downloads_delete_control(theme, visual, callbacks))
        .on_mouse_down(MouseButton::Left, press)
        .on_mouse_up(MouseButton::Left, release)
        .on_hover(hover)
}
