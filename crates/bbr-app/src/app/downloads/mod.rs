//! Downloads panel composition state (DESIGN.md §11.12).
//!
//! The app-side half of the Phase-8 downloads surface: the controller-owned
//! attempt/history store ([`bbr_browser::DownloadController`], built on the
//! L6 portable contract), the anchored-popover presentation state (open,
//! keyboard-cursor focus, scroll anchor), the pure panel layout math, and
//! the render wrapper that builds the [`DownloadsPanelModel`] DTO and wires
//! the production listener bundle onto the `bbr-ui` downloads surface. The
//! deterministic row-model derivation (the DTO building from controller
//! records) lives in the sibling [`rows`] module. The native `WKDownload`
//! adapter lives in `bbr-web-macos`; the composition
//! (`app::downloads_behavior`) wires actions and events between the
//! controller and the adapter.
//!
//! Machine fidelity (L6): the panel renders exactly the L6 states and never
//! invents a transition; the first terminal event wins; cancel is a request
//! that stays active until the engine's terminal event lands; retry exists
//! on `Failed` and restart on `Cancelled` (the settled §17.8 decision) —
//! both link a fresh attempt via `retry_of`; progress is monotonic and a
//! zero-byte download succeeds with a full bar. All of that is the
//! controller's contract — this module only maps records to rows.
//!
//! Deterministic ordering: Active by ascending `DownloadId`, History
//! most-recent-terminal first, so the panel row index `<n>` is stable for
//! E2E. Bounded rendering: at most [`DOWNLOADS_MAX_VISIBLE`] rows in the
//! viewport, a [`DOWNLOADS_WINDOW_ROWS`] render window (visible range plus
//! the §11.6 8-row overscan), and the full history is never laid out.
//!
//! The §11.12 presentation moved to `bbr-ui::surfaces::downloads` in the
//! G2 Group 5 cutover (packet `component-gallery-browser-components.md`);
//! this module keeps the controller, the lifecycle (the `dur:fine`
//! opacity transition), and the render wrapper, and the old app-owned
//! render modules were deleted with no duplicate and no re-export alias.

use std::collections::HashMap;
use std::rc::Rc;

use bbr_browser::DownloadController;
use bbr_ui::surfaces::downloads::{
    DownloadsCallbacks, DownloadsClear, DownloadsInsidePress, DownloadsKeyDown,
    DownloadsPanelModel, DownloadsRowAction, DownloadsRowDelete, DownloadsRowState, PanelLayout,
    DOWNLOADS_MAX_VISIBLE, DOWNLOADS_WINDOW_ROWS,
};
use bbr_web::downloads::DownloadId;
use gpui::{Context, Div, MouseDownEvent, ScrollHandle, Stateful, Transition, Window};

use super::BbrApp;

mod rows;

// The row-model derivation is the module's public API surface (the render
// wrapper, the testkit snapshot, the behavior/benchmark modules, and the
// unit tests consume it through `crate::app::downloads::*`); the glob
// re-export keeps every pub symbol of [`rows`] reachable at the module
// root exactly as before the split.
pub use rows::*;

/// DESIGN.md §11.12: at most 6 rows are visible in the panel viewport at
/// once (beyond that the popover scrolls). The constants live in
/// `bbr-ui::surfaces::downloads` (the §11.12 vocabulary moved in the G2
/// Group 5 cutover); the app imports them at the use sites.
/// What the last render actually built (recorded during render so the
/// testkit snapshot reports exactly what the shell built).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DownloadsRenderState {
    /// First rendered combined row index.
    pub first_rendered: usize,
    /// How many rows the render built (bounded by [`DOWNLOADS_WINDOW_ROWS`]).
    pub rendered_rows: usize,
    /// The full deterministic row list length.
    pub item_count: usize,
    /// First row at the viewport top, derived from the production scroll
    /// offset.
    pub scroll_anchor: usize,
    /// The scroll viewport's row count (`min(6, total)`).
    pub viewport_rows: usize,
}

/// The Reveal-in-Finder eligibility of one succeeded record (DESIGN.md
/// §11.12: enabled only when the resolved path exists on disk; otherwise
/// §10.8 `disabled` with a reason). Computed through the native adapter —
/// never in a portable crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevealEligibility {
    pub eligible: bool,
    pub reason: Option<String>,
}

/// App-side downloads composition state. The controller is the portable
/// authority; this struct owns only presentation state (open, focus, scroll,
/// hover/press, reveal cache, last render state).
pub struct DownloadsState {
    pub controller: DownloadController,
    /// The anchored popover is open.
    pub open: bool,
    /// The keyboard-cursor row (no persistent selection — focus is the
    /// cursor; DESIGN.md §11.12).
    pub focus: usize,
    pub hovered_row: Option<usize>,
    pub pressed_row: Option<usize>,
    /// Production scroll state shared by the render, the keyboard traversal,
    /// and the native wheel listener.
    pub scroll: ScrollHandle,
    /// What the last render built (testkit snapshot evidence).
    pub render_state: DownloadsRenderState,
    /// Reveal eligibility cache keyed by attempt id (computed through the
    /// native adapter at success time and on panel open).
    pub reveal: HashMap<DownloadId, RevealEligibility>,
    /// The §11.12 open/close opacity fade (`dur:fine` only — never translate
    /// or scale). Created lazily on the first render that needs it; the goal
    /// is `1.0` while open and `0.0` on dismissal, and the render keeps the
    /// panel mounted (fading out) until the value reaches zero.
    pub opacity: Option<Transition<f32>>,
}

impl Default for DownloadsState {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadsState {
    pub fn new() -> Self {
        Self {
            controller: DownloadController::new(),
            open: false,
            focus: 0,
            hovered_row: None,
            pressed_row: None,
            scroll: ScrollHandle::new(),
            render_state: DownloadsRenderState::default(),
            reveal: HashMap::new(),
            opacity: None,
        }
    }

    /// The full deterministic row list (active ascending `DownloadId`, then
    /// history most-recent-terminal first) — `row index <n>` is stable.
    pub fn row_ids(&self) -> Vec<DownloadId> {
        self.controller.rows()
    }

    /// The scroll anchor derived from the production scroll offset (negative
    /// as the user scrolls down), clamped to the deterministic row list.
    pub fn anchor_from_offset(&self, row_h: f32) -> usize {
        let total = self.controller.len();
        if total == 0 {
            return 0;
        }
        let offset = self.scroll.offset().y;
        let row_h = row_h.max(1.0);
        let scrolled = (-f32::from(offset)).max(0.0);
        let anchor = (scrolled / row_h).floor() as usize;
        total.saturating_sub(DOWNLOADS_WINDOW_ROWS).min(anchor)
    }

    /// The bounded render window over the combined row list.
    pub fn window_for(&self, anchor: usize) -> std::ops::Range<usize> {
        let total = self.controller.len();
        if total == 0 {
            return 0..0;
        }
        let anchor = total.saturating_sub(DOWNLOADS_WINDOW_ROWS).min(anchor);
        let end = (anchor + DOWNLOADS_WINDOW_ROWS).min(total);
        anchor..end
    }

    /// The scroll viewport's row count: at most [`DOWNLOADS_MAX_VISIBLE`].
    pub fn viewport_rows(&self, total: usize) -> usize {
        DOWNLOADS_MAX_VISIBLE.min(total)
    }
}

impl BbrApp {
    /// The §11.12 anchored popover surface (the G2 Group 5 render wrapper):
    /// the app keeps the transition lifecycle (the `dur:fine` opacity fade
    /// is created lazily — it needs the window state), the DTO building
    /// (layout/anchor/window/row models from the controller), the render
    /// evidence recording, and the production listener bundle; the `bbr-ui`
    /// surface owns the anatomy. `None` while closed and the fade-out has
    /// completed.
    pub(crate) fn downloads_panel_surface(
        &mut self,
        theme: &bbr_ui::theme::Theme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Stateful<Div>> {
        // §11.12 open/close: opacity fade `dur:fine` only — never translate
        // or scale. The transition is created lazily on the first render
        // that needs it (the render context owns the window state); the goal
        // is driven by open/close. While the fade-out runs the panel stays
        // mounted; at zero it unmounts.
        if self.downloads.opacity.is_none() {
            let transition =
                window.use_transition(cx, bbr_ui::theme::Motion::fine(), |_, _| 0.0_f32);
            if self.downloads.open {
                transition.update(cx, |value, cx| {
                    *value = 1.0;
                    cx.notify();
                });
            }
            self.downloads.opacity = Some(transition);
        }
        let fade = self.downloads.opacity.as_ref().expect("armed above");
        let alpha = *fade.evaluate(window, cx);
        if !self.downloads.open && alpha <= 0.001 {
            return None;
        }

        // The §11.12 `type:badge` group label: badge text step × leading +
        // 2 × `sp:1` padding — the exact height the label divs render (both
        // sides use the same token-derived constant, so the scroll math and
        // the layout never drift).
        let row_h = f32::from(theme.density.row());
        let label_h = f32::from(theme.type_.badge.size) * theme.type_.badge.leading
            + 2.0 * f32::from(bbr_ui::theme::Spacing::sp1());
        let controller = &self.downloads.controller;
        let total = controller.len();
        let layout = PanelLayout {
            label_h,
            row_h,
            active_count: controller.active_ids().len(),
            history_count: controller.history_ids().len(),
            hairline: 1.0,
        };
        let anchor = self.downloads.anchor_from_offset(row_h);
        let row_window = self.downloads.window_for(anchor);
        let visuals = row_visuals(&self.downloads, Some(self.downloads.focus));

        // Record what this render built so the testkit snapshot reports the
        // bounded window and the deterministic scroll anchor (the snapshot
        // mirrors the last frame, exactly like the dense tab list).
        self.downloads.render_state = DownloadsRenderState {
            first_rendered: row_window.start,
            rendered_rows: row_window.len(),
            item_count: total,
            scroll_anchor: anchor,
            viewport_rows: self.downloads.viewport_rows(total),
        };

        let panel_focused = self.downloads_panel_handle.is_focused(window);

        // Render milestone for the downloads presentation probe (DESIGN.md
        // §15 motion gate): record the panel presentation this frame actually
        // drew (open/focused plus the bounded row window — never the full
        // history), so a pending downloads benchmark can commit only from a
        // render that presented the armed identity. A frame that omitted the
        // panel records nothing.
        #[cfg(feature = "testkit")]
        {
            use crate::performance::downloads::observe_row;
            let rows = visuals
                .iter()
                .filter(|row| row_window.contains(&row.index))
                .map(observe_row)
                .collect::<Vec<_>>();
            self.performance
                .observe_downloads_state(self.downloads.open, panel_focused, rows);
        }

        let model = DownloadsPanelModel {
            open: self.downloads.open,
            focused: panel_focused,
            alpha,
            width: self.downloads_panel_width(window),
            rows: visuals,
            window: row_window,
            layout,
            scroll: &self.downloads.scroll,
            focus: &self.downloads_panel_handle,
        };

        // The production listener bundle (the Library wrapper pattern): the
        // entity handle re-enters the app's behavior/history authorities on
        // real pointer/keyboard events.
        let entity = cx.entity();
        let on_key_down: DownloadsKeyDown = Rc::new(move |event, window, cx| {
            entity.update(cx, |view, cx| view.on_downloads_key_down(event, window, cx));
        });
        let entity = cx.entity();
        let on_inside_press: DownloadsInsidePress =
            Rc::new(move |_e: &MouseDownEvent, _window, cx| {
                entity.update(cx, |view, _cx| view.mark_downloads_press_inside());
                // §13: one pointer action — the inside-press marker stops
                // propagation so the root outside-press handler never dismisses
                // the panel.
                cx.stop_propagation();
            });
        let entity = cx.entity();
        let on_row_action: DownloadsRowAction = Rc::new(move |index, window, cx| {
            entity.update(cx, |view, cx| {
                crate::app::downloads_behavior::download_click_row_action(view, index, window, cx)
            });
        });
        let entity = cx.entity();
        let on_row_press: DownloadsRowState = Rc::new(move |index, pressed, _window, cx| {
            entity.update(cx, |view, cx| {
                view.set_downloads_pressed(if pressed { Some(index) } else { None }, cx);
            });
        });
        let entity = cx.entity();
        let on_row_hover: DownloadsRowState = Rc::new(move |index, hovered, _window, cx| {
            entity.update(cx, |view, cx| view.set_downloads_hover(index, hovered, cx));
        });
        let entity = cx.entity();
        let on_row_delete: DownloadsRowDelete = Rc::new(move |id, _window, cx| {
            entity.update(cx, |view, cx| {
                let _ = crate::app::downloads_history::download_delete_row(
                    view,
                    bbr_web::downloads::DownloadId(id),
                    cx,
                );
            });
        });
        let entity = cx.entity();
        let on_clear: DownloadsClear = Rc::new(move |_window, cx| {
            entity.update(cx, |view, cx| {
                crate::app::downloads_history::downloads_clear(view, cx);
            });
        });
        let callbacks = DownloadsCallbacks {
            on_key_down,
            on_inside_press,
            on_row_action,
            on_row_press,
            on_row_hover,
            on_row_delete,
            on_clear,
        };
        bbr_ui::surfaces::downloads::downloads_panel_surface(theme, &model, &callbacks)
    }

    /// §11.12 panel width: the sidebar column's current width − 2 ×
    /// `density:row-inset` (S `sp:1` 4 / N `sp:2` 8 / XL `sp:3` 12), clamped
    /// to stay fully inside the window. The leading edge is flush with the
    /// column's leading edge, so the popover never overlaps the native web
    /// view (it is confined to the sidebar column by construction).
    fn downloads_panel_width(&self, window: &Window) -> f32 {
        let density = self.theme().density;
        let window_width = f32::from(window.bounds().size.width);
        let sidebar_width = if self.sidebar_visible {
            self.sidebar.effective_width(window_width)
        } else {
            0.0
        };
        (sidebar_width - 2.0 * f32::from(density.row_inset())).max(0.0)
    }

    /// The §11.12 panel layout tokens for the visible-id derivation (the
    /// same token-derived constants the render uses, so the ids mirror the
    /// shell's gating exactly).
    #[cfg(feature = "testkit")]
    pub(crate) fn downloads_layout(&self) -> PanelLayout {
        let theme = self.theme();
        PanelLayout {
            label_h: f32::from(theme.type_.badge.size) * theme.type_.badge.leading
                + 2.0 * f32::from(bbr_ui::theme::Spacing::sp1()),
            row_h: f32::from(theme.density.row()),
            active_count: self.downloads.controller.active_ids().len(),
            history_count: self.downloads.controller.history_ids().len(),
            hairline: 1.0,
        }
    }
}

#[cfg(test)]
mod tests;
