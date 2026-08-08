//! Pure mapping between the portable find contract and the native strategy.
//!
//! This module contains **no** AppKit/WebKit imports and no `unsafe` code, so
//! the entire portable↔native mapping can be contract-tested without a display
//! server. The strategy script itself (`js/find_strategy.js`) is executed by
//! WebKit's public `evaluateJavaScript` bridge; the script's determinism
//! contract is documented at the top of that file and proven by the real-app
//! E2E fixtures in a later delivery step.
//!
//! ## API verdict (why a scripted strategy exists)
//!
//! WKWebView's public find surface is `findString:withConfiguration:
//! completionHandler:`, whose `WKFindResult` reports only `matchFound: Bool`
//! (verified against the SDK header, the runtime method list, and WebKit
//! source: `WebPageProxy::findString` completes with the boolean alone because
//! the public wrapper sends `maxCount = 1` without `DetermineMatchIndex`).
//! The count/ordinal APIs that once existed (`findAllString:`,
//! `WKFindConfiguration.findAllMatches`) were removed from modern SDKs and
//! survive only as private SPI, which this adapter deliberately does not use.
//! The public JS bridge therefore is the pinned API that expresses the
//! contract: count, ordinal, wrap/no-wrap, whole-word and case semantics all
//! come from one deterministic WebKit-executed strategy.
//!
//! ## Mapping table (portable lifecycle → native call → callback)
//!
//! | Portable (`FindSession`)        | Adapter (`FindAdapterCore`)        | Native (WKWebView)                        | Callback                      |
//! |---------------------------------|------------------------------------|-------------------------------------------|-------------------------------|
//! | `start(q, opts) -> id`          | `issue_start(id, q, opts)`         | `evaluateJavaScript(build_find_script)`   | `report(id, {count, ordinal})`|
//! | `find_next() -> id`             | `issue_step(id, Next)`             | `evaluateJavaScript(build_find_script)`   | `report(id, {count, ordinal})`|
//! | `find_previous() -> id`         | `issue_step(id, Previous)`         | `evaluateJavaScript(build_find_script)`   | `report(id, {count, ordinal})`|
//! | `report(id, result)`            | `complete(id, outcome)`            | —                                         | `FindCompletion::Reported`    |
//! | `finish()`                      | `finish()`                         | `evaluateJavaScript(build_clear_script)`  | none (fire-and-forget)        |
//! | `invalidate()`                  | `invalidate()`                     | `evaluateJavaScript(build_clear_script)`  | none (fire-and-forget)        |
//!
//! Request correlation: the adapter stores the single in-flight `FindRequestId`
//! echoed from the portable session. A completion whose id is not current is
//! dropped as [`FindCompletion::StaleDropped`]; a completion after
//! `finish`/`invalidate` is likewise dropped. The portable session
//! independently rejects stale ids, so the two layers agree. WebKit itself
//! makes no staleness guarantee: in-flight `evaluateJavaScript` completions are
//! not cancelled by navigation, which is exactly why the adapter owns
//! correlation.

use std::fmt::Write;

use bbr_web::find_in_page::{FindDirection, FindError, FindOptions, FindRequestId, FindResult};

/// The strategy script executed inside the web view. Tokens are filled by
/// [`build_find_script`] / [`build_clear_script`].
pub const FIND_STRATEGY_JS: &str = include_str!("../js/find_strategy.js");

const TOKEN_MODE: &str = "\"__BBR_MODE__\"";
const TOKEN_QUERY: &str = "\"__BBR_QUERY__\"";
const TOKEN_CASE: &str = "\"__BBR_CASE__\"";
const TOKEN_WORD: &str = "\"__BBR_WORD__\"";
const TOKEN_WRAP: &str = "\"__BBR_WRAP__\"";
const TOKEN_DIR: &str = "\"__BBR_DIR__\"";

/// A native find operation handed to the strategy for execution. The script
/// is fully built here (pure, testable) so the strategy stays a thin runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrategyOp {
    /// Search, select the first match, highlight all; reports count/ordinal.
    Start(String),
    /// Step from the current match in a direction; reports count/ordinal.
    Step(String),
    /// Clear selection and highlights; no completion is routed.
    Clear(String),
}

/// Outcome reported by the native strategy for one request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyOutcome {
    /// A result computed by WebKit. `ordinal` is `None` exactly for zero
    /// matches. `mutated` signals a DOM content change since the previous
    /// request, which makes earlier results stale.
    Ok {
        count: usize,
        ordinal: Option<usize>,
        mutated: bool,
    },
    /// The engine could not produce a usable result.
    Failed { error: FindError },
}

/// What the adapter does with a strategy completion for one request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindCompletion {
    /// A result for `request_id`. `mutated` tells the composition that the
    /// document changed since the previous request (it may invalidate the
    /// portable session).
    Reported {
        request_id: FindRequestId,
        result: FindResult,
        mutated: bool,
    },
    /// The strategy could not produce a usable result for `request_id`.
    Failed {
        request_id: FindRequestId,
        error: FindError,
    },
    /// The completion belongs to a superseded or invalidated request; dropped.
    StaleDropped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingKind {
    Start,
    Step { direction: FindDirection },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExpectedFind {
    request_id: FindRequestId,
    kind: PendingKind,
}

/// Pure adapter state: open/closed lifecycle plus the single in-flight
/// request. `query`/`options` outlive the in-flight request so a step after a
/// completed start still knows what to search for.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FindAdapterCore {
    open: bool,
    query: Option<String>,
    options: FindOptions,
    expected: Option<ExpectedFind>,
}

impl FindAdapterCore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Issue a query start for `request_id` and return the scripted strategy
    /// op. The query is trimmed (mirroring [`FindSession::start`]); an empty
    /// query is rejected without changing state.
    pub fn issue_start(
        &mut self,
        request_id: FindRequestId,
        query: &str,
        options: FindOptions,
    ) -> Result<StrategyOp, FindError> {
        let query = query.trim().to_owned();
        if query.is_empty() {
            return Err(FindError::EmptyQuery);
        }
        self.expected = Some(ExpectedFind {
            request_id,
            kind: PendingKind::Start,
        });
        self.query = Some(query.clone());
        self.options = options;
        self.open = true;
        Ok(StrategyOp::Start(build_find_script(&query, options, None)))
    }

    /// Issue a navigation step for `request_id`. Requires an open session; the
    /// zero-match case is rejected by the portable session before reaching the
    /// adapter, so the core deliberately does not re-implement match counting.
    pub fn issue_step(
        &mut self,
        request_id: FindRequestId,
        direction: FindDirection,
    ) -> Result<StrategyOp, FindError> {
        if !self.open {
            return Err(FindError::Inactive);
        }
        let Some(query) = self.query.as_ref() else {
            return Err(FindError::Inactive);
        };
        let options = self.options;
        self.expected = Some(ExpectedFind {
            request_id,
            kind: PendingKind::Step { direction },
        });
        Ok(StrategyOp::Step(build_find_script(
            query,
            options,
            Some(direction),
        )))
    }

    /// Route one strategy completion. Drops completions whose id is not the
    /// current request (superseded by a replacement start/step, or invalidated
    /// by `finish`/`invalidate`). Validates the count/ordinal pair before
    /// reporting.
    pub fn complete(
        &mut self,
        request_id: FindRequestId,
        outcome: StrategyOutcome,
    ) -> FindCompletion {
        let Some(expected_id) = self.expected.as_ref().map(|expected| expected.request_id) else {
            return FindCompletion::StaleDropped;
        };
        if expected_id != request_id {
            return FindCompletion::StaleDropped;
        }
        self.expected = None;
        match outcome {
            StrategyOutcome::Failed { error } => FindCompletion::Failed { request_id, error },
            StrategyOutcome::Ok {
                count,
                ordinal,
                mutated,
            } => {
                let result = FindResult {
                    match_count: count,
                    current_match: ordinal,
                };
                if !result.is_consistent() {
                    return FindCompletion::Failed {
                        request_id,
                        error: FindError::InvalidResult,
                    };
                }
                FindCompletion::Reported {
                    request_id,
                    result,
                    mutated,
                }
            }
        }
    }

    /// End the session: drop the in-flight request and clear highlights at the
    /// strategy. Returns the clear op, or `None` when already closed.
    pub fn finish(&mut self) -> Option<StrategyOp> {
        self.close()
    }

    /// Mark all current and in-flight results stale after a document change:
    /// drop the in-flight request and clear highlights. Same native effect as
    /// [`Self::finish`]; the portable distinction lives in `FindSession`.
    pub fn invalidate(&mut self) -> Option<StrategyOp> {
        self.close()
    }

    /// Whether a session is open (started, not finished/invalidated).
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// The in-flight request id, if any.
    pub fn pending_request(&self) -> Option<FindRequestId> {
        self.expected.as_ref().map(|expected| expected.request_id)
    }

    fn close(&mut self) -> Option<StrategyOp> {
        if !self.open && self.expected.is_none() {
            return None;
        }
        self.open = false;
        self.query = None;
        self.expected = None;
        Some(StrategyOp::Clear(build_clear_script()))
    }
}

/// Build the find script for `query`/`options`. `direction` `None` produces a
/// start (select first match); `Some` produces a step.
pub fn build_find_script(
    query: &str,
    options: FindOptions,
    direction: Option<FindDirection>,
) -> String {
    let mode = if direction.is_some() { "step" } else { "start" };
    let dir = match direction {
        None | Some(FindDirection::Next) => 1,
        Some(FindDirection::Previous) => -1,
    };
    replace_tokens(&[
        (TOKEN_MODE, quote_double(mode)),
        (TOKEN_QUERY, js_quote(query)),
        (TOKEN_CASE, options.case_sensitive.to_string()),
        (TOKEN_WORD, options.whole_word.to_string()),
        (TOKEN_WRAP, options.wrap_around.to_string()),
        (TOKEN_DIR, dir.to_string()),
    ])
}

/// Build the clear script (mode `clear`; other tokens remain placeholders).
pub fn build_clear_script() -> String {
    replace_tokens(&[(TOKEN_MODE, quote_double("clear"))])
}

fn quote_double(value: &str) -> String {
    format!("\"{value}\"")
}

/// Quote a string as a single-quoted JS string literal. Every control char is
/// escaped so the embedded query cannot break out of the literal.
pub fn js_quote(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 2);
    out.push('\'');
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('\'');
    out
}

/// Fill the strategy template's tokens. Each token must occur exactly once.
fn replace_tokens(replacements: &[(&str, String)]) -> String {
    let mut out = FIND_STRATEGY_JS.to_owned();
    for (token, value) in replacements {
        let occurrences = out.matches(token).count();
        assert_eq!(
            occurrences, 1,
            "strategy template token {token} must occur exactly once (found {occurrences})"
        );
        out = out.replace(token, value);
    }
    out
}

/// Parse the strategy's text reply. Grammar:
/// `OK:<count>:<ordinal|-1>:<mutated 0|1>` or `ERR:<code>`. A missing or
/// malformed count/ordinal makes the whole reply a failed outcome, so the
/// composition never sees a fabricated count.
pub fn parse_outcome(reply: &str) -> StrategyOutcome {
    // Exactly four colon-separated fields: tag, count, ordinal, mutated.
    let parts: Vec<&str> = reply.splitn(5, ':').collect();
    if parts.len() != 4 || parts[0] != "OK" {
        return StrategyOutcome::Failed {
            error: FindError::InvalidResult,
        };
    }
    let Ok(count) = parts[1].parse::<usize>() else {
        return StrategyOutcome::Failed {
            error: FindError::InvalidResult,
        };
    };
    let ordinal = match parts[2] {
        "-1" => None,
        s => match s.parse::<i64>() {
            Ok(value) if value >= 1 => Some(value as usize),
            _ => {
                return StrategyOutcome::Failed {
                    error: FindError::InvalidResult,
                }
            }
        },
    };
    let mutated = parts[3] == "1";
    StrategyOutcome::Ok {
        count,
        ordinal,
        mutated,
    }
}

#[cfg(test)]
mod tests;
