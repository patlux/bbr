//! Testkit-only §11.3 swipe-commit benchmark (DESIGN.md §11.3 / task packet
//! `space-footer-strip-appearance-ui`, `swipe_commit` performance layer).
//!
//! The measured sample spans the **real production gesture path**: the app
//! drives `Started` and `Moved` wheel updates plus a **zero-delta
//! `TouchPhase::Ended`** through the exact handler the sidebar's
//! `on_scroll_wheel` listener calls (`BbrApp::on_sidebar_scroll_wheel`), and
//! that ended release resolves the tracking gesture into exactly one existing
//! `activate_space` commit — never a direct activation call and never a
//! benchmark-only state machine or second switching authority. The sample
//! commits only from the authoritative post-switch snapshot showing the exact
//! persistent destination, its warm selected view, the Idle machine phase,
//! and the unchanged session count (fast-but-wrong guard, same warm class as
//! `benchmark_activate_space` — no session recreation).

use bbr_browser::SpaceId;
use gpui::{point, px, Context, ScrollDelta, ScrollWheelEvent, TouchPhase, Window};

use super::BbrApp;

impl BbrApp {
    /// Benchmark one §11.3 swipe commit into the adjacent persistent
    /// `destination` space through the production gesture path.
    ///
    /// The gesture is driven with real `Started`/`Moved` wheel updates (the
    /// signed horizontal delta is horizontal-dominant, so the production
    /// handler begins and feeds the tracking gesture; the sign follows the
    /// canonical display-order direction toward `destination`) and a real
    /// **zero-delta `TouchPhase::Ended`** — the exact release shape a macOS
    /// trackpad produces — which resolves the gesture. Above-threshold
    /// progress commits exactly one switch through `activate_space`; the
    /// machine returns Idle, so a repeated `Ended` can never double-commit.
    ///
    /// The benchmark rejects any gesture that would not be a genuine
    /// adjacent commit (unknown/identical source, boundary, or
    /// non-adjacent destination): it cancels and returns `false` instead of
    /// measuring a no-op, so absence/skipping and boundary samples are
    /// impossible by construction. The destination must stay fixed at
    /// gesture begin — the driver alternates directions so every sample is a
    /// real commit.
    pub fn benchmark_swipe_commit(
        &mut self,
        operation_id: bbr_testkit::protocol::BenchmarkOperationId,
        destination: SpaceId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self
            .performance
            .begin(
                operation_id,
                bbr_testkit::protocol::BenchmarkKind::SwipeCommit,
            )
            .is_err()
        {
            return false;
        }

        // The source is the active space; the destination must be the one
        // adjacent persistent space in the canonical chip order (the same
        // order the render, snapshot, semantics, and keyboard traversal
        // consume). Anything else is a boundary/no-op and must never be
        // measured as a commit.
        let source = self.controller.snapshot().active_space;
        let order: Vec<SpaceId> = self
            .space_chip_visuals()
            .into_iter()
            .map(|visual| visual.space)
            .collect();
        let source_at = order.iter().position(|id| *id == source);
        let destination_at = order.iter().position(|id| *id == destination);
        let Some((source_at, destination_at)) = source_at.zip(destination_at) else {
            self.performance.cancel();
            return false;
        };
        let step = destination_at as isize - source_at as isize;
        if !matches!(step, -1 | 1) {
            self.performance.cancel();
            return false;
        }
        // Display-order direction: positive delta travels `Next` (toward the
        // space after the source), negative `Prev`. A deterministic full
        // swipe (Started already beyond the commit threshold, Moved topping
        // out the clamped progress) so the zero-delta Ended resolves into
        // exactly one commit.
        let sign = if step > 0 { 1.0 } else { -1.0 };

        // The real production gesture: Started → Moved → zero-delta Ended,
        // each routed through the exact handler the sidebar's
        // `on_scroll_wheel` element listener calls. The ended release
        // resolves the machine and commits through the existing
        // `activate_space` authority (chrome focus, profile-hint, and
        // warm-session rules identical to pointer and keyboard activation).
        let started = ScrollWheelEvent {
            position: point(px(0.0), px(0.0)),
            delta: ScrollDelta::Pixels(point(px(sign * 200.0), px(0.0))),
            modifiers: gpui::Modifiers::default(),
            touch_phase: TouchPhase::Started,
        };
        self.on_sidebar_scroll_wheel(&started, window, cx);
        let moved = ScrollWheelEvent {
            position: point(px(0.0), px(0.0)),
            delta: ScrollDelta::Pixels(point(px(sign * 120.0), px(0.0))),
            modifiers: gpui::Modifiers::default(),
            touch_phase: TouchPhase::Moved,
        };
        self.on_sidebar_scroll_wheel(&moved, window, cx);
        let ended = ScrollWheelEvent {
            position: point(px(0.0), px(0.0)),
            delta: ScrollDelta::Pixels(point(px(0.0), px(0.0))),
            modifiers: gpui::Modifiers::default(),
            touch_phase: TouchPhase::Ended,
        };
        self.on_sidebar_scroll_wheel(&ended, window, cx);

        // The ended release must have committed exactly the destination: the
        // authoritative post-switch controller state is the semantic truth
        // (a double commit or a wrong destination is rejected here, and the
        // driver re-checks the typed snapshot afterwards).
        let snapshot = self.controller.snapshot();
        if snapshot.active_space != destination {
            self.performance.cancel();
            return false;
        }
        let active_space = bbr_testkit::protocol::TestSpaceId(snapshot.active_space.0);
        self.performance.observe_active_space(active_space);
        self.performance.expect_space(active_space);
        self.performance.expect_state(
            snapshot.active_view,
            self.sidebar_visible,
            self.chrome_focused(window),
            self.command_bar_mode.into(),
        );
        // §11.3 same-profile rule: a same-profile swipe must not announce a
        // profile transition, or the sample is rejected in the app.
        let hint = self.command_profile_hint.clone();
        self.performance.observe_profile_hint(hint.clone());
        self.performance.expect_profile_hint(hint);
        self.performance.commit_current_presentation();

        // The in-app semantic guard rejected the sample (fast-but-wrong or a
        // dropped commit): report a no-op so the gate fails loudly instead
        // of timing out silently.
        if self.performance.presentation_pending()() {
            self.performance.cancel();
            return false;
        }
        true
    }
}
