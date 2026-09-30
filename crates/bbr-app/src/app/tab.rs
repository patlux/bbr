//! Tab-list pointer and keyboard interaction (DESIGN.md §11.1/§11.2).
//!
//! The visible tab surface lives in `app::sidebar_view`; this module owns
//! the production interaction paths: pointer selection/close through the row
//! listeners, keyboard traversal of the focused list (`↑`/`↓`, Enter,
//! backspace), reopen, and the crash-panel reload. Every mutation routes
//! through the controller and preserves the chrome focus domain, matching
//! the interaction contract.

use bbr_browser::ControllerEvent;
use gpui::{px, Context, Window};

use super::BbrApp;

impl BbrApp {
    /// Pointer selection of a visible tab row: chrome focus domain preserved,
    /// selection routed through the same controller mutation the keyboard
    /// path uses (DESIGN.md §9 / interaction contract).
    pub fn select_row_via_pointer(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_tab_list(window, cx);
        if self.select_tab(index, cx).is_some() {
            cx.notify();
        }
    }

    /// Pointer close of a visible tab row: chrome focus domain preserved.
    /// Closing the final tab opens the empty New Tab command surface, the
    /// same outcome the Cmd-W path produces (interaction contract: empty
    /// space offers an obvious way to open a new tab).
    pub fn close_row_via_pointer(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_tab_list(window, cx);
        let closed = self.close_tab(index, cx).is_some();
        if closed && self.controller.read().snapshot().active_view.is_none() {
            self.enter_empty_new_tab(window, cx);
        }
    }

    /// Blueprint D21 (`⌥⌘S`): move keyboard focus into the rail on the
    /// selected row — the tab list takes chrome focus with its ring on the
    /// selected tab, scrolled into view; ↑↓ move and Return selects and
    /// returns focus to the page (the existing tab-list keyboard). An open
    /// command surface closes first; a hidden rail floats (blueprint D02).
    pub fn focus_rail(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.dismiss_command_portal(cx);
        // Blueprint D02: a hidden rail floats to take the focus (⌘S docks).
        if !self.sidebar_visible {
            self.float_rail(window, cx);
        }
        self.focus_tab_list(window, cx);
        self.ensure_active_visible();
        cx.notify();
    }

    /// Focus the tab list (chrome domain) without activating the command bar.
    pub(super) fn focus_tab_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_bar_mode = crate::command_bar::CommandBarMode::Inactive;
        self.command_bar_return = None;
        self.command_ranking.close();
        window.focus(&self.tab_list_handle, cx);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
    }

    /// Keyboard navigation of the visible tab list (DESIGN.md §11.1/§11.6):
    /// `↑/↓` move selection, `PgUp/PgDn` move both selection and viewport,
    /// `backspace` closes the active tab, Enter activates. In a dense
    /// collection the arrow keys clamp at the list edges instead of wrapping
    /// (a wrap would teleport the viewport across 500 rows); small
    /// collections keep the established wrap-around traversal. Every move
    /// scrolls the selection into view instantly. Runs only while the tab
    /// list owns chrome focus and no command mode is active.
    pub(super) fn on_tab_list_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.command_bar_mode.is_active() {
            return;
        }
        let snapshot = self.controller.read().snapshot();
        let Some(active) = snapshot.active_index else {
            return;
        };
        let len = snapshot.tabs.len();
        match event.keystroke.key.as_str() {
            "up" | "down" => {
                if len > 1 {
                    let delta = if event.keystroke.key == "up" { -1 } else { 1 };
                    let dense = len > crate::dense::DENSE_WINDOW_ROWS;
                    // §11.11: traversal follows the rendered rows — loose
                    // pinned, expanded folders' members, then the regular
                    // region (a member of a collapsed folder keeps the
                    // index-order step). Favorites render as tiles only
                    // (§11.10) and closed rows are skipped: stepping onto
                    // one would reopen it (§10.12).
                    let organization = self
                        .organization
                        .bookkeep()
                        .organization_snapshot(&self.controller.read());
                    let grouped = crate::shell::organized::hidden_pinned(
                        &organization.folders,
                        &organization.favorites,
                        &snapshot,
                    );
                    let presentation = &self.u2.folder_presentation;
                    let expanded = crate::shell::organized::expanded_member_indices(
                        &organization.folders,
                        &snapshot,
                        |id| presentation.is_expanded(id),
                    );
                    let pinned = crate::shell::pinned_count(&snapshot.tabs);
                    let mut order =
                        crate::shell::organized::traversal_order(len, pinned, &grouped, &expanded);
                    order.retain(|index| {
                        snapshot
                            .tabs
                            .get(*index)
                            .is_some_and(|tab| tab.lifecycle != bbr_browser::TabLifecycle::Closed)
                    });
                    let stepped = crate::shell::organized::step(&order, active, delta, !dense);
                    // An active row outside the rendered order (a collapsed
                    // member or a Favorite) keeps the index-order step,
                    // still past closed rows.
                    let next = stepped.or_else(|| {
                        let mut index = active;
                        for _ in 0..len {
                            index = if dense {
                                // Dense: clamp at the edges (no wrap across
                                // the collection).
                                (index as isize + delta).clamp(0, len as isize - 1) as usize
                            } else {
                                (index as isize + delta).rem_euclid(len as isize) as usize
                            };
                            if snapshot.tabs[index].lifecycle != bbr_browser::TabLifecycle::Closed {
                                return Some(index);
                            }
                        }
                        None
                    });
                    if next.is_some_and(|next| self.select_tab(next, cx).is_some()) {
                        self.ensure_active_visible();
                        cx.notify();
                    }
                }
                cx.stop_propagation();
            }
            "pageup" | "pagedown" => {
                if len > 1 {
                    let delta = if event.keystroke.key == "pageup" {
                        -(self.dense_page_rows() as isize)
                    } else {
                        self.dense_page_rows() as isize
                    };
                    let next = (active as isize + delta).clamp(0, len as isize - 1) as usize;
                    if next != active && self.select_tab(next, cx).is_some() {
                        self.ensure_active_visible();
                        cx.notify();
                    }
                }
                cx.stop_propagation();
            }
            "enter" => {
                // DESIGN.md §11.1 / interaction contract: Enter activates the
                // selected tab — the chrome traversal commits and keyboard
                // focus moves into the page (the same outcome as a committed
                // New-Tab command). No-op when no live view is presented
                // (crashed tab or empty space); selection never changes here.
                if self.controller.read().snapshot().active_view.is_some() {
                    self.focus_web(window, cx);
                    #[cfg(feature = "testkit")]
                    {
                        self.performance.expect_state(
                            self.controller.read().snapshot().active_view,
                            self.sidebar_visible,
                            self.chrome_focused(window),
                            self.command_bar_mode.into(),
                        );
                        self.commit_benchmark_sync(window);
                    }
                }
                cx.stop_propagation();
            }
            "backspace" => {
                let _ = self.close_active_tab(window, cx);
                // Closing the final tab already opened the empty New Tab
                // command surface (command-bar focus, NewTab mode — the same
                // outcome as the pointer path). Only a close that leaves tabs
                // behind keeps the list focused so repeated keys keep working.
                if self.controller.read().snapshot().active_view.is_some() {
                    self.focus_tab_list(window, cx);
                }
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    /// DESIGN.md §11.6 selection visibility: after any selection or space
    /// activation, the selected row is scrolled into view **instantly** (no
    /// animated scroll) within the same frame. Runs after every selection
    /// mutation; the pinned region is always visible, and a regular-row
    /// selection outside the rendered window sets the production scroll
    /// offset so the next frame renders it. A no-op while the sidebar is
    /// collapsed (there is no scroll container to move).
    pub(super) fn ensure_active_visible(&self) {
        if !self.sidebar_visible {
            return;
        }
        let browser = self.controller.read().snapshot();
        let Some(active) = browser.active_index else {
            return;
        };
        let pinned = crate::shell::pinned_count(&browser.tabs);
        if active < pinned {
            return; // the pinned block is always on screen
        }
        let regular = active - pinned;
        let total = browser.tabs.len() - pinned;
        // §8 pitch: rows render at surface + gap (`density:pitch`), so
        // scroll-into-view offsets the dense window in exact pitch units
        // (§11.6).
        let pitch = self.theme().density.pitch();
        let anchor = self.dense_scroll.offset().y;
        let current = crate::dense::anchor_from_offset(anchor, pitch, total);
        if !crate::dense::window_contains(&crate::dense::window_for(current, total), regular) {
            let target = crate::dense::clamp_anchor(regular, total);
            self.dense_scroll.set_offset(gpui::point(
                px(0.0),
                crate::dense::offset_for_anchor(target, pitch),
            ));
        }
    }

    /// The dense page step for `PgUp`/`PgDn`: one rendered window of rows
    /// (DESIGN.md §11.6 keyboard scroll moves both selection and viewport by
    /// a page).
    fn dense_page_rows(&self) -> usize {
        crate::dense::DENSE_WINDOW_ROWS.max(1)
    }

    /// Dense-list scroll entry (DESIGN.md §11.6): move the tracked
    /// production scroll state by `delta_px` in the same units and sign as
    /// the native wheel listener (`delta_px < 0` scrolls down; the container
    /// offset is negative as the user scrolls down). This mutates the exact
    /// `ScrollHandle` the wheel listener, the keyboard traversal
    /// (`ensure_active_visible`), and the render share — the render derives
    /// the bounded row window from it identically for a wheel, a `PgDn`, or
    /// this entry. GPUI 0.2.2 exposes no public synthetic wheel dispatch
    /// (`Window::dispatch_event` returns a crate-private result type), so
    /// E2E drives this production scroll state instead; the container's own
    /// prepaint clamps it to the collection extent like any other source.
    #[cfg(feature = "testkit")]
    pub fn scroll_dense_list(&self, delta_px: f32) {
        let current = self.dense_scroll.offset().y;
        self.dense_scroll
            .set_offset(gpui::point(px(0.0), current + px(delta_px)));
    }

    /// Reload the crashed active tab from the crash panel (fresh session).
    pub fn reload_active_from_panel(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        // The crash-panel Reload is the contract's `Restore` recovery for
        // the crash incident the adapter classified: drop the tracked
        // incident so the surface never re-presents a stale crash after the
        // session is rebuilt (the crash panel is the visible surface for
        // the process-crash class).
        let active = self.controller.read().snapshot().active_view;
        if let Some(view) = active {
            self.error_surface.get().clear_view(view);
        }
        self.reload();
        cx.notify();
    }

    /// Reopen the most recently closed tab (`Cmd-Shift-T`). Contract: the
    /// tab keeps its identity/URL/title, receives a fresh native session, and
    /// the focus domain of the invoking command is preserved (chrome stays in
    /// the tab list, web stays in web content).
    pub fn reopen_tab(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<bbr_browser::ControllerEvent> {
        let chrome_focused = self.chrome_focused(window);
        let result = self
            .controller
            .get()
            .reopen_tab()
            .map_err(|error| {
                log::error!("could not reopen tab: {error}");
                error
            })
            .ok();
        self.after_reopen(result, chrome_focused, window, cx)
    }

    /// Blueprint D10: reopen one recently closed tab from History (the
    /// page takes the keyboard afterwards).
    pub fn reopen_closed_tab(
        &mut self,
        tab: bbr_domain::TabId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<bbr_browser::ControllerEvent> {
        let result = self
            .controller
            .get()
            .reopen_closed_tab(tab)
            .map_err(|error| {
                log::error!("could not reopen tab: {error}");
                error
            })
            .ok();
        self.after_reopen(result, false, window, cx)
    }

    fn after_reopen(
        &mut self,
        result: Option<bbr_browser::ControllerEvent>,
        chrome_focused: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<bbr_browser::ControllerEvent> {
        if let Some(outcome) = &result {
            self.clear_command_profile_hint();
            self.sync_command_bar();
            // A fresh session for the reopened tab ends any find
            // presentation bound to the previous active view.
            self.find_discard_if_unbound(cx);
            self.emit_browser(outcome);
            self.sync_peek_surface(cx);
            if chrome_focused {
                self.focus_tab_list(window, cx);
            } else {
                self.focus_web(window, cx);
            }
            self.ensure_active_visible();
            // The reopened tab restores a recently-closed identity into the
            // live model: a structural mutation that must survive a force
            // quit.
            self.persist_structure();
            cx.notify();
        }
        result
    }

    /// Pin the tab at `index` (moves it into the pinned region). Testkit-only
    /// until the pin keyboard command ships.
    #[cfg(feature = "testkit")]
    pub fn pin_tab(
        &mut self,
        index: usize,
        cx: &mut Context<Self>,
    ) -> Option<bbr_browser::ControllerEvent> {
        let result = self
            .controller
            .get()
            .pin_tab(index)
            .map_err(|error| {
                log::error!("could not pin tab {index}: {error}");
                error
            })
            .ok();
        if let Some(outcome) = &result {
            self.clear_command_profile_hint();
            self.sync_command_bar();
            self.emit_browser(outcome);
            // Pin moves the identity into the pinned region: a structural
            // mutation that must survive a force quit.
            self.persist_structure();
            cx.notify();
        }
        result
    }

    /// Toggle the pin state of the tab at `index` through the production
    /// controller path (pin → unpin → pin). The selected tab stays selected
    /// and the live session is preserved. `window` is threaded for symmetry
    /// with the other mutation paths; focus is left untouched so the invoking
    /// focus domain is preserved (interaction contract: tab mutations keep
    /// the domain that initiated them).
    pub fn toggle_pin(&mut self, index: usize, cx: &mut Context<Self>) -> Option<ControllerEvent> {
        let snapshot = self.controller.read().snapshot();
        let pinned = snapshot.tabs.get(index).is_some_and(|tab| tab.pinned);
        let result = if pinned {
            self.controller.get().unpin_tab(index)
        } else {
            self.controller.get().pin_tab(index)
        }
        .map_err(|error| {
            log::error!("could not toggle pin on tab {index}: {error}");
            error
        })
        .ok();
        if let Some(outcome) = &result {
            self.clear_command_profile_hint();
            self.sync_command_bar();
            self.emit_browser(outcome);
            // Pin/unpin changes the pinned flag and the tab order: a
            // structural mutation that must survive a force quit.
            self.persist_structure();
            cx.notify();
        }
        result
    }

    /// `Cmd-Shift-P`: pin or unpin the active tab, preserving the invoking
    /// focus domain and the live session.
    pub fn pin_active_tab(&mut self, cx: &mut Context<Self>) -> Option<ControllerEvent> {
        let index = self.controller.read().snapshot().active_index?;
        self.toggle_pin(index, cx)
    }

    /// `Cmd-Shift-↑`/`Cmd-Shift-↓`: move the active tab one step within its
    /// current region. A step that would cross the pinned/regular boundary is
    /// rejected whole (no partial mutation; the order and sessions stay
    /// untouched), matching the interaction contract's reorder rule. Focus is
    /// left untouched so the invoking focus domain is preserved.
    pub fn reorder_active_tab(
        &mut self,
        delta: isize,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let from = self.controller.read().snapshot().active_index?;
        let to = (from as isize + delta).max(0) as usize;
        self.reorder_tab_at(from, to, cx)
    }

    /// Reorder the tab at `from` to `to` through the production controller
    /// path. The step must stay inside the source region: the domain rejects
    /// a pinned/regular boundary crossing whole (no partial mutation), and
    /// the app additionally refuses to even attempt a step outside the
    /// region so the tick cannot suggest a boundary crossing the domain will
    /// reject. Sessions and selection are untouched.
    pub fn reorder_tab_at(
        &mut self,
        from: usize,
        to: usize,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let snapshot = self.controller.read().snapshot();
        let len = snapshot.tabs.len();
        if len < 2 {
            return None;
        }
        let pinned = crate::shell::pinned_count(&snapshot.tabs);
        // Region-bound: the pinned region is [0, pinned), the regular region
        // [pinned, len). A step outside the source region is the boundary
        // crossing the domain rejects; never wrap around.
        let in_region = if from < pinned {
            (0..pinned).contains(&to)
        } else {
            (pinned..len).contains(&to)
        };
        if !in_region {
            return None;
        }
        let result = self
            .controller
            .get()
            .reorder_tab(from, to)
            .map_err(|error| {
                log::error!("could not reorder tab {from} -> {to}: {error}");
                error
            })
            .ok();
        if let Some(outcome) = &result {
            self.clear_command_profile_hint();
            self.sync_command_bar();
            self.emit_browser(outcome);
            // Blueprint D23: a pane moved out of its block leaves the split.
            self.split_reorder_left_block(to, cx);
            // The tab order is part of the persisted model: poke the one
            // browser-state writer so a force quit recovers the reordered
            // order.
            self.persist_structure();
            cx.notify();
        }
        result
    }

    /// Move the active tab into `space` through the production controller
    /// path: same-profile moves keep the live session; cross-profile moves
    /// recreate it exactly once in the destination profile's store. The
    /// source space's selection is repaired by the domain; chrome focus stays
    /// in the tab list (both invocation paths — the staged move command and
    /// the drag — are chrome interactions).
    pub fn move_active_tab_to_space(
        &mut self,
        space: bbr_browser::SpaceId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let index = self.controller.read().snapshot().active_index?;
        self.move_tab_to_space_at(index, space, window, cx)
    }

    /// Move the tab at `index` into `space` through the production
    /// controller path (the shared mutation of the staged move command and
    /// the pointer drag). Same-profile moves keep the live session;
    /// cross-profile moves recreate it exactly once in the destination
    /// profile's store. The source space's selection is repaired by the
    /// domain; chrome focus stays in the tab list.
    pub fn move_tab_to_space_at(
        &mut self,
        index: usize,
        space: bbr_browser::SpaceId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let from_space = self.controller.read().snapshot().active_space;
        let result = self
            .controller
            .get()
            .move_tab(index, space)
            .map_err(|error| {
                log::error!("could not move tab {index} to {space:?}: {error}");
                error
            })
            .ok();
        // Blueprint D07: a cross-profile move recreated the page; a toast
        // says so, with Undo.
        if let Some(ControllerEvent::TabMoved {
            tab,
            to_space,
            recreated: true,
        }) = &result
        {
            self.announce_profile_move(*tab, from_space, *to_space, cx);
        }
        if let Some(outcome) = &result {
            self.clear_command_profile_hint();
            self.sync_command_bar();
            // A cross-profile move recreates the session; a same-profile
            // move can also change the active view. Either ends a find
            // presentation bound elsewhere.
            self.find_discard_if_unbound(cx);
            self.emit_browser(outcome);
            self.sync_peek_surface(cx);
            self.focus_tab_list(window, cx);
            self.ensure_active_visible();
            // The tab changed space: a structural mutation (space membership
            // and the source space's selection) that must survive a force
            // quit.
            self.persist_structure();
            cx.notify();
        }
        result
    }

    /// Unpin the tab at `index` (moves it to the end of the regular region).
    /// Testkit-only until the unpin keyboard command ships alongside the pin
    /// toggle (`Cmd-Shift-P` toggles; this seam seeds the unpinned state in
    /// E2E).
    #[cfg(feature = "testkit")]
    pub fn unpin_tab(
        &mut self,
        index: usize,
        cx: &mut Context<Self>,
    ) -> Option<bbr_browser::ControllerEvent> {
        let result = self
            .controller
            .get()
            .unpin_tab(index)
            .map_err(|error| {
                log::error!("could not unpin tab {index}: {error}");
                error
            })
            .ok();
        if let Some(outcome) = &result {
            self.clear_command_profile_hint();
            self.sync_command_bar();
            self.emit_browser(outcome);
            // Unpin moves the identity to the end of the regular region: a
            // structural mutation that must survive a force quit.
            self.persist_structure();
            cx.notify();
        }
        result
    }

    /// Reorder the tab at `from` to `to` (E2E semantic seam for setup; the
    /// user paths are the keyboard steps and the pointer drag, both of which
    /// enforce the region boundary before the controller rejects it).
    #[cfg(feature = "testkit")]
    pub fn reorder_tab_seam(
        &mut self,
        from: usize,
        to: usize,
        cx: &mut Context<Self>,
    ) -> Option<bbr_browser::ControllerEvent> {
        let result = self
            .controller
            .get()
            .reorder_tab(from, to)
            .map_err(|error| {
                log::error!("could not reorder tab {from} -> {to}: {error}");
                error
            })
            .ok();
        if let Some(outcome) = &result {
            self.clear_command_profile_hint();
            self.sync_command_bar();
            self.emit_browser(outcome);
            // The reorder is a structural mutation (tab order is persisted):
            // poke the one browser-state writer so the E2E reorder survives a
            // force quit.
            self.persist_structure();
            cx.notify();
        }
        result
    }
}
