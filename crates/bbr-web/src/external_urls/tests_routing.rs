//! Routing contract tests: the frontmost-eligible-window rule, profile
//! isolation, space ownership, and deterministic caller order.

use crate::external_urls::{
    decide_routing, RoutingDecision, RoutingInput, SpaceId, WindowId, WindowSnapshot,
};
use crate::ProfileId;

fn window(window: u64, profile: u64, space: u64, accepts_external_urls: bool) -> WindowSnapshot {
    WindowSnapshot {
        window: WindowId(window),
        profile: ProfileId(profile),
        space: SpaceId(space),
        accepts_external_urls,
    }
}

fn input(windows: &[WindowSnapshot], default_profile: u64, default_space: u64) -> RoutingInput<'_> {
    RoutingInput {
        windows,
        default_profile: ProfileId(default_profile),
        default_space: SpaceId(default_space),
    }
}

/// An accepted URL reuses the frontmost eligible window: the first window in
/// the caller's front-to-back order.
#[test]
fn reuse_frontmost_eligible_window() {
    let windows = [
        window(1, 10, 100, true),
        window(2, 10, 200, true),
        window(3, 20, 300, true),
    ];
    let decision = decide_routing(&input(&windows, 99, 999));
    assert_eq!(
        decision,
        RoutingDecision::ReuseWindow {
            window: WindowId(1),
            profile: ProfileId(10),
            space: SpaceId(100),
        }
    );
}

/// A frontmost ineligible window (private/incognito, utility, devtools) is
/// skipped; the next eligible window is reused.
#[test]
fn reuse_skips_ineligible_front_window() {
    let windows = [
        window(1, 10, 100, false),
        window(2, 10, 200, true),
        window(3, 20, 300, true),
    ];
    let decision = decide_routing(&input(&windows, 99, 999));
    assert_eq!(
        decision,
        RoutingDecision::ReuseWindow {
            window: WindowId(2),
            profile: ProfileId(10),
            space: SpaceId(200),
        }
    );
}

/// With no eligible window the URL opens in a new window in the
/// caller-declared default profile and default space.
#[test]
fn new_window_falls_back_to_declared_defaults() {
    let windows = [window(1, 10, 100, false), window(2, 20, 200, false)];
    let decision = decide_routing(&input(&windows, 42, 7));
    assert_eq!(
        decision,
        RoutingDecision::NewWindow {
            profile: ProfileId(42),
            space: SpaceId(7),
        }
    );
}

/// With no windows at all the same new-window fallback applies.
#[test]
fn new_window_when_no_windows_exist() {
    let decision = decide_routing(&input(&[], 42, 7));
    assert_eq!(
        decision,
        RoutingDecision::NewWindow {
            profile: ProfileId(42),
            space: SpaceId(7),
        }
    );
}

/// Profile isolation: a reuse takes the reused window's own profile, never
/// the default profile and never any other profile — an external URL can
/// never silently cross a profile boundary.
#[test]
fn reuse_never_crosses_profile_boundaries() {
    // The frontmost window belongs to profile 10 while the declared default
    // is profile 99; the decision must stay in profile 10.
    let windows = [window(1, 10, 100, true)];
    let decision = decide_routing(&input(&windows, 99, 999));
    assert_eq!(
        decision,
        RoutingDecision::ReuseWindow {
            window: WindowId(1),
            profile: ProfileId(10),
            space: SpaceId(100),
        }
    );
}

/// Space ownership: a reused tab joins exactly the reused window's space,
/// never any other.
#[test]
fn reuse_keeps_the_windows_space() {
    let windows = [window(1, 10, 100, true), window(2, 10, 500, true)];
    let decision = decide_routing(&input(&windows, 99, 999));
    assert_eq!(
        decision,
        RoutingDecision::ReuseWindow {
            window: WindowId(1),
            profile: ProfileId(10),
            space: SpaceId(100),
        }
    );
}

/// The rule is deterministic: the first eligible window in caller order wins,
/// so equal front-to-back states always produce the same decision.
#[test]
fn first_eligible_window_in_caller_order_wins() {
    let windows = [
        window(1, 10, 100, false),
        window(2, 10, 200, false),
        window(3, 20, 300, true),
        window(4, 20, 400, true),
    ];
    let decision = decide_routing(&input(&windows, 99, 999));
    assert_eq!(
        decision,
        RoutingDecision::ReuseWindow {
            window: WindowId(3),
            profile: ProfileId(20),
            space: SpaceId(300),
        }
    );
}

/// Repeated decisions on the same input are stable (pure function).
#[test]
fn decisions_are_pure_and_repeatable() {
    let windows = [window(1, 10, 100, true)];
    let first = decide_routing(&input(&windows, 99, 999));
    let second = decide_routing(&input(&windows, 99, 999));
    assert_eq!(first, second);
}
