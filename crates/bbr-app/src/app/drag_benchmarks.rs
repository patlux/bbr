//! Testkit-only drag-to-chip benchmark driver (F2a, DESIGN.md §11.3/§10.9).
//!
//! The production drag-to-chip move (`BenchmarkDragMoveTab`): the concrete
//! row `from_index` is pressed, the pointer is moved onto the destination
//! space's rendered chip (the exact production chip-hit geometry derived
//! from the strip layout), and the release commits through the drag drop →
//! the existing single tab-move authority. Same typed kinds, report keys,
//! and budget classes as the direct move — only the measured production
//! entry is the drag-to-chip path, never a second move authority.
//!
//! The destination chip must be **visible** for the pointer to enter it
//! truthfully (the strip clips and scrolls, and its scroll-into-view
//! authority follows the selected chip). The driver therefore scrolls the
//! destination chip fully into view through the production tracked
//! `space_strip_scroll` handle and settles one bounded main-window draw —
//! the pending scroll applies in prepaint — before deriving the visible
//! span and pressing, exactly the render-inclusive commit pattern the
//! command/find/context-menu benchmarks use. A chip with no visible span
//! after the settle cancels the sample (never hidden hit geometry), and the
//! fast-but-wrong guard still rejects a drag that resolves a different
//! target.

use bbr_browser::SpaceId;
use gpui::{
    point, px, App, AsyncApp, Context, Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent,
    Window,
};

use super::BbrApp;

impl BbrApp {
    /// Benchmark the same tab move through the production **drag-to-chip**
    /// entry: the concrete row `from_index` is pressed, the pointer is
    /// moved onto the destination space's rendered chip, and the release
    /// commits through the drag drop → the existing single move authority.
    ///
    /// The destination chip may be scrolled out of the strip viewport, so
    /// the sample first brings it fully into view through the production
    /// scroll-into-view authority and a settled main-window draw, then
    /// enters the visible span — the pointer never targets hidden hit
    /// geometry. The commit runs after the settle-draw (render-inclusive,
    /// the same model the palette/find benchmarks use); the driver awaits
    /// the wire event + presentation measurement from the request cursor.
    pub fn benchmark_drag_move_tab(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        from_index: usize,
        to_space: SpaceId,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let snapshot = self.controller.read().snapshot();
        if snapshot.active_space == to_space || from_index >= snapshot.tabs.len() {
            return false;
        }
        let same_profile = snapshot
            .spaces
            .iter()
            .find(|s| s.id == to_space)
            .is_some_and(|s| s.profile == snapshot.active_profile);
        let kind = if same_profile {
            bbr_testkit::protocol::BenchmarkKind::TabMoveSameProfile
        } else {
            bbr_testkit::protocol::BenchmarkKind::TabMoveCrossProfile
        };
        if self.performance.begin(operation_id, kind).is_err() {
            return false;
        }
        // The §11.3 destination chip in canonical strip order (the same
        // `space_chip_visuals` vector the render, the snapshot, the
        // semantics, and the scroll seam consume). A space without a chip
        // cannot be a drag destination.
        let dest_index = self
            .space_chip_visuals()
            .iter()
            .position(|visual| visual.space == to_space);
        let Some(dest_index) = dest_index else {
            self.performance.cancel();
            return false;
        };
        // Scroll the destination chip fully into view through the same
        // tracked handle the activation path's scroll-into-view uses. The
        // pending item applies in prepaint of the next draw — the settle
        // happens outside the mutable `BbrApp` lease (the reentrancy rule
        // the command/find benchmarks follow), so the pointer point derives
        // from the visible, clipped geometry.
        self.space_strip_scroll.scroll_to_item(dest_index);
        let Some(window_handle) = cx.windows().into_iter().next() else {
            self.performance.cancel();
            return false;
        };
        let weak = cx.weak_entity();
        cx.spawn(async move |_, async_cx: &mut AsyncApp| {
            // GPUI-CE `AsyncApp::update` is infallible (no `.ok()`).
            async_cx.update(|cx| {
                let _ = window_handle.update(cx, |_root, window, cx| {
                    // Settle the queued scroll-into-view: the pending item
                    // applies in prepaint during the bounded draw, so the
                    // span derived next is the visible one.
                    let app: &mut App = cx;
                    let _ = window.draw(app);
                    // The drag continuation runs on the main thread inside
                    // the same window update, outside the mutable `BbrApp`
                    // lease of the original request.
                    let _: bool = weak
                        .update(cx, |this, cx| {
                            this.benchmark_drag_move_tab_visible(from_index, to_space, window, cx)
                        })
                        .unwrap_or(false);
                });
            });
        })
        .detach();
        true
    }

    /// The draw-settled continuation of [`Self::benchmark_drag_move_tab`]:
    /// derive the destination chip's visible span from the settled strip
    /// layout, press the concrete row, move the pointer onto the chip, and
    /// release through the production drag listeners. Cancels truthfully
    /// when the chip still has no visible span or the hit geometry resolves
    /// a different target (a fast-but-wrong move is never measured).
    fn benchmark_drag_move_tab_visible(
        &mut self,
        from_index: usize,
        to_space: SpaceId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        // The exact rendered chip of the destination space (production
        // strip layout). No chip span means the pointer cannot truthfully
        // enter the chip — the sample cancels instead of measuring a fake
        // path.
        let Some(chip) = self.chip_span_for(to_space) else {
            self.performance.cancel();
            return false;
        };
        // The production press/move/release sequence — the exact methods
        // the row mouse-down, the root mouse-move, and the mouse-up
        // listeners drive. The press anchors above the footer (the tab
        // list); the move enters the destination chip's rendered span; the
        // release commits the drop.
        let press = point(px(130.0), px(chip.center_y() - 300.0));
        let release = point(px(chip.center_x()), px(chip.center_y()));
        self.begin_tab_drag(
            &MouseDownEvent {
                button: MouseButton::Left,
                position: press,
                modifiers: Modifiers::default(),
                click_count: 1,
                first_mouse: true,
            },
            from_index,
            window,
            cx,
        );
        let row_surface = self.theme().density.row();
        self.set_tab_drag_press_in_row(f32::from(row_surface) / 2.0);
        // The activation move (past the 4 pt distance) presents the drag;
        // the next move resolves the chip target.
        self.on_tab_drag_move(
            &MouseMoveEvent {
                position: point(press.x, press.y + px(6.0)),
                pressed_button: Some(MouseButton::Left),
                modifiers: Modifiers::default(),
            },
            window,
            cx,
        );
        self.on_tab_drag_move(
            &MouseMoveEvent {
                position: release,
                pressed_button: Some(MouseButton::Left),
                modifiers: Modifiers::default(),
            },
            window,
            cx,
        );
        // Fast-but-wrong guard: the production hit geometry must resolve
        // exactly the destination chip before the release.
        if self.tab_drag.chip_target() != Some(to_space) {
            self.cancel_tab_drag(window, cx);
            self.performance.cancel();
            return false;
        }
        let committed = self.finish_tab_drag(window, cx);
        cx.notify();
        if !committed {
            self.performance.cancel();
            return false;
        }
        self.commit_organize_benchmark(window);
        true
    }
}
