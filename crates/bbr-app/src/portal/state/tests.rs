use super::*;

fn centered_request(id: u64, host: PortalHost, priority: usize) -> PortalRequest {
    PortalRequest {
        presentation: PresentationId(id),
        host,
        placement: PortalPlacement::Centered {
            size: Sz::new(480.0, 360.0),
            margin: Edges::all(8.0),
        },
        priority,
        dismissal: DismissalPolicy::any(),
    }
}

fn anchored_request(id: u64, priority: usize) -> PortalRequest {
    PortalRequest::deferred(
        PresentationId(id),
        PortalPlacement::Anchored(AnchoredPlacement {
            anchor_point: Pt::new(24.0, 300.0),
            anchor: PortalAnchor::TopLeft,
            size: Sz::new(320.0, 200.0),
            offset: Pt::new(0.0, 4.0),
            fit: FitMode::SnapToWindowWithMargin(Edges::all(8.0)),
        }),
        priority,
        DismissalPolicy::any(),
    )
}

#[test]
fn present_shows_exactly_one_active_presentation() {
    let mut state = PortalState::new(PortalId(1));
    assert_eq!(state.active(), None);

    assert_eq!(
        state.present(centered_request(1, PortalHost::GpuiDeferred, 1)),
        PortalTransition::Presented
    );
    assert_eq!(state.active().unwrap().id, PresentationId(1));
    assert_eq!(state.host(), Some(PortalHost::GpuiDeferred));

    // A different id replaces the active presentation — never two.
    assert_eq!(
        state.present(centered_request(2, PortalHost::NativeAboveWeb, 2)),
        PortalTransition::Replaced
    );
    let active = state.active().unwrap();
    assert_eq!(active.id, PresentationId(2));
    assert_eq!(active.host, PortalHost::NativeAboveWeb);
    assert_eq!(state.active(), Some(active), "exactly one active");
}

#[test]
fn same_presentation_id_updates_in_place() {
    let mut state = PortalState::new(PortalId(1));
    assert_eq!(
        state.present(centered_request(7, PortalHost::GpuiDeferred, 1)),
        PortalTransition::Presented
    );
    assert_eq!(
        state.present(centered_request(7, PortalHost::GpuiDeferred, 5)),
        PortalTransition::Updated,
        "same id refreshes, never replaces"
    );
    let active = state.active().unwrap();
    assert_eq!(active.id, PresentationId(7), "id stays stable");
    assert_eq!(active.priority, 5, "fields refresh in place");
    assert_eq!(state.active(), Some(active), "still exactly one active");
}

#[test]
fn priority_ordering_is_deterministic_and_retained() {
    let mut state = PortalState::new(PortalId(1));
    state.present(centered_request(1, PortalHost::GpuiDeferred, 1));
    state.present(centered_request(2, PortalHost::NativeAboveWeb, 3));
    assert_eq!(state.active().unwrap().priority, 3);
    // The latest request wins regardless of priority value; the value is
    // retained for the deferred draw order / native z-order.
    state.present(centered_request(3, PortalHost::GpuiDeferred, 2));
    assert_eq!(state.active().unwrap().priority, 2);
    assert_eq!(state.active().unwrap().host, PortalHost::GpuiDeferred);
}

#[test]
fn host_selection_reflects_the_request() {
    assert_eq!(PortalHost::GpuiDeferred.label(), "gpui-deferred");
    assert_eq!(PortalHost::NativeAboveWeb.label(), "native-above-web");
    assert!(!PortalHost::GpuiDeferred.crosses_native_boundary());
    assert!(PortalHost::NativeAboveWeb.crosses_native_boundary());

    let mut state = PortalState::new(PortalId(1));
    state.present(PortalRequest::deferred(
        PresentationId(1),
        PortalPlacement::Centered {
            size: Sz::new(480.0, 360.0),
            margin: Edges::all(8.0),
        },
        1,
        DismissalPolicy::any(),
    ));
    assert_eq!(state.host(), Some(PortalHost::GpuiDeferred));

    state.present(PortalRequest::native(
        PresentationId(2),
        PortalPlacement::Centered {
            size: Sz::new(480.0, 360.0),
            margin: Edges::all(8.0),
        },
        1,
        DismissalPolicy::any(),
    ));
    assert_eq!(state.host(), Some(PortalHost::NativeAboveWeb));
}

#[test]
fn dismiss_is_deterministic_and_idempotent() {
    let mut state = PortalState::new(PortalId(1));
    assert!(!state.dismiss(), "dismissing an empty portal is a no-op");
    assert_eq!(state.active(), None);
    assert!(
        !state.is_dismissed(),
        "nothing was shown, nothing dismissed"
    );

    state.present(centered_request(1, PortalHost::GpuiDeferred, 1));
    assert!(!state.is_dismissed());
    assert!(state.dismiss(), "an active presentation was dismissed");
    assert_eq!(state.active(), None, "nothing active after dismissal");
    assert!(state.is_dismissed(), "dismissal state is recorded");
    assert!(!state.dismiss(), "dismissing again is a no-op");
}

#[test]
fn presenting_after_dismiss_clears_dismissal_state() {
    let mut state = PortalState::new(PortalId(1));
    state.present(centered_request(1, PortalHost::NativeAboveWeb, 1));
    assert!(state.dismiss());
    assert!(state.is_dismissed());

    // A fresh request re-activates the host; the dismissed marker clears.
    assert_eq!(
        state.present(centered_request(1, PortalHost::NativeAboveWeb, 1)),
        PortalTransition::Presented
    );
    assert!(!state.is_dismissed());
    assert_eq!(state.active().unwrap().id, PresentationId(1));
}

#[test]
fn presentation_id_stays_stable_across_lifecycle() {
    let mut state = PortalState::new(PortalId(9));
    state.present(centered_request(42, PortalHost::GpuiDeferred, 1));
    state.dismiss();
    assert_eq!(
        state.present(centered_request(42, PortalHost::GpuiDeferred, 2)),
        PortalTransition::Presented,
        "a dismissed id presents fresh but keeps its identity"
    );
    assert_eq!(state.active().unwrap().id, PresentationId(42));
    assert_eq!(state.active().unwrap().priority, 2);
}

#[test]
fn outside_press_dismissal_is_policy_gated() {
    let mut state = PortalState::new(PortalId(1));
    assert!(
        !state.dismisses_on_outside_press(),
        "no presentation means no outside-press dismissal"
    );

    // A presentation without outside-pointer dismissal never closes on
    // an outside press.
    state.present(PortalRequest {
        presentation: PresentationId(1),
        host: PortalHost::GpuiDeferred,
        placement: PortalPlacement::Centered {
            size: Sz::new(480.0, 360.0),
            margin: Edges::all(8.0),
        },
        priority: 1,
        dismissal: DismissalPolicy::escape(),
    });
    assert!(!state.dismisses_on_outside_press());

    // The DESIGN §13 default and the outside-pointer policy both allow
    // the one-outside-pointer-action dismissal.
    state.present(PortalRequest {
        presentation: PresentationId(2),
        host: PortalHost::NativeAboveWeb,
        placement: PortalPlacement::Centered {
            size: Sz::new(480.0, 360.0),
            margin: Edges::all(8.0),
        },
        priority: 1,
        dismissal: DismissalPolicy::outside_pointer(),
    });
    assert!(state.dismisses_on_outside_press());
    assert!(state.dismiss());
    assert!(!state.dismisses_on_outside_press());
}

#[test]
fn resolve_centered_matches_the_pure_geometry() {
    let presentation = PortalPresentation::from(centered_request(1, PortalHost::GpuiDeferred, 1));
    let rect = presentation.resolve(Sz::new(1080.0, 700.0), Edges::none());
    assert_eq!(
        rect,
        Rect::new(Pt::new(300.0, 170.0), Sz::new(480.0, 360.0))
    );
}

#[test]
fn resolve_below_start_anchored_matches_the_pure_geometry() {
    let presentation = PortalPresentation::from(anchored_request(1, 1));
    let rect = presentation.resolve(Sz::new(1080.0, 700.0), Edges::none());
    assert_eq!(
        rect,
        Rect::new(Pt::new(24.0, 304.0), Sz::new(320.0, 200.0)),
        "DESIGN §11.4 field-anchored overlay geometry"
    );
}

#[test]
fn compact_window_resolution_clamps() {
    // The centered request carries the DESIGN 8 pt margin; the compact
    // clamp pins the left/top edge to that margin (never to zero), and
    // the size is never shrunk.
    let presentation = PortalPresentation::from(centered_request(1, PortalHost::GpuiDeferred, 1));
    let rect = presentation.resolve(Sz::new(320.0, 200.0), Edges::none());
    assert_eq!(rect.x, 8.0, "wider-than-window palette pins to the margin");
    assert_eq!(rect.y, 8.0);
    assert_eq!(rect.width, 480.0, "size is never shrunk by clamping");
    assert_eq!(rect.height, 360.0);
}
