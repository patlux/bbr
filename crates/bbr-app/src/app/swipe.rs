//! DESIGN.md §11.3 sidebar swipe — the F2a swipe product kernel.
//!
//! A **pure** gesture state machine with explicit Idle/Tracking phases, a
//! source space and **one adjacent persistent [`SpaceId`] destination**
//! resolved from the canonical `space_chip_visuals` ordering at begin,
//! signed progress clamped to [-1, 1], and explicit update/end/cancel. The
//! machine never mutates browser state: an in-progress gesture changes no
//! [`BrowserController`] state and emits no [`ControllerEvent`]. Only the
//! app layer may commit an above-threshold release, and it does so
//! exclusively through the existing `activate_space` authority (the same
//! focus/profile-hint/session path pointer and keyboard activation use) —
//! there is never a second switching path.
//!
//! # Gesture semantics
//!
//! - [`SwipeGesture::begin`] resolves the destination as the adjacent space
//!   in canonical display order (`Next` = +1, `Prev` = -1). Boundary
//!   gestures (no adjacent space) fail to begin and leave the machine Idle.
//! - [`SwipeGesture::feed_raw`] converts a raw platform scroll delta
//!   (positive = display `Next`, negative = `Prev`) into progress toward
//!   the fixed destination, so a full swipe in **either** direction reaches
//!   `+1`; reversal pushes progress back toward and past zero.
//! - [`SwipeGesture::end`] returns exactly one typed commit intent
//!   (`Some(destination)`) when `progress >= SWIPE_COMMIT_THRESHOLD`,
//!   otherwise `None` (below-threshold release cancels); the machine
//!   returns Idle either way, so a repeated `end` can never double-commit.
//! - [`SwipeGesture::cancel`] returns Idle with no commit intent.
//!
//! # Delta normalization and phase routing (deterministic)
//!
//! [`normalized_delta_x`] maps a GPUI [`ScrollDelta`] onto a signed
//! progress step: precise trackpad pixels divide by [`SWIPE_PIXELS_TO_FULL`],
//! wheel lines by [`SWIPE_LINES_TO_FULL`]. [`is_horizontal_dominant`] keeps
//! `Started`/`Moved` deltas out of the gesture unless horizontal-dominant,
//! so ordinary vertical strip/tab scrolling is preserved — **but `Ended`
//! resolves whenever a gesture tracks, independent of the delta**, because
//! a real macOS release event carries a zero (or residual) `scrollingDelta`
//! (gpui mac/events.rs maps `NSEventPhaseEnded` → `TouchPhase::Ended`).
//!
//! # Chrome-only interpolation
//!
//! While tracking, [`BbrApp::swipe_tint`] derives the source/destination
//! chrome presentations from the one central [`crate::theme::space_tint`]
//! resolver and blends them by the signed progress
//! ([`interpolate_tint`], progress clamped to [0, 1] so a reversal returns
//! the exact source). Only BBR-owned sidebar material and the active-chip
//! presentation consume the interpolation; text, icons, hairlines, accent
//! rails, WebViews, Portal, and native geometry are never tinted, animated,
//! sampled, or reparented. Cancel returns the exact source presentation;
//! commit lands on the exact destination presentation.

use bbr_browser::{SpaceId, SpaceTheme};
use gpui::{Context, Rgba, ScrollDelta, ScrollWheelEvent, TouchPhase, Window};

use crate::theme::{space_tint, SpaceTint};

use super::BbrApp;

/// The signed progress magnitude that commits on release. Deterministic
/// and presentation-only: 0.5 means "halfway toward the adjacent space".
pub const SWIPE_COMMIT_THRESHOLD: f64 = 0.5;
/// Natural trackpad pixels required to reach full commit progress (a
/// deterministic presentation constant — never a platform read).
const SWIPE_PIXELS_TO_FULL: f64 = 240.0;
/// Non-precise wheel lines required to reach full commit progress (a
/// deterministic line → progress mapping for plain mouse wheels).
const SWIPE_LINES_TO_FULL: f64 = 4.0;

/// The explicit machine phase (DESIGN.md §11.3: in-progress gestures
/// commit nothing).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipePhase {
    Idle,
    Tracking,
}

impl SwipePhase {
    /// The canonical machine-readable form the typed snapshot exposes.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // §15 snapshot seam
    pub fn name(self) -> &'static str {
        match self {
            SwipePhase::Idle => "idle",
            SwipePhase::Tracking => "tracking",
        }
    }
}

/// The display-order direction a gesture travels from its source space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeDirection {
    /// Toward the next space in canonical display order (positive delta).
    Next,
    /// Toward the previous space in canonical display order (negative delta).
    Prev,
}

impl SwipeDirection {
    /// The canonical machine-readable form the typed snapshot exposes.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // §15 snapshot seam
    pub fn name(self) -> &'static str {
        match self {
            SwipeDirection::Next => "next",
            SwipeDirection::Prev => "prev",
        }
    }
}

/// One in-flight swipe gesture: the source space, the one adjacent
/// persistent destination resolved at begin, the display-order direction,
/// and the signed progress. `progress` is clamped to [-1, 1] where positive
/// is toward `destination`; the destination never changes mid-gesture.
#[derive(Debug, Clone, PartialEq)]
pub struct SwipeGesture {
    phase: SwipePhase,
    source: SpaceId,
    destination: SpaceId,
    direction: SwipeDirection,
    progress: f64,
}

impl SwipeGesture {
    /// Begin a gesture from `source` toward its adjacent space in the
    /// canonical display `order` (the same `space_chip_visuals` ordering the
    /// render, the snapshot, and the semantic tree consume — persistent ids,
    /// never render positions). Boundary gestures (first space swiped `Prev`,
    /// last space swiped `Next`, or an unknown source) return `None` and the
    /// machine stays Idle. The destination is fixed at begin.
    pub fn begin(
        order: &[SpaceId],
        source: SpaceId,
        direction: SwipeDirection,
    ) -> Option<SwipeGesture> {
        let index = order.iter().position(|id| *id == source)?;
        let step: isize = match direction {
            SwipeDirection::Next => 1,
            SwipeDirection::Prev => -1,
        };
        let target = index as isize + step;
        let destination = (0..order.len() as isize)
            .contains(&target)
            .then(|| order[target as usize])?;
        Some(SwipeGesture {
            phase: SwipePhase::Tracking,
            source,
            destination,
            direction,
            progress: 0.0,
        })
    }

    /// Feed a signed normalized delta toward the fixed destination; the
    /// progress is clamped to [-1, 1]. A no-op once the gesture has
    /// ended/cancelled (repeated `end` cannot double-commit). `delta` is
    /// **progress toward the destination** (positive toward it) — the
    /// machine contract (see also [`SwipeGesture::feed_raw`] for raw
    /// platform deltas).
    pub fn update(&mut self, delta: f64) {
        if self.phase != SwipePhase::Tracking {
            return;
        }
        self.progress = (self.progress + delta).clamp(-1.0, 1.0);
    }

    /// Feed a **raw signed scroll delta** as the production handler receives
    /// it (positive magnitude = display `Next`, negative = display `Prev`),
    /// converted to progress toward this gesture's fixed destination — so a
    /// full swipe in either display direction reaches `+1.0` and the
    /// threshold commit and the chrome interpolation behave identically in
    /// both directions. Reversing direction pushes progress back toward and
    /// past zero (the destination stays fixed at begin).
    pub fn feed_raw(&mut self, delta: f64) {
        let sign = match self.direction {
            SwipeDirection::Next => 1.0,
            SwipeDirection::Prev => -1.0,
        };
        self.update(delta * sign);
    }

    /// End the gesture: returns exactly one commit intent (`Some(destination)`)
    /// when the signed progress reaches [`SWIPE_COMMIT_THRESHOLD`], otherwise
    /// `None` (below-threshold release restores the source with no switch).
    /// The machine returns Idle either way — a second `end` is a no-op.
    pub fn end(&mut self) -> Option<SpaceId> {
        if self.phase != SwipePhase::Tracking {
            return None;
        }
        self.phase = SwipePhase::Idle;
        (self.progress >= SWIPE_COMMIT_THRESHOLD).then_some(self.destination)
    }

    /// Cancel the gesture: Idle, no commit intent, exact source preserved.
    pub fn cancel(&mut self) {
        self.phase = SwipePhase::Idle;
    }

    pub fn phase(&self) -> SwipePhase {
        self.phase
    }

    pub fn source(&self) -> SpaceId {
        self.source
    }

    pub fn destination(&self) -> SpaceId {
        self.destination
    }

    /// The display-order direction resolved at begin.
    #[cfg_attr(not(feature = "testkit"), allow(dead_code))] // §15 snapshot seam
    pub fn direction(&self) -> SwipeDirection {
        self.direction
    }

    pub fn progress(&self) -> f64 {
        self.progress
    }
}

/// The app-owned swipe state: exactly one gesture at a time. Idle unless a
/// horizontal trackpad gesture is in progress; commits only through the
/// existing activation authority.
#[derive(Default)]
pub struct SwipeState {
    gesture: Option<SwipeGesture>,
}

impl SwipeState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn gesture(&self) -> Option<&SwipeGesture> {
        self.gesture.as_ref()
    }

    pub fn set_gesture(&mut self, gesture: Option<SwipeGesture>) {
        self.gesture = gesture;
    }

    pub fn take_gesture(&mut self) -> Option<SwipeGesture> {
        self.gesture.take()
    }
}

/// The deterministic signed horizontal progress step of one [`ScrollDelta`]:
/// precise trackpad pixels divide by [`SWIPE_PIXELS_TO_FULL`], wheel lines
/// by [`SWIPE_LINES_TO_FULL`]. The sign is the display-order direction.
pub fn normalized_delta_x(delta: &ScrollDelta) -> f64 {
    match delta {
        ScrollDelta::Pixels(pixels) => f64::from(pixels.x) / SWIPE_PIXELS_TO_FULL,
        ScrollDelta::Lines(lines) => f64::from(lines.x) / SWIPE_LINES_TO_FULL,
    }
}

/// The raw `(x, y)` magnitudes of one delta on its own scale (pixels for
/// precise trackpad events, lines for wheel events) — the dominance test
/// compares within one event type, never across scales.
fn axis_magnitudes(delta: &ScrollDelta) -> (f64, f64) {
    match delta {
        ScrollDelta::Pixels(pixels) => (f64::from(pixels.x).abs(), f64::from(pixels.y).abs()),
        ScrollDelta::Lines(lines) => (f64::from(lines.x).abs(), f64::from(lines.y).abs()),
    }
}

/// Horizontal-dominant deltas only: a wheel event participates in the swipe
/// exactly when its horizontal magnitude strictly exceeds its vertical
/// magnitude. Vertical-dominant and equal-axis deltas are never gestures, so
/// ordinary strip/tab scrolling (including GPUI's axis fallback on the
/// horizontal strip) is preserved untouched.
pub fn is_horizontal_dominant(delta: &ScrollDelta) -> bool {
    let (x, y) = axis_magnitudes(delta);
    x > y
}

/// Blend two resolved chrome presentations by `t` (clamped to [0, 1]) —
/// the chrome-only interpolation: source at `t = 0`, destination at
/// `t = 1`, per-channel linear blend. Negative progress (a reversal past
/// the source) clamps to the exact source presentation.
pub fn interpolate_tint(source: SpaceTint, destination: SpaceTint, t: f64) -> SpaceTint {
    let t = t.clamp(0.0, 1.0) as f32;
    let lerp = |a: Rgba, b: Rgba| Rgba {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: 1.0,
    };
    SpaceTint {
        hue: lerp(source.hue, destination.hue),
        chrome: lerp(source.chrome, destination.chrome),
        chip_active: lerp(source.chip_active, destination.chip_active),
    }
}

/// What the production handler does with one scroll-wheel event — the pure
/// phase/delta/tracking routing of [`BbrApp::on_sidebar_scroll_wheel`],
/// shared by the handler and its tests (exactly one authority, no window
/// needed).
///
/// The critical rule: **`Ended` resolves whenever a gesture tracks,
/// independent of horizontal dominance** — a real macOS release event
/// carries a zero (or residual) delta (gpui mac/events.rs maps
/// `NSEventPhaseEnded` → `TouchPhase::Ended` from the raw
/// `scrollingDeltaX/Y`), so gating `Ended` on dominance would strand the
/// machine in Tracking forever. Dominance applies only where `Started`/
/// `Moved` deltas participate, so ordinary vertical strip/tab scrolling
/// stays untouched.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum SwipeEventRoute {
    /// A horizontal-dominant `Started`: attempt to begin a tracking gesture
    /// and feed its begin delta.
    Begin { delta: f64 },
    /// A horizontal-dominant `Moved` while tracking: feed the delta.
    Feed { delta: f64 },
    /// An `Ended` while tracking: resolve the gesture — below-threshold
    /// release returns Idle with no switch, above-threshold release commits
    /// exactly one switch; the machine is Idle either way, so a repeated
    /// `Ended` can never double-commit.
    ResolveEnd,
    /// Everything else — vertical/non-horizontal deltas, idle `Started`/
    /// `Moved`, releases without a tracking gesture — is a no-op.
    Ignore,
}

/// The deterministic phase/delta routing of the production handler.
pub(crate) fn route_event(
    phase: TouchPhase,
    delta: &ScrollDelta,
    tracking: bool,
) -> SwipeEventRoute {
    match (phase, tracking) {
        (TouchPhase::Ended, true) => SwipeEventRoute::ResolveEnd,
        (TouchPhase::Started, _) if is_horizontal_dominant(delta) => SwipeEventRoute::Begin {
            delta: normalized_delta_x(delta),
        },
        (TouchPhase::Moved, true) if is_horizontal_dominant(delta) => SwipeEventRoute::Feed {
            delta: normalized_delta_x(delta),
        },
        _ => SwipeEventRoute::Ignore,
    }
}

impl BbrApp {
    /// The §11.3 production sidebar wheel handler — the exact method the
    /// sidebar's `on_scroll_wheel` element listener calls.
    ///
    /// - `Ended` resolves a tracking gesture **independent of the delta**:
    ///   a real macOS release event carries a zero (or residual)
    ///   `scrollingDelta` (gpui mac/events.rs maps `NSEventPhaseEnded` →
    ///   `TouchPhase::Ended` from the raw deltas), so gating `Ended` on
    ///   horizontal dominance would strand the machine in Tracking forever.
    ///   Above-threshold release commits exactly one switch through
    ///   [`BbrApp::activate_space`] — the same authority pointer and
    ///   keyboard activation use; below-threshold releases restore the
    ///   source with no event. A release without a tracking gesture is a
    ///   no-op (a repeated `Ended` can never double-commit).
    /// - `Started` begins a tracking gesture (boundary gestures stay Idle);
    ///   the begin delta already feeds progress.
    /// - `Moved` feeds progress only while tracking; an idle `Moved` (e.g.
    ///   momentum after a committed gesture) never starts a new gesture.
    /// - `Started`/`Moved` deltas participate only when horizontal-dominant
    ///   (vertical-dominant and non-horizontal deltas are ignored and never
    ///   stopped), so ordinary strip/tab scrolling is preserved.
    ///
    /// In-progress handling mutates only the app swipe state and the
    /// chrome-only presentation: no [`BrowserController`] state and no
    /// [`ControllerEvent`] until a commit.
    pub(crate) fn on_sidebar_scroll_wheel(
        &mut self,
        event: &ScrollWheelEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match route_event(
            event.touch_phase,
            &event.delta,
            self.swipe.gesture().is_some(),
        ) {
            SwipeEventRoute::Begin { delta } => {
                self.begin_swipe(event, cx);
                if let Some(gesture) = self.swipe.gesture.as_mut() {
                    // Raw signed delta → progress toward the fixed
                    // destination (positive toward it in either display
                    // direction), so a full `Prev` swipe commits exactly
                    // like a full `Next` swipe.
                    gesture.feed_raw(delta);
                }
                cx.notify();
                cx.stop_propagation();
            }
            SwipeEventRoute::Feed { delta } => {
                if let Some(gesture) = self.swipe.gesture.as_mut() {
                    gesture.feed_raw(delta);
                    cx.notify();
                    cx.stop_propagation();
                }
            }
            SwipeEventRoute::ResolveEnd => {
                if let Some(mut gesture) = self.swipe.take_gesture() {
                    if let Some(destination) = gesture.end() {
                        // The one activation authority: same focus/hint/
                        // session rules as pointer and keyboard activation.
                        self.activate_space(destination, window, cx);
                    }
                    cx.notify();
                    cx.stop_propagation();
                }
            }
            SwipeEventRoute::Ignore => {}
        }
    }

    /// Resolve the adjacent persistent destination from the canonical
    /// `space_chip_visuals` ordering at gesture begin. Boundary/no-adjacent
    /// gestures leave the machine Idle (they never commit).
    fn begin_swipe(&mut self, event: &ScrollWheelEvent, cx: &mut Context<Self>) {
        let order: Vec<SpaceId> = self
            .space_chip_visuals()
            .into_iter()
            .map(|visual| visual.space)
            .collect();
        let source = self.controller.snapshot().active_space;
        let direction = if normalized_delta_x(&event.delta) >= 0.0 {
            SwipeDirection::Next
        } else {
            SwipeDirection::Prev
        };
        self.swipe
            .set_gesture(SwipeGesture::begin(&order, source, direction));
        let _ = cx;
    }

    /// Explicitly cancel a tracking gesture: no commit, the chrome tint
    /// returns to the exact source presentation. A no-op while Idle. Also
    /// invoked by [`BbrApp::activate_space`] so any manual activation ends a
    /// stale gesture (a stale destination must never commit a second switch).
    pub(crate) fn cancel_swipe(&mut self, cx: &mut Context<Self>) {
        if let Some(mut gesture) = self.swipe.take_gesture() {
            gesture.cancel();
            cx.notify();
        }
    }

    /// The interpolated chrome presentation while a swipe tracks: the source
    /// and destination `SpaceTint`s resolved from the one central
    /// `space_tint` resolver, blended by the signed progress (a reversal
    /// returns the exact source). `None` while Idle.
    pub(crate) fn swipe_tint(&self) -> Option<SpaceTint> {
        let gesture = self.swipe.gesture()?;
        if gesture.phase() != SwipePhase::Tracking {
            return None;
        }
        let theme = self.theme();
        let resolve = |space: SpaceId| {
            self.controller
                .space_appearance(space)
                .map(|appearance| appearance.theme)
                .unwrap_or(SpaceTheme::Ink)
        };
        let source = space_tint(resolve(gesture.source()), &theme);
        let destination = space_tint(resolve(gesture.destination()), &theme);
        Some(interpolate_tint(source, destination, gesture.progress()))
    }

    /// The §11.3 sidebar rail chrome the render paints: the interpolated
    /// chrome while a swipe tracks, otherwise the active space's resolved
    /// chrome from the one central resolver. Chrome-only — text, hairlines,
    /// accents, icons, and web content keep their own tokens.
    pub(crate) fn active_chrome_tint(&self) -> Rgba {
        self.swipe_tint().map_or_else(
            || space_tint(self.active_space_appearance().theme, &self.theme()).chrome,
            |tint| tint.chrome,
        )
    }

    /// The typed §11.3 swipe wire evidence: the truthful active space at
    /// every phase plus the gesture's source/destination/direction/progress
    /// and the resolved interpolated tints while tracking.
    #[cfg(feature = "testkit")]
    pub(crate) fn swipe_snapshot(&self) -> bbr_testkit::protocol::SwipeSnapshot {
        use bbr_testkit::protocol::TestSpaceId;
        let gesture = self.swipe.gesture();
        let tint = self.swipe_tint();
        bbr_testkit::protocol::SwipeSnapshot {
            active: TestSpaceId(self.controller.snapshot().active_space.0),
            phase: gesture
                .map_or("idle", |gesture| gesture.phase().name())
                .to_string(),
            source: gesture.map(|gesture| TestSpaceId(gesture.source().0)),
            destination: gesture.map(|gesture| TestSpaceId(gesture.destination().0)),
            direction: gesture.map(|gesture| gesture.direction().name().to_string()),
            progress: gesture.map_or(0.0, |gesture| gesture.progress()),
            interpolated_chrome_tint: tint.map(|tint| crate::theme::hex(tint.chrome)),
            interpolated_chip_tint: tint.map(|tint| crate::theme::hex(tint.chip_active)),
        }
    }
}

#[cfg(test)]
mod tests;
