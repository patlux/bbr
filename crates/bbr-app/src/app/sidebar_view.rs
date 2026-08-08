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
use crate::theme::{Radius, Spacing, Theme};

use super::BbrApp;

mod command_bar;

impl BbrApp {
    /// The sidebar rail at the given effective width: the §11 column (brand,
    /// command field, spaces region, favorites grid, pinned rows, folders,
    /// regular tabs, status, toggle, plus menu, footer row) plus the §5 4 pt
    /// resize divider on the right edge. The divider is focusable and
    /// carries the stable §15 id; the collapse control keeps the `Cmd-B`
    /// pointer entry. The U2 surfaces (spaces region, favorites, folders,
    /// plus menu, footer) are composed by `u2_surface` helpers.
    pub(crate) fn sidebar_surface(
        &mut self,
        theme: &Theme,
        sidebar_width: f32,
        chrome_focused: bool,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let p = theme.palette;
        // The U2 rail inputs (favorites tiles, folder rows/members) derived
        // once per frame and shared by the tab/favorites/folders surfaces.
        let u2 = self.u2_rail_data();
        let tab_surface = self.tab_surface(theme, window, cx, &u2.folder_rows, &u2.folder_members);
        let space_surface = self.space_surface(theme, window, cx);
        let command_bar = self.command_bar(theme, chrome_focused, cx);
        // §11.5 result palette: the only chrome surface opening over the
        // rail's own content (native web views are real child views).
        let command_results = self.command_result_surface(theme);
        let favorites = self.favorites_surface(theme, &u2.favorite_tiles, cx);
        let plus_menu = self.plus_menu_surface(theme, cx);
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
                    // Strict height bound: the column must never grow to its
                    // content, or the dense scroll container below would be
                    // laid out at the full collection height (a 500-tab list
                    // would hand the container ~16 000 pt) and the container's
                    // prepaint would clamp the production scroll offset to 0.
                    .min_h_0()
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
                    .when(command_results.is_some(), |column| {
                        column.child(command_results.expect("checked above"))
                    })
                    // §11.4: regions stack separated by `density:section-gap`
                    // breathing (S `sp:3` / N `sp:4` / XL `sp:5`).
                    .child(div().h(theme.density.section_gap()).flex_shrink_0())
                    // §11.4 region order: spaces (while revealed) → favorites →
                    // tabs → status → toggle → plus menu → footer.
                    .map(|column| self.u2_pre_tab_children(column, space_surface, favorites))
                    .child(div().h(theme.density.section_gap()).flex_shrink_0())
                    .child(tab_surface)
                    .child(div().h(theme.density.section_gap()).flex_shrink_0())
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
                    )
                    .map(|column| self.u2_post_tab_children(column, plus_menu, theme, cx)),
            )
            .child(sidebar_divider(
                theme,
                self.divider_handle.clone(),
                divider_focused,
                begin_drag,
                divider_keys,
            ))
    }

    /// The controller-backed tab surface: the pinned region as a fixed block
    /// above the regular region's scroll view (DESIGN.md §11.2/§11.6), with
    /// the §11.11 folders list between the pinned block and the Tabs
    /// section. Rows are wired to the production
    /// pointer path (`select_row_via_pointer` / `close_row_via_pointer` /
    /// `toggle_row_pin` / `begin_tab_drag`), and an active drag renders the
    /// §10.9 insertion tick at its target slot (muted when the drop would be
    /// illegal).
    ///
    /// The regular region renders only the bounded window around the
    /// production scroll anchor (`dense::window_for`, ≤ `DENSE_WINDOW_ROWS`
    /// rows) plus spacers that keep the container's content height equal to
    /// the full collection — the native scrollbar and wheel listener behave
    /// as if every row were laid out, while a 500-tab list never lays out
    /// 500 rows per frame. The pinned region renders all of its rows: it is
    /// a small, stable block and must never shift on selection (§11.2).
    fn tab_surface(
        &self,
        theme: &Theme,
        window: &Window,
        cx: &mut Context<Self>,
        folder_rows: &[crate::components::FolderRowSpec],
        folder_members: &[crate::components::FolderMemberVisual],
    ) -> Stateful<Div> {
        let browser = self.controller.snapshot();
        let pinned = crate::shell::pinned_count(&browser.tabs);
        let regular = &browser.tabs[pinned..];
        let row_h = theme.density.row();
        let offset_y = self.dense_scroll.offset().y;
        let anchor = crate::dense::anchor_from_offset(offset_y, row_h, regular.len());
        let row_window = crate::dense::window_for(anchor, regular.len());
        let drag_source = self.tab_drag.source();
        let pinned_rows = self.row_visuals(&browser.tabs[..pinned], drag_source);
        let window_rows = self.row_visuals(&regular[row_window.clone()], drag_source);
        let tick = self.tab_drag.tab_slot();
        let tick_valid = self.tab_drag.valid();
        let list_focused = self.tab_list_handle.is_focused(window);
        let p = theme.palette;

        // Record what this render built so the testkit snapshot reports the
        // bounded window and the deterministic scroll anchor (the snapshot is
        // taken on demand; the recorded state mirrors the last frame).
        self.dense_window.set(crate::dense::DenseRenderState {
            first_rendered: pinned + row_window.start,
            rendered_rows: row_window.len(),
            item_count: regular.len(),
            scroll_anchor: anchor,
        });

        let mut children: Vec<gpui::AnyElement> = Vec::new();
        if !pinned_rows.is_empty() {
            children.push(components::section_label(theme, "Pinned", list_focused).into_any());
            for row in &pinned_rows {
                if tick == Some(row.index) {
                    children.push(components::drag_tick(theme, row.index, !tick_valid).into_any());
                }
                children.push(self.render_row(theme, row, cx).into_any());
            }
            if tick == Some(pinned) {
                children.push(components::drag_tick(theme, pinned, !tick_valid).into_any());
            }
            children.push(
                div()
                    .h(px(1.0))
                    .w_full()
                    .bg(theme.palette.hairline)
                    .into_any(),
            );
        }
        // The §11.11 folders list sits between the pinned block and the Tabs
        // section (hidden entirely while no folder exists).
        let folders = self.folders_surface(theme, folder_rows, folder_members, window, cx);
        if let Some(folders) = folders {
            children.push(folders.into_any());
        }
        if !regular.is_empty() {
            children.push(components::section_label(theme, "Tabs", list_focused).into_any());
        }

        // The scroll container: spacers keep the total content height equal
        // to the full collection, the rendered window follows the production
        // scroll offset, and the drag tick renders only inside the window.
        let top = crate::dense::scroll_top_for_anchor(row_window.start, row_h);
        let bottom = (regular.len().saturating_sub(row_window.end)) as f32 * f32::from(row_h);
        let mut scroll_children: Vec<gpui::AnyElement> = Vec::new();
        if top > 0.0 {
            scroll_children.push(div().h(px(top)).into_any());
        }
        for row in &window_rows {
            if tick == Some(row.index) {
                scroll_children
                    .push(components::drag_tick(theme, row.index, !tick_valid).into_any());
            }
            scroll_children.push(self.render_row(theme, row, cx).into_any());
        }
        if bottom > 0.0 {
            scroll_children.push(div().h(px(bottom)).into_any());
        }
        children.push(
            div()
                .id("bbr-tab-scroll")
                .flex_1()
                .min_h_0()
                .min_w_0()
                .overflow_y_scroll()
                .track_scroll(&self.dense_scroll)
                .children(scroll_children)
                .into_any(),
        );

        div()
            .id("bbr-tab-list")
            .track_focus(&self.tab_list_handle)
            .flex_1()
            .flex()
            .flex_col()
            .min_w_0()
            .min_h_0()
            // §9 keyboard-focus ring: 2 pt accent outline inside the focused
            // list's bounds + the brightened section labels — focus is a
            // shape and a text step, never color-only.
            .when(list_focused, |el| {
                el.border_2()
                    .border_color(p.accent)
                    .rounded(Radius.medium())
            })
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
        let pin = cx.listener(move |this, e: &MouseDownEvent, window, cx| {
            cx.stop_propagation();
            this.toggle_row_pin(index, window, cx);
            let _ = e;
        });
        // Pointer drag entry: pressing the row starts a drag of that row.
        // The row's own controls stop propagation before this listener runs
        // (their handlers come first in the bubble phase), so a press on the
        // pin or close control never starts a drag.
        let begin_drag = cx.listener(move |this, e: &MouseDownEvent, window, cx| {
            this.begin_tab_drag(e, index, window, cx);
        });
        // §10.2/§10.3 production pointer-state paths: the press wash while
        // the primary button is down, the hover wash + close reveal on
        // enter/leave. Separate listeners from select/drag so the visual
        // states never interfere with the mutations.
        let press = cx.listener(move |this, _e: &MouseDownEvent, _window, cx| {
            this.set_row_pressed(index, cx);
        });
        let release = cx.listener(move |this, _e: &gpui::MouseUpEvent, _window, cx| {
            this.clear_row_pressed(cx);
        });
        let hover = cx.listener(move |this, hovered: &bool, _window, cx| {
            this.set_row_hover(index, *hovered, cx);
        });
        components::tab_row(theme, row, select, close, pin, begin_drag)
            .on_mouse_down(MouseButton::Left, press)
            .on_mouse_up(MouseButton::Left, release)
            .on_hover(hover)
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
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let browser = self.controller.snapshot();
        let budget = crate::shell::space_title_budget_chars(theme);
        let drag_target_space = self.tab_drag.space_target();
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
                let mut row = row.clone();
                if drag_target_space == Some(space) {
                    row.state = crate::components::SpaceVisualState::DragTarget;
                }
                children.push(components::space_row(theme, &row, select).into_any());
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
            // §9 keyboard-focus ring on the focused space list (the same
            // structural cue the tab list uses).
            .when(self.space_list_handle.is_focused(window), |el| {
                el.border_2()
                    .border_color(theme.palette.accent)
                    .rounded(theme.radius.medium())
            })
            .on_key_down(cx.listener(Self::on_space_list_key_down))
            .children(children)
    }

    /// Map a tab slice to per-row visuals for the current theme. The §4
    /// one-line title truncation is applied here so the snapshot's row
    /// titles and the rendered rows share one source. `drag_source` marks
    /// the row currently being dragged (§10.9 `dragging` state); the row's
    /// `pinned` region identity drives the pin control. The pointer-state
    /// fields (hover/press) and the §10.2 close-reveal derive from the same
    /// app state the production listeners mutate, so the snapshot's
    /// `tab_states` always match the render.
    pub fn row_visuals(
        &self,
        tabs: &[bbr_browser::TabSnapshot],
        drag_source: Option<usize>,
    ) -> Vec<RowVisuals> {
        let budget = crate::shell::title_budget_chars(&self.theme());
        tabs.iter()
            .map(|tab| RowVisuals {
                index: tab.index,
                title: SharedString::from(crate::shell::truncate_title(
                    &crate::shell::display_title(&tab.url, &tab.title),
                    budget,
                )),
                state: if drag_source == Some(tab.index) {
                    crate::components::TabVisualState::Dragging
                } else {
                    crate::shell::row_state(
                        tab.active,
                        tab.loading,
                        tab.crashed,
                        self.pressed_row == Some(tab.index),
                        self.hovered_row == Some(tab.index),
                    )
                },
                favicon: Some(components::FaviconSpec {
                    letter: crate::shell::monogram_letter(&tab.url),
                    hue: crate::shell::monogram_hue(&tab.url),
                }),
                pinned: tab.pinned,
                show_close: true,
                reveal_close: self.hovered_row == Some(tab.index)
                    || self.pressed_row == Some(tab.index)
                    || tab.active,
            })
            .collect()
    }
}
