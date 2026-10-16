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
use gpui::{Context, Div, MouseDownEvent, ScrollHandle, Stateful, Window};

use super::BbrApp;

mod fade;
mod rows;

use fade::DownloadsFade;

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
    /// Shared by every window (blueprint D19); the rest is this window's.
    pub controller: crate::shared::Shared<DownloadController>,
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
    pub reveal: crate::shared::Shared<HashMap<DownloadId, RevealEligibility>>,
    /// The §11.12 open/close opacity fade (`dur:fine` only — never translate
    /// or scale). Created lazily on the first render that needs it; the goal
    /// is `1.0` while open and `0.0` on dismissal, and the render keeps the
    /// panel mounted (fading out) until the value reaches zero.
    pub opacity: Option<DownloadsFade>,
    /// Ordinary testkit evidence from the panel's actual paint, not its model.
    #[cfg(feature = "testkit")]
    pub painted: Rc<std::cell::RefCell<Option<bbr_testkit::protocol::DownloadsPanelPaint>>>,
}

impl Default for DownloadsState {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadsState {
    pub fn new() -> Self {
        Self {
            controller: crate::shared::Shared::new(DownloadController::new()),
            open: false,
            focus: 0,
            hovered_row: None,
            pressed_row: None,
            scroll: ScrollHandle::new(),
            render_state: DownloadsRenderState::default(),
            reveal: Default::default(),
            opacity: None,
            #[cfg(feature = "testkit")]
            painted: Default::default(),
        }
    }

    /// Blueprint D19: another window's panel over the same downloads.
    pub fn sibling(&self) -> Self {
        Self {
            controller: self.controller.clone(),
            open: false,
            focus: 0,
            hovered_row: None,
            pressed_row: None,
            scroll: ScrollHandle::new(),
            render_state: DownloadsRenderState::default(),
            reveal: self.reveal.clone(),
            opacity: None,
            #[cfg(feature = "testkit")]
            painted: Default::default(),
        }
    }

    /// The full deterministic row list (active ascending `DownloadId`, then
    /// history most-recent-terminal first) — `row index <n>` is stable.
    pub fn row_ids(&self) -> Vec<DownloadId> {
        self.controller.read().rows()
    }

    /// The scroll anchor derived from the production scroll offset (negative
    /// as the user scrolls down), clamped to the deterministic row list. The
    /// layout maps the offset across the mixed Active/History row heights.
    pub fn anchor_from_offset(&self, layout: &PanelLayout) -> usize {
        let total = self.controller.read().len();
        if total == 0 {
            return 0;
        }
        let scrolled = (-f32::from(self.scroll.offset().y)).max(0.0);
        total
            .saturating_sub(DOWNLOADS_WINDOW_ROWS)
            .min(layout.anchor_at(scrolled))
    }

    /// The bounded render window over the combined row list.
    pub fn window_for(&self, anchor: usize) -> std::ops::Range<usize> {
        let total = self.controller.read().len();
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
    /// Advance the shared fade in its owning browser render. The floating
    /// rail is assembled outside that window's draw phase and only reads it.
    pub(crate) fn prime_downloads_fade(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        #[cfg(feature = "testkit")]
        self.downloads.painted.borrow_mut().take();
        let fade = self.downloads.opacity.get_or_insert_with(|| {
            let fade = DownloadsFade::new(cx);
            fade.set_open(self.downloads.open, cx);
            fade
        });
        fade.evaluate(window, cx);
    }

    /// Build the §11.12 production model and listeners. The app owns fade
    /// state, controller mapping and render observations; bbr-ui owns anatomy.
    /// Returns None only when logically closed and the fade-out has completed.
    pub(crate) fn downloads_panel_surface(
        &mut self,
        theme: &bbr_ui::theme::Theme,
        rail_width: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Stateful<Div>> {
        // Share the browser render's presented value with floating composition;
        // never evaluate against the browser window outside its draw phase.
        let alpha = self.downloads.opacity.as_ref()?.rendered_value();
        if !self.downloads.open && alpha <= 0.001 {
            return None;
        }

        // The §11.12 layout authority (Paper M-4 metrics): the same
        // token-derived heights the render paints and the snapshot reports.
        let (total, active_count, history_count) = {
            let controller = self.downloads.controller.read();
            (
                controller.len(),
                controller.active_ids().len(),
                controller.history_ids().len(),
            )
        };
        let layout = PanelLayout::for_theme(theme, active_count, history_count);
        let anchor = self.downloads.anchor_from_offset(&layout);
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
            width: (rail_width - 2.0 * f32::from(theme.density.row_inset())).max(0.0),
            rows: visuals,
            window: row_window,
            layout,
            scroll: &self.downloads.scroll,
            focus: &self.downloads_panel_handle,
            check: self.download_check(),
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
        let panel =
            bbr_ui::surfaces::downloads::downloads_panel_surface(theme, &model, &callbacks)?;
        #[cfg(feature = "testkit")]
        let panel = {
            use gpui::{prelude::*, Styled};
            let painted = self.downloads.painted.clone();
            // An absolute, non-painting observer occupies no flow space. Its
            // bounds are the panel interior (inside the production border).
            panel.relative().child(
                gpui::canvas(
                    |_, _, _| (),
                    move |bounds, (), _window, _cx| {
                        *painted.borrow_mut() = Some(bbr_testkit::protocol::DownloadsPanelPaint {
                            rect: [
                                f32::from(bounds.origin.x),
                                f32::from(bounds.origin.y),
                                f32::from(bounds.size.width),
                                f32::from(bounds.size.height),
                            ],
                            alpha,
                        });
                    },
                )
                .absolute()
                .inset_0(),
            )
        };
        Some(panel)
    }

    /// The §11.12 panel layout (the same token-derived heights the render
    /// paints), used by the render, the keyboard scroll, the benchmarks and
    /// the visible-id derivation.
    pub(crate) fn downloads_layout(&self) -> PanelLayout {
        let controller = self.downloads.controller.read();
        PanelLayout::for_theme(
            &self.theme(),
            controller.active_ids().len(),
            controller.history_ids().len(),
        )
    }
}

#[cfg(test)]
mod tests;
