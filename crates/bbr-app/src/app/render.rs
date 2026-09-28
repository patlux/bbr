//! GPUI rendering for the browser shell.
//!
//! The root renders the sidebar rail (surface composition lives in
//! `app::sidebar_view`) next to the native web-view slot. All visuals come
//! from the Chassis tokens and components (DESIGN.md); the tab rows are
//! built from the controller snapshot, never from a second navigation-state
//! copy. The root also carries the production sidebar-drag listeners:
//! mouse-move follows an active drag anywhere over the window and mouse-up
//! commits it, so the 4 pt divider keeps working when the cursor leaves it.

use gpui::{
    div, prelude::*, AnyElement, App, Context, MouseButton, MouseDownEvent, Render, Window,
};

use crate::slot::NativeWebViewSlot;
use bbr_ui::components;
use bbr_ui::theme::Theme;
use bbr_web::WebViewId;

#[cfg(feature = "testkit")]
use super::command_rank::CommandResultState;
use super::BbrApp;

impl Render for BbrApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // §18.5 idle evidence: every animation frame re-renders this view.
        #[cfg(feature = "testkit")]
        {
            self.motion.root_renders += 1;
        }
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
        // The bookmark-commit benchmark's render gate: the menu closes on
        // activation (the portal never draws it again), so the main-window
        // render records the post-activation closed state while such a
        // sample is pending. A render that still shows the menu open (the
        // pre-activation state) can never commit the sample; the
        // `context_menu_open` benchmark's observation stays exclusively the
        // portal render's, so this gate never changes its semantics.
        #[cfg(feature = "testkit")]
        if matches!(
            self.performance.pending_kind(),
            Some(bbr_testkit::protocol::BenchmarkKind::ContextMenuBookmarkCommit)
        ) {
            let s = &self.context_menu.state;
            self.performance
                .observe_context_menu_state(s.open, s.focused, s.bound_view, s.point);
        }

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
        // §2 seated content card: one frame for the chassis, the card and
        // every native slot's clip.
        let card = self.content_card_frame(window);

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
        // D10: the drawn pane illustration restarts on each appearance.
        let illustration_kind = self.content_illustration();
        let illustration = self.observe_pane_illustration(illustration_kind);
        let content: gpui::AnyElement = if self.library.open {
            // The Saved Pages Library is an internal chrome surface: it
            // replaces the active view's native slot (the view was hidden
            // by `sync_library_visibility`; never destroyed — identity is
            // untouched) and wins over the error surface while open.
            self.library_surface(&theme, window, cx).into_any()
        } else if let Some(spec) = self.active_error_surface_spec() {
            // The surface replaces the active view's native slot (the view
            // was hidden by `sync_error_surface_visibility`; never
            // destroyed — identity is untouched). The panel is the
            // extracted `bbr-ui` surface (G2 Error/Crash cut); the app
            // keeps the incident lifecycle and builds the DTO.
            let mut model = spec.to_model();
            if model.illustration.is_some() {
                model.illustration = Some(illustration);
            }
            bbr_ui::surfaces::error::error_surface(
                &theme,
                &model,
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
                // §11.8a: the unresponsive pane replaces the exact active
                // view's native slot while its renderer is reported
                // unresponsive (the session and tab stay alive); every
                // other pane keeps its native slot and the slot machinery
                // is untouched. The predicate is shared with the visibility
                // sync, so the drawn surface and the hidden native view can
                // never disagree.
                if this.unresponsive_pane_over(id) {
                    return components::unresponsive_panel(
                        &theme,
                        &this.unresponsive_wait_handle,
                        &this.unresponsive_reload_handle,
                        &this.unresponsive_close_handle,
                        cx.listener(|this, _e, window, cx| {
                            this.wait_unresponsive_active(window, cx)
                        }),
                        cx.listener(|this, _e, window, cx| {
                            this.reload_unresponsive_active(window, cx)
                        }),
                        cx.listener(|this, _e, window, cx| {
                            let _ = this.close_active_tab(window, cx);
                        }),
                    )
                    .into_any();
                }
                let slot =
                    NativeWebViewSlot::new(this.engine.clone(), id).corner_radius(card.radius);
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
                illustration,
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
                illustration,
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
                !self.library.open
                    && !self.find.state.open
                    && !self.auth_sheet_is_presented()
                    && !self.context_menu.state.open
                    && !self.tab_menu.state.open,
                |root| root.track_focus(&self.chrome_handle),
            )
            .when(self.library.open, |root| {
                root.track_focus(&self.library_handle)
            })
            .when(self.find.state.open, |root| {
                root.track_focus(&self.find_handle)
            })
            .when(self.auth_sheet_is_presented(), |root| {
                root.track_focus(&self.auth_handle)
            })
            .when(self.context_menu.state.open, |root| {
                root.track_focus(&self.menu_handle)
            })
            .when(self.tab_menu.state.open, |root| {
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
                    this.on_root_left_press(window, cx);
                }),
            )
            // §11.13: a secondary press outside the open Tab/Favorite menu
            // dismisses it too (the menu card's row listeners mark presses
            // inside and stop propagation; a sidebar row's right-click
            // re-binds the menu to that row instead of dismissing).
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, _e: &MouseDownEvent, window, cx| {
                    this.dismiss_tab_menu_on_outside_press(window, cx);
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
                // native web view): clear the row and folder hover.
                // Entering the root never clears a row the pointer is
                // already over.
                if !*hovered {
                    this.clear_row_hover(cx);
                    if this.hovered_folder.take().is_some() {
                        cx.notify();
                    }
                }
            }))
            // The tab-drag listeners run alongside the sidebar-drag ones on
            // the same root; each handler is a no-op unless its own drag is
            // active, so the two drag surfaces never interfere.
            .on_mouse_move(cx.listener(Self::on_tab_drag_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::end_tab_drag))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::end_tab_drag))
            .when(self.sidebar_visible, |root| root.child(sidebar))
            .child(self.seated_content(&theme, card, content, cx))
            // §11.13: the open Tab/Favorite menu overlay — ordinary GPUI
            // chrome at the resolved clamped placement, drawn above the
            // sidebar and the web content slot (the caller owns the
            // outside-press dismissal; the card does not stop propagation).
            .when(self.tab_menu.state.open, |root| {
                root.child(self.tab_menu_overlay(&theme, window, cx))
            })
    }
}

impl BbrApp {
    /// The open Tab/Favorite menu overlay (task packet
    /// `tab-favorite-context-menus`; DESIGN.md §11.13): the production
    /// `bbr-ui` card at the resolved clamped placement with the row
    /// callbacks re-entering the app entity (the established sidebar
    /// wrapper pattern). `None` while the menu is closed.
    fn tab_menu_overlay(
        &mut self,
        theme: &Theme,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // Render milestone for the tab-menu presentation probe (DESIGN.md
        // §15 motion gate): record the menu presentation this frame actually
        // draws (open/focused, the bound source tab, the source key, the
        // cursor, the open submenu, and the clamped anchor), so a pending
        // tab-menu benchmark can commit only from a render that presented
        // the armed identity. A frame that omitted the menu records nothing.
        #[cfg(feature = "testkit")]
        {
            let s = &self.tab_menu.state;
            self.performance.observe_tab_menu_state(
                s.open,
                s.focused,
                s.bound_tab.map(|tab| tab.0),
                &s.source_key,
                s.selected,
                s.open_submenu,
                s.submenu_selected,
                [s.anchor.x, s.anchor.y, s.anchor.width, s.anchor.height],
            );
        }
        let model = self.tab_menu.state.model();
        let anchor = self.tab_menu.state.anchor;
        let submenu_rect = self.tab_menu.state.submenu_rect;
        let entity = cx.entity();
        let on_row_hover =
            move |index: usize, hovered: &bool, window: &mut Window, cx: &mut App| {
                let hovered = *hovered;
                entity.update(cx, |view, cx| {
                    view.tab_menu_select(index, hovered, window, cx)
                });
            };
        let entity = cx.entity();
        let on_row_press =
            move |index: usize, _event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                entity.update(cx, |view, cx| view.on_tab_menu_row_press(index, window, cx));
            };
        let entity = cx.entity();
        let on_submenu_hover =
            move |index: usize, hovered: &bool, window: &mut Window, cx: &mut App| {
                let hovered = *hovered;
                entity.update(cx, |view, cx| {
                    view.tab_menu_submenu_select(index, hovered, window, cx)
                });
            };
        let entity = cx.entity();
        let on_submenu_press =
            move |index: usize, _event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                entity.update(cx, |view, cx| {
                    view.on_tab_menu_submenu_press(index, window, cx)
                });
            };
        bbr_ui::surfaces::tab_menu::tab_menu_card(
            theme,
            &model,
            anchor,
            submenu_rect,
            on_row_hover,
            on_row_press,
            on_submenu_hover,
            on_submenu_press,
        )
        .into_any()
    }
}
