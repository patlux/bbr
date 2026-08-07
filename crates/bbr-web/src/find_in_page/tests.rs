//! Deterministic tests for the portable find-in-page contract.

use super::*;

fn started(query: &str, options: FindOptions) -> (FindSession, FindRequestId) {
    let mut session = FindSession::default();
    let request_id = session.start(query, options).expect("query must start");
    (session, request_id)
}

fn active(query: &str, count: usize, current: Option<usize>) -> FindSession {
    let (mut session, request_id) = started(query, FindOptions::default());
    session
        .report(
            request_id,
            FindResult {
                match_count: count,
                current_match: current,
            },
        )
        .expect("report must land");
    session
}

#[test]
fn defaults_and_query_normalization_are_explicit() {
    assert_eq!(FindOptions::default(), FindOptions::new(false, false, true));
    assert_eq!(normalize_query("  hello  "), "hello");
    assert!(is_empty_query(" \t\n"));
    assert!(!is_empty_query("x"));
}

#[test]
fn empty_query_leaves_session_unchanged() {
    let mut session = FindSession::default();
    assert_eq!(
        session.start("", FindOptions::default()),
        Err(FindError::EmptyQuery)
    );
    assert_eq!(
        session.start("  ", FindOptions::default()),
        Err(FindError::EmptyQuery)
    );
    assert!(session.is_idle());
    assert_eq!(session.query(), None);
}

#[test]
fn start_trims_and_returns_a_pending_request() {
    let options = FindOptions::new(true, true, false);
    let (mut session, request_id) = started("  hello  ", options);
    assert!(session.is_pending());
    assert_eq!(session.query(), Some("hello"));
    assert_eq!(session.options(), options);
    assert_eq!(session.match_count(), Err(FindError::ResultsPending));
    assert_eq!(session.find_next(), Err(FindError::ResultsPending));
    assert_ne!(request_id, FindRequestId::new(0));
}

#[test]
fn matching_request_lands_results_and_zero_matches_is_active() {
    let (mut session, request_id) = started("rust", FindOptions::default());
    session
        .report(
            request_id,
            FindResult {
                match_count: 3,
                current_match: Some(1),
            },
        )
        .unwrap();
    assert!(session.is_active());
    assert_eq!(session.match_count(), Ok(3));
    assert_eq!(session.current_match(), Ok(Some(1)));

    let (mut none, request_id) = started("missing", FindOptions::default());
    none.report(
        request_id,
        FindResult {
            match_count: 0,
            current_match: None,
        },
    )
    .unwrap();
    assert!(none.is_active());
    assert_eq!(none.match_count(), Ok(0));
    assert_eq!(none.current_match(), Ok(None));
    assert_eq!(none.find_next(), Err(FindError::NoMatches));
}

#[test]
fn stale_completion_is_rejected_after_replacement_start() {
    let (mut session, old_request) = started("old", FindOptions::default());
    let new_request = session.start("new", FindOptions::default()).unwrap();
    assert_ne!(old_request, new_request);
    assert_eq!(
        session.report(
            old_request,
            FindResult {
                match_count: 1,
                current_match: Some(1)
            }
        ),
        Err(FindError::StaleResults)
    );
    assert!(session.is_pending());
    session
        .report(
            new_request,
            FindResult {
                match_count: 2,
                current_match: Some(2),
            },
        )
        .unwrap();
    assert_eq!(session.query(), Some("new"));
    assert_eq!(session.match_count(), Ok(2));
}

#[test]
fn stale_completion_is_rejected_after_invalidate_then_restart() {
    let (mut session, old_request) = started("old", FindOptions::default());
    session.invalidate();
    let new_request = session.start("new", FindOptions::default()).unwrap();
    assert_eq!(
        session.report(
            old_request,
            FindResult {
                match_count: 1,
                current_match: Some(1)
            }
        ),
        Err(FindError::StaleResults)
    );
    session
        .report(
            new_request,
            FindResult {
                match_count: 1,
                current_match: Some(1),
            },
        )
        .unwrap();
    assert!(session.is_active());
}

#[test]
fn mismatched_navigation_completion_is_rejected_and_pending_is_preserved() {
    let mut session = active("a", 3, Some(1));
    let request = session.find_next().unwrap();
    assert_eq!(session.find_previous(), Err(FindError::ResultsPending));
    assert_eq!(session.match_count(), Err(FindError::ResultsPending));
    assert_eq!(
        session.report(
            FindRequestId::new(request.0 + 1),
            FindResult {
                match_count: 3,
                current_match: Some(2)
            }
        ),
        Err(FindError::StaleResults)
    );
    assert!(session.is_pending());
    session
        .report(
            request,
            FindResult {
                match_count: 3,
                current_match: Some(2),
            },
        )
        .unwrap();
    assert_eq!(session.current_match(), Ok(Some(2)));
}

#[test]
fn navigation_is_serialized_until_confirmation() {
    let mut session = active("a", 3, Some(1));
    let first = session.find_next().unwrap();
    assert_eq!(session.find_next(), Err(FindError::ResultsPending));
    session
        .report(
            first,
            FindResult {
                match_count: 3,
                current_match: Some(2),
            },
        )
        .unwrap();
    let second = session.find_next().unwrap();
    assert_ne!(first, second);
    session
        .report(
            second,
            FindResult {
                match_count: 3,
                current_match: Some(3),
            },
        )
        .unwrap();
    assert_eq!(session.current_match(), Ok(Some(3)));
}

#[test]
fn next_and_previous_wrap_when_enabled() {
    let mut next = active("a", 3, Some(3));
    let request = next.find_next().unwrap();
    next.report(
        request,
        FindResult {
            match_count: 3,
            current_match: Some(1),
        },
    )
    .unwrap();
    assert_eq!(next.current_match(), Ok(Some(1)));

    let mut previous = active("a", 3, Some(1));
    let request = previous.find_previous().unwrap();
    previous
        .report(
            request,
            FindResult {
                match_count: 3,
                current_match: Some(3),
            },
        )
        .unwrap();
    assert_eq!(previous.current_match(), Ok(Some(3)));
}

#[test]
fn boundary_navigation_without_wrap_is_confirmed_noop() {
    let options = FindOptions::new(false, false, false);
    let (mut session, request) = started("a", options);
    session
        .report(
            request,
            FindResult {
                match_count: 2,
                current_match: Some(2),
            },
        )
        .unwrap();
    let request = session.find_next().unwrap();
    session
        .report(
            request,
            FindResult {
                match_count: 2,
                current_match: Some(2),
            },
        )
        .unwrap();
    assert_eq!(session.current_match(), Ok(Some(2)));
}

#[test]
fn invalid_result_does_not_consume_request() {
    let (mut session, request) = started("x", FindOptions::default());
    assert_eq!(
        session.report(
            request,
            FindResult {
                match_count: 2,
                current_match: None
            }
        ),
        Err(FindError::InvalidResult)
    );
    assert!(session.is_pending());
    session
        .report(
            request,
            FindResult {
                match_count: 1,
                current_match: Some(1),
            },
        )
        .unwrap();
    assert!(session.is_active());
}

#[test]
fn invalid_lifecycle_operations_have_distinct_errors() {
    let mut idle = FindSession::default();
    assert_eq!(
        idle.report(
            FindRequestId::new(1),
            FindResult {
                match_count: 1,
                current_match: Some(1)
            }
        ),
        Err(FindError::Inactive)
    );
    assert_eq!(idle.find_next(), Err(FindError::Inactive));

    let mut active = active("x", 1, Some(1));
    assert_eq!(
        active.report(
            FindRequestId::new(1),
            FindResult {
                match_count: 1,
                current_match: Some(1)
            }
        ),
        Err(FindError::NoRequestPending)
    );
}

#[test]
fn invalidate_rejects_inflight_and_access_until_restart() {
    let (mut session, request) = started("x", FindOptions::default());
    session.invalidate();
    assert!(session.is_stale());
    assert_eq!(
        session.report(
            request,
            FindResult {
                match_count: 1,
                current_match: Some(1)
            }
        ),
        Err(FindError::StaleResults)
    );
    assert_eq!(session.match_count(), Err(FindError::StaleResults));
    assert_eq!(session.find_next(), Err(FindError::StaleResults));
}

#[test]
fn finish_clears_every_state_and_is_idempotent() {
    let mut session = active("x", 1, Some(1));
    session.finish();
    session.finish();
    assert!(session.is_idle());
    assert_eq!(session.query(), None);
    assert_eq!(session.match_count(), Err(FindError::Inactive));
}

#[test]
fn result_consistency_is_strict() {
    for result in [
        FindResult {
            match_count: 0,
            current_match: None,
        },
        FindResult {
            match_count: 1,
            current_match: Some(1),
        },
        FindResult {
            match_count: 4,
            current_match: Some(4),
        },
    ] {
        assert!(result.is_consistent());
    }
    for result in [
        FindResult {
            match_count: 0,
            current_match: Some(1),
        },
        FindResult {
            match_count: 3,
            current_match: None,
        },
        FindResult {
            match_count: 3,
            current_match: Some(0),
        },
        FindResult {
            match_count: 3,
            current_match: Some(4),
        },
    ] {
        assert!(!result.is_consistent());
    }
}

#[test]
fn direction_and_error_are_plain_typed_values() {
    assert_ne!(FindDirection::Next, FindDirection::Previous);
    fn assert_error(error: FindError) -> String {
        error.to_string()
    }
    for error in [
        FindError::Inactive,
        FindError::EmptyQuery,
        FindError::ResultsPending,
        FindError::NoRequestPending,
        FindError::NoMatches,
        FindError::UnsupportedOption,
        FindError::StaleResults,
        FindError::InvalidResult,
    ] {
        assert!(!assert_error(error).is_empty());
    }
}
