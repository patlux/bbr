//! Testkit-only context-menu benchmark driver (queue item 12, part B).
//!
//! `benchmark_context_menu_open` measures the production event-driven open
//! path (the exact seam the native bridge's `WebEvent::ContextMenuRequested`
//! routes to: `menu_open_from_event`): the row model build, the anchored
//! placement, the portal presentation, and the focus handoff. The sample
//! commits only from a render of the one `NativeAboveWeb` portal window that
//! drew the menu open and focused over the same active view at the armed
//! pointer anchor (fast-but-wrong guard: a render that omitted the menu,
//! changed the bound view, or lost the anchor rejects in the app and the
//! driver reports failure).
//!
//! Opening the menu never touches a tab, session, profile, space, or focus
//! domain (the menu owns a non-chrome focus handle); the armed browser-state
//! expectations reject any sample that mutates them.

use bbr_testkit::protocol::{BenchmarkKind, BenchmarkOperationId};
use gpui::{App, AsyncApp, Context, Window};

use super::BbrApp;

impl BbrApp {
    /// Benchmark the production context-menu open over the active web view:
    /// the sample starts at the exact event-driven open seam
    /// (`menu_open_from_event`) and commits only from a render of the portal
    /// window that drew the menu open with the field focused over the same
    /// active view at the armed anchor. The active view/sidebar/focus-domain
    /// are unchanged and no command surface is active; the armed
    /// expectations reject any render that omits the menu or touches the
    /// session.
    pub fn benchmark_context_menu_open(
        &mut self,
        operation_id: BenchmarkOperationId,
        view: bbr_web::WebViewId,
        point: [f32; 2],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::ContextMenuOpen)
            .is_err()
        {
            return false;
        }
        // Production event-driven open: the exact seam the native bridge's
        // `ContextMenuRequested` routes to (the point is in window content
        // coordinates, points, top-left origin).
        let rect = bbr_web::WebRect::new(point[0], point[1], 0.0, 0.0);
        self.menu_open_from_event(view, rect, window, cx);
        let active_view = self.controller.read().snapshot().active_view;
        let mode: bbr_testkit::protocol::CommandBarMode = self.command_bar_mode.into();
        // Fast-but-wrong guard: the same active view stays selected, the
        // sidebar stays visible, the menu focus is a non-chrome domain (so
        // the chrome-focus state is unchanged), and no command surface is
        // active. Refresh the probe from the authoritative post-mutation
        // state so the presentation commit sees the real state (the menu
        // render has not painted yet).
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            self.chrome_focused(window),
            mode,
        );
        self.performance.expect_state(
            active_view,
            self.sidebar_visible,
            self.chrome_focused(window),
            mode,
        );
        self.performance.expect_context_menu_open(view, point);
        // Render-inclusive commit: draw the portal window (the menu renders
        // there) so the render records the drawn menu state; the probe then
        // commits only if the armed open identity and browser state match.
        self.schedule_context_menu_render_commit(cx);
        true
    }

    /// Schedule the render-inclusive commit for a pending context-menu
    /// benchmark: a bounded forced draw of the portal window (the menu
    /// renders in the one `NativeAboveWeb` host, so the main-window draw
    /// used by the other command benchmarks cannot present it), followed by
    /// the explicit commit through the app-entity contract. The draw runs on
    /// the main thread after this update completes, outside the mutable
    /// `BbrApp` lease (the same reentrancy rule as `dispatch_portal`).
    pub(crate) fn schedule_context_menu_render_commit(&self, cx: &mut Context<Self>) {
        if !matches!(
            self.performance.pending_kind(),
            Some(BenchmarkKind::ContextMenuOpen)
        ) {
            return;
        }
        let Some(portal) = self.portal_frame_handle() else {
            return;
        };
        let weak = cx.weak_entity();
        cx.spawn(async move |_, async_cx: &mut AsyncApp| {
            // GPUI-CE `AsyncApp::update` is infallible (no `.ok()`).
            async_cx.update(|cx| {
                // Draw the portal window: `PortalRoot` renders the menu from
                // the app's live `ContextMenuSurface` and records the drawn
                // menu state into the probe (the render milestone). A render
                // that omitted the menu records nothing and can never commit.
                let _ = portal.update(cx, |_root, window, cx| {
                    let app: &mut App = cx;
                    let _ = window.draw(app);
                });
                // Post-render commit through the app-entity contract: the
                // explicit path validates the armed identity + browser state
                // and cancels truthfully when the render did not present the
                // menu (no observation → no commit).
                let _: bool = weak
                    .update(cx, |this, _cx| this.commit_context_menu_benchmark())
                    .unwrap_or(false);
            });
        })
        .detach();
    }

    /// Commit the pending context-menu sample from the render milestone: the
    /// probe compares the armed open identity against the observation the
    /// portal render recorded and the browser-state expectations against the
    /// state observed at the mutation site. Returns false and cancels when
    /// the render omitted the menu or the identity/state was wrong — a
    /// truthful rejection, never a fast-but-wrong pass.
    fn commit_context_menu_benchmark(&self) -> bool {
        self.performance.commit_current_presentation();
        if matches!(
            self.performance.pending_kind(),
            Some(BenchmarkKind::ContextMenuOpen)
        ) {
            self.performance.cancel();
            return false;
        }
        true
    }

    /// Record the menu state the portal render actually drew (the render
    /// milestone; `PortalRoot` calls this after consuming the app's live
    /// `ContextMenuSurface`). The probe commits context-menu samples only
    /// from this observation.
    #[cfg(feature = "testkit")]
    pub(crate) fn observe_portal_context_menu_render(&self) {
        let s = &self.context_menu.state;
        self.performance
            .observe_context_menu_state(s.open, s.focused, s.bound_view, s.point);
    }

    /// Benchmark the production bookmark-row activation (task packet
    /// `page-background-context-menu.md`): the sample starts at the exact
    /// `menu_activate` seam a pointer press on the `Bookmark Page` /
    /// `Remove Bookmark` row dispatches — B2's one app-owned
    /// `SavedPageCommand::toggle` against the captured focused-page
    /// URL/title plus the menu dismissal — and commits only from a render
    /// that drew the menu closed (the main-window render observes the
    /// post-activation state; the portal never draws the closed menu
    /// again). The menu must be open over `view` at `point` with the
    /// bookmark row enabled (the mutation-site guard); the B2 store truth
    /// (the record flipped exactly as the row title stated) is verified
    /// right after the toggle. Same warm-interaction class as the other
    /// command commits; no tab or session is created or touched.
    pub fn benchmark_context_menu_bookmark(
        &mut self,
        operation_id: BenchmarkOperationId,
        view: bbr_web::WebViewId,
        point: [f32; 2],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(operation_id, BenchmarkKind::ContextMenuBookmarkCommit)
            .is_err()
        {
            return false;
        }
        // Mutation-site guard: the menu is open over the armed view at the
        // armed anchor with the bookmark row present and enabled — the only
        // state from which the activation can be measured truthfully.
        let open = self.context_menu.state.open
            && self.context_menu.state.bound_view == Some(view)
            && self.context_menu.state.point == point;
        let bookmark_index =
            self.context_menu.state.items.iter().position(|item| {
                item.key == crate::app::context_menu::MENU_BOOKMARK && item.enabled
            });
        let (Some(index), true) = (bookmark_index, open) else {
            self.performance.cancel();
            return false;
        };
        let url = self.context_menu.state.url.clone();
        // The B2 truth before the toggle (the direction the row title
        // states: Bookmark Page → unsaved, Remove Bookmark → saved).
        let saved_before = url
            .as_deref()
            .is_some_and(|u| self.saved_pages.can_remove(self, u));
        // Arm the post-activation render gate: a render that drew the menu
        // CLOSED. Arming clears the open observation, so the pre-activation
        // open state can never commit the sample.
        self.performance.expect_context_menu_bookmark_commit();
        let active_view = self.controller.read().snapshot().active_view;
        let mode: bbr_testkit::protocol::CommandBarMode = self.command_bar_mode.into();
        // Production activation: the exact seam the row's mouse-down
        // listener calls — toggle through B2, dismiss the menu, restore the
        // origin focus domain.
        self.menu_activate(index, window, cx);
        let chrome_focused = self.chrome_focused(window);
        self.performance.observe_render_state(
            active_view,
            self.sidebar_visible,
            chrome_focused,
            mode,
        );
        self.performance
            .expect_state(active_view, self.sidebar_visible, chrome_focused, mode);
        // The store truth at the mutation site: the record flipped exactly
        // as the row title stated (unsaved → saved, or saved → removed). A
        // toggle that did not flip the store rejects truthfully.
        let saved_after = url
            .as_deref()
            .is_some_and(|u| self.saved_pages.can_remove(self, u));
        if saved_after == saved_before {
            self.performance.cancel();
            return false;
        }
        // Render-inclusive commit: the harness's browser window is never
        // shown, so it draws only when asked (no display link); a forced
        // draw renders the closed menu and the slot paint commits the
        // sample when the armed state matches.
        self.schedule_bookmark_render_commit(cx);
        true
    }

    /// Schedule the render-inclusive commit for a pending bookmark sample:
    /// a bounded forced draw of this browser window after this update
    /// completes (outside the mutable `BbrApp` lease), then the explicit
    /// commit, which cancels truthfully when the render did not present the
    /// closed menu over the armed state (no observation → no commit).
    fn schedule_bookmark_render_commit(&self, cx: &mut Context<Self>) {
        let Some(window) = self.browser_window else {
            return;
        };
        let weak = cx.weak_entity();
        cx.spawn(async move |_, async_cx: &mut AsyncApp| {
            // GPUI-CE `AsyncApp::update` is infallible (no `.ok()`).
            async_cx.update(|cx| {
                let _ = (*window).update(cx, |_root, window, cx| {
                    let app: &mut App = cx;
                    let _ = window.draw(app);
                });
                let _: bool = weak
                    .update(cx, |this, _cx| this.commit_bookmark_benchmark())
                    .unwrap_or(false);
            });
        })
        .detach();
    }

    /// Commit the pending bookmark sample from the render milestone; false
    /// (and cancelled) when the render did not present the closed menu.
    fn commit_bookmark_benchmark(&self) -> bool {
        if self.performance.pending_kind() != Some(BenchmarkKind::ContextMenuBookmarkCommit) {
            return true;
        }
        self.performance.commit_current_presentation();
        if self.performance.pending_kind() == Some(BenchmarkKind::ContextMenuBookmarkCommit) {
            self.performance.cancel();
            return false;
        }
        true
    }
}
