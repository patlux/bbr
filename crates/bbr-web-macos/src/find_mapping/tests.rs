//! Contract tests for the pure find mapping (see the parent module).
//!
//! Every test here runs without a display server: `FindAdapterCore`, script
//! building, reply parsing, and the request-id echo are pure. Request ids are
//! minted through the authoritative portable `FindSession`, exactly as the
//! composition will.

use super::*;
use bbr_web::find_in_page::FindSession;

fn options() -> FindOptions {
    FindOptions::new(false, false, true)
}

/// Mint a real portable request id through the authoritative session, the
/// same way the composition will.
fn mint_id(session: &mut FindSession, query: &str) -> FindRequestId {
    session
        .start(query, options())
        .expect("minting a test request id must succeed")
}

#[test]
fn js_quote_escapes_literal_breaking_characters() {
    assert_eq!(js_quote("plain"), "'plain'");
    assert_eq!(js_quote("it's"), "'it\\'s'");
    assert_eq!(js_quote("back\\slash"), "'back\\\\slash'");
    assert_eq!(js_quote("line\nbreak"), "'line\\nbreak'");
    assert_eq!(js_quote("tab\there"), "'tab\\there'");
    assert_eq!(js_quote("cr\rhere"), "'cr\\rhere'");
    assert_eq!(js_quote("\u{2028}sep\u{2029}"), "'\\u2028sep\\u2029'");
    assert_eq!(js_quote("ctrl\u{01}"), "'ctrl\\u0001'");
    assert_eq!(js_quote("emoji 🦄 ok"), "'emoji 🦄 ok'");
}

#[test]
fn build_find_script_embeds_start_configuration() {
    let script = build_find_script("it's", options(), None);
    assert!(script.contains("mode: \"start\""), "{script}");
    assert!(script.contains("query: 'it\\'s'"), "{script}");
    assert!(script.contains("caseSensitive: false"), "{script}");
    assert!(script.contains("wholeWord: false"), "{script}");
    assert!(script.contains("wrapAround: true"), "{script}");
    assert!(script.contains("direction: 1"), "{script}");
    assert!(!script.contains(TOKEN_QUERY), "query token must be filled");
}

#[test]
fn build_find_script_embeds_step_direction_and_flags() {
    let script = build_find_script(
        "word",
        FindOptions::new(true, true, false),
        Some(FindDirection::Previous),
    );
    assert!(script.contains("mode: \"step\""), "{script}");
    assert!(script.contains("caseSensitive: true"), "{script}");
    assert!(script.contains("wholeWord: true"), "{script}");
    assert!(script.contains("wrapAround: false"), "{script}");
    assert!(script.contains("direction: -1"), "{script}");
}

#[test]
fn build_clear_script_switches_mode_only() {
    let script = build_clear_script();
    assert!(script.contains("mode: \"clear\""), "{script}");
    assert!(
        script.contains(TOKEN_QUERY),
        "clear script must not embed a query"
    );
    assert!(
        !script.contains("query: '"),
        "no quoted query in clear script"
    );
}

#[test]
fn issue_start_rejects_empty_query_without_state_change() {
    let mut core = FindAdapterCore::new();
    assert_eq!(
        core.issue_start(
            mint_id(&mut FindSession::default(), "probe"),
            "   ",
            options()
        ),
        Err(FindError::EmptyQuery)
    );
    assert!(!core.is_open());
    assert_eq!(core.pending_request(), None);
}

#[test]
fn start_report_completes_to_reported_result() {
    let mut core = FindAdapterCore::new();
    let mut session = FindSession::default();
    let id1 = mint_id(&mut session, "foo");

    let op = core.issue_start(id1, "foo", options()).unwrap();
    assert!(matches!(op, StrategyOp::Start(_)));
    assert!(core.is_open());
    assert_eq!(core.pending_request(), Some(id1));

    let completion = core.complete(
        id1,
        StrategyOutcome::Ok {
            count: 3,
            ordinal: Some(1),
            mutated: false,
        },
    );
    assert_eq!(
        completion,
        FindCompletion::Reported {
            request_id: id1,
            result: FindResult {
                match_count: 3,
                current_match: Some(1),
            },
            mutated: false,
        }
    );
    // A second completion for the same request is stale.
    assert_eq!(
        core.complete(
            id1,
            StrategyOutcome::Ok {
                count: 3,
                ordinal: Some(2),
                mutated: false,
            },
        ),
        FindCompletion::StaleDropped
    );
    assert!(core.is_open(), "completed session stays open for stepping");
}

#[test]
fn step_after_start_reports_new_ordinal() {
    let mut core = FindAdapterCore::new();
    let mut session = FindSession::default();
    let id1 = mint_id(&mut session, "foo");
    core.issue_start(id1, "foo", options()).unwrap();
    core.complete(
        id1,
        StrategyOutcome::Ok {
            count: 3,
            ordinal: Some(1),
            mutated: false,
        },
    );

    let id2 = mint_id(&mut session, "foo");
    let op = core.issue_step(id2, FindDirection::Next).unwrap();
    assert!(matches!(op, StrategyOp::Step(_)));
    assert_eq!(core.pending_request(), Some(id2));

    let completion = core.complete(
        id2,
        StrategyOutcome::Ok {
            count: 3,
            ordinal: Some(2),
            mutated: false,
        },
    );
    assert_eq!(
        completion,
        FindCompletion::Reported {
            request_id: id2,
            result: FindResult {
                match_count: 3,
                current_match: Some(2),
            },
            mutated: false,
        }
    );
}

#[test]
fn zero_matches_report_without_ordinal() {
    let mut core = FindAdapterCore::new();
    let mut session = FindSession::default();
    let id1 = mint_id(&mut session, "absent");
    core.issue_start(id1, "absent", options()).unwrap();
    let completion = core.complete(
        id1,
        StrategyOutcome::Ok {
            count: 0,
            ordinal: None,
            mutated: false,
        },
    );
    assert_eq!(
        completion,
        FindCompletion::Reported {
            request_id: id1,
            result: FindResult {
                match_count: 0,
                current_match: None,
            },
            mutated: false,
        }
    );
}

#[test]
fn replacement_start_makes_previous_completion_stale() {
    let mut core = FindAdapterCore::new();
    let mut session = FindSession::default();
    let id1 = mint_id(&mut session, "first");
    let id2 = mint_id(&mut session, "second");
    core.issue_start(id1, "first", options()).unwrap();
    core.issue_start(id2, "second", options()).unwrap();
    assert_eq!(core.pending_request(), Some(id2));

    // The first request's late completion is dropped, not delivered.
    assert_eq!(
        core.complete(
            id1,
            StrategyOutcome::Ok {
                count: 1,
                ordinal: Some(1),
                mutated: false,
            },
        ),
        FindCompletion::StaleDropped
    );
    assert_eq!(
        core.complete(
            id2,
            StrategyOutcome::Ok {
                count: 1,
                ordinal: Some(1),
                mutated: false,
            },
        ),
        FindCompletion::Reported {
            request_id: id2,
            result: FindResult {
                match_count: 1,
                current_match: Some(1),
            },
            mutated: false,
        }
    );
}

#[test]
fn mutated_flag_is_forwarded_to_the_composition() {
    let mut core = FindAdapterCore::new();
    let mut session = FindSession::default();
    let id1 = mint_id(&mut session, "foo");
    core.issue_start(id1, "foo", options()).unwrap();
    let completion = core.complete(
        id1,
        StrategyOutcome::Ok {
            count: 2,
            ordinal: Some(1),
            mutated: true,
        },
    );
    assert_eq!(
        completion,
        FindCompletion::Reported {
            request_id: id1,
            result: FindResult {
                match_count: 2,
                current_match: Some(1),
            },
            mutated: true,
        }
    );
}

#[test]
fn inconsistent_outcomes_are_rejected_not_reported() {
    let mut core = FindAdapterCore::new();
    let mut session = FindSession::default();
    let id1 = mint_id(&mut session, "foo");
    core.issue_start(id1, "foo", options()).unwrap();
    // Ordinal beyond the count.
    assert_eq!(
        core.complete(
            id1,
            StrategyOutcome::Ok {
                count: 2,
                ordinal: Some(5),
                mutated: false,
            },
        ),
        FindCompletion::Failed {
            request_id: id1,
            error: FindError::InvalidResult,
        }
    );

    let mut core = FindAdapterCore::new();
    let id1 = mint_id(&mut session, "foo");
    core.issue_start(id1, "foo", options()).unwrap();
    // Ordinal without a match count.
    assert_eq!(
        core.complete(
            id1,
            StrategyOutcome::Ok {
                count: 0,
                ordinal: Some(1),
                mutated: false,
            },
        ),
        FindCompletion::Failed {
            request_id: id1,
            error: FindError::InvalidResult,
        }
    );
}

#[test]
fn strategy_failure_is_routed_as_failed_completion() {
    let mut core = FindAdapterCore::new();
    let mut session = FindSession::default();
    let id1 = mint_id(&mut session, "foo");
    core.issue_start(id1, "foo", options()).unwrap();
    assert_eq!(
        core.complete(
            id1,
            StrategyOutcome::Failed {
                error: FindError::InvalidResult
            }
        ),
        FindCompletion::Failed {
            request_id: id1,
            error: FindError::InvalidResult,
        }
    );
}

#[test]
fn finish_drops_pending_and_returns_clear_op() {
    let mut core = FindAdapterCore::new();
    let mut session = FindSession::default();
    let id1 = mint_id(&mut session, "foo");
    core.issue_start(id1, "foo", options()).unwrap();
    let op = core.finish().expect("finish returns a clear op");
    assert!(matches!(op, StrategyOp::Clear(_)));
    assert!(!core.is_open());
    assert_eq!(core.pending_request(), None);

    // A late completion after finish is dropped.
    assert_eq!(
        core.complete(
            id1,
            StrategyOutcome::Ok {
                count: 1,
                ordinal: Some(1),
                mutated: false,
            },
        ),
        FindCompletion::StaleDropped
    );
    // Stepping a closed session is inactive.
    let id2 = mint_id(&mut session, "foo");
    assert_eq!(
        core.issue_step(id2, FindDirection::Next),
        Err(FindError::Inactive)
    );
    // Finishing an already-closed session is a no-op.
    assert_eq!(core.finish(), None);
}

#[test]
fn invalidate_drops_pending_and_returns_clear_op() {
    let mut core = FindAdapterCore::new();
    let mut session = FindSession::default();
    let id1 = mint_id(&mut session, "foo");
    core.issue_start(id1, "foo", options()).unwrap();
    let op = core.invalidate().expect("invalidate returns a clear op");
    assert!(matches!(op, StrategyOp::Clear(_)));
    assert!(!core.is_open());
    assert_eq!(core.pending_request(), None);
    assert_eq!(
        core.complete(
            id1,
            StrategyOutcome::Ok {
                count: 1,
                ordinal: Some(1),
                mutated: false,
            },
        ),
        FindCompletion::StaleDropped
    );
    let id2 = mint_id(&mut session, "foo");
    assert_eq!(
        core.issue_step(id2, FindDirection::Next),
        Err(FindError::Inactive)
    );
    assert_eq!(core.invalidate(), None);
}

#[test]
fn parse_outcome_accepts_ok_replies() {
    assert_eq!(
        parse_outcome("OK:3:2:0"),
        StrategyOutcome::Ok {
            count: 3,
            ordinal: Some(2),
            mutated: false,
        }
    );
    assert_eq!(
        parse_outcome("OK:0:-1:1"),
        StrategyOutcome::Ok {
            count: 0,
            ordinal: None,
            mutated: true,
        }
    );
    assert_eq!(
        parse_outcome("OK:7:7:1"),
        StrategyOutcome::Ok {
            count: 7,
            ordinal: Some(7),
            mutated: true,
        }
    );
}

#[test]
fn parse_outcome_rejects_malformed_replies() {
    for reply in [
        "",
        "OK",
        "OK:3",
        "OK:3:0", // ordinal 0 is not a valid 1-based ordinal
        "OK:3:2:0:extra",
        "ERR:internal",
        "garbage",
        "OK:x:y:z",
    ] {
        let outcome = parse_outcome(reply);
        assert!(
            matches!(outcome, StrategyOutcome::Failed { .. }),
            "reply {reply:?} must fail, got {outcome:?}"
        );
    }
}

#[test]
fn consistency_check_catches_strategy_bugs() {
    // A strategy that claims a count without an ordinal is consistent at
    // parse time but must be rejected by `complete`.
    let mut core = FindAdapterCore::new();
    let mut session = FindSession::default();
    let id1 = mint_id(&mut session, "foo");
    core.issue_start(id1, "foo", options()).unwrap();
    assert_eq!(
        core.complete(id1, parse_outcome("OK:3:-1:0")),
        FindCompletion::Failed {
            request_id: id1,
            error: FindError::InvalidResult,
        }
    );
}
