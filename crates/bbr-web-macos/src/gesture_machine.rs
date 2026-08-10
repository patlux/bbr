//! The pure content-gesture phase machine (task packet
//! `web-content-navigation-gestures`).
//!
//! Deterministic and AppKit-free: one instance tracks one in-flight logical
//! gesture and returns at most one [`ContentGestureCommit`] per gesture.
//! The native bridge ([`crate::content_gesture`]) feeds it device-normalized
//! finger deltas; this module owns the phase logic and the commit threshold.
//!
//! Exactly-once semantics are structural: a terminal `Ended` resolves the
//! tracking state back to `Idle` before returning the commit, so a repeated
//! terminal event can never commit twice. Cancellation, vertical dominance,
//! and sub-threshold releases all return `Idle` without committing.

use bbr_web::{ContentGestureCommit, ContentGestureDirection, ContentGesturePhase, WebViewId};

/// Full-swipe finger movement in points; a gesture commits when the
/// accumulated horizontal finger delta reaches this magnitude (the same
/// commit feel as the sidebar space swipe's half-progress threshold).
const CONTENT_SWIPE_COMMIT_POINTS: f64 = 120.0;

/// The pure gesture phase machine (deterministic, unit-tested without
/// AppKit). One instance tracks one in-flight gesture.
pub(crate) struct GestureMachine {
    state: MachineState,
}

#[derive(Debug, Clone, PartialEq)]
enum MachineState {
    Idle,
    Tracking {
        view: WebViewId,
        /// Signed accumulated finger movement (points; positive = fingers
        /// moved rightward, after natural-scroll normalization).
        finger_dx: f64,
        /// Accumulated absolute vertical finger movement (dominance test).
        finger_dy_abs: f64,
    },
}

impl GestureMachine {
    pub(crate) fn new() -> Self {
        Self {
            state: MachineState::Idle,
        }
    }

    /// Feed one logical gesture phase with the device-normalized finger
    /// deltas of the event. Returns the single commit of the gesture, if any.
    pub(crate) fn feed(
        &mut self,
        phase: ContentGesturePhase,
        view: WebViewId,
        finger_dx: f64,
        finger_dy: f64,
    ) -> Option<ContentGestureCommit> {
        match phase {
            ContentGesturePhase::Started => {
                // (Re)start tracking on the exact pane under the pointer. A
                // repeated Started restarts (defensive; AppKit emits one).
                self.state = MachineState::Tracking {
                    view,
                    finger_dx,
                    finger_dy_abs: finger_dy.abs(),
                };
                None
            }
            ContentGesturePhase::Moved => {
                let MachineState::Tracking {
                    view: _,
                    finger_dx: dx,
                    finger_dy_abs,
                } = &mut self.state
                else {
                    return None;
                };
                *dx += finger_dx;
                *finger_dy_abs += finger_dy.abs();
                let vertical_dominant = *finger_dy_abs > dx.abs();
                if vertical_dominant {
                    // Vertical dominance: this is a scroll, never a history
                    // gesture. The gesture is abandoned whole.
                    self.state = MachineState::Idle;
                }
                None
            }
            ContentGesturePhase::Ended => {
                let MachineState::Tracking {
                    view, finger_dx, ..
                } = self.state.clone()
                else {
                    return None;
                };
                self.state = MachineState::Idle;
                if finger_dx.abs() < CONTENT_SWIPE_COMMIT_POINTS {
                    return None;
                }
                let direction = if finger_dx > 0.0 {
                    // Fingers moved rightward: the current page slides right
                    // and the previous page is revealed (the iOS/Safari
                    // edge-swipe metaphor).
                    ContentGestureDirection::Back
                } else {
                    ContentGestureDirection::Forward
                };
                Some(ContentGestureCommit { view, direction })
            }
            ContentGesturePhase::Cancelled => {
                self.state = MachineState::Idle;
                None
            }
        }
    }

    pub(crate) fn is_idle(&self) -> bool {
        matches!(self.state, MachineState::Idle)
    }

    /// The view of the in-flight tracked gesture, when one exists. The
    /// native bridge uses it to keep feeding phases to the exact pane the
    /// gesture began over (pointer re-resolution happens only at `Started`).
    pub(crate) fn tracking_view(&self) -> Option<WebViewId> {
        match self.state {
            MachineState::Tracking { view, .. } => Some(view),
            MachineState::Idle => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn machine() -> GestureMachine {
        GestureMachine::new()
    }

    fn feed(
        m: &mut GestureMachine,
        phase: ContentGesturePhase,
        dx: f64,
        dy: f64,
    ) -> Option<ContentGestureCommit> {
        m.feed(phase, WebViewId(1), dx, dy)
    }

    #[test]
    fn started_moved_ended_commits_exactly_one_back() {
        let mut m = machine();
        assert_eq!(feed(&mut m, ContentGesturePhase::Started, 0.0, 0.0), None);
        assert_eq!(feed(&mut m, ContentGesturePhase::Moved, 60.0, 0.0), None);
        assert_eq!(feed(&mut m, ContentGesturePhase::Moved, 60.0, 0.0), None);
        assert_eq!(
            feed(&mut m, ContentGesturePhase::Ended, 0.0, 0.0),
            Some(ContentGestureCommit {
                view: WebViewId(1),
                direction: ContentGestureDirection::Back,
            })
        );
        // A repeated terminal event never commits twice (the Ended resolved
        // the machine back to Idle before returning the commit).
        assert_eq!(feed(&mut m, ContentGesturePhase::Ended, 0.0, 0.0), None);
    }

    #[test]
    fn fingers_left_commit_forward() {
        let mut m = machine();
        feed(&mut m, ContentGesturePhase::Started, 0.0, 0.0);
        feed(&mut m, ContentGesturePhase::Moved, -140.0, 0.0);
        assert_eq!(
            feed(&mut m, ContentGesturePhase::Ended, 0.0, 0.0),
            Some(ContentGestureCommit {
                view: WebViewId(1),
                direction: ContentGestureDirection::Forward,
            })
        );
    }

    #[test]
    fn below_threshold_release_commits_nothing() {
        let mut m = machine();
        feed(&mut m, ContentGesturePhase::Started, 0.0, 0.0);
        feed(&mut m, ContentGesturePhase::Moved, 40.0, 0.0);
        assert_eq!(feed(&mut m, ContentGesturePhase::Ended, 0.0, 0.0), None);
    }

    #[test]
    fn vertical_dominance_abandons_the_gesture() {
        let mut m = machine();
        feed(&mut m, ContentGesturePhase::Started, 0.0, 0.0);
        feed(&mut m, ContentGesturePhase::Moved, 30.0, 0.0);
        // Vertical movement now dominates: the whole gesture is abandoned.
        assert_eq!(feed(&mut m, ContentGesturePhase::Moved, 30.0, 120.0), None);
        assert!(m.is_idle());
        assert_eq!(feed(&mut m, ContentGesturePhase::Ended, 0.0, 0.0), None);
    }

    #[test]
    fn cancel_commits_nothing() {
        let mut m = machine();
        feed(&mut m, ContentGesturePhase::Started, 0.0, 0.0);
        feed(&mut m, ContentGesturePhase::Moved, 200.0, 0.0);
        assert_eq!(feed(&mut m, ContentGesturePhase::Cancelled, 0.0, 0.0), None);
        assert!(m.is_idle());
    }

    #[test]
    fn events_without_a_gesture_are_ignored() {
        let mut m = machine();
        assert_eq!(feed(&mut m, ContentGesturePhase::Moved, 10.0, 0.0), None);
        assert_eq!(feed(&mut m, ContentGesturePhase::Ended, 0.0, 0.0), None);
        assert_eq!(feed(&mut m, ContentGesturePhase::Cancelled, 0.0, 0.0), None);
    }

    #[test]
    fn restarted_gesture_accumulates_fresh() {
        let mut m = machine();
        feed(&mut m, ContentGesturePhase::Started, 0.0, 0.0);
        feed(&mut m, ContentGesturePhase::Moved, 130.0, 0.0);
        feed(&mut m, ContentGesturePhase::Started, 0.0, 0.0);
        feed(&mut m, ContentGesturePhase::Moved, 10.0, 0.0);
        // The restart discarded the earlier accumulation: below threshold.
        assert_eq!(feed(&mut m, ContentGesturePhase::Ended, 0.0, 0.0), None);
    }
}
