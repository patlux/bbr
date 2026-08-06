//! GPUI rendering for the browser shell.
//!
//! The sidebar renders the brand, command bar, the controller-backed tab
//! surface (pinned + regular regions), status line, and the collapse control
//! next to the native web-view slot. All visuals come from the Chassis tokens
//! and components (DESIGN.md); the tab rows are built from the controller
//! snapshot, never from a second navigation-state copy.

use gpui::{
    div, prelude::*, px, Context, Div, MouseButton, MouseDownEvent, Render, SharedString, Stateful,
    Window,
};

use crate::components::{self, RowVisuals};
use crate::slot::NativeWebViewSlot;
use crate::theme::{Spacing, Theme};

use super::BbrApp;

impl Render for BbrApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme();
        let p = theme.palette;
        let chrome_focused = self.chrome_handle.is_focused(window);
        let browser = self.controller.snapshot();
        let status_text: SharedString = crate::shell::status_line(&browser).into();
        #[cfg(feature = "testkit")]
        self.performance.observe_render_state(
            browser.active_view,
            self.sidebar_visible,
            chrome_focused,
            self.command_bar_mode.into(),
        );

        let pinned_count = crate::shell::pinned_count(&browser.tabs);
        let pinned_rows = self.row_visuals(&browser.tabs[..pinned_count]);
        let regular_rows = self.row_visuals(&browser.tabs[pinned_count..]);

        let tab_surface = self.tab_surface(&theme, pinned_rows, regular_rows, cx);
        let space_surface = self.space_surface(&theme, &browser, cx);
        let command_bar = self.command_bar(&theme, chrome_focused);

        let sidebar = div()
            .id("bbr-sidebar")
            .w(theme.density.sidebar_width())
            .h_full()
            .flex()
            .flex_col()
            .bg(p.surface_sidebar)
            .border_r_1()
            .border_color(p.hairline)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e: &MouseDownEvent, window, cx| this.focus_chrome(window, cx)),
            )
            .child(
                div()
                    .id("bbr-brand")
                    .px(Spacing::sp4())
                    .py(Spacing::sp3())
                    .text_size(theme.type_.title.size)
                    .font_weight(theme.type_.title.weight)
                    .text_color(p.ink)
                    .child("BBR"),
            )
            .child(command_bar)
            .child(space_surface)
            .child(tab_surface)
            .child(
                div()
                    .id("bbr-status")
                    .px(Spacing::sp4())
                    .py(Spacing::sp2())
                    .text_size(theme.type_.meta.size)
                    .text_color(p.ink_faint)
                    .child(status_text),
            )
            .child(
                div()
                    .id("bbr-sidebar-toggle")
                    .px(Spacing::sp4())
                    .py(Spacing::sp2())
                    .text_size(theme.type_.meta.size)
                    .text_color(p.ink_faint)
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e: &MouseDownEvent, window, cx| {
                            this.toggle_sidebar(window, cx)
                        }),
                    )
                    .child("Sidebar (Cmd-B)"),
            );

        let content: gpui::AnyElement = if browser.active_view.is_some() {
            let slot = NativeWebViewSlot::new(self.engine.clone());
            #[cfg(feature = "testkit")]
            let slot = slot.on_presented(
                self.performance.presentation_pending(),
                self.performance.presentation_callback(),
            );
            slot.into_any()
        } else if browser
            .active_index
            .and_then(|index| browser.tabs.get(index))
            .is_some_and(|tab| tab.crashed)
        {
            // The crash destroyed the session (and with it the session nav
            // state), so the error surface derives from the tab's persistent
            // crash flag, never from `browser.nav` (DESIGN.md §10.7).
            components::crash_panel(
                &theme,
                cx.listener(|this, _e, window, cx| this.reload_active_from_panel(window, cx)),
                cx.listener(|this, _e, window, cx| {
                    let _ = this.close_active_tab(window, cx);
                }),
            )
            .into_any()
        } else {
            components::empty_space_pane(
                &theme,
                &crate::shell::profile_line(
                    browser.active_profile_title.as_deref().unwrap_or("Default"),
                ),
                cx.listener(|this, _e, window, cx| this.activate_new_tab(window, cx)),
            )
            .into_any()
        };

        div()
            .id("bbr-root")
            .track_focus(&self.chrome_handle)
            .key_context("bbr_chrome")
            .flex()
            .flex_row()
            .size_full()
            .bg(p.surface_canvas)
            .on_key_down(cx.listener(Self::on_chrome_key_down))
            .when(self.sidebar_visible, |root| root.child(sidebar))
            .child(content)
    }
}

impl BbrApp {
    /// The resolved theme tokens for the current shell mode. Fixed
    /// dark/standard/normal density until the light, contrast, and density
    /// slices introduce mode switching; the §15 mode fields are already
    /// reported by the snapshot.
    pub fn theme(&self) -> Theme {
        Theme::dark()
    }

    /// The command bar component (mirrors the live URL / staged destination).
    fn command_bar(&self, theme: &Theme, chrome_focused: bool) -> Stateful<Div> {
        let p = theme.palette;
        div()
            .id("bbr-command-bar")
            .mx(Spacing::sp3())
            .my(Spacing::sp2())
            .px(Spacing::sp3())
            .py(Spacing::sp2())
            .rounded(theme.radius.large())
            .bg(if chrome_focused {
                p.surface_field
            } else {
                p.surface_sidebar
            })
            .border_1()
            .border_color(if chrome_focused { p.accent } else { p.hairline })
            .flex()
            .flex_row()
            .gap(Spacing::sp2())
            .child(
                div()
                    .text_size(theme.type_.meta.size)
                    .text_color(p.ink_faint)
                    .child("▶"),
            )
            .child(
                div()
                    .id("bbr-command-text")
                    .flex_1()
                    .text_size(theme.type_.mono.size)
                    .text_color(p.ink)
                    .child(if self.command_bar_text.is_empty() {
                        SharedString::from("Type a URL…")
                    } else {
                        SharedString::from(self.command_bar_text.clone())
                    }),
            )
            .when(chrome_focused && self.command_bar_mode.is_active(), |el| {
                el.child(
                    div()
                        .id("bbr-command-cursor")
                        .w(px(2.0))
                        .h_full()
                        .bg(p.accent),
                )
            })
    }

    /// The controller-backed tab surface: the focused tab list with the
    /// pinned region fixed above the regular region (§11.2). Rows are wired
    /// to the production pointer path (`select_row_via_pointer` /
    /// `close_row_via_pointer`).
    fn tab_surface(
        &self,
        theme: &Theme,
        pinned_rows: Vec<RowVisuals>,
        regular_rows: Vec<RowVisuals>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let mut children: Vec<gpui::AnyElement> = Vec::new();
        if !pinned_rows.is_empty() {
            children.push(components::section_label(theme, "Pinned").into_any());
            for row in &pinned_rows {
                children.push(self.render_row(theme, row, cx).into_any());
            }
            children.push(
                div()
                    .h(px(1.0))
                    .w_full()
                    .bg(theme.palette.hairline)
                    .into_any(),
            );
        }
        if !regular_rows.is_empty() {
            children.push(components::section_label(theme, "Tabs").into_any());
            for row in &regular_rows {
                children.push(self.render_row(theme, row, cx).into_any());
            }
        }

        div()
            .id("bbr-tab-list")
            .track_focus(&self.tab_list_handle)
            .flex_1()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .on_key_down(cx.listener(Self::on_tab_list_key_down))
            .children(children)
    }

    fn render_row(&self, theme: &Theme, row: &RowVisuals, cx: &mut Context<Self>) -> Stateful<Div> {
        let index = row.index;
        let select = cx.listener(move |this, _e: &MouseDownEvent, window, cx| {
            this.select_row_via_pointer(index, window, cx)
        });
        let close = cx.listener(move |this, e: &MouseDownEvent, window, cx| {
            cx.stop_propagation();
            this.close_row_via_pointer(index, window, cx);
            let _ = e;
        });
        components::tab_row(theme, row, select, close)
    }

    /// The controller-backed space surface (DESIGN.md §11.3): every space
    /// grouped under its profile with a per-row chip and a hairline between
    /// profile groups, plus the New Space row at the foot. The active space
    /// renders the selected rail; rows run the production pointer activation
    /// path. The list owns its own GPUI focus handle so `↑/↓`/Enter traverse
    /// the spaces region (DESIGN.md §9 order: command field → spaces → tabs).
    fn space_surface(
        &self,
        theme: &Theme,
        browser: &bbr_browser::ControllerSnapshot,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let budget = crate::shell::space_title_budget_chars(theme);
        let groups = crate::shell::space_groups(browser, budget);
        let mut children: Vec<gpui::AnyElement> = Vec::new();
        for group in &groups {
            if group.hairline_before {
                children.push(
                    div()
                        .h(px(1.0))
                        .w_full()
                        .bg(theme.palette.hairline)
                        .into_any(),
                );
            }
            for row in &group.rows {
                let space = browser
                    .spaces
                    .iter()
                    .find(|s| s.index == row.index)
                    .expect("space row derives from the snapshot")
                    .id;
                let select = cx.listener(move |this, _e: &MouseDownEvent, window, cx| {
                    this.select_space_via_pointer(space, window, cx)
                });
                children.push(components::space_row(theme, row, select).into_any());
            }
        }
        let new_space = cx
            .listener(|this, _e: &MouseDownEvent, window, cx| this.activate_new_space(window, cx));
        children.push(components::space_new_row(theme, new_space).into_any());

        div()
            .id("bbr-space-list")
            .track_focus(&self.space_list_handle)
            .flex()
            .flex_col()
            .px(Spacing::sp1())
            .on_key_down(cx.listener(Self::on_space_list_key_down))
            .children(children)
    }

    /// Map a tab slice to per-row visuals for the current theme. The §4
    /// one-line title truncation is applied here so the snapshot's row
    /// titles and the rendered rows share one source.
    pub fn row_visuals(&self, tabs: &[bbr_browser::TabSnapshot]) -> Vec<RowVisuals> {
        let budget = crate::shell::title_budget_chars(&self.theme());
        tabs.iter()
            .map(|tab| RowVisuals {
                index: tab.index,
                title: SharedString::from(crate::shell::truncate_title(
                    &crate::shell::display_title(&tab.url, &tab.title),
                    budget,
                )),
                state: crate::shell::row_state(tab.active, tab.loading, tab.crashed),
                favicon: Some(components::FaviconSpec {
                    letter: crate::shell::monogram_letter(&tab.url),
                    hue: crate::shell::monogram_hue(&tab.url),
                }),
                show_close: true,
            })
            .collect()
    }

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
        if closed && self.controller.snapshot().active_view.is_none() {
            self.enter_empty_new_tab(window, cx);
        }
    }

    /// Focus the tab list (chrome domain) without activating the command bar.
    pub(super) fn focus_tab_list(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_bar_mode = crate::command_bar::CommandBarMode::Inactive;
        self.command_bar_return = None;
        window.focus(&self.tab_list_handle);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
    }

    /// Keyboard navigation of the visible tab list (DESIGN.md §11.1):
    /// `↑/↓` move selection, `backspace` closes the active tab, Enter
    /// activates. Runs only while the tab list owns chrome focus and no
    /// command mode is active.
    pub(super) fn on_tab_list_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.command_bar_mode.is_active() {
            return;
        }
        let snapshot = self.controller.snapshot();
        let Some(active) = snapshot.active_index else {
            return;
        };
        match event.keystroke.key.as_str() {
            "up" | "down" => {
                let len = snapshot.tabs.len();
                if len > 1 {
                    let delta = if event.keystroke.key == "up" { -1 } else { 1 };
                    let next = (active as isize + delta).rem_euclid(len as isize) as usize;
                    if self.select_tab(next, cx).is_some() {
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
                if self.controller.snapshot().active_view.is_some() {
                    self.focus_web(window, cx);
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
                cx.stop_propagation();
            }
            "backspace" => {
                let _ = self.close_active_tab(window, cx);
                // Closing the final tab already opened the empty New Tab
                // command surface (command-bar focus, NewTab mode — the same
                // outcome as the pointer path). Only a close that leaves tabs
                // behind keeps the list focused so repeated keys keep working.
                if self.controller.snapshot().active_view.is_some() {
                    self.focus_tab_list(window, cx);
                }
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    /// Reload the crashed active tab from the crash panel (fresh session).
    pub fn reload_active_from_panel(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
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
            .reopen_tab()
            .map_err(|error| {
                log::error!("could not reopen tab: {error}");
                error
            })
            .ok();
        if let Some(outcome) = &result {
            self.sync_command_bar();
            self.emit_browser(outcome);
            if chrome_focused {
                self.focus_tab_list(window, cx);
            } else {
                self.focus_web(window, cx);
            }
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
            .pin_tab(index)
            .map_err(|error| {
                log::error!("could not pin tab {index}: {error}");
                error
            })
            .ok();
        if let Some(outcome) = &result {
            self.sync_command_bar();
            self.emit_browser(outcome);
            cx.notify();
        }
        result
    }
}
