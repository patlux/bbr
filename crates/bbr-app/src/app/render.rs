//! GPUI rendering for the browser shell.
//!
//! The root renders the sidebar rail (surface composition lives in
//! `app::sidebar_view`) next to the native web-view slot. All visuals come
//! from the Chassis tokens and components (DESIGN.md); the tab rows are
//! built from the controller snapshot, never from a second navigation-state
//! copy. The root also carries the production sidebar-drag listeners:
//! mouse-move follows an active drag anywhere over the window and mouse-up
//! commits it, so the 4 pt divider keeps working when the cursor leaves it.

use gpui::{div, prelude::*, Context, MouseButton, MouseDownEvent, Render, Window};

use crate::slot::NativeWebViewSlot;
use bbr_ui::components;
use bbr_ui::theme::{ContrastMode, Theme, ThemeMode};
use bbr_web::WebViewId;

#[cfg(feature = "testkit")]
use super::command_rank::CommandResultState;
use super::BbrApp;

impl Render for BbrApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme();
        let p = theme.palette;
        let chrome_focused = self.chrome_handle.is_focused(window);
        let browser = self.controller.snapshot();
        #[cfg(feature = "testkit")]
        self.performance.observe_render_state(
            browser.active_view,
            self.sidebar_visible,
            // The presentation probe reports the chrome-focus **domain** (the
            // union of chrome handles, DESIGN.md §9/§11.12: the open
            // downloads panel is chrome focus) — not only the root chrome
            // handle. Draw-path benchmarks commit from this observation, and
            // the panel's own `chrome_focused(window)` is the authoritative
            // domain value (the snapshot reports the same union).
            self.chrome_focused(window),
            self.command_bar_mode.into(),
        );

        let window_width = f32::from(window.bounds().size.width);
        // Phase-7 warm-restore persist source: record the last rendered
        // window **content** rect (origin + size in points) so the
        // structural persistence DTO carries the truthful window geometry
        // the next startup restores through the window options. GPUI's
        // window options apply the persisted bounds as the content rect, so
        // the content rect is measured natively (via the live session's
        // window `contentLayoutRect`, which excludes the title bar) — a
        // frame-based value would grow by the title-bar height every
        // launch. Before the first native session exists the frame bounds
        // are the fallback. The repository validation rejects
        // non-positive/non-finite values on write and recovers them on
        // restore, so this raw observation is safe to persist as-is.
        {
            let bounds = window.bounds();
            let content = {
                #[cfg(target_os = "macos")]
                {
                    self.geometry_engine.as_ref().and_then(|engine| {
                        self.controller
                            .snapshot()
                            .active_view
                            .and_then(|view| engine.borrow().window_content_rect(view))
                    })
                }
                #[cfg(not(target_os = "macos"))]
                {
                    None
                }
            };
            self.window_geometry.replace(Some(content.unwrap_or([
                bounds.origin.x.into(),
                bounds.origin.y.into(),
                bounds.size.width.into(),
                bounds.size.height.into(),
            ])));
        }
        let sidebar_width = if self.sidebar_visible {
            self.sidebar.effective_width(window_width)
        } else {
            0.0
        };
        let sidebar = self.sidebar_surface(&theme, sidebar_width, chrome_focused, window, cx);

        // §11.5 presentation milestone: the result palette was constructed
        // into the sidebar above. When a `CommandType` benchmark is pending,
        // the probe records the command identity this render actually drew —
        // generation, first/selected result keys, typed state — so the
        // sample can commit only from a frame that genuinely presented the
        // ranked rows. A frame that omits the palette (surface inactive, a
        // non-Results typed state, or the rail hidden) records no
        // observation and can never satisfy the armed expectation.
        #[cfg(feature = "testkit")]
        self.performance.observe_rendered_command(
            self.command_ranking.generation,
            self.sidebar_visible
                && self.command_ranking.state == CommandResultState::Results
                && !self.command_ranking.results.is_empty(),
            self.command_ranking
                .results
                .first()
                .map(|row| row.key.as_str()),
            self.command_ranking
                .results
                .get(self.command_ranking.selected)
                .map(|row| row.key.as_str()),
            self.command_ranking.state.into(),
            &self
                .command_ranking
                .results
                .iter()
                .map(|row| row.key.clone())
                .collect::<Vec<_>>(),
            self.command_ranking.total,
        );

        // §11.8-style error-surface presentation (task packet
        // `error-surface-contract`): the classified incident the content
        // area renders for the active view (TLS, navigation, offline — the
        // process-crash class is the DESIGN §11.8 crash panel below). The
        // render records the drawn surface identity for the presentation
        // probe (the render milestone); a frame without the surface records
        // the empty observation so a stale surface can never satisfy a
        // later benchmark arm.
        #[cfg(feature = "testkit")]
        {
            let surface = self.active_error_surface_spec();
            self.performance.observe_error_surface(
                surface.is_some(),
                surface.as_ref().map(|spec| spec.incident.reason.category()),
            );
        }
        let content: gpui::AnyElement = if let Some(spec) = self.active_error_surface_spec() {
            // The surface replaces the active view's native slot (the view
            // was hidden by `sync_error_surface_visibility`; never
            // destroyed — identity is untouched).
            crate::app::error_surface::error_surface(
                &theme,
                &spec,
                cx.listener(|this, _e, window, cx| this.error_recover(window, cx)),
                cx.listener(|this, _e, window, cx| this.error_dismiss(window, cx)),
            )
            .into_any()
        } else if let Some(layout) = self.controller.pane_layout() {
            // The authoritative pane layout drives the native surface:
            // one slot per visible pane, each addressing its own
            // `WebViewId`. Single-pane geometry is exactly the previous
            // sidebar/content contract; split renders two equal,
            // adjacent, strictly positive horizontal panes with no
            // divider or polish in this anchor.
            #[cfg(feature = "testkit")]
            {
                let mut keep = vec![layout.primary];
                if let Some(secondary) = layout.secondary {
                    keep.push(secondary);
                }
                self.performance.prune_pane_rects(&keep);
            }
            let build_slot = |this: &BbrApp, id: WebViewId, _focused: bool| {
                let slot = NativeWebViewSlot::new(this.engine.clone(), id);
                #[cfg(feature = "testkit")]
                let slot = slot.on_keyed_geometry(this.performance.keyed_geometry_callback());
                #[cfg(feature = "testkit")]
                let slot = if _focused {
                    // The focused/active slot feeds the legacy
                    // `content_rect` and the presentation-commit path; a
                    // secondary pane must never satisfy either.
                    slot.on_geometry(this.performance.slot_geometry_callback())
                        .on_presented(
                            this.performance.presentation_pending(),
                            this.performance.presentation_callback(),
                        )
                } else {
                    slot
                };
                slot.into_any()
            };
            match layout.secondary {
                Some(_) => div()
                    .flex()
                    .flex_row()
                    .flex_grow()
                    .size_full()
                    .child(build_slot(
                        self,
                        layout.primary,
                        layout.focused == layout.primary,
                    ))
                    .child(build_slot(
                        self,
                        layout.secondary.expect("split has two panes"),
                        layout.focused == layout.secondary.expect("split has two panes"),
                    ))
                    .into_any(),
                None => build_slot(self, layout.primary, true),
            }
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
            .relative()
            // While a portal presentation is open the root tracks that
            // surface's focus handle (the portal window is never key, so
            // keystrokes remain handled by the main window); otherwise the
            // chrome handle owns the root focus.
            .when(
                !self.find.state.open
                    && !self.auth_sheet_is_presented()
                    && !self.context_menu.state.open,
                |root| root.track_focus(&self.chrome_handle),
            )
            .when(self.find.state.open, |root| {
                root.track_focus(&self.find_handle)
            })
            .when(self.auth_sheet_is_presented(), |root| {
                root.track_focus(&self.auth_handle)
            })
            .when(self.context_menu.state.open, |root| {
                root.track_focus(&self.menu_handle)
            })
            .key_context("bbr_chrome")
            .flex()
            .flex_row()
            .size_full()
            .bg(p.surface_canvas)
            .on_key_down(cx.listener(Self::on_chrome_key_down))
            // §11.4 plus-menu / §11.12 downloads-panel outside-press
            // dismissal: any GPUI pointer press outside the open surface
            // closes it (one pointer action, §13). The surface and its
            // owning footer controls stop propagation for their own presses,
            // so a press inside never reaches this root handler; a press
            // anywhere else (a tab row, a space row, another footer control,
            // the divider) both performs its normal action and dismisses the
            // open surface. Clicks on the native web view are platform
            // child-view events and never reach GPUI's tree.
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _e: &MouseDownEvent, window, cx| {
                    this.dismiss_plus_menu_on_outside_press(window, cx);
                    this.dismiss_downloads_on_outside_press(window, cx);
                }),
            )
            .on_mouse_move(cx.listener(Self::on_sidebar_drag_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_sidebar_drag_end))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_sidebar_drag_end))
            // §10.3 release path: a pointer press ends with the primary
            // button up anywhere over (or out of) the window.
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _e: &gpui::MouseUpEvent, _window, cx| {
                    this.clear_row_pressed(cx)
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _e: &gpui::MouseUpEvent, _window, cx| {
                    this.clear_row_pressed(cx)
                }),
            )
            .on_hover(cx.listener(|this, hovered: &bool, _window, cx| {
                // The pointer left the whole GPUI chrome (e.g. into the
                // native web view): clear the row hover. Entering the root
                // never clears a row the pointer is already over.
                if !*hovered {
                    this.clear_row_hover(cx);
                }
            }))
            // The tab-drag listeners run alongside the sidebar-drag ones on
            // the same root; each handler is a no-op unless its own drag is
            // active, so the two drag surfaces never interfere.
            .on_mouse_move(cx.listener(Self::on_tab_drag_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::end_tab_drag))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::end_tab_drag))
            .when(self.sidebar_visible, |root| root.child(sidebar))
            .child(content)
    }
}

impl BbrApp {
    /// The resolved theme tokens for the current shell mode. Density is
    /// automatic from the regular-tab collection size (DESIGN.md §8); the
    /// §15 mode fields are already reported by the snapshot.
    pub fn theme(&self) -> Theme {
        let browser = self.controller.snapshot();
        let pinned = crate::shell::pinned_count(&browser.tabs);
        let regular = browser.tabs.len().saturating_sub(pinned);
        Theme::for_collection(ThemeMode::Dark, ContrastMode::Standard, regular)
    }
}
