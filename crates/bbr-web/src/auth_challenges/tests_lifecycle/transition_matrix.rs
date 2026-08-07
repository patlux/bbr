//! Lifecycle transition-matrix test: the exhaustive state × event matrix.

use super::support::*;
use crate::auth_challenges::apply_event;

/// Exhaustive state × event matrix: every one of the 6 states crossed with
/// every one of the 5 events yields exactly the hand-derived outcome, so no
/// legal transition is missing and no illegal one slips through.
#[test]
fn lifecycle_transition_matrix_is_exhaustive() {
    let challenge = basic_challenge();
    for (state_name, state) in states() {
        for (event_name, event) in events() {
            let expected = expected_outcome(state_name, event_name);
            let actual = classify(apply_event(&challenge, &state, event));
            assert_eq!(actual, expected, "state {state_name} × event {event_name}");
        }
    }
}
