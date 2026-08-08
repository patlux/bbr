//! Visible profile/space navigation (DESIGN.md §11.3, §12).
//!
//! The sidebar's footer renders every space as a chip in the permanent
//! §11.3/§11.4 space strip; this module owns the production activation
//! paths: pointer chip selection, keyboard traversal (`↑`/`↓`/Enter on the
//! focused space strip), and the `Cmd-Shift-N` / New Space row staging that
//! creates a space in the active profile through the command bar.
//!
//! Activation keeps the chrome focus domain and routes through the portable
//! controller, so switching spaces restores each space's own active tab while
//! warm sessions stay warm — including across profile boundaries (recreation
//! stays reserved for actual cross-profile tab moves).

use bbr_browser::{ControllerEvent, SpaceAppearance, SpaceId, SpaceTheme};
use gpui::{Context, KeyDownEvent, Window};

use crate::command_bar::CommandBarMode;

use super::BbrApp;

impl BbrApp {
    /// The production space activation path: switch the controller's active
    /// space, keep chrome focus, and mirror the destination's tab into the
    /// command bar. Warm tabs and native sessions are preserved by the
    /// controller's show/hide policy. Cross-profile activations set the §11.3
    /// cross-context command-field hint for one interaction; same-profile
    /// switching never announces a profile transition (and clears any prior
    /// hint).
    pub fn activate_space(
        &mut self,
        space: SpaceId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let source_profile = self.controller.active_profile();
        let result = self
            .controller
            .activate_space(space)
            .map_err(|error| {
                log::error!("could not activate space {space:?}: {error}");
                error
            })
            .ok();
        if let Some(outcome) = &result {
            self.sync_command_bar();
            // Space activation switches the active session; a find
            // presentation bound to the previous space's view ends.
            self.find_discard_if_unbound(cx);
            self.emit_browser(outcome);
            self.focus_space_strip(window, cx);
            self.ensure_active_visible();
            // DESIGN.md §11.3: the selected chip is scrolled fully into view
            // deterministically (no reordering, no wrapping) whenever the
            // strip overflows.
            self.scroll_selected_chip_into_view();
            // DESIGN.md §11.3 cross-context rule: only a cross-profile
            // activation announces the destination profile at the top of the
            // command field for one interaction; same-profile switching must
            // never misleadingly announce a profile transition.
            if self.controller.active_profile() != source_profile {
                let title = self
                    .controller
                    .snapshot()
                    .active_profile_title
                    .clone()
                    .unwrap_or_else(|| "other".to_string());
                self.command_profile_hint = Some(crate::shell::profile_line(&title));
            } else {
                self.command_profile_hint = None;
            }
            #[cfg(feature = "testkit")]
            self.commit_benchmark_sync(window);
            cx.notify();
        }
        result
    }

    /// Pointer activation of a visible space chip: chrome focus domain
    /// preserved, activation routed through the same controller mutation the
    /// keyboard path uses (DESIGN.md §11.3 / interaction contract).
    pub fn select_space_via_pointer(
        &mut self,
        space: SpaceId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.focus_space_strip(window, cx);
        if self.activate_space(space, window, cx).is_some() {
            cx.notify();
        }
    }

    /// Focus the space strip (chrome domain) without activating the command
    /// bar — the pointer path into the §11.3 chips.
    pub(super) fn focus_space_strip(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.command_bar_mode = CommandBarMode::Inactive;
        self.command_bar_return = None;
        self.command_ranking.close();
        window.focus(&self.space_strip_handle, cx);
        self.engine.borrow_mut().focus_host_chrome();
        cx.notify();
        self.emit_focus(window);
    }

    /// DESIGN.md §11.3 overflow contract: after any space activation the
    /// selected chip is brought fully into view through the strip's tracked
    /// scroll handle (minimal scroll, no reordering). The selected chip's
    /// canonical rendered position is its index in the same
    /// [`Self::space_chip_visuals`] vector the footer render consumes, so the
    /// tracked handle scrolls exactly the chip that is visible on screen —
    /// no separate shell authority derives the position. GPUI applies the
    /// pending item during the next layout of the scroll container, so the
    /// next frame presents the selected chip fully visible.
    fn scroll_selected_chip_into_view(&self) {
        let position = self
            .space_chip_visuals()
            .iter()
            .position(|visual| visual.selected);
        if let Some(position) = position {
            self.space_strip_scroll.scroll_to_item(position);
        }
    }

    /// The effective appearance of the active space from the one controller
    /// authority (the §11.3 chrome-tint source). The deterministic fallback
    /// only covers the invariant-forbidden empty browser; the controller's
    /// effective appearance (explicit record or profile-hue default) is
    /// authoritative for every real space.
    pub(crate) fn active_space_appearance(&self) -> SpaceAppearance {
        let active = self.controller.snapshot().active_space;
        self.controller
            .space_appearance(active)
            .unwrap_or_else(|| SpaceAppearance::default_for(active))
    }

    /// Set the curated theme token of `space` through the one controller
    /// appearance authority (the explicit record becomes authoritative over
    /// the derived profile-hue default). `theme` is the canonical token
    /// name; unknown names reject whole with a message.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // testkit seam
    pub(crate) fn set_space_theme(
        &mut self,
        space: SpaceId,
        theme: &str,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        let token = SpaceTheme::ALL
            .iter()
            .find(|candidate| candidate.name() == theme)
            .copied()
            .ok_or_else(|| format!("unknown theme {theme:?}"))?;
        self.controller
            .set_space_theme(space, token)
            .map_err(|error| error.to_string())?;
        cx.notify();
        Ok(())
    }

    /// The §11.3 chip visuals for the current render, snapshot, and semantic
    /// tree: one chip per space in canonical grouped order (profile groups,
    /// creation order within each profile), each consuming the effective
    /// space appearance from the one controller authority and the app's
    /// current drag-to-chip state. The render, the snapshot, and the
    /// semantic tree all call this same method so the chip ids and states
    /// can never drift.
    pub(crate) fn space_chip_visuals(&self) -> Vec<crate::components::SpaceChipVisuals> {
        let browser = self.controller.snapshot();
        let budget = crate::shell::space_title_budget_chars(&self.theme());
        // The one drag-state source: the exact persistent chip the pointer
        // currently hovers (None at rest / over another zone).
        let drag_chip = self.tab_drag.chip_target();
        crate::shell::space_groups(&browser, budget)
            .into_iter()
            .flat_map(|group| group.rows.into_iter())
            .map(|row| {
                let space = browser
                    .spaces
                    .iter()
                    .find(|s| s.index == row.index)
                    .expect("space row derives from the snapshot");
                let appearance = self
                    .controller
                    .space_appearance(space.id)
                    .expect("space exists");
                let targeted = drag_chip == Some(space.id);
                crate::components::SpaceChipVisuals {
                    // The persistent id is the chip identity; the vector
                    // position is display order only.
                    space: space.id,
                    title: row.title.to_string(),
                    identity: appearance.identity.name(),
                    theme: appearance.theme,
                    selected: space.active,
                    // §10.9/§11.3: the targeted chip renders the drag-target
                    // treatment plus the §12 destination-profile companion
                    // (the destination space's profile — monogram + hue +
                    // the profile chip's creation-order index), derived from
                    // the same controller snapshot the chip order consumes.
                    drag_target: targeted,
                    companion: targeted
                        .then(|| {
                            browser
                                .profiles
                                .iter()
                                .find(|profile| profile.id == space.profile)
                                .map(|profile| crate::components::ProfileCompanion {
                                    profile_index: profile.index,
                                    letter: crate::shell::space::profile_monogram(&profile.title),
                                    hue: profile.hue,
                                })
                        })
                        .flatten(),
                }
            })
            .collect()
    }

    /// Keyboard navigation of the space strip (DESIGN.md §9 order):
    /// `↑/↓` activate the previous/next space in canonical render order,
    /// Enter re-activates the focused chip. Runs only while the strip owns
    /// chrome focus and no command mode is active. Activation keeps the
    /// chrome domain — the keyboard equivalent of clicking a chip.
    pub(super) fn on_space_strip_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.command_bar_mode.is_active() {
            return;
        }
        let snapshot = self.controller.snapshot();
        if snapshot.spaces.is_empty() {
            return;
        }
        // Canonical order is the grouped sidebar order (DESIGN.md §11.3).
        let order: Vec<SpaceId> = crate::shell::space_groups(&snapshot, usize::MAX)
            .into_iter()
            .flat_map(|group| group.rows.into_iter())
            .map(|row| {
                snapshot
                    .spaces
                    .iter()
                    .find(|s| s.index == row.index)
                    .expect("space row derives from the snapshot")
                    .id
            })
            .collect();
        let current = order
            .iter()
            .position(|id| *id == snapshot.active_space)
            .unwrap_or(0);
        match event.keystroke.key.as_str() {
            "up" | "down" => {
                let len = order.len();
                let delta = if event.keystroke.key == "up" { -1 } else { 1 };
                let next = (current as isize + delta).rem_euclid(len as isize) as usize;
                if self.activate_space(order[next], window, cx).is_some() {
                    cx.notify();
                }
                cx.stop_propagation();
            }
            "enter" => {
                // Re-activate the focused space (no-op when already active).
                if self.activate_space(order[current], window, cx).is_some() {
                    cx.notify();
                }
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    /// §11.3/§11.4 read-only strip scroll evidence for the E2E snapshot:
    /// the tracked handle's current viewport, signed offset, max offset, and
    /// one raw layout bounds rect per chip in canonical display order (the
    /// same vector the render, the snapshot, and the scroll seam consume).
    /// The E2E combines a raw chip rect with the signed offset to assert the
    /// selected chip lies fully inside the viewport after an activation
    /// (the §11.3 overflow rule). Never mutates the handle.
    #[cfg(feature = "testkit")]
    pub(crate) fn space_strip_scroll_snapshot(
        &self,
    ) -> bbr_testkit::protocol::SpaceStripScrollSnapshot {
        let scroll = &self.space_strip_scroll;
        let viewport = scroll.bounds();
        let offset = scroll.offset();
        let max_offset = scroll.max_offset();
        // GPUI's `Pixels` field is crate-private; the public `From<Pixels>
        // for f32` conversion keeps the f32 wire values without reaching
        // into the tuple field.
        let px = f32::from;
        let count = self.space_chip_visuals().len();
        let chip_bounds = (0..count)
            .map(|index| {
                scroll.bounds_for_item(index).map_or([0.0; 4], |bounds| {
                    [
                        px(bounds.left()),
                        px(bounds.top()),
                        px(bounds.size.width),
                        px(bounds.size.height),
                    ]
                })
            })
            .collect();
        bbr_testkit::protocol::SpaceStripScrollSnapshot {
            viewport: [
                px(viewport.left()),
                px(viewport.top()),
                px(viewport.size.width),
                px(viewport.size.height),
            ],
            offset: [px(offset.x), px(offset.y)],
            max_offset: [px(max_offset.x), px(max_offset.y)],
            chip_bounds,
        }
    }

    /// `Cmd-Shift-N` / New Space row: stage a space name without creating a
    /// profile or space until Enter commits it (the NewTab staging pattern).
    pub fn activate_new_space(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // A rail mode entry replaces any open palette/overlay presentation
        // (the surface state couples to Location/NewTab only).
        self.dismiss_command_portal(cx);
        self.clear_command_profile_hint();
        if self.command_bar_mode != CommandBarMode::NewSpace {
            self.command_bar_return = Some(crate::command_bar::CommandBarReturn {
                mode: self.command_bar_mode,
                text: self.command_text.text.clone(),
                chrome_focused: self.chrome_focused(window),
            });
        }
        self.command_bar_mode = CommandBarMode::NewSpace;
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

    /// Commit a staged New Space name: create the space in the active
    /// profile and activate it. Chrome focus stays in the space strip — a
    /// fresh space is empty, so there is no page to hand focus to.
    pub fn commit_new_space(
        &mut self,
        name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<ControllerEvent> {
        let name = name.trim();
        if name.is_empty() {
            return None;
        }
        let profile = self.controller.active_profile();
        let result = self
            .controller
            .create_space_event(profile, name)
            .map_err(|error| {
                log::error!("could not create space {name:?}: {error}");
                error
            })
            .ok();
        let outcome = result?;
        let space = match outcome {
            ControllerEvent::SpaceCreated { space, .. } => space,
            other => {
                log::error!("unexpected create-space outcome: {other:?}");
                return None;
            }
        };
        // The creation itself is a semantic browser mutation and must be
        // observable like every other mutation path (the gateway's CreateSpace
        // emits SpaceCreated; the keyboard path must emit the same event so
        // E2E and future consumers can distinguish creation from activation).
        self.emit_browser(&outcome);
        self.activate_space(space, window, cx);
        // The new space is a structural mutation: poke the one browser-state
        // writer so a force quit recovers the committed model with the new
        // space.
        self.persist_structure();
        Some(outcome)
    }

    #[cfg(feature = "testkit")]
    pub fn benchmark_activate_space(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        space: SpaceId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::WarmSpaceSwitch,
            )
            .is_err()
        {
            return false;
        }
        // The production activation path, exactly as the row mouse-down
        // listener drives it (chrome focus domain preserved). The benchmark
        // commits synchronously from the authoritative post-mutation snapshot
        // with the destination space as the semantic state check.
        let outcome = self.activate_space(space, window, cx);
        if outcome.is_none() {
            self.performance.cancel();
            return false;
        }
        let snapshot = self.controller.snapshot();
        let active_space = bbr_testkit::protocol::TestSpaceId(snapshot.active_space.0);
        self.performance.observe_active_space(active_space);
        self.performance.expect_space(active_space);
        self.performance.expect_state(
            snapshot.active_view,
            self.sidebar_visible,
            self.chrome_focused(window),
            self.command_bar_mode.into(),
        );
        // The §11.3 cross-context hint is part of the presented state: a
        // cross-profile activation must show it and a same-profile activation
        // must not, or the sample is rejected in the app (fast-but-wrong
        // guard).
        let hint = self.command_profile_hint.clone();
        self.performance.observe_profile_hint(hint.clone());
        self.performance.expect_profile_hint(hint);
        self.performance.commit_current_presentation();
        true
    }
}
