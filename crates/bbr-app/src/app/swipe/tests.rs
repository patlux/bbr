//! Unit tests for the pure sidebar-swipe gesture machine, its deterministic
//! delta normalization/dominance math, the chrome-only interpolation, and
//! the production handler's phase/delta routing (DESIGN.md §11.3).
//!
//! The routing tests cover the exact gating the window `on_scroll_wheel`
//! listener applies — most importantly that a **zero-delta `Ended` resolves
//! a tracking gesture** (a real macOS release event carries a zero delta),
//! in both the below-threshold cancel and the above-threshold commit case,
//! and that a repeated `Ended` without a tracking gesture is a no-op. The
//! machine itself stays pure; these tests never construct a window.

use super::*;
use gpui::{point, px};

fn spaces(ids: &[u64]) -> Vec<SpaceId> {
    ids.iter().map(|id| SpaceId(*id)).collect()
}

/// The delta a real macOS trackpad release carries: zero on both axes.
fn zero_delta() -> ScrollDelta {
    ScrollDelta::Pixels(gpui::Point::default())
}

#[test]
fn begin_resolves_the_adjacent_persistent_destination_in_canonical_order() {
    let order = spaces(&[2, 4, 7, 6]);
    let next =
        SwipeGesture::begin(&order, SpaceId(4), SwipeDirection::Next).expect("adjacent next");
    assert_eq!(next.phase(), SwipePhase::Tracking);
    assert_eq!(next.source(), SpaceId(4));
    assert_eq!(next.destination(), SpaceId(7), "adjacent persistent id");
    assert_eq!(next.direction(), SwipeDirection::Next);
    assert_eq!(next.progress(), 0.0);

    let prev =
        SwipeGesture::begin(&order, SpaceId(4), SwipeDirection::Prev).expect("adjacent prev");
    assert_eq!(prev.destination(), SpaceId(2));
    assert_eq!(prev.direction(), SwipeDirection::Prev);
}

#[test]
fn boundary_gestures_never_begin() {
    let order = spaces(&[2, 4, 7, 6]);
    // First space swiped `Prev` and last space swiped `Next` have no
    // adjacent space in display order — the machine stays Idle.
    assert!(SwipeGesture::begin(&order, SpaceId(2), SwipeDirection::Prev).is_none());
    assert!(SwipeGesture::begin(&order, SpaceId(6), SwipeDirection::Next).is_none());
    // A single-space collection has no adjacent space in either direction.
    let single = spaces(&[2]);
    assert!(SwipeGesture::begin(&single, SpaceId(2), SwipeDirection::Next).is_none());
    assert!(SwipeGesture::begin(&single, SpaceId(2), SwipeDirection::Prev).is_none());
    // An unknown source never begins.
    assert!(SwipeGesture::begin(&order, SpaceId(999), SwipeDirection::Next).is_none());
}

#[test]
fn update_accumulates_and_clamps_signed_progress() {
    let order = spaces(&[2, 4, 7]);
    let mut gesture = SwipeGesture::begin(&order, SpaceId(2), SwipeDirection::Next).unwrap();
    gesture.update(0.3);
    gesture.update(0.4);
    assert_eq!(gesture.progress(), 0.7);
    // Clamp at +1.
    gesture.update(2.0);
    assert_eq!(gesture.progress(), 1.0);
    // Reversal moves back past zero and clamps at -1.
    gesture.update(-2.0);
    assert_eq!(gesture.progress(), -1.0);
    gesture.update(0.25);
    assert_eq!(gesture.progress(), -0.75);
}

#[test]
fn below_threshold_release_cancels_with_no_switch() {
    let order = spaces(&[2, 4, 7]);
    let mut gesture = SwipeGesture::begin(&order, SpaceId(2), SwipeDirection::Next).unwrap();
    gesture.update(0.4); // below the 0.5 threshold
    assert_eq!(
        gesture.end(),
        None,
        "below-threshold release commits nothing"
    );
    assert_eq!(gesture.phase(), SwipePhase::Idle);
}

#[test]
fn above_threshold_release_commits_exactly_one_destination() {
    let order = spaces(&[2, 4, 7]);
    let mut gesture = SwipeGesture::begin(&order, SpaceId(2), SwipeDirection::Next).unwrap();
    gesture.update(0.5);
    assert_eq!(
        gesture.end(),
        Some(SpaceId(4)),
        "exactly the adjacent destination"
    );
    assert_eq!(gesture.phase(), SwipePhase::Idle);
}

#[test]
fn repeated_end_cannot_double_commit() {
    let order = spaces(&[2, 4, 7]);
    let mut gesture = SwipeGesture::begin(&order, SpaceId(2), SwipeDirection::Next).unwrap();
    gesture.update(1.0);
    assert_eq!(gesture.end(), Some(SpaceId(4)));
    assert_eq!(gesture.end(), None, "a second end is a no-op");
    assert_eq!(gesture.update(1.0), ());
    assert_eq!(gesture.end(), None);
}

#[test]
fn cancel_returns_idle_without_commit() {
    let order = spaces(&[2, 4, 7]);
    let mut gesture = SwipeGesture::begin(&order, SpaceId(2), SwipeDirection::Next).unwrap();
    gesture.update(1.0);
    gesture.cancel();
    assert_eq!(gesture.phase(), SwipePhase::Idle);
    assert_eq!(gesture.end(), None);
}

#[test]
fn progress_direction_commits_in_both_display_directions() {
    let order = spaces(&[2, 4, 7]);
    let mut next = SwipeGesture::begin(&order, SpaceId(4), SwipeDirection::Next).unwrap();
    next.update(1.0);
    assert_eq!(next.end(), Some(SpaceId(7)));
    let mut prev = SwipeGesture::begin(&order, SpaceId(4), SwipeDirection::Prev).unwrap();
    prev.update(1.0);
    assert_eq!(prev.end(), Some(SpaceId(2)));
}

#[test]
fn natural_pixel_normalization_is_deterministic() {
    let pixels = |x, y| ScrollDelta::Pixels(point(px(x), px(y)));
    assert_eq!(normalized_delta_x(&pixels(120.0, 0.0)), 0.5);
    assert_eq!(normalized_delta_x(&pixels(240.0, 0.0)), 1.0);
    assert_eq!(normalized_delta_x(&pixels(-120.0, 0.0)), -0.5);
    // 1 px = 1/240 of a full swipe; a tiny deterministic step.
    assert!((normalized_delta_x(&pixels(1.0, 0.0)) - 1.0 / 240.0).abs() < 1e-9);
}

#[test]
fn wheel_line_normalization_is_deterministic() {
    let lines = |x, y| ScrollDelta::Lines(gpui::point(x, y));
    assert_eq!(normalized_delta_x(&lines(1.0, 0.0)), 0.25);
    assert_eq!(normalized_delta_x(&lines(4.0, 0.0)), 1.0);
    assert_eq!(normalized_delta_x(&lines(-2.0, 0.0)), -0.5);
}

#[test]
fn dominance_filter_keeps_vertical_and_equal_deltas_out() {
    let pixels = |x, y| ScrollDelta::Pixels(point(px(x), px(y)));
    assert!(is_horizontal_dominant(&pixels(100.0, 50.0)));
    assert!(!is_horizontal_dominant(&pixels(50.0, 100.0)));
    assert!(!is_horizontal_dominant(&pixels(0.0, 100.0)));
    assert!(
        !is_horizontal_dominant(&pixels(100.0, 100.0)),
        "equal axes are not horizontal-dominant"
    );
    assert!(!is_horizontal_dominant(&ScrollDelta::Pixels(
        gpui::Point::default()
    )));
    let lines = |x, y| ScrollDelta::Lines(gpui::point(x, y));
    assert!(!is_horizontal_dominant(&lines(0.0, 1.0)));
    assert!(is_horizontal_dominant(&lines(-1.0, 0.25)));
}

#[test]
fn interpolation_is_chrome_only_and_returns_exact_endpoints() {
    let theme = bbr_ui::theme::Theme::dark();
    let source = space_tint(SpaceTheme::Cobalt, &theme);
    let destination = space_tint(SpaceTheme::Pine, &theme);
    let at_source = interpolate_tint(source, destination, 0.0);
    assert_eq!(at_source, source, "cancel returns the exact source");
    let at_destination = interpolate_tint(source, destination, 1.0);
    assert_eq!(
        at_destination, destination,
        "commit lands on the exact destination"
    );
    // Negative progress (reversal past the source) clamps to source.
    assert_eq!(interpolate_tint(source, destination, -0.5), source);
    // A strictly-between progress is strictly between the two chrome
    // washes, channel by channel.
    let middle = interpolate_tint(source, destination, 0.5);
    assert_ne!(middle.chrome, source.chrome);
    assert_ne!(middle.chrome, destination.chrome);
}

// ---- Handler routing: zero-delta Ended resolution (F1) --------------------

/// A real macOS release event carries a zero delta — not horizontal-dominant
/// — yet while a gesture tracks it must resolve (never be dominance-gated).
#[test]
fn zero_delta_ended_resolves_a_tracking_gesture_regardless_of_dominance() {
    assert_eq!(
        route_event(TouchPhase::Ended, &zero_delta(), true),
        SwipeEventRoute::ResolveEnd,
        "zero-delta Ended resolves while tracking"
    );
    // Dominance is irrelevant to the release itself even for a
    // vertical-dominant residual delta.
    let pixels = |x, y| ScrollDelta::Pixels(point(px(x), px(y)));
    assert_eq!(
        route_event(TouchPhase::Ended, &pixels(0.0, 100.0), true),
        SwipeEventRoute::ResolveEnd
    );
    // Without a tracking gesture the release is a no-op.
    assert_eq!(
        route_event(TouchPhase::Ended, &zero_delta(), false),
        SwipeEventRoute::Ignore
    );
}

/// The full handler resolution of a zero-delta `Ended` in the **cancel**
/// case: below-threshold progress commits nothing and the machine returns
/// Idle (no switch, no double resolution).
#[test]
fn zero_delta_ended_below_threshold_cancels_with_no_switch() {
    let order = spaces(&[2, 4, 7]);
    let mut state = SwipeState::new();
    let mut gesture = SwipeGesture::begin(&order, SpaceId(2), SwipeDirection::Next).unwrap();
    gesture.update(0.4); // below the 0.5 commit threshold
    state.set_gesture(Some(gesture));

    // The handler route: zero-delta Ended while tracking → resolve.
    assert_eq!(
        route_event(TouchPhase::Ended, &zero_delta(), state.gesture().is_some()),
        SwipeEventRoute::ResolveEnd
    );
    let mut gesture = state.take_gesture().expect("tracking gesture");
    assert_eq!(
        gesture.end(),
        None,
        "below-threshold zero-delta release commits nothing"
    );
    assert_eq!(gesture.phase(), SwipePhase::Idle);
    // A repeated zero-delta Ended now finds nothing tracking — no-op.
    assert_eq!(
        route_event(TouchPhase::Ended, &zero_delta(), state.gesture().is_some()),
        SwipeEventRoute::Ignore
    );
}

/// The full handler resolution of a zero-delta `Ended` in the **commit**
/// case: above-threshold progress yields exactly one destination, the
/// machine returns Idle, and a repeated `Ended` never double-commits.
#[test]
fn zero_delta_ended_above_threshold_commits_exactly_once() {
    let order = spaces(&[2, 4, 7]);
    let mut state = SwipeState::new();
    let mut gesture = SwipeGesture::begin(&order, SpaceId(2), SwipeDirection::Next).unwrap();
    gesture.update(1.0); // at/above the 0.5 commit threshold
    state.set_gesture(Some(gesture));

    assert_eq!(
        route_event(TouchPhase::Ended, &zero_delta(), state.gesture().is_some()),
        SwipeEventRoute::ResolveEnd
    );
    let mut gesture = state.take_gesture().expect("tracking gesture");
    assert_eq!(
        gesture.end(),
        Some(SpaceId(4)),
        "above-threshold zero-delta release commits exactly the destination"
    );
    assert_eq!(gesture.phase(), SwipePhase::Idle);
    // Repeated resolution while nothing tracks cannot double-commit.
    assert_eq!(
        route_event(TouchPhase::Ended, &zero_delta(), state.gesture().is_some()),
        SwipeEventRoute::Ignore,
        "repeated Ended without a tracking gesture never resolves again"
    );
}

/// The handler feeds raw signed scroll deltas (positive = display `Next`,
/// negative = display `Prev`); `feed_raw` converts them to progress toward
/// the gesture's fixed destination so a full `Prev` swipe reaches `+1` and
/// commits exactly like a full `Next` swipe.
#[test]
fn prev_release_with_raw_negative_delta_commits_exactly_one_destination() {
    let order = spaces(&[2, 4, 7]);
    let mut gesture = SwipeGesture::begin(&order, SpaceId(4), SwipeDirection::Prev).unwrap();
    // A full Prev swipe is a negative scroll delta on the raw axis.
    gesture.feed_raw(-1.0);
    assert_eq!(
        gesture.progress(),
        1.0,
        "progress toward the prev destination"
    );
    assert_eq!(
        gesture.end(),
        Some(SpaceId(2)),
        "above-threshold prev release commits exactly the previous space"
    );
    assert_eq!(gesture.phase(), SwipePhase::Idle);

    // Reversing direction (positive raw delta mid-Prev) pushes progress
    // back toward and past zero.
    let mut reversing = SwipeGesture::begin(&order, SpaceId(4), SwipeDirection::Prev).unwrap();
    reversing.feed_raw(-0.6);
    reversing.feed_raw(0.2); // reversal: back toward zero
    assert!(
        (reversing.progress() - 0.4).abs() < 1e-9,
        "reversal pushes progress back toward zero (got {})",
        reversing.progress()
    );
    reversing.feed_raw(0.6); // continued reversal: past zero
    assert!(
        (reversing.progress() + 0.2).abs() < 1e-9,
        "continued reversal pushes progress past zero (got {})",
        reversing.progress()
    );
}

/// Dominance applies only where `Started`/`Moved` deltas participate: an
/// idle `Moved` never begins a gesture, a vertical-dominant `Started` never
/// begins, and `Ended` is resolved independent of both.
#[test]
fn dominance_gate_applies_only_to_started_and_moved_deltas() {
    let pixels = |x, y| ScrollDelta::Pixels(point(px(x), px(y)));
    // Vertical-dominant Started never begins a gesture.
    assert_eq!(
        route_event(TouchPhase::Started, &pixels(0.0, 100.0), false),
        SwipeEventRoute::Ignore
    );
    // Horizontal-dominant Started begins.
    assert_eq!(
        route_event(TouchPhase::Started, &pixels(100.0, 0.0), false),
        SwipeEventRoute::Begin {
            delta: 100.0 / SWIPE_PIXELS_TO_FULL
        }
    );
    // Horizontal-dominant Moved feeds only while tracking.
    assert_eq!(
        route_event(TouchPhase::Moved, &pixels(50.0, 0.0), true),
        SwipeEventRoute::Feed {
            delta: 50.0 / SWIPE_PIXELS_TO_FULL
        }
    );
    assert_eq!(
        route_event(TouchPhase::Moved, &pixels(50.0, 0.0), false),
        SwipeEventRoute::Ignore,
        "an idle Moved never starts a gesture"
    );
    // Vertical-dominant Moved while tracking is not a gesture delta.
    assert_eq!(
        route_event(TouchPhase::Moved, &pixels(0.0, 100.0), true),
        SwipeEventRoute::Ignore
    );
}
