//! Ranked result activation (DESIGN.md §11.5): Enter / Cmd-Enter commit the
//! selected row's typed activation — or the typed navigate/search fallback
//! while the debounce is pending — through existing app/controller methods.
//! Profile isolation and warm-session rules remain controller-owned; no
//! engine or session mutation happens here.

use bbr_browser::SpaceId;
use bbr_command::id::ActionId;
use bbr_command::parse::{Destination, Mode, Query};
use bbr_command::rank::Activation;
use bbr_command::search_settings::render_search_url;
use bbr_web::WebAction;
use gpui::{Context, Window};

use crate::command_bar::CommandBarMode;

use super::command_rank::CommandResultState;
use super::BbrApp;

/// The effective search URL for `terms` (Phase 7): the persisted global
/// search-engine settings from `bbr-persistence` feed the existing
/// provider-neutral fallback rendering. The default DuckDuckGo settings
/// render byte-identically to the settled command-bar fallback
/// (`https://duckduckgo.com/?q=<percent-encoded terms>`); a custom engine
/// is constructor-validated, so dangerous `file:`/`javascript:`/`data:`
/// templates cannot exist here. E2E asserts the encoded intent, never the
/// live provider.
pub fn search_url(
    settings: &bbr_command::search_settings::settings::SearchSettingsV1,
    terms: &str,
) -> String {
    render_search_url(settings.engine(), terms)
}

impl BbrApp {
    /// Enter commits the command bar: Location activates the selected result
    /// in the current context; NewTab opens the selected/typed destination in
    /// a new tab; staged NewSpace/MoveTab keep their existing behavior.
    pub fn submit_command(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.command_bar_mode {
            CommandBarMode::Location => {
                let Some(activation) = self.selected_or_typed_activation() else {
                    return;
                };
                self.dispatch_activation(activation, false, window, cx);
            }
            CommandBarMode::NewTab => {
                let Some(activation) = self.selected_or_typed_activation() else {
                    return;
                };
                // Blueprint D04: on an empty ⌘T a recent tab row returns to
                // that tab (⌘T ↵ goes back to the last thing); typed
                // destinations keep the staged new-tab semantics.
                let recent_tab = self.command_text.text.trim().is_empty()
                    && matches!(activation, Activation::SelectTab { .. });
                self.dispatch_activation(activation, !recent_tab, window, cx);
            }
            _ => self.submit_staged_command_bar(window, cx),
        }
    }

    /// Cmd-Enter: open a Location/NewTab destination in a new tab and focus
    /// the new page (exactly one new tab/session); non-destination rows
    /// (spaces/actions) run in the current context through the same
    /// controller-owned methods.
    pub fn submit_command_new_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.command_bar_mode {
            CommandBarMode::Location | CommandBarMode::NewTab => {
                let Some(activation) = self.selected_or_typed_activation() else {
                    return;
                };
                self.dispatch_activation(activation, true, window, cx);
            }
            _ => self.submit_staged_command_bar(window, cx),
        }
    }

    /// The activation to commit: the selected published row, or the typed
    /// navigate/search fallback when nothing is published yet (debounce
    /// pending or empty). `None` only for empty text.
    fn selected_or_typed_activation(&self) -> Option<Activation> {
        if let Some(row) = self
            .command_ranking
            .results
            .get(self.command_ranking.selected)
        {
            return Some(row.activation.clone());
        }
        let mode = match self.command_bar_mode {
            CommandBarMode::Location => Mode::Location,
            CommandBarMode::NewTab => Mode::NewTab,
            _ => return None,
        };
        let query = Query::new(mode, self.command_text.text.trim());
        // Blueprint D04: the `>` actions mode never falls back to a
        // navigation or a search; only a listed action activates.
        if query.mode == Mode::Action {
            return None;
        }
        match query.destination {
            Destination::Navigate(url) => Some(Activation::Navigate { url }),
            Destination::Search(terms) => Some(Activation::Search { terms }),
            Destination::Empty => None,
        }
    }

    /// Route one typed activation through existing app/controller methods.
    /// Returns false when the destination is unavailable, which surfaces the
    /// truthful `ActivationError` state (never a silent no-op).
    fn dispatch_activation(
        &mut self,
        activation: Activation,
        new_tab: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let ok = match activation {
            Activation::SelectTab { tab, .. } => {
                // Persistent identity: locate the target in the complete
                // global tab collection by its domain id, never by an
                // active-space position guess. Routing uses the tab's
                // CURRENT owning space and index from the live `all_tabs`
                // record — the ranked activation's `space` may be stale if
                // the tab moved spaces between ranking and Enter, so it is
                // never used to route (selecting the stale numeric index in
                // the old space could activate an unrelated tab).
                let snapshot = self.controller.snapshot();
                let Some(global) = snapshot.all_tabs.iter().find(|global| global.id.0 == tab.0)
                else {
                    return false;
                };
                if new_tab {
                    // Cmd-Enter on a tab result opens that tab's exact
                    // destination URL in a fresh tab (exactly one new
                    // tab/session) and focuses the new page — same terminal
                    // web activation as the navigate/search destinations.
                    if self.open_tab(&global.url, cx).is_some() {
                        self.finish_web_activation(window, cx);
                        true
                    } else {
                        false
                    }
                } else if global.space == snapshot.active_space {
                    // The tab's current owning space is the active one:
                    // select its exact current index (warm session reuse
                    // stays controller-owned).
                    if self.select_tab(global.index, cx).is_some() {
                        self.finish_web_activation(window, cx);
                        true
                    } else {
                        false
                    }
                } else {
                    // The tab lives in another space (it may have moved
                    // there after ranking): activate its CURRENT owning
                    // space first (it restores its own active tab and warm
                    // sessions through the controller), then re-read the
                    // identity from the post-activation `all_tabs` and
                    // verify the exact persistent id still lives in the
                    // owning space we just activated before selecting its
                    // current index. Profile isolation is controller-owned:
                    // cross-profile activation switches the active profile;
                    // warm sessions stay warm.
                    if self.activate_space(global.space, window, cx).is_none() {
                        return false;
                    }
                    let snapshot = self.controller.snapshot();
                    let current = snapshot.all_tabs.iter().find(|global| global.id.0 == tab.0);
                    match current {
                        Some(current)
                            if current.space == global.space
                                && self.select_tab(current.index, cx).is_some() =>
                        {
                            self.finish_web_activation(window, cx);
                            true
                        }
                        _ => false,
                    }
                }
            }
            Activation::SelectSpace { space } => {
                let activated = self.activate_space(SpaceId(space.0), window, cx).is_some();
                if activated {
                    // `activate_space` keeps the chrome focus domain and
                    // already set the mode to Inactive; close the ranking
                    // surface state.
                    self.command_ranking.close();
                }
                activated
            }
            Activation::OpenHistory { history } => {
                // Phase 7: a ranked history row activates its record — the
                // URL is looked up in the in-memory provider (never SQLite)
                // and opened like any other destination; an unavailable
                // record is rejected truthfully.
                match self.store.history_entry(history) {
                    Some((url, title)) => self.open_destination(&url, &title, new_tab, window, cx),
                    None => false,
                }
            }
            Activation::OpenSavedPage { bookmark } => {
                // B2: a ranked Saved Page row activates through the one
                // app-owned command authority — the record is revalidated in
                // the in-memory provider and exactly one fresh regular tab
                // opens in the currently active Space/Profile. A stale or
                // deleted `BookmarkId` rejects truthfully; an existing
                // matching tab is never silently selected.
                let command = self.saved_pages;
                command.open(self, bbr_command::id::BookmarkId(bookmark.0), window, cx)
            }
            Activation::RunAction { action } => self.run_registry_action(action, window, cx),
            Activation::Navigate { url } => {
                // Every destination handed to a web session must be
                // absolute: WebKit rejects a scheme-less URL before any
                // provisional navigation starts, leaving the fresh view
                // blank (the schemeless-nav regression: Cmd-T → "golem.de"
                // → Enter rendered white while the scheme'd twin loaded).
                // `open_destination` normalizes (idempotent for
                // already-absolute URLs), so NewTab and Location deliver
                // the identical absolute URL to the controller.
                self.open_destination(&url, "", new_tab, window, cx)
            }
            Activation::Search { terms } => {
                // The persisted global search-engine settings feed the
                // provider-neutral fallback rendering (Phase 7): the
                // default DuckDuckGo settings render the settled command-bar
                // endpoint byte-identically, and a custom template is
                // constructor-validated.
                let url = search_url(self.store.effective_settings(), &terms);
                self.open_destination(&url, "", new_tab, window, cx)
            }
        };
        if !ok {
            self.command_activation_error(window, cx);
        }
        ok
    }

    /// Opens `url` in the current or a new tab and, when accepted, records
    /// the visit in the app-owned history provider (Phase 7): the in-memory
    /// provider is updated on the main thread and the SQLite write is a
    /// bounded poke on the background writer — never on the render path.
    /// Returns false when the destination could not be opened.
    fn open_destination(
        &mut self,
        url: &str,
        title: &str,
        new_tab: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let url = bbr_web::normalize_url(url);
        if url.is_empty() {
            return false;
        }
        let accepted = if new_tab {
            if self.open_tab(&url, cx).is_some() {
                self.finish_web_activation(window, cx);
                true
            } else {
                false
            }
        } else {
            self.navigate(&url, window, cx);
            true
        };
        if accepted {
            self.store.record_visit(&url, title);
        }
        accepted
    }

    /// Run a registered action through the existing app/controller method.
    /// Staged actions (New Tab / New Space / Move Tab) enter their staged
    /// mode and return `true` without closing the surface; the rest are
    /// terminal and close it.
    fn run_registry_action(
        &mut self,
        action: ActionId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        match action.0 {
            1 => {
                // New Tab: enter the staged NewTab mode (no tab yet).
                self.activate_new_tab(window, cx);
                true
            }
            2 => {
                // Close Tab (destructive; no confirmation dialog in v1) is a
                // deliberate terminal action: `close_active_tab` routes the
                // aftermath (empty space → staged new-tab surface; otherwise
                // the chrome domain with the neighbor selected), and the
                // result surface is dismissed — re-ranking for the closed
                // context is never a thing.
                let closed = self.close_active_tab(window, cx).is_some();
                if closed && self.controller.snapshot().active_view.is_some() {
                    // Non-empty aftermath: the palette closes and the chrome
                    // domain keeps focus in the tab list (the initiating
                    // focus domain was chrome/command).
                    self.command_bar_mode = CommandBarMode::Inactive;
                    self.dismiss_command_portal(cx);
                    self.command_ranking.close();
                    self.focus_tab_list(window, cx);
                }
                closed
            }
            3 => {
                let reopened = self.reopen_tab(window, cx).is_some();
                if reopened {
                    self.finish_action_activation(window, cx);
                }
                reopened
            }
            4 => {
                let pinned = self.pin_active_tab(cx).is_some();
                if pinned {
                    self.finish_action_activation(window, cx);
                }
                pinned
            }
            5 => {
                // Move Tab to Space: enter the staged MoveTab mode.
                self.activate_move_tab(window, cx);
                true
            }
            6 => {
                // New Space: enter the staged NewSpace mode.
                self.activate_new_space(window, cx);
                true
            }
            7 => {
                // Back: rejected truthfully when the active session has no
                // back history (the nav flags are engine-authoritative).
                if self.controller.snapshot().nav.can_go_back {
                    self.go_back();
                    self.finish_web_activation(window, cx);
                    true
                } else {
                    false
                }
            }
            8 => {
                // Forward: rejected truthfully when there is no forward
                // history.
                if self.controller.snapshot().nav.can_go_forward {
                    self.go_forward();
                    self.finish_web_activation(window, cx);
                    true
                } else {
                    false
                }
            }
            9 => {
                // Reload: the controller's navigate result governs success
                // (no active tab/session → rejected, never a silent no-op).
                if self.controller.navigate(WebAction::Reload).is_ok() {
                    self.finish_web_activation(window, cx);
                    true
                } else {
                    false
                }
            }
            10 => {
                // Stop: the controller's navigate result governs success.
                if self.controller.navigate(WebAction::Stop).is_ok() {
                    self.finish_web_activation(window, cx);
                    true
                } else {
                    false
                }
            }
            11 => {
                self.toggle_sidebar(window, cx);
                self.finish_action_activation(window, cx);
                true
            }
            12 => {
                let reordered = self.reorder_active_tab(-1, cx).is_some();
                if reordered {
                    self.finish_action_activation(window, cx);
                }
                reordered
            }
            13 => {
                let reordered = self.reorder_active_tab(1, cx).is_some();
                if reordered {
                    self.finish_action_activation(window, cx);
                }
                reordered
            }
            14 => {
                // Set as Default Browser (queue item 13): a user-triggered
                // LaunchServices claim for http/https under the main bundle
                // identifier — never invoked automatically. The request is
                // terminal; its result is reported through the app log.
                self.request_default_browser();
                self.finish_action_activation(window, cx);
                true
            }
            _ => false,
        }
    }

    /// A destination activation that ends in web content: close the surface
    /// and move focus into the page.
    pub(super) fn finish_web_activation(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_web(window, cx);
        // The sync commit must observe the *post-focus* state: focus_web has
        // already cleared the mode and blurred the chrome, so the probe sees
        // the real presented state (web focus, inactive mode) and a wrong
        // destination/focus/mode is rejected inside `commit_presentation`.
        #[cfg(feature = "testkit")]
        {
            // While a command benchmark is pending, the benchmark driver owns
            // the commit: it must validate caller-specific destination/session
            // state (fresh view, tab/session counts) before any sample leaves
            // the process. The generic commit would otherwise emit before that
            // validation; with no pending benchmark the normal path stays.
            if !self.performance.command_benchmark_pending() {
                self.performance.expect_state(
                    self.controller.snapshot().active_view,
                    self.sidebar_visible,
                    false,
                    self.command_bar_mode.into(),
                );
                self.commit_benchmark_sync(window);
            }
        }
    }

    /// A terminal action activation that keeps the chrome focus domain:
    /// close the surface state (mode Inactive, ranking closed) without
    /// moving focus.
    #[cfg_attr(not(feature = "testkit"), allow(unused_variables))]
    fn finish_action_activation(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
        self.command_bar_mode = CommandBarMode::Inactive;
        self.command_bar_return = None;
        self.dismiss_command_portal(cx);
        self.command_ranking.close();
        cx.notify();
    }

    /// A rejected activation surfaces the truthful `ActivationError` state
    /// with a stable reason; the surface stays open so the user can edit.
    fn command_activation_error(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.command_ranking.activation_error = Some("destination is unavailable".to_string());
        self.command_ranking.state = CommandResultState::ActivationError;
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_command::search_settings::settings::SearchSettingsV1;

    #[test]
    fn search_fallback_renders_engine_encoded_terms() {
        // The default (DuckDuckGo) settings render byte-identically to the
        // settled command-bar fallback; the settings feed the provider-
        // neutral rendering, so the encoded intent is asserted, never the
        // live provider.
        let defaults = SearchSettingsV1::defaults();
        assert_eq!(
            search_url(&defaults, "hello world"),
            "https://duckduckgo.com/?q=hello%20world"
        );
        assert_eq!(
            search_url(&defaults, "a&b=c"),
            "https://duckduckgo.com/?q=a%26b%3Dc"
        );
        assert_eq!(
            search_url(&defaults, "ümlaut"),
            "https://duckduckgo.com/?q=%C3%BCmlaut"
        );
        // Unreserved characters stay readable.
        assert_eq!(
            search_url(&defaults, "a-z_0.9~"),
            "https://duckduckgo.com/?q=a-z_0.9~"
        );
        // A persisted custom engine feeds the same provider-neutral renderer
        // (constructor-validated, so no dangerous template can exist).
        let custom = SearchSettingsV1::with_engine(
            bbr_command::search_settings::engine::SearchEngine::Custom(
                bbr_command::search_settings::engine::CustomUrlTemplate::new(
                    "https://search.example.test/?q=%s&lang=en",
                )
                .unwrap(),
            ),
        );
        assert_eq!(
            search_url(&custom, "hello world"),
            "https://search.example.test/?q=hello%20world&lang=en"
        );
    }
}
