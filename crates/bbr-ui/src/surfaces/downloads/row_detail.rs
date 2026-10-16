//! Downloads row/panel detail controls (DESIGN.md §11.12, Paper M-4): the
//! row's text column (name, the Active progress bar, the secondary line),
//! the single contextual primary row action, and the settled post-MVP
//! controls — the §17.8 row Delete control and the §17.7 Clear panel
//! control.
//!
//! Split out of `render` to keep both files within the ~500-line structure
//! gate. These are presentation only — the machine state and the offered
//! action arrive as a [`super::model::DownloadRowModel`] DTO and every
//! pointer path routes through the [`super::callbacks::DownloadsCallbacks`]
//! bundle; nothing here derives or changes a transition.

use gpui::{div, prelude::*, px, App, Div, MouseButton, MouseDownEvent, Stateful, Styled, Window};

use crate::icons::Icon;
use crate::theme::Theme;

use super::callbacks::DownloadsCallbacks;
use super::ids::{download_slot_id, DOWNLOADS_CLEAR_ID};
use super::model::{
    action_label, DownloadActionKind, DownloadGroupKind, DownloadMachine, DownloadRowModel,
    DownloadsMetrics,
};

/// Paper M-4: the action pill's horizontal padding (content-inset).
const PILL_PAD_X: f32 = 10.0;
/// Paper M-4: the indeterminate sweep frame starts 40 % into the track and
/// covers 30 % of it.
const SWEEP_START: f32 = 0.40;
const SWEEP_WIDTH: f32 = 0.30;

/// The §11.12 text column: the name line (`type:label`, `ink`, one line
/// with end-ellipsis), then — on Active rows only — the 4 pt progress bar
/// (track `wash.active`, fill `color:accent`; a known total fills
/// `fraction()`, an unknown length shows the §10.5 sweep frame), then the
/// secondary line (`type:meta`, `ink.faint`; a failure reason in
/// `status.error`). Terminal rows carry no bar: their secondary line names
/// the size and outcome. The name carries `bbr-download-name-<n>`, the bar
/// `bbr-download-progress-<n>`, the secondary line `bbr-download-meta-<n>`.
pub fn downloads_row_text(theme: &Theme, visual: &DownloadRowModel) -> Div {
    let p = theme.palette;
    let metrics = DownloadsMetrics::for_theme(theme);
    let index = visual.index;
    let name = div()
        .id(download_slot_id("name", index))
        .debug_selector(move || download_slot_id("name", index))
        .w_full()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .text_size(theme.type_.label.size)
        .font_weight(theme.type_.label.weight)
        .line_height(px(metrics.line))
        .text_color(p.ink)
        .child(visual.filename.clone());
    let bar = (visual.group == DownloadGroupKind::Active).then(|| {
        let (offset, width) = match visual.progress_fraction {
            Some(fraction) => (0.0, fraction.clamp(0.0, 1.0) as f32),
            None => (SWEEP_START, SWEEP_WIDTH),
        };
        div()
            .id(download_slot_id("progress", index))
            .debug_selector(move || download_slot_id("progress", index))
            .flex_shrink_0()
            .w(px(metrics.bar_w))
            .max_w_full()
            .h(px(metrics.bar))
            .rounded(theme.radius.full())
            .bg(p.wash_active)
            .child(
                div()
                    .h_full()
                    .ml(gpui::DefiniteLength::Fraction(offset))
                    .w(gpui::DefiniteLength::Fraction(width))
                    .rounded(theme.radius.full())
                    .bg(p.accent),
            )
    });
    let meta_colour = if visual.machine == DownloadMachine::Failed {
        p.status_error
    } else {
        p.ink_faint
    };
    let meta = div()
        .id(download_slot_id("meta", index))
        .debug_selector(move || download_slot_id("meta", index))
        .w_full()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .text_size(theme.type_.meta.size)
        .font_weight(theme.type_.meta.weight)
        .line_height(px(metrics.line))
        .text_color(meta_colour)
        .child(visual.meta.clone());
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(metrics.gap))
        .child(name)
        .children(bar)
        .child(meta)
}

/// The exactly-one contextual primary action per row (DESIGN.md §11.12):
/// active → **Cancel**, failed → **Retry**, succeeded →
/// **Reveal-in-Finder** (the conventional macOS "Show in Finder" copy;
/// §10.8 `disabled` with a reason when the resolved path does not
/// exist), cancelled → **Restart** (the settled §17.8 decision — a
/// fresh attempt linked via `retry_of`). The visible control is the Paper
/// 24 pt pill (`type:label`, `ink.secondary`; Retry as the bordered field
/// button); its hit area stays at least `density:hit-min`.
pub fn downloads_action(
    theme: &Theme,
    visual: &DownloadRowModel,
    index: usize,
    activate: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let p = theme.palette;
    let density = theme.density;
    let metrics = DownloadsMetrics::for_theme(theme);
    let label = action_label(visual.action);
    if label.is_empty() {
        // No primary action (an unreachable state after §17.8 — every
        // L6 state offers one) — the trailing slot stays quiet.
        return div().id(download_slot_id("action", index));
    }
    let enabled = visual.action_enabled;
    let bordered = enabled && visual.action == DownloadActionKind::Retry;
    let pill = div()
        .id(("download-action-pill", index))
        .h(px(metrics.action_h))
        .px(px(PILL_PAD_X))
        .rounded(theme.radius.medium())
        .flex()
        .items_center()
        .text_size(theme.type_.label.size)
        .font_weight(theme.type_.label.weight)
        .line_height(px(metrics.line))
        .when(bordered, |el| {
            el.border_1()
                .border_color(p.border_strong)
                .bg(p.surface_field)
                .text_color(p.ink)
        })
        .when(enabled && !bordered, |el| el.text_color(p.ink_secondary))
        .when(!enabled, |el| el.text_color(p.ink_faint))
        .when(enabled, |el| el.hover(|el| el.bg(p.wash_hover)))
        .child(label);
    div()
        .id(download_slot_id("action", index))
        .debug_selector(move || download_slot_id("action", index))
        .flex_shrink_0()
        .h(density.hit_min())
        .min_w(density.hit_min())
        .flex()
        .items_center()
        .justify_center()
        .when(enabled, |el| el.cursor_pointer())
        // A disabled action (a vanished reveal target) is a no-op; the
        // reason rides the semantic tree / snapshot (§10.8; the tooltip
        // presentation is deferred with DESIGN.md §17.2).
        .when(enabled, |el| el.on_mouse_down(MouseButton::Left, activate))
        .child(pill)
}

/// The §17.8 settled row Delete control: a cancelled row offers
/// **Restart** (the primary action) **and** Delete — this
/// `bbr-download-delete-<n>` control removes exactly that one terminal
/// record durably (the app's `download_delete_row` callback — the exact
/// seam the testkit `ClickDownloadRowDelete` request drives). Paper M-4
/// draws it as the trash glyph; its accessible name stays "Delete".
/// Rendered only when `visual.delete_available` (cancelled rows).
pub fn downloads_delete_control(
    theme: &Theme,
    visual: &DownloadRowModel,
    callbacks: &DownloadsCallbacks,
) -> Option<Stateful<Div>> {
    if !visual.delete_available {
        return None;
    }
    let p = theme.palette;
    let density = theme.density;
    let index = visual.index;
    let on_row_delete = callbacks.on_row_delete.clone();
    let id = visual.id;
    let delete = move |_e: &MouseDownEvent, window: &mut Window, cx: &mut App| {
        on_row_delete(id, window, cx);
        cx.stop_propagation();
    };
    Some(
        div()
            .id(download_slot_id("delete", index))
            .debug_selector(move || download_slot_id("delete", index))
            .flex_shrink_0()
            .size(density.hit_min_close())
            .rounded(theme.radius.small())
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|el| el.bg(p.wash_hover))
            .on_mouse_down(MouseButton::Left, delete)
            .child(Icon::Trash.canvas_with(theme, density.icon(), p.ink_secondary)),
    )
}

/// The §11.12 Clear control (DESIGN.md §17.7 settled): a trailing
/// `type:label` control below the scroll area, visible while any history
/// exists (Paper M-4: right-aligned, 6 pt above, 2 pt below). Pointer press
/// clears the controller history and the repository durably (the app's
/// `downloads_clear` callback — the exact seam the testkit
/// `ClickDownloadsClear` request drives) and stops propagation so the root
/// outside-press handler never dismisses the panel mid-action.
pub fn downloads_clear_control(theme: &Theme, callbacks: &DownloadsCallbacks) -> Div {
    let p = theme.palette;
    let metrics = DownloadsMetrics::for_theme(theme);
    let on_clear = callbacks.on_clear.clone();
    let clear = move |_e: &MouseDownEvent, window: &mut Window, cx: &mut App| {
        on_clear(window, cx);
        cx.stop_propagation();
    };
    div()
        .w_full()
        .flex()
        .flex_row()
        .justify_end()
        .pt(px(6.0))
        .pb(px(2.0))
        .px(px(4.0))
        .child(
            div()
                .id(DOWNLOADS_CLEAR_ID)
                .debug_selector(|| DOWNLOADS_CLEAR_ID.to_string())
                .h(px(metrics.action_h))
                .px(px(PILL_PAD_X))
                .rounded(theme.radius.medium())
                .flex()
                .items_center()
                .cursor_pointer()
                .text_size(theme.type_.label.size)
                .font_weight(theme.type_.label.weight)
                .line_height(px(metrics.line))
                .text_color(p.ink_secondary)
                .hover(|el| el.bg(p.wash_hover))
                .on_mouse_down(MouseButton::Left, clear)
                .child("Clear"),
        )
}
