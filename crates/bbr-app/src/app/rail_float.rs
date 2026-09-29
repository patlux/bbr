//! Blueprint D02 (DESIGN §2 §11.4): the floating rail's app state — when
//! it shows ([`super::rail_reveal`]), where (the notched panel over the
//! card), and what holds it open. The rail itself is the unchanged docked
//! composition, drawn by the one Portal (`crate::portal::rail`); the card
//! and every web view keep their collapsed geometry, so nothing reflows.
//!
//! Keyboard focus stays in the browser window: the rail's handles are
//! focused there and a keyboard twin (`rail_float_view`) routes their keys
//! to the same listeners, so the floating rail's keyboard behaves exactly
//! like the docked one.

use std::time::Duration;

use bbr_ui::sidebar::floating::FloatingRailGeometry;
use gpui::{Context, Task, Window};

use super::rail_reveal::{
    PointerSample, RailReveal, RevealAction, RevealContext, DWELL_MS, EDGE_FULL_SCREEN,
    EDGE_WINDOWED, OVERSHOOT_MS,
};
use super::BbrApp;

/// The floating rail's app-owned state.
#[derive(Default)]
pub(crate) struct FloatingRail {
    pub reveal: RailReveal,
    /// The panel geometry the last browser-window frame resolved.
    pub geometry: Option<FloatingRailGeometry>,
    /// The entrance key (a new key replays the slide-in).
    pub entrance: u64,
    dwell: Option<Task<()>>,
    overshoot: Option<Task<()>>,
    /// The panel sliding out after a hide (its geometry and exit key); it
    /// takes no pointer and lasts `dur:fine`.
    pub leaving: Option<(FloatingRailGeometry, u64)>,
    leave: Option<Task<()>>,
    /// Exits started (diagnostics and E2E).
    pub exits: u64,
    /// The native pointer observer, kept alive for the window's lifetime.
    pub observer: Option<Box<dyn std::any::Any>>,
}

impl BbrApp {
    /// Whether the floating rail shows.
    pub(crate) fn rail_floating(&self) -> bool {
        self.floating_rail.reveal.is_floating()
    }

    /// Resolve the panel geometry for this frame (browser window): the
    /// remembered rail width, inset over the card, notched around the
    /// traffic lights. A change re-syncs the Portal's capture regions.
    pub(crate) fn record_floating_rail(&mut self, window: &Window, cx: &mut Context<Self>) {
        if !self.rail_floating() {
            self.floating_rail.geometry = None;
            return;
        }
        let size = window.bounds().size;
        let size = [f32::from(size.width), f32::from(size.height)];
        let width = bbr_ui::sidebar::SidebarGeometry::effective_width_for(
            self.sidebar.last_expanded(),
            size[0],
        );
        let lights = self.safe_area(window).traffic_lights;
        let geometry = FloatingRailGeometry::resolve(size, width, lights);
        if self.floating_rail.geometry != Some(geometry) {
            self.floating_rail.geometry = Some(geometry);
            self.sync_portal_overlays(cx);
        }
    }

    /// The Portal's capture regions for the floating rail (empty while it
    /// is hidden).
    pub(crate) fn floating_rail_regions(&self) -> Vec<[f32; 4]> {
        match (self.rail_floating(), self.floating_rail.geometry) {
            (true, Some(geometry)) => geometry.capture_rects(),
            _ => Vec::new(),
        }
    }

    /// A reveal is blocked: the rail is docked, or a surface that owns the
    /// pointer or the keyboard is open.
    fn rail_reveal_blocked(&self) -> bool {
        self.sidebar_visible
            || self.command_surface.is_open()
            || self.context_menu.state.open
            || self.tab_menu.state.open
            || self.nav_history_open()
            || self.auth_sheet_is_presented()
            || self.space_delete_open()
            || self.library.open
            || self.peek_presented()
    }

    /// An interaction holds the shown rail: a drag, an open menu or panel,
    /// an edit in its field (the rail and what it opened form one domain).
    fn rail_held(&self) -> bool {
        self.tab_drag.is_active()
            || self.tab_drag.is_armed()
            || self.sidebar.is_dragging()
            || self.command_bar_mode.is_active()
            || self.command_surface.is_open()
            || self.tab_menu.state.open
            || self.context_menu.state.open
            || self.nav_history_open()
            || self.space_config.open
            || self.u2.plus_menu.open
            || self.downloads.open
            || self.folder_rename_view().is_some()
    }

    fn reveal_context(&self, window: &Window) -> RevealContext {
        let height = f32::from(window.bounds().size.height);
        RevealContext {
            edge: if self.is_full_screen(window) {
                EDGE_FULL_SCREEN
            } else {
                EDGE_WINDOWED
            },
            rail: self
                .floating_rail
                .geometry
                .map_or([0.0; 4], |geometry| geometry.rect),
            window_height: height,
            blocked: self.rail_reveal_blocked(),
            held: self.rail_held(),
        }
    }

    /// One pointer sample from the native observer (or the testkit seam).
    pub fn on_rail_pointer(
        &mut self,
        sample: PointerSample,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.sidebar_visible {
            self.floating_rail.reveal.reset();
            return;
        }
        let context = self.reveal_context(window);
        let action = self.floating_rail.reveal.on_sample(sample, context);
        self.apply_rail_reveal(action, window, cx);
    }

    fn apply_rail_reveal(
        &mut self,
        action: RevealAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            RevealAction::None => {}
            RevealAction::ArmDwell(generation) => {
                self.floating_rail.dwell = Some(cx.spawn_in(window, async move |this, cx| {
                    cx.background_executor()
                        .timer(Duration::from_millis(DWELL_MS))
                        .await;
                    let _ = this.update_in(cx, |view, window, cx| {
                        let action = view.floating_rail.reveal.on_dwell_elapsed(generation);
                        view.apply_rail_reveal(action, window, cx);
                    });
                }));
            }
            RevealAction::Show => self.present_floating_rail(window, cx),
            RevealAction::ShowOvershoot(generation) => {
                self.present_floating_rail(window, cx);
                self.floating_rail.overshoot = Some(cx.spawn_in(window, async move |this, cx| {
                    cx.background_executor()
                        .timer(Duration::from_millis(OVERSHOOT_MS))
                        .await;
                    let _ = this.update_in(cx, |view, window, cx| {
                        let held = view.rail_held();
                        let action = view
                            .floating_rail
                            .reveal
                            .on_overshoot_elapsed(generation, held);
                        view.apply_rail_reveal(action, window, cx);
                    });
                }));
            }
            RevealAction::Hide => self.withdraw_floating_rail(true, window, cx),
        }
    }

    fn present_floating_rail(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.floating_rail.leaving = None;
        self.floating_rail.leave = None;
        self.floating_rail.entrance = self.floating_rail.reveal.generation();
        self.record_floating_rail(window, cx);
        self.sync_portal_overlays(cx);
        cx.notify();
    }

    /// The rail went away: keyboard focus inside it returns to the page,
    /// and the Portal lets go of its regions. With `exit`, the panel slides
    /// out over `dur:fine` first (drawn, but no longer taking the pointer).
    fn withdraw_floating_rail(&mut self, exit: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.floating_rail.dwell = None;
        self.floating_rail.overshoot = None;
        let geometry = self.floating_rail.geometry.take();
        self.floating_rail.leave = None;
        self.floating_rail.leaving = None;
        if let Some(geometry) = geometry.filter(|_| exit) {
            let key = self.floating_rail.reveal.generation();
            self.floating_rail.leaving = Some((geometry, key));
            self.floating_rail.exits += 1;
            self.floating_rail.leave = Some(cx.spawn_in(window, async move |this, cx| {
                cx.background_executor()
                    .timer(bbr_ui::theme::Motion::fine())
                    .await;
                let _ = this.update(cx, |view, cx| {
                    if view.floating_rail.leaving.is_some_and(|(_, k)| k == key) {
                        view.floating_rail.leaving = None;
                        view.sync_portal_overlays(cx);
                    }
                });
            }));
        }
        let rail_focused = self.tab_list_handle.is_focused(window)
            || self.u2.folders_handle.is_focused(window)
            || self.u2.plus_menu_handle.is_focused(window)
            || self.space_strip_handle.is_focused(window)
            || self.divider_handle.is_focused(window);
        if rail_focused {
            self.focus_web(window, cx);
        }
        self.sync_portal_overlays(cx);
        cx.notify();
    }

    /// ⌘L / ⌥⌘S on a hidden rail: float it (never dock).
    pub(crate) fn float_rail(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let action = self.floating_rail.reveal.show_now();
        self.apply_rail_reveal(action, window, cx);
        // The rail is on screen this frame (its field anchors ⌘L).
        self.record_floating_rail(window, cx);
    }

    /// Escape with nothing deeper to cancel: hide the floating rail.
    /// Returns whether it was shown.
    pub(crate) fn escape_floating_rail(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let action = self.floating_rail.reveal.hide_now();
        let hid = action == RevealAction::Hide;
        self.apply_rail_reveal(action, window, cx);
        hid
    }

    /// Docking, deactivation, minimize or close end the floating rail
    /// without an exit presentation.
    pub(crate) fn end_floating_rail(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.rail_floating() {
            self.floating_rail.reveal.reset();
            self.withdraw_floating_rail(false, window, cx);
        } else {
            self.floating_rail.reveal.reset();
            self.floating_rail.dwell = None;
            if self.floating_rail.leaving.take().is_some() {
                self.floating_rail.leave = None;
                self.sync_portal_overlays(cx);
            }
        }
    }
}

#[cfg(feature = "testkit")]
impl BbrApp {
    /// The reveal phase on the wire.
    pub(crate) fn rail_reveal_label(&self) -> &'static str {
        use super::rail_reveal::RevealPhase;
        match self.floating_rail.reveal.phase() {
            RevealPhase::Idle => "idle",
            RevealPhase::Dwelling { .. } => "dwelling",
            RevealPhase::Floating { .. } => "floating",
        }
    }

    /// The floating panel on the wire (while it shows).
    pub(crate) fn floating_rail_snapshot(
        &self,
    ) -> Option<bbr_testkit::protocol::FloatingRailSnapshot> {
        let geometry = self
            .floating_rail
            .geometry
            .filter(|_| self.rail_floating())?;
        Some(bbr_testkit::protocol::FloatingRailSnapshot {
            rect: geometry.rect,
            notch: geometry.notch,
            capture: geometry.capture_rects(),
            entrance: self.floating_rail.entrance,
        })
    }
}
