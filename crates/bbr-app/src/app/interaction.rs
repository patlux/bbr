//! Command-bar, navigation, and native-focus choreography.

use bbr_web::{WebAction, WebEvent};
use gpui::{Context, KeyDownEvent, Window};

use crate::app::palette::CommandSurface;
use crate::command_bar::{CommandBarMode, CommandBarReturn};

use super::BbrApp;

impl BbrApp {
    /// Navigate the active view (used by location-mode Enter and the E2E gateway).
    pub fn navigate(&mut self, input: &str, window: &mut Window, cx: &mut Context<Self>) {
        let url = bbr_web::normalize_url(input);
        if url.is_empty() {
            return;
        }
        if self
            .controller
            .navigate(WebAction::Load { url: url.clone() })
            .is_err()
        {
            return;
        }
        self.command_text.set_text(url);
        self.focus_web(window, cx);
    }

    /// Staged-mode submit (NewSpace / MoveTab): the unchanged staged behavior
    /// — Enter commits the staged name/move, Escape cancels. Location/NewTab
    /// route through the ranked result activation instead (`submit_command`).
    pub(super) fn submit_staged_command_bar(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let input = self.command_text.text.clone();
        match self.command_bar_mode {
            CommandBarMode::Inactive | CommandBarMode::Location | CommandBarMode::NewTab => {}
            CommandBarMode::NewSpace => {
                // Create the space in the active profile and activate it;
                // chrome focus stays in the space strip (a fresh space is
                // empty, so there is no page to hand focus to).
                let name = input.trim().to_string();
                if name.is_empty() {
                    return;
                }
                self.command_text.set_text("");
                self.command_bar_return = None;
                if self.commit_new_space(&name, window, cx).is_some() {
                    #[cfg(feature = "testkit")]
                    {
                        self.performance.expect_state(
                            self.controller.snapshot().active_view,
                            self.sidebar_visible,
                            true,
                            self.command_bar_mode.into(),
                        );
                        self.commit_benchmark_sync(window);
                    }
                }
            }
            CommandBarMode::MoveTab => {
                // Move the active tab into the destination space named by the
                // staged text (exact title match, else the first space whose
                // title starts with it). Same-profile moves keep the session;
                // cross-profile moves recreate it once in the destination
                // store. Chrome focus stays in the tab list.
                let Some((space, _, _)) = self.resolve_move_space(&input) else {
                    return;
                };
                self.command_text.set_text("");
                self.command_bar_return = None;
                if self.move_active_tab_to_space(space, window, cx).is_some() {
                    #[cfg(feature = "testkit")]
                    {
                        self.performance.expect_state(
                            self.controller.snapshot().active_view,
                            self.sidebar_visible,
                            self.chrome_focused(window),
                            self.command_bar_mode.into(),
                        );
                        self.commit_benchmark_sync(window);
                    }
                }
            }
        }
    }

    pub fn go_back(&mut self) {
        let _ = self.controller.navigate(WebAction::Back);
    }

    pub fn go_forward(&mut self) {
        let _ = self.controller.navigate(WebAction::Forward);
    }

    pub fn reload(&mut self) {
        let _ = self.controller.navigate(WebAction::Reload);
    }

    /// Apply an engine event to the owning controller tab and refresh chrome.
    pub fn on_web_event(&mut self, event: WebEvent, window: &mut Window, cx: &mut Context<Self>) {
        // A new navigation started on the bound find view: the current
        // results are stale (the adapter drops the old document's in-flight
        // completions by correlation; the portable session rejects further
        // steps until a fresh query). Page-initiated navigations and the
        // WebAction back/forward/reload paths both surface here.
        if let WebEvent::LoadingChanged(view, true) = &event {
            if self.find.state.open && self.find.bound_view == Some(*view) {
                self.find_navigation_invalidated(cx);
            }
            // A recovery in flight for the view completes when the retried
            // navigation starts: the incident lifecycle moves to Recovered,
            // the surface clears, and the native view we hid is restored
            // (identity untouched). If the retry fails again, the adapter
            // reports a fresh incident that supersedes this one.
            if let Some(action) = self.error_surface.recovery_in_flight(*view) {
                if self.error_surface.recovered(*view, action) {
                    self.sync_error_surface_visibility();
                    // The incident cleared: the active view may now be
                    // reported unresponsive, so the §11.8a pane
                    // re-evaluates.
                    self.sync_unresponsive_visibility();
                }
            }
        }
        // A navigation landed on a view: the per-site zoom level follows the
        // destination origin (same site keeps its level; a different site
        // switches to its own; a fresh view restores the stored level).
        if let WebEvent::UrlChanged(view, url) = &event {
            self.zoom_navigation(*view, url);
            // Blueprint D17: the first navigation ends the restore notice.
            self.restore_notice_navigation(url, cx);
        }
        // A right-click over the active view was classified BBR-owned (page
        // background, not a WebKit interactive element): present the BBR
        // context menu through the one native portal host, anchored at the
        // pointer (queue item 12, part B).
        if let WebEvent::ContextMenuRequested(view, point) = &event {
            self.menu_open_from_event(*view, *point, window, cx);
        }
        // D5: when this event ends the active tab's load (commit, finish,
        // failure or crash all clear the flag), the address line plays its
        // fill-and-fade after the state is applied.
        let loading_before = self.active_load();
        let _ = self.controller.apply_event(event.clone());
        if let Some(view) = loading_before {
            if self.active_load().is_none() {
                self.note_load_finished(view, cx);
            }
        }
        if !self.command_bar_mode.is_active() {
            self.sync_command_bar();
        }
        // An engine event can change the renderer availability (a probe
        // answer, a fail-closed timeout, a termination): re-assert the
        // §11.8a pane visibility for the active view. A freshly unresponsive
        // active view hands focus to the pane's Wait control only when the
        // user is not operating chrome (the sidebar / command bar stay
        // usable while the page hangs).
        let active_view = self.controller.snapshot().active_view;
        if let WebEvent::UnresponsiveChanged(view, true) = &event {
            if active_view == Some(*view) && !self.chrome_focused(window) {
                window.focus(&self.unresponsive_wait_handle, cx);
                self.emit_focus(window);
            }
        }
        self.sync_unresponsive_visibility();
        #[cfg(feature = "testkit")]
        self.performance.note_web_event(&event);
        cx.notify();
    }

    /// `Cmd-L`: edit the active tab's location. The exact origin is captured
    /// before the URL is selected so Escape restores the pre-edit text and
    /// the originating focus domain (DESIGN §11.5 / interaction contract).
    /// The origin transient is captured once: a repeated `Cmd-L` while
    /// Location mode is already active re-selects the live URL without
    /// overwriting the original return point, so Escape still restores the
    /// original web/chrome origin (not an intermediate edit state).
    pub fn activate_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // A rail-mode entry replaces any open find presentation (exactly one
        // active portal surface per window); the command surface owns the
        // focus afterwards.
        self.find_discard(cx);
        self.clear_command_profile_hint();
        // A mode entry preserves the surface presentation only when its
        // coupled mode is re-entered (the overlay in Location); any other
        // presentation closes back to the in-rail command bar (DESIGN.md
        // §11.4 mutual exclusion).
        self.command_surface = self
            .command_surface
            .coupled_surface(CommandBarMode::Location);
        if self.command_bar_mode != CommandBarMode::Location {
            self.command_bar_return = Some(CommandBarReturn {
                mode: self.command_bar_mode,
                text: self.command_text.text.clone(),
                chrome_focused: self.chrome_focused(window),
            });
        }
        self.command_bar_mode = CommandBarMode::Location;
        self.sync_command_bar();
        // Cmd-L selects the whole active URL; typing replaces the selection.
        self.command_text.select_all();
        self.open_command_ranking(cx);
        self.focus_command_bar(window, cx);
    }

    /// `Cmd-T`: stage a destination without creating a tab or WKWebView.
    pub fn activate_new_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.find_discard(cx);
        self.clear_command_profile_hint();
        // A mode entry preserves the surface presentation only when its
        // coupled mode is re-entered (the centered palette in New Tab
        // staging); any other presentation closes back to the in-rail
        // command bar (DESIGN.md §11.4 mutual exclusion).
        self.command_surface = self.command_surface.coupled_surface(CommandBarMode::NewTab);
        if self.command_bar_mode != CommandBarMode::NewTab {
            self.command_bar_return = Some(CommandBarReturn {
                mode: self.command_bar_mode,
                text: self.command_text.text.clone(),
                chrome_focused: self.chrome_focused(window),
            });
        }
        self.command_bar_mode = CommandBarMode::NewTab;
        self.command_text.set_text("");
        self.open_command_ranking(cx);
        self.focus_command_bar(window, cx);
        #[cfg(feature = "testkit")]
        {
            self.performance.expect_state(
                self.controller.snapshot().active_view,
                self.sidebar_visible,
                true,
                self.command_bar_mode.into(),
            );
            self.commit_benchmark_sync(window);
        }
    }

    /// `Cmd-Shift-M`: stage a destination space for the active tab without
    /// moving anything until Enter commits it. The staged text is matched
    /// against space titles; Escape restores the previous command-bar state
    /// and focus domain exactly like the other staged modes.
    pub fn activate_move_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.controller.snapshot().active_index.is_none() {
            return;
        }
        // A rail mode entry replaces any open find presentation.
        self.find_discard(cx);
        // A rail mode entry replaces any open palette/overlay presentation
        // (the surface state couples to Location/NewTab only).
        self.dismiss_command_portal(cx);
        // The staged move has its own hint (command_bar_hint); the §11.3
        // space-switch transient must not coexist with it.
        self.clear_command_profile_hint();
        if self.command_bar_mode != CommandBarMode::MoveTab {
            self.command_bar_return = Some(CommandBarReturn {
                mode: self.command_bar_mode,
                text: self.command_text.text.clone(),
                chrome_focused: self.chrome_focused(window),
            });
        }
        self.command_bar_mode = CommandBarMode::MoveTab;
        self.command_text.set_text("");
        self.command_ranking.close();
        self.focus_command_bar(window, cx);
        #[cfg(feature = "testkit")]
        {
            self.performance.expect_state(
                self.controller.snapshot().active_view,
                self.sidebar_visible,
                true,
                self.command_bar_mode.into(),
            );
            self.commit_benchmark_sync(window);
        }
    }

    /// Resolve a staged move destination: the space whose title matches the
    /// text exactly (trimmed), else the first space whose title starts with
    /// it. Returns the space id, its title, and its profile id.
    pub(super) fn resolve_move_space(
        &self,
        text: &str,
    ) -> Option<(bbr_browser::SpaceId, String, bbr_web::ProfileId)> {
        let needle = text.trim();
        if needle.is_empty() {
            return None;
        }
        let snapshot = self.controller.snapshot();
        let exact = snapshot
            .spaces
            .iter()
            .find(|space| space.title.trim() == needle);
        let matched = exact.or_else(|| {
            snapshot
                .spaces
                .iter()
                .find(|space| space.title.trim().starts_with(needle))
        })?;
        let profile = snapshot.profiles.iter().find(|p| p.id == matched.profile)?;
        Some((
            bbr_browser::SpaceId(matched.id.0),
            matched.title.clone(),
            profile.id,
        ))
    }

    /// The §11.3 command-field destination hint for a staged move: "→ Space"
    /// within the active profile, "in Profile · <profile> → Space" when the
    /// destination crosses a profile boundary (the contract's cross-context
    /// communication). `None` when no move is staged or nothing matches.
    /// Rendered in the command field so the user sees the destination before
    /// committing.
    pub(super) fn command_bar_hint(&self) -> Option<String> {
        if self.command_bar_mode != CommandBarMode::MoveTab {
            return None;
        }
        let (_, title, profile) = self.resolve_move_space(&self.command_text.text)?;
        if profile == self.controller.active_profile() {
            Some(format!("→ {title}"))
        } else {
            let snapshot = self.controller.snapshot();
            let profile_title = snapshot
                .profiles
                .iter()
                .find(|p| p.id == profile)
                .map(|p| p.title.clone())
                .unwrap_or_else(|| "other".to_string());
            Some(format!("in Profile · {profile_title} → {title}"))
        }
    }

    pub(super) fn enter_empty_new_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_bar_return = None;
        self.command_bar_mode = CommandBarMode::NewTab;
        // The empty-pane surface is the in-rail command bar (§11.8), not a
        // palette/overlay presentation.
        self.dismiss_command_portal(cx);
        self.command_text.set_text("");
        self.open_command_ranking(cx);
        self.focus_command_bar(window, cx);
    }

    pub(super) fn focus_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.chrome_handle, cx);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
    }

    /// DESIGN.md §10.2 production hover path: the pointer entered row
    /// `index` (`hovered == true`) or left it (`false`). This is the exact
    /// app method the row's `on_hover` element listener calls, so the
    /// testkit seam drives the same path a real pointer move runs.
    pub fn set_row_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        let changed = if hovered {
            self.hovered_row != Some(index)
        } else {
            self.hovered_row == Some(index)
        };
        self.hovered_row = if hovered { Some(index) } else { None };
        if changed {
            cx.notify();
        }
    }

    /// Clear the hover state regardless of the row (the `UnhoverTabRow` seam
    /// and a pointer leaving the window).
    pub fn clear_row_hover(&mut self, cx: &mut Context<Self>) {
        if self.hovered_row.take().is_some() {
            cx.notify();
        }
    }

    /// DESIGN.md §10.2/§11.11 production hover path: the pointer entered
    /// folder row `id` (`hovered == true`) or left it (`false`). The exact
    /// app method the folder row's `on_hover` element listener calls, so
    /// the hover/focus-revealed count follows the same production pointer
    /// path as the row hover wash.
    pub fn set_folder_hover(
        &mut self,
        id: bbr_domain::FolderId,
        hovered: bool,
        cx: &mut Context<Self>,
    ) {
        let changed = if hovered {
            self.hovered_folder != Some(id)
        } else {
            self.hovered_folder == Some(id)
        };
        self.hovered_folder = if hovered { Some(id) } else { None };
        if changed {
            cx.notify();
        }
    }

    /// Testkit pointer-leave seam: clear the folder hover so the revealed
    /// count returns quiet. Production leave events use
    /// `set_folder_hover(id, false)` through the folder callback.
    #[cfg(feature = "testkit")]
    pub fn clear_folder_hover(&mut self, cx: &mut Context<Self>) {
        if self.hovered_folder.take().is_some() {
            cx.notify();
        }
    }

    /// DESIGN.md §10.3 production press path: the row's primary button went
    /// down. The exact app method the row's mouse-down listener calls.
    pub fn set_row_pressed(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.pressed_row != Some(index) {
            self.pressed_row = Some(index);
            cx.notify();
        }
    }

    /// DESIGN.md §10.3 production release path: the primary button went up
    /// (or left the window). The exact app method the row's mouse-up / the
    /// root's mouse-up-out listeners call.
    pub fn clear_row_pressed(&mut self, cx: &mut Context<Self>) {
        if self.pressed_row.take().is_some() {
            cx.notify();
        }
    }

    /// Escape cancels the command surface: location mode restores the exact
    /// origin text and the originating focus domain (web or chrome, as
    /// captured at Cmd-L); staged tab/space/move modes restore their captured
    /// prior state. An active tab drag is cancelled first (chrome focus stays
    /// in the tab list), an open plus menu is dismissed without action
    /// (§11.4 — the menu's own focused handler stops propagation, this is the
    /// fallback for any Escape that reaches the app while the menu is open),
    /// and an open downloads panel is dismissed (§11.12 — the panel's own
    /// key handler can never see Escape, because GPUI dispatches bound
    /// actions first and actions stop propagation, so this route is the
    /// panel's primary Escape dismissal).
    pub fn cancel_command_bar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Blueprint D14: Escape closes the Back/Forward history menu.
        if self.nav_history_open() {
            self.nav_history_close(window, cx);
            return;
        }
        // Blueprint D05: Escape cancels the Delete Space sheet (the global
        // binding routes here before the sheet's key handler).
        if self.space_delete_open() {
            self.space_delete_cancel(window, cx);
            return;
        }
        // Blueprint D20: Escape during an inline folder rename keeps the
        // committed title (the global `escape → FocusWeb` binding routes
        // here before the region's key handler could see it).
        if self.folder_rename_open() {
            self.cancel_folder_rename(cx);
            return;
        }
        // Escape while the Saved Pages Library is open closes it and
        // restores the exact native view the Library hid (DESIGN.md §11.16;
        // the global `escape → FocusWeb` binding routes here).
        if self.library.open {
            self.close_saved_pages_library(window, cx);
            return;
        }
        // Escape while a credential sheet is open cancels the challenge
        // through the auth contract: the request fails and never retries
        // (the global `escape → FocusWeb` binding routes here).
        if self.auth_sheet_is_presented() {
            self.auth_cancel(window, cx);
            return;
        }
        // Escape while a transient Peek preview is open dismisses it: the
        // preview session is destroyed and focus returns to the exact warm
        // origin (the global `escape → FocusWeb` binding routes here; the
        // preview owns browser-command focus while open).
        if self.controller.preview_open() {
            self.peek_close(window, cx);
            return;
        }
        // Escape while a context menu is open closes it and restores the
        // origin focus domain (queue item 12, part B; the global
        // `escape → FocusWeb` binding routes here).
        if self.context_menu.state.open {
            self.menu_close(window, cx);
            return;
        }
        // Escape while the Tab/Favorite menu is open closes it and restores
        // the origin focus domain (task packet `tab-favorite-context-menus`;
        // the same global binding routes here).
        if self.tab_menu.state.open {
            self.tab_menu_close(window, cx);
            return;
        }
        // Escape while a find presentation is open closes it and restores
        // the origin focus domain (the global `escape → FocusWeb` binding
        // routes here).
        if self.find.state.open {
            self.find_close(window, cx);
            return;
        }
        // Escape while the §11.3 Space-configuration surface is open cancels
        // it without applying the draft: the created Space and its valid
        // defaults stay, and chrome focus returns to the permanent space
        // strip (the global `escape → FocusWeb` binding routes here; the
        // surface's own key handler can never see Escape, because GPUI
        // dispatches bound actions first and actions stop propagation, so
        // this route is the surface's primary Escape dismissal).
        if self.space_config.open {
            self.space_config_cancel(window, cx);
            return;
        }
        if self.tab_drag.is_active() {
            self.cancel_tab_drag(window, cx);
            return;
        }
        if self.u2.plus_menu.open {
            self.close_plus_menu(window, cx);
            return;
        }
        // DESIGN.md §11.12 Dismiss: Escape dismisses the downloads panel
        // unconditionally (independent of content — the empty panel
        // included) and returns focus to the footer control.
        if self.downloads.open {
            crate::app::downloads_behavior::downloads_close(self, window, cx);
            return;
        }
        // Escaping a palette/overlay presentation closes it (DESIGN.md
        // §11.4); the restoration below returns to the captured origin
        // surface state, which never includes an open presentation.
        self.dismiss_command_portal(cx);
        if !matches!(
            self.command_bar_mode,
            CommandBarMode::Location
                | CommandBarMode::NewTab
                | CommandBarMode::NewSpace
                | CommandBarMode::MoveTab
        ) {
            // No open command surface: the Escape binding still hands focus
            // to the page (the pre-composition behavior).
            self.focus_web(window, cx);
            return;
        }
        if let Some(restore) = self.command_bar_return.take() {
            // Restore the exact origin mode and text, re-open the ranking
            // surface only when the restored mode is a ranking mode, then
            // hand focus back to the captured origin domain. A restored
            // Location origin re-presents the field-anchored overlay: the
            // §11.5 address control is display-only, so address editing
            // always lives in the overlay — never back in the sidebar rail.
            self.command_bar_mode = restore.mode;
            self.command_text.set_text(restore.text);
            if self.command_bar_mode == CommandBarMode::Location {
                // The overlay re-presents with the restored URL selected, so
                // one keystroke replaces it (the §11.4 open contract).
                self.command_surface = CommandSurface::Overlay;
                self.command_text.select_all();
                self.present_command_portal(CommandSurface::Overlay, window, cx);
                self.open_command_ranking(cx);
            } else if self.command_bar_mode == CommandBarMode::NewTab {
                self.open_command_ranking(cx);
            } else {
                self.command_ranking.close();
            }
            if restore.chrome_focused {
                self.focus_command_bar(window, cx);
            } else {
                self.focus_web(window, cx);
            }
        } else if self.controller.snapshot().active_view.is_some() {
            self.sync_command_bar();
            self.focus_web(window, cx);
        }
    }

    /// Move GPUI focus + AppKit first responder back to the web view and
    /// close the ranking surface (a command-surface exit path). While a
    /// transient preview is open the preview session owns web focus.
    pub fn focus_web(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_bar_mode = CommandBarMode::Inactive;
        self.command_bar_return = None;
        self.dismiss_command_portal(cx);
        self.command_ranking.close();
        window.blur();
        let target = self
            .controller
            .snapshot()
            .preview
            .map(|preview| preview.view)
            .or_else(|| self.controller.snapshot().active_view);
        if let Some(view) = target {
            self.engine.borrow_mut().focus_native_web(view);
        }
        cx.notify();
        self.emit_focus(window);
    }

    /// GPUI chrome gained focus without activating the command bar (click):
    /// a pointer action outside the open palette/overlay dismisses it
    /// (DESIGN.md §13 — one pointer action), and the in-rail command surface
    /// closes like the pre-composition click path.
    pub(super) fn focus_chrome(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_bar_mode = CommandBarMode::Inactive;
        self.command_bar_return = None;
        self.dismiss_command_portal(cx);
        self.command_ranking.close();
        window.focus(&self.chrome_handle, cx);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
    }

    /// DESIGN.md §11.5 production hover path: the pointer entered the
    /// address pill (`hovered == true`) or left it (`false`). This is the
    /// exact app method the pill's `on_hover` element listener calls, so
    /// the testkit seam drives the same path a real pointer move runs.
    pub fn set_address_hover(&mut self, hovered: bool, cx: &mut Context<Self>) {
        if self.address_hovered != hovered {
            self.address_hovered = hovered;
            cx.notify();
        }
    }

    /// The live address the §11.5 pill derives its domain from: the nav URL
    /// of the active session, falling back to the active tab's URL (the
    /// same single source `sync_command_bar` mirrors into the input model).
    pub(super) fn active_url(&self) -> String {
        let snapshot = self.controller.snapshot();
        snapshot
            .nav
            .url
            .or_else(|| {
                snapshot
                    .active_index
                    .and_then(|index| snapshot.tabs.get(index).map(|tab| tab.url.clone()))
            })
            .unwrap_or_default()
    }

    pub(super) fn sync_command_bar(&mut self) {
        self.command_text.set_text(self.active_url());
    }

    /// Handle printable typing + editing/selection keys while the command bar
    /// is active. Every mutation routes through the UTF-8-safe text model
    /// (replace-selection typing, selection-first backspace, horizontal
    /// cursor movement) and restarts the cancellable ranking debounce.
    pub(super) fn on_chrome_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // A credential sheet owns the keyboard while it is open: printable
        // chars edit the focused field, Tab moves between fields, Enter
        // submits, Escape cancels (the sheet routes every decision through
        // the merged auth contract).
        if self.auth_sheet_is_presented() {
            self.on_auth_key_down(event, window, cx);
            return;
        }
        // Blueprint D05: the Delete Space sheet owns the keyboard while it
        // is open (Return triggers the focused button, Tab moves).
        if self.space_delete_open() {
            self.on_space_delete_key_down(event, window, cx);
            return;
        }
        // A context-menu presentation owns the keyboard while it is open:
        // Up/Down move the cursor (skipping disabled rows), Enter activates,
        // Escape dismisses (queue item 12, part B).
        if self.context_menu.state.open {
            self.on_menu_key_down(event, window, cx);
            return;
        }
        // Blueprint D14: the Back/Forward history menu owns the keyboard.
        if self.nav_history_open() {
            self.on_nav_history_key_down(event, window, cx);
            return;
        }
        // The Tab/Favorite menu owns the keyboard while it is open: Up/Down
        // move the cursor (top-level or inside the open submenu), Right/Left
        // enter/leave submenus, Enter commits, Escape dismisses (task packet
        // `tab-favorite-context-menus`).
        if self.tab_menu.state.open {
            self.on_tab_menu_key_down(event, window, cx);
            return;
        }
        // A find presentation owns the keyboard while it is open: printable
        // chars edit the query, Enter/Shift-Enter step, Escape closes.
        if self.find.state.open {
            self.on_find_key_down(event, window, cx);
            return;
        }
        // The §11.3 Space-configuration surface owns the keyboard while it
        // is open: printable chars edit the rename draft, Backspace deletes,
        // Enter commits, Escape cancels.
        if self.space_config.open {
            self.on_space_config_key_down(event, window, cx);
            return;
        }
        // The Saved Pages Library owns the keyboard while it is open:
        // printable chars edit the search query, backspace deletes, Escape
        // closes (handled by the bound action before this handler).
        if self.library.open {
            self.on_library_key_down(event, window, cx);
            return;
        }
        if !self.command_bar_mode.is_active() {
            return;
        }
        let ks = &event.keystroke;
        match ks.key.as_str() {
            "backspace" => {
                self.command_text.backspace();
                self.on_command_text_changed(cx);
                cx.stop_propagation();
            }
            "enter" => {
                cx.stop_propagation();
                if ks.modifiers.platform {
                    self.submit_command_new_tab(window, cx);
                } else {
                    self.submit_command(window, cx);
                }
            }
            // Up/Down select results without moving focus out of the field.
            "up" | "down" => {
                let delta = if ks.key == "up" { -1 } else { 1 };
                self.command_move_selection(delta, cx);
                cx.stop_propagation();
            }
            // Horizontal cursor movement (UTF-8-safe; collapses any
            // selection in the direction of movement).
            "left" | "right" => {
                if ks.key == "left" {
                    self.command_text.move_left();
                } else {
                    self.command_text.move_right();
                }
                cx.stop_propagation();
            }
            _ => {
                if let Some(ch) = ks.key_char.as_deref() {
                    if !ch.is_empty() {
                        self.command_text.replace_selection(ch);
                        self.on_command_text_changed(cx);
                        cx.stop_propagation();
                    }
                }
            }
        }
        cx.notify();
    }
}
