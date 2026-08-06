//! The GPUI sidebar surface (DESIGN.md §11.1–§11.4).
//!
//! Owns the rail composition: brand block, command bar, the controller-backed
//! space and tab surfaces, the status line, the collapse control, and the
//! adjustable-width resize divider. All visuals come from the Chassis tokens
//! and components; the rail width is the effective sidebar geometry (the
//! requested width clamped to the window), never a local magic value. The
//! root render wires the drag-move/up listeners; this module owns the
//! surfaces themselves.

use gpui::{
    div, prelude::*, px, Context, Div, MouseButton, MouseDownEvent, SharedString, Stateful, Window,
};

use crate::components::{self, RowVisuals};
use crate::sidebar::sidebar_divider;
use crate::theme::{Spacing, Theme};

use super::BbrApp;

impl BbrApp {
    /// The sidebar rail at the given effective width: the §11 column (brand,
    /// command field, spaces, tabs, status, toggle) plus the §5 4 pt resize
    /// divider on the right edge. The divider is focusable and carries the
    /// stable §15 id; the collapse control keeps the `Cmd-B` pointer entry.
    pub(crate) fn sidebar_surface(
        &self,
        theme: &Theme,
        sidebar_width: f32,
        chrome_focused: bool,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let p = theme.palette;
        let tab_surface = self.tab_surface(theme, cx);
        let space_surface = self.space_surface(theme, cx);
        let command_bar = self.command_bar(theme, chrome_focused);
        let status_text: SharedString =
            crate::shell::status_line(&self.controller.snapshot()).into();
        let divider_focused = self.divider_handle.is_focused(window);
        let begin_drag = cx.listener(|this, event, window, cx| {
            this.begin_sidebar_drag(event, window, cx);
            cx.stop_propagation();
        });
        let divider_keys = cx.listener(Self::on_sidebar_divider_key_down);

        div()
            .id("bbr-sidebar")
            .w(px(sidebar_width))
            .h_full()
            // The rail is a physical fixed-width surface: GPUI's default
            // flex_shrink is 1, so without this the rail would absorb part
            // of the slot's relative(1.0) width and render narrower than its
            // effective geometry — breaking the §13 content boundary and the
            // §5 clamp (the content floor must stay exact).
            .flex_shrink_0()
            .flex()
            .flex_row()
            .bg(p.surface_sidebar)
            .border_r_1()
            .border_color(p.hairline)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e: &MouseDownEvent, window, cx| this.focus_chrome(window, cx)),
            )
            .child(
                div()
                    .id("bbr-sidebar-column")
                    .flex_1()
                    .flex()
                    .flex_col()
                    .min_w_0()
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
                    ),
            )
            .child(sidebar_divider(
                theme,
                self.divider_handle.clone(),
                divider_focused,
                begin_drag,
                divider_keys,
            ))
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
    fn tab_surface(&self, theme: &Theme, cx: &mut Context<Self>) -> Stateful<Div> {
        let browser = self.controller.snapshot();
        let pinned_count = crate::shell::pinned_count(&browser.tabs);
        let pinned_rows = self.row_visuals(&browser.tabs[..pinned_count]);
        let regular_rows = self.row_visuals(&browser.tabs[pinned_count..]);
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
    fn space_surface(&self, theme: &Theme, cx: &mut Context<Self>) -> Stateful<Div> {
        let browser = self.controller.snapshot();
        let budget = crate::shell::space_title_budget_chars(theme);
        let groups = crate::shell::space_groups(&browser, budget);
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
}
