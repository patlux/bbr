//! Downloads panel render (DESIGN.md §11.12).
//!
//! The anchored popover (`bbr-downloads-panel`) confined to the sidebar
//! column: an **Active** group (non-terminal attempts, ascending
//! `DownloadId`) above a **History** group (terminal attempts,
//! most-recent-terminal first), with `type:badge` group labels hidden when
//! a group is empty, per-row `[status glyph] [name + secondary] [action]`
//! anatomy, the §10.2/§10.3 hover/press washes, exactly one contextual
//! primary action per row, the bounded 6-row window (§11.6 overscan +
//! scrollbar), the empty hint, and the `dur:fine` opacity open/close fade.
//!
//! The panel is ordinary GPUI sidebar chrome: it is a flex child of the
//! sidebar column placed directly above the footer row (bottom edge flush
//! with the footer's top edge, leading edge flush with the column's leading
//! edge, width = sidebar width − 2 × `density:row-inset`), so it never
//! overlaps the native web view and never uses the NativeAboveWeb Portal
//! host. All geometry derives from the settled S1 tokens — no invented
//! sizes. Rows render exactly the L6 machine states; the panel never
//! invents a transition (the controller is the authority; this module only
//! maps records to rows).

use gpui::{
    div, prelude::*, px, Context, Div, MouseButton, MouseDownEvent, Stateful, Styled, Window,
};

use crate::components::Icon;
use crate::theme::{Spacing, Theme};

use super::downloads::{row_visuals, DownloadMachine, PanelLayout, DOWNLOADS_MAX_VISIBLE};
use super::BbrApp;

mod row_detail;

/// §11.12/§15 assert ids of the downloads panel.
pub const DOWNLOADS_PANEL_ID: &str = "bbr-downloads-panel";
pub const DOWNLOADS_ACTIVE_ID: &str = "bbr-downloads-active";
pub const DOWNLOADS_HISTORY_ID: &str = "bbr-downloads-history";
pub const DOWNLOADS_EMPTY_ID: &str = "bbr-downloads-empty";

/// The `bbr-download-row-<n>` id of row `index` (deterministic panel index).
pub fn download_row_id(index: usize) -> String {
    format!("bbr-download-row-{index}")
}

/// The `bbr-download-<slot>-<n>` ids of one row (DESIGN.md §11.12).
pub(crate) fn download_slot_id(slot: &str, index: usize) -> String {
    format!("bbr-download-{slot}-{index}")
}

impl BbrApp {
    /// The §11.12 panel layout tokens for the visible-id derivation (the
    /// same token-derived constants the render uses, so the ids mirror the
    /// shell's gating exactly).
    #[cfg(feature = "testkit")]
    pub(crate) fn downloads_layout(&self) -> PanelLayout {
        let theme = self.theme();
        PanelLayout {
            label_h: f32::from(theme.type_.badge.size) * theme.type_.badge.leading
                + 2.0 * f32::from(Spacing::sp1()),
            row_h: f32::from(theme.density.row()),
            active_count: self.downloads.controller.active_ids().len(),
            history_count: self.downloads.controller.history_ids().len(),
            hairline: 1.0,
        }
    }

    /// The §11.12 anchored popover surface. `None` while closed and the
    /// `dur:fine` fade-out has completed. The panel owns the keyboard
    /// cursor while open (`↑`/`↓`/`PgUp`/`PgDn`/Enter/Tab/Escape via
    /// [`BbrApp::on_downloads_key_down`]), marks pointer presses inside so
    /// the root outside-press handler never dismisses it (§13 one pointer
    /// action), and carries the §9 focus ring while it owns chrome focus.
    pub(crate) fn downloads_panel_surface(
        &mut self,
        theme: &Theme,
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
                window.use_transition(cx, crate::theme::Motion::fine(), |_, _| 0.0_f32);
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

        let p = theme.palette;
        let density = theme.density;
        let row_h = f32::from(density.row());
        // The §11.12 `type:badge` group label: badge text step × leading +
        // 2 × `sp:1` padding — the exact height the label divs render (both
        // sides use the same token-derived constant, so the scroll math and
        // the layout never drift).
        let label_h = f32::from(theme.type_.badge.size) * theme.type_.badge.leading
            + 2.0 * f32::from(Spacing::sp1());
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

        // The §11.12 6-row bound: at most `DOWNLOADS_MAX_VISIBLE` rows are
        // visible; beyond that the scroll viewport is exactly 6 rows tall
        // (labels scroll with the content) and the full history is never
        // laid out (the render window is the visible range plus the §11.6
        // 8-row overscan).
        let content_h = layout.content_height();
        let viewport_h = if total > DOWNLOADS_MAX_VISIBLE {
            DOWNLOADS_MAX_VISIBLE as f32 * row_h
        } else {
            content_h
        };

        // Record what this render built so the testkit snapshot reports the
        // bounded window and the deterministic scroll anchor (the snapshot
        // mirrors the last frame, exactly like the dense tab list).
        self.downloads.render_state = super::downloads::DownloadsRenderState {
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

        // The scroll container's flow children: the top spacer (everything
        // above the rendered window), the window's own label/hairline/row
        // items, and the bottom spacer. The content height equals the full
        // content, so the native scrollbar and wheel behave as if every row
        // were laid out while only the bounded window is.
        let window_items = layout.items_in_window(&row_window);
        let top = layout.top_spacer(&row_window);
        let bottom = layout.bottom_spacer(&row_window);
        let mut scroll_children: Vec<gpui::AnyElement> = Vec::new();
        if top > 0.0 {
            scroll_children.push(div().h(px(top)).into_any());
        }
        if window_items.active_label {
            scroll_children.push(self.downloads_group_label(theme, true).into_any());
        }
        for visual in visuals.iter().filter(|row| row_window.contains(&row.index)) {
            scroll_children.push(self.downloads_row(theme, visual, cx).into_any());
        }
        if window_items.hairline {
            scroll_children.push(
                div()
                    .h(px(layout.hairline))
                    .w_full()
                    .bg(p.hairline)
                    .into_any(),
            );
        }
        if window_items.history_label {
            scroll_children.push(self.downloads_group_label(theme, false).into_any());
        }
        if bottom > 0.0 {
            scroll_children.push(div().h(px(bottom)).into_any());
        }

        let panel = div()
            .id(DOWNLOADS_PANEL_ID)
            .track_focus(&self.downloads_panel_handle)
            .w(px(self.downloads_panel_width(window)))
            .opacity(alpha)
            .rounded(theme.radius.large())
            .bg(p.surface_overlay)
            .border_1()
            .border_color(p.hairline)
            .shadow(elev2_shadow(theme))
            .overflow_hidden()
            // §9 ring while the panel owns chrome focus (the same structural
            // cue as the tab/space lists; the selected-tab rail stays the
            // only accent).
            .when(panel_focused, |el| {
                el.border_2()
                    .border_color(p.accent)
                    .rounded(theme.radius.large())
            })
            .on_key_down(cx.listener(Self::on_downloads_key_down))
            // §13: the inside-press marker runs deepest in the bubble phase
            // and stops propagation, so a press inside the panel never
            // dismisses it and the sidebar chrome-focus handler never steals
            // focus (§11.12: the panel is chrome focus while open).
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e: &MouseDownEvent, _window, cx| {
                    this.mark_downloads_press_inside();
                    cx.stop_propagation();
                }),
            );

        // The panel content: the empty hint or the bounded scroll window.
        let panel = if total == 0 {
            panel.child(
                div()
                    .id(DOWNLOADS_EMPTY_ID)
                    .h(px(3.0 * row_h))
                    .flex()
                    .items_center()
                    .justify_center()
                    .px(Spacing::sp3())
                    .text_size(theme.type_.meta.size)
                    .text_color(p.ink_secondary)
                    .child("No downloads yet"),
            )
        } else {
            panel.child(
                div()
                    .id("bbr-downloads-scroll")
                    .h(px(viewport_h))
                    .w_full()
                    .overflow_y_scroll()
                    .track_scroll(&self.downloads.scroll)
                    .children(scroll_children),
            )
        };
        Some(panel)
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

    /// One §11.12 `type:badge` group label ("Active"/"History"), hidden by
    /// the caller when the group is empty. The label height is the
    /// token-derived constant the scroll math uses.
    fn downloads_group_label(&self, theme: &Theme, active: bool) -> Stateful<Div> {
        let p = theme.palette;
        let label_h = f32::from(theme.type_.badge.size) * theme.type_.badge.leading
            + 2.0 * f32::from(Spacing::sp1());
        div()
            .id(if active {
                DOWNLOADS_ACTIVE_ID
            } else {
                DOWNLOADS_HISTORY_ID
            })
            .h(px(label_h))
            .px(Spacing::sp3())
            .flex()
            .items_center()
            .text_size(theme.type_.badge.size)
            .font_weight(theme.type_.badge.weight)
            .text_color(p.ink_faint)
            .child(if active { "ACTIVE" } else { "HISTORY" })
    }

    /// One §11.12 download row: `[status glyph] [name line + secondary
    /// line] [action]` at `density:row` tall with `density:row-padding-x`,
    /// the §10.2/§10.3 inset hover/press washes (`density:row-inset`,
    /// `density:radius`), the deterministic `bbr-download-*` ids, and the
    /// production pointer paths (hover/press state, action activation).
    fn downloads_row(
        &self,
        theme: &Theme,
        visual: &super::downloads::DownloadRowVisual,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let p = theme.palette;
        let density = theme.density;
        let index = visual.index;
        let icon_size = density.icon();
        let status = match visual.machine {
            DownloadMachine::Waiting | DownloadMachine::Running => {
                // §10.5 spinner arc (accent, transient indication — allowed).
                Icon::LoadingArc.canvas(theme, icon_size).into_any()
            }
            DownloadMachine::Succeeded => {
                // §3.3 `status.ok` check with its non-color companion (the
                // status glyph is always next to the name line).
                Icon::Check
                    .canvas_with(theme, icon_size, p.status_ok)
                    .into_any()
            }
            DownloadMachine::Failed => Icon::Lifering.canvas(theme, icon_size).into_any(),
            DownloadMachine::Cancelled => {
                // Neutral marker at `ink.secondary`.
                Icon::Stopped.canvas(theme, icon_size).into_any()
            }
        };
        let secondary = self.downloads_secondary_line(theme, visual);
        let activate = cx.listener(move |this, _e: &MouseDownEvent, window, cx| {
            crate::app::downloads_behavior::download_click_row_action(this, index, window, cx);
            cx.stop_propagation();
        });
        let press = cx.listener(move |this, _e: &MouseDownEvent, _window, cx| {
            this.set_downloads_pressed(Some(index), cx);
        });
        let release = cx.listener(move |this, _e: &gpui::MouseUpEvent, _window, cx| {
            this.set_downloads_pressed(None, cx);
        });
        let hover = cx.listener(move |this, hovered: &bool, _window, cx| {
            this.set_downloads_hover(index, *hovered, cx);
        });

        div()
            .id(download_row_id(index))
            .h(density.row())
            .mx(density.row_inset())
            .rounded(density.density_radius())
            .px(density.row_padding_x())
            .gap(Spacing::sp2())
            .items_center()
            .flex()
            .flex_row()
            .min_w_0()
            .when(
                visual.visual == super::downloads::DownloadVisual::Hover,
                |el| el.bg(p.wash_hover),
            )
            .when(
                visual.visual == super::downloads::DownloadVisual::Pressed,
                |el| el.bg(p.wash_active),
            )
            .when(
                visual.visual == super::downloads::DownloadVisual::Focused,
                |el| {
                    el.border_2()
                        .border_color(p.accent)
                        .rounded(density.density_radius())
                },
            )
            .child(
                div()
                    .id(download_slot_id("status", index))
                    .size(icon_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(status),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap(px(1.0))
                    .child(
                        div()
                            .id(download_slot_id("name", index))
                            .overflow_hidden()
                            .text_ellipsis()
                            .text_size(theme.type_.title.size)
                            .font_weight(theme.type_.title.weight)
                            .text_color(p.ink)
                            .child(visual.filename.clone()),
                    )
                    .child(secondary),
            )
            .child(self.downloads_action(theme, visual, index, activate))
            .on_mouse_down(MouseButton::Left, press)
            .on_mouse_up(MouseButton::Left, release)
            .on_hover(hover)
    }
}

/// The §2 `elev:2` shadow token (the palette's shadow class): light
/// `0 1 0 rgba(0,0,0,0.04) + 0 12 24 rgba(23,26,29,0.10)`, dark
/// `0 0 0 1px rgba(255,255,255,0.05) + 0 16 32 rgba(0,0,0,0.45)` — bounded
/// GPUI shadows, no blur beyond the token.
fn elev2_shadow(theme: &Theme) -> Vec<gpui::BoxShadow> {
    match theme.mode {
        crate::theme::ThemeMode::Light => vec![
            gpui::BoxShadow {
                color: gpui::rgba(0x0000000a).into(),
                offset: gpui::point(px(0.0), px(1.0)),
                blur_radius: px(0.0),
                spread_radius: px(0.0),
                inset: false,
            },
            gpui::BoxShadow {
                color: gpui::rgba(0x171a1d1a).into(),
                offset: gpui::point(px(0.0), px(12.0)),
                blur_radius: px(24.0),
                spread_radius: px(0.0),
                inset: false,
            },
        ],
        crate::theme::ThemeMode::Dark => vec![
            gpui::BoxShadow {
                color: gpui::rgba(0xffffff0d).into(),
                offset: gpui::point(px(0.0), px(0.0)),
                blur_radius: px(0.0),
                spread_radius: px(1.0),
                inset: false,
            },
            gpui::BoxShadow {
                color: gpui::rgba(0x00000073).into(),
                offset: gpui::point(px(0.0), px(16.0)),
                blur_radius: px(32.0),
                spread_radius: px(0.0),
                inset: false,
            },
        ],
    }
}

/// The §11.12/§15 download id set for the visible-ids derivation: the panel
/// surface plus the group labels, the bounded rendered rows (never the full
/// history), and the empty hint. Mirrors the render's gating exactly.
#[cfg(feature = "testkit")]
pub fn downloads_visible_ids(
    total: usize,
    window: &std::ops::Range<usize>,
    layout: &PanelLayout,
) -> Vec<String> {
    let mut ids = vec![DOWNLOADS_PANEL_ID.to_string()];
    if layout.has_active() {
        ids.push(DOWNLOADS_ACTIVE_ID.to_string());
    }
    if layout.has_history() {
        ids.push(DOWNLOADS_HISTORY_ID.to_string());
    }
    if total == 0 {
        ids.push(DOWNLOADS_EMPTY_ID.to_string());
    } else {
        for index in window.clone() {
            if index >= total {
                break;
            }
            ids.push(download_row_id(index));
            ids.push(download_slot_id("status", index));
            ids.push(download_slot_id("name", index));
            ids.push(download_slot_id("meta", index));
            ids.push(download_slot_id("progress", index));
            ids.push(download_slot_id("action", index));
        }
    }
    ids
}
