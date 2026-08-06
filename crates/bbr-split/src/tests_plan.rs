//! Focused stateless-plan tests: open validation, focus evaluation against
//! caller-supplied values, close outcomes, divider resize evaluation, the
//! typed horizontal direction, and the no-stored-state authority model.

use crate::geometry::{DividerPosition, DEFAULT_POSITION};
use crate::plan::{
    close_outcome, evaluate_focus, evaluate_resize, validate_open, CloseOutcome, FocusIntent,
    FocusOutcome, OpenIntent, OpenPlan, OpenRejection, ResizeIntent, ResizeOutcome, SplitDirection,
};

const A: u64 = 1;
const B: u64 = 2;
const C: u64 = 3;

fn intent(primary: u64, secondary: u64, focused: u64) -> OpenIntent<u64> {
    OpenIntent {
        primary,
        secondary,
        focused,
        direction: SplitDirection::Horizontal,
    }
}

fn plan(primary: u64, secondary: u64, focused: u64) -> OpenPlan<u64> {
    validate_open(intent(primary, secondary, focused)).expect("valid open intent")
}

fn half() -> DividerPosition {
    DividerPosition::new(0.5).expect("0.5 is finite")
}

/// The validated plan reproduces the ordered identities, keeps the pre-split
/// selected pane focused, carries the typed horizontal direction, and adopts
/// the even 50/50 initial divider position.
#[test]
fn validate_open_produces_typed_plan() {
    let plan = plan(A, B, A);
    assert_eq!(
        plan,
        OpenPlan {
            primary: A,
            secondary: B,
            focused: A,
            direction: SplitDirection::Horizontal,
            initial_position: DividerPosition::new(DEFAULT_POSITION).expect("default is finite"),
        }
    );
}

/// The pre-split selected pane stays focused even when it is the secondary
/// identity; pane order stays primary-then-secondary.
#[test]
fn validate_open_keeps_pre_split_focus_on_secondary() {
    let plan = plan(A, B, B);
    assert_eq!(plan.primary, A);
    assert_eq!(plan.secondary, B);
    assert_eq!(plan.focused, B);
    assert_eq!(plan.direction, SplitDirection::Horizontal);
}

/// The typed direction is part of the intent and the plan; only Horizontal
/// exists in this crate, so no vertical behavior can be expressed.
#[test]
fn direction_is_horizontal_only() {
    let plan = plan(A, B, A);
    assert_eq!(plan.direction, SplitDirection::Horizontal);
    // The intent carries the same direction through validation.
    assert_eq!(intent(A, B, A).direction, SplitDirection::Horizontal);
}

/// The distinct-pane requirement: identical pane identities are rejected
/// before any plan exists.
#[test]
fn validate_open_rejects_identical_panes() {
    let rejection =
        validate_open(intent(A, A, A)).expect_err("identical pane identities must be rejected");
    assert_eq!(
        rejection,
        OpenRejection::IdenticalPanes {
            primary: A,
            secondary: A
        }
    );
}

/// An outside identity cannot become split focus at open time.
#[test]
fn validate_open_rejects_focused_outside_members() {
    let rejection =
        validate_open(intent(A, B, C)).expect_err("an outside focused identity must be rejected");
    assert_eq!(rejection, OpenRejection::FocusNotMember { focused: C });
}

/// Selecting a member changes focus; the ordered pane identities are the
/// caller's and are never reordered by this crate.
#[test]
fn focus_member_changes_focus_without_reordering() {
    let outcome = evaluate_focus(A, B, A, FocusIntent { pane: B });
    assert_eq!(outcome, FocusOutcome::Changed { focused: B });
    // The caller keeps its own order; the outcome only reports the new focus.
    assert_eq!(
        evaluate_focus(A, B, B, FocusIntent { pane: A }),
        FocusOutcome::Changed { focused: A }
    );
}

/// Requesting the already-focused member is an explicit Unchanged outcome;
/// only a real change reports Changed.
#[test]
fn focus_unchanged_is_explicit_and_distinct() {
    let unchanged = evaluate_focus(A, B, A, FocusIntent { pane: A });
    assert_eq!(unchanged, FocusOutcome::Unchanged { focused: A });
    assert_ne!(unchanged, FocusOutcome::Changed { focused: A });
    assert_eq!(
        evaluate_focus(A, B, B, FocusIntent { pane: B }),
        FocusOutcome::Unchanged { focused: B }
    );
}

/// An outside identity cannot become split focus: the request is rejected
/// regardless of the caller-supplied current focus.
#[test]
fn focus_outside_member_rejected() {
    assert_eq!(
        evaluate_focus(A, B, A, FocusIntent { pane: C }),
        FocusOutcome::Rejected { pane: C }
    );
    assert_eq!(
        evaluate_focus(A, B, B, FocusIntent { pane: C }),
        FocusOutcome::Rejected { pane: C }
    );
}

/// The caller-supplied authoritative `current` must itself be a member: an
/// outside current is rejected before any Unchanged result, so it can never
/// be reported as the focused pane. The regression: `current == pane` with
/// both outside the members previously returned an Unchanged outcome.
#[test]
fn focus_rejects_outside_current_before_unchanged() {
    // Outside current with a request for the same outside value must not
    // report Unchanged; the invalid authoritative baseline is reported.
    assert_eq!(
        evaluate_focus(A, B, C, FocusIntent { pane: C }),
        FocusOutcome::InvalidCurrent { current: C }
    );
    // An outside current invalidates every request, even one targeting a
    // member: the baseline is inconsistent, so no outcome can be trusted.
    assert_eq!(
        evaluate_focus(A, B, C, FocusIntent { pane: A }),
        FocusOutcome::InvalidCurrent { current: C }
    );
    assert_eq!(
        evaluate_focus(A, B, C, FocusIntent { pane: B }),
        FocusOutcome::InvalidCurrent { current: C }
    );
    // A member current keeps the existing behavior: Unchanged for the
    // already-focused member, Changed on a real change.
    assert_eq!(
        evaluate_focus(A, B, A, FocusIntent { pane: A }),
        FocusOutcome::Unchanged { focused: A }
    );
    assert_eq!(
        evaluate_focus(A, B, A, FocusIntent { pane: B }),
        FocusOutcome::Changed { focused: B }
    );
}

/// The stateless focus evaluation answers identically for equal authoritative
/// inputs regardless of call history: it stores no focus state.
#[test]
fn focus_evaluation_is_pure_and_stateless() {
    let first = evaluate_focus(A, B, A, FocusIntent { pane: B });
    // Interleave unrelated evaluations; the same input still yields the same
    // outcome.
    evaluate_focus(A, B, B, FocusIntent { pane: C });
    evaluate_focus(B, A, B, FocusIntent { pane: A });
    let second = evaluate_focus(A, B, A, FocusIntent { pane: B });
    assert_eq!(first, second);
}

/// Close reports the focused survivor while retaining both pane identities
/// for the caller.
#[test]
fn close_reports_focused_survivor_and_both_identities() {
    assert_eq!(
        close_outcome(A, B, B),
        CloseOutcome {
            primary: A,
            secondary: B,
            survivor: B,
        }
    );
    assert_eq!(
        close_outcome(A, B, A),
        CloseOutcome {
            primary: A,
            secondary: B,
            survivor: A,
        }
    );
}

/// Divider resize moves the divider and reports that it changed.
#[test]
fn resize_moves_and_reports_changed() {
    let outcome = evaluate_resize(half(), ResizeIntent { position: 0.25 });
    assert_eq!(
        outcome,
        ResizeOutcome::Resized {
            position: DividerPosition::new(0.25).expect("finite"),
            changed: true,
        }
    );
}

/// Low positions clamp to the bounded zero edge.
#[test]
fn resize_low_clamp_bounded() {
    let outcome = evaluate_resize(half(), ResizeIntent { position: -0.5 });
    assert_eq!(
        outcome,
        ResizeOutcome::Resized {
            position: DividerPosition::new(0.0).expect("finite"),
            changed: true,
        }
    );
}

/// High positions clamp to the bounded one edge.
#[test]
fn resize_high_clamp_bounded() {
    let outcome = evaluate_resize(half(), ResizeIntent { position: 1.5 });
    assert_eq!(
        outcome,
        ResizeOutcome::Resized {
            position: DividerPosition::new(1.0).expect("finite"),
            changed: true,
        }
    );
}

/// An unchanged divider input is a no-op under f32 equality: the outcome
/// reports `changed: false`.
#[test]
fn resize_unchanged_is_no_op() {
    let outcome = evaluate_resize(half(), ResizeIntent { position: 0.5 });
    assert_eq!(
        outcome,
        ResizeOutcome::Resized {
            position: half(),
            changed: false,
        }
    );
}

/// A position that clamps onto the current value is also a no-op.
#[test]
fn resize_clamped_to_current_is_no_op() {
    let current = DividerPosition::new(0.0).expect("finite");
    let outcome = evaluate_resize(current, ResizeIntent { position: -0.25 });
    assert_eq!(
        outcome,
        ResizeOutcome::Resized {
            position: DividerPosition::new(0.0).expect("finite"),
            changed: false,
        }
    );
}

/// Non-finite resize input is rejected and the caller's current divider value
/// is left untouched (compared by variant because `NaN != NaN`).
#[test]
fn resize_rejects_non_finite() {
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        match evaluate_resize(half(), ResizeIntent { position: bad }) {
            ResizeOutcome::Rejected { position } => assert!(position.is_nan() || position == bad),
            other => panic!("expected rejection for {bad:?}, got {other:?}"),
        }
    }
}

/// Resize evaluation depends only on the two inputs, never on hidden state.
#[test]
fn resize_evaluation_is_pure_and_stateless() {
    let first = evaluate_resize(half(), ResizeIntent { position: 0.75 });
    let second = evaluate_resize(half(), ResizeIntent { position: 0.75 });
    assert_eq!(first, second);
    // A different current value changes only the `changed` flag.
    let current = DividerPosition::new(0.75).expect("finite");
    assert_eq!(
        evaluate_resize(current, ResizeIntent { position: 0.75 }),
        ResizeOutcome::Resized {
            position: DividerPosition::new(0.75).expect("finite"),
            changed: false,
        }
    );
}

/// The plan API works with any `Copy + Eq + Debug` identity type, not just
/// numbers.
#[test]
fn plan_is_generic_over_pane_identity() {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Pane(u64);

    let left = Pane(1);
    let right = Pane(2);
    let plan = validate_open(OpenIntent {
        primary: left,
        secondary: right,
        focused: left,
        direction: SplitDirection::Horizontal,
    })
    .expect("valid open intent");
    assert_eq!(plan.focused, left);
    assert_eq!(
        evaluate_focus(left, right, left, FocusIntent { pane: right }),
        FocusOutcome::Changed { focused: right }
    );
    assert_eq!(close_outcome(left, right, right).survivor, right);
}

/// Exact debug representations for the intent, plan, and outcome shapes.
#[test]
fn debug_strings_are_exact() {
    assert_eq!(
        format!(
            "{:?}",
            OpenIntent {
                primary: A,
                secondary: B,
                focused: A,
                direction: SplitDirection::Horizontal,
            }
        ),
        "OpenIntent { primary: 1, secondary: 2, focused: 1, direction: Horizontal }"
    );
    assert_eq!(
        format!(
            "{:?}",
            OpenRejection::IdenticalPanes {
                primary: A,
                secondary: A
            }
        ),
        "IdenticalPanes { primary: 1, secondary: 1 }"
    );
    assert_eq!(
        format!("{:?}", plan(A, B, A)),
        "OpenPlan { primary: 1, secondary: 2, focused: 1, direction: Horizontal, \
         initial_position: DividerPosition(0.5) }"
    );
    assert_eq!(
        format!("{:?}", FocusIntent { pane: B }),
        "FocusIntent { pane: 2 }"
    );
    assert_eq!(
        format!("{:?}", FocusOutcome::Changed { focused: B }),
        "Changed { focused: 2 }"
    );
    assert_eq!(
        format!("{:?}", FocusOutcome::Unchanged { focused: A }),
        "Unchanged { focused: 1 }"
    );
    assert_eq!(
        format!("{:?}", FocusOutcome::Rejected { pane: C }),
        "Rejected { pane: 3 }"
    );
    assert_eq!(
        format!("{:?}", FocusOutcome::InvalidCurrent { current: C }),
        "InvalidCurrent { current: 3 }"
    );
    assert_eq!(
        format!("{:?}", ResizeIntent { position: 0.25 }),
        "ResizeIntent { position: 0.25 }"
    );
    assert_eq!(
        format!(
            "{:?}",
            CloseOutcome {
                primary: A,
                secondary: B,
                survivor: A
            }
        ),
        "CloseOutcome { primary: 1, secondary: 2, survivor: 1 }"
    );
}
