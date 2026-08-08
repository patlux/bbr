//! The macOS find adapter facade: owns the pure mapping core and the native
//! strategy, and routes completions to the composition's sink.
//!
//! One adapter instance targets **one** web view. The composition (a later
//! delivery step) enforces the product invariant of exactly one active find
//! presentation per browser window and re-targets/invalidates on tab or
//! navigation changes.
//!
//! The composition supplies request ids minted by the authoritative portable
//! [`FindSession`](bbr_web::find_in_page::FindSession); the adapter only
//! echoes them. Every completion flows through [`FindAdapterCore::complete`],
//! which drops stale ids, validates the count/ordinal pair, and reports.
//! A completion sink installed via [`Self::set_completion_sink`] receives
//! [`FindCompletion::Reported`] / [`FindCompletion::Failed`]; the composition
//! maps those back into the portable session (`report` / `invalidate`).

use std::cell::RefCell;
use std::rc::Rc;

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_web_kit::WKWebView;

use bbr_web::find_in_page::{FindDirection, FindError, FindOptions, FindRequestId};

use crate::find_mapping::{FindAdapterCore, FindCompletion};
use crate::find_strategy::{FindStrategy, OutcomeRouter};
use crate::find_web_kit::WebKitFindStrategy;

/// The production adapter type, bound to a real WKWebView.
pub type NativeFindAdapter = WebKitFindAdapter<WebKitFindStrategy>;

/// The composition-facing completion sink. Cleared by the composition when
/// the adapter is torn down so late completions become no-ops.
pub type FindCompletionSink = Rc<RefCell<Option<Box<dyn FnMut(FindCompletion)>>>>;

/// Adapter over a native find strategy (main-thread only). Generic over the
/// strategy so contract tests can inject a deterministic fake.
pub struct WebKitFindAdapter<S: FindStrategy> {
    core: Rc<RefCell<FindAdapterCore>>,
    strategy: S,
    router: OutcomeRouter,
    sink: FindCompletionSink,
}

impl WebKitFindAdapter<WebKitFindStrategy> {
    /// Create the production adapter bound to a live web view's `WKWebView`.
    pub fn for_web_view(mtm: MainThreadMarker, web_view: Retained<WKWebView>) -> Self {
        Self::with_strategy(WebKitFindStrategy::new(mtm, web_view))
    }
}

impl<S: FindStrategy> WebKitFindAdapter<S> {
    /// Adapter over an injected strategy; contract tests inject a fake.
    pub fn with_strategy(strategy: S) -> Self {
        let core = Rc::new(RefCell::new(FindAdapterCore::new()));
        let sink: FindCompletionSink = Rc::new(RefCell::new(None));
        let router = install_router(core.clone(), sink.clone());
        Self {
            core,
            strategy,
            router,
            sink,
        }
    }

    /// The underlying strategy (test access).
    pub fn strategy(&self) -> &S {
        &self.strategy
    }

    /// Install the composition-facing completion sink.
    pub fn set_completion_sink(&self, sink: Box<dyn FnMut(FindCompletion)>) {
        *self.sink.borrow_mut() = Some(sink);
    }

    /// Start a find for `request_id` (minted by the portable session).
    pub fn start(
        &self,
        request_id: FindRequestId,
        query: &str,
        options: FindOptions,
    ) -> Result<(), FindError> {
        let op = self
            .core
            .borrow_mut()
            .issue_start(request_id, query, options)?;
        self.strategy.execute(op, Some(request_id), &self.router);
        Ok(())
    }

    /// Navigate to the next match for `request_id`.
    pub fn find_next(&self, request_id: FindRequestId) -> Result<(), FindError> {
        self.step(request_id, FindDirection::Next)
    }

    /// Navigate to the previous match for `request_id`.
    pub fn find_previous(&self, request_id: FindRequestId) -> Result<(), FindError> {
        self.step(request_id, FindDirection::Previous)
    }

    /// End the session: drop in-flight requests and clear highlights.
    pub fn finish(&self) {
        if let Some(op) = self.core.borrow_mut().finish() {
            self.strategy.execute(op, None, &self.router);
        }
    }

    /// Invalidate after a document change: drop in-flight requests and clear
    /// highlights. The composition also invalidates the portable session.
    pub fn invalidate(&self) {
        if let Some(op) = self.core.borrow_mut().invalidate() {
            self.strategy.execute(op, None, &self.router);
        }
    }

    fn step(&self, request_id: FindRequestId, direction: FindDirection) -> Result<(), FindError> {
        let op = self.core.borrow_mut().issue_step(request_id, direction)?;
        self.strategy.execute(op, Some(request_id), &self.router);
        Ok(())
    }
}

fn install_router(core: Rc<RefCell<FindAdapterCore>>, sink: FindCompletionSink) -> OutcomeRouter {
    let router: OutcomeRouter =
        Rc::new(RefCell::new(Some(Box::new(move |request_id, outcome| {
            let completion = core.borrow_mut().complete(request_id, outcome);
            if let FindCompletion::StaleDropped = completion {
                return;
            }
            if let Some(sink) = sink.borrow_mut().as_mut() {
                sink(completion);
            }
        }))));
    router
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbr_web::find_in_page::{FindResult, FindSession};

    use crate::find_mapping::{StrategyOp, StrategyOutcome};

    /// Deterministic fake strategy: records ops and lets the test route
    /// outcomes explicitly, so out-of-order completions (the stale paths) can
    /// be exercised without a display server.
    struct FakeStrategy {
        executed: RefCell<Vec<(StrategyOp, Option<FindRequestId>)>>,
        router: RefCell<Option<OutcomeRouter>>,
    }

    impl FakeStrategy {
        fn new() -> Self {
            Self {
                executed: RefCell::new(Vec::new()),
                router: RefCell::new(None),
            }
        }

        /// Route one outcome for `request_id`, as a WebKit completion would.
        fn route(&self, request_id: FindRequestId, outcome: StrategyOutcome) {
            if let Some(router) = self.router.borrow().as_ref() {
                if let Some(router) = router.borrow_mut().as_mut() {
                    router(request_id, outcome);
                }
            }
        }

        fn executed_kinds(&self) -> Vec<&'static str> {
            self.executed
                .borrow()
                .iter()
                .map(|(op, _)| match op {
                    StrategyOp::Start(_) => "start",
                    StrategyOp::Step(_) => "step",
                    StrategyOp::Clear(_) => "clear",
                })
                .collect()
        }
    }

    impl FindStrategy for FakeStrategy {
        fn execute(
            &self,
            op: StrategyOp,
            request_id: Option<FindRequestId>,
            router: &OutcomeRouter,
        ) {
            self.router.replace(Some(router.clone()));
            self.executed.borrow_mut().push((op, request_id));
        }
    }

    fn options() -> FindOptions {
        FindOptions::new(false, false, true)
    }

    fn recording_adapter() -> (
        WebKitFindAdapter<FakeStrategy>,
        Rc<RefCell<Vec<FindCompletion>>>,
    ) {
        let adapter = WebKitFindAdapter::with_strategy(FakeStrategy::new());
        let completions: Rc<RefCell<Vec<FindCompletion>>> = Rc::new(RefCell::new(Vec::new()));
        let recorded = completions.clone();
        adapter.set_completion_sink(Box::new(move |completion| {
            recorded.borrow_mut().push(completion);
        }));
        (adapter, completions)
    }

    fn ok(count: usize, ordinal: usize) -> StrategyOutcome {
        StrategyOutcome::Ok {
            count,
            ordinal: Some(ordinal),
            mutated: false,
        }
    }

    #[test]
    fn start_and_step_echo_portable_ids_through_the_full_loop() {
        let (adapter, completions) = recording_adapter();

        let mut session = FindSession::default();
        let id1 = session.start("foo", options()).unwrap();
        adapter.start(id1, "foo", options()).unwrap();
        adapter.strategy().route(id1, ok(3, 1));
        // The composition feeds the adapter's completion back into the session.
        session
            .report(
                id1,
                FindResult {
                    match_count: 3,
                    current_match: Some(1),
                },
            )
            .unwrap();
        assert!(session.is_active());

        let id2 = session.find_next().unwrap();
        adapter.find_next(id2).unwrap();
        adapter.strategy().route(id2, ok(3, 2));
        session
            .report(
                id2,
                FindResult {
                    match_count: 3,
                    current_match: Some(2),
                },
            )
            .unwrap();

        assert_eq!(
            completions.borrow().as_slice(),
            [
                FindCompletion::Reported {
                    request_id: id1,
                    result: FindResult {
                        match_count: 3,
                        current_match: Some(1),
                    },
                    mutated: false,
                },
                FindCompletion::Reported {
                    request_id: id2,
                    result: FindResult {
                        match_count: 3,
                        current_match: Some(2),
                    },
                    mutated: false,
                },
            ]
        );
        assert_eq!(adapter.strategy().executed_kinds(), ["start", "step"]);
    }

    #[test]
    fn replacement_start_drops_the_superseded_completion() {
        let (adapter, completions) = recording_adapter();

        let mut session = FindSession::default();
        let id1 = session.start("first", options()).unwrap();
        let id2 = session.start("second", options()).unwrap();
        adapter.start(id1, "first", options()).unwrap();
        adapter.start(id2, "second", options()).unwrap();

        // The superseded request's completion arrives after the replacement
        // was issued: it must be dropped, never delivered.
        adapter.strategy().route(id1, ok(1, 1));
        assert_eq!(completions.borrow().len(), 0);

        adapter.strategy().route(id2, ok(1, 1));
        assert_eq!(
            completions.borrow().as_slice(),
            [FindCompletion::Reported {
                request_id: id2,
                result: FindResult {
                    match_count: 1,
                    current_match: Some(1),
                },
                mutated: false,
            }]
        );
    }

    #[test]
    fn finish_routes_clear_and_stops_late_completions() {
        let (adapter, completions) = recording_adapter();

        let mut session = FindSession::default();
        let id1 = session.start("foo", options()).unwrap();
        adapter.start(id1, "foo", options()).unwrap();
        adapter.strategy().route(id1, ok(1, 1));
        assert_eq!(completions.borrow().len(), 1);

        adapter.finish();
        assert_eq!(adapter.strategy().executed_kinds(), ["start", "clear"]);
        // A completion arriving after finish must not reach the sink.
        adapter.strategy().route(id1, ok(1, 1));
        assert_eq!(completions.borrow().len(), 1);

        let id2 = session.start("bar", options()).unwrap();
        adapter.start(id2, "bar", options()).unwrap();
        adapter.strategy().route(id2, ok(1, 1));
        assert_eq!(completions.borrow().len(), 2);
    }

    #[test]
    fn invalidate_clears_and_drops_late_completions() {
        let (adapter, completions) = recording_adapter();

        let mut session = FindSession::default();
        let id1 = session.start("foo", options()).unwrap();
        adapter.start(id1, "foo", options()).unwrap();
        adapter.strategy().route(id1, ok(1, 1));

        adapter.invalidate();
        assert_eq!(adapter.strategy().executed_kinds(), ["start", "clear"]);
        adapter.strategy().route(id1, ok(1, 1));
        assert_eq!(completions.borrow().len(), 1);
    }

    #[test]
    fn zero_match_outcome_is_reported() {
        let (adapter, completions) = recording_adapter();

        let mut session = FindSession::default();
        let id1 = session.start("absent", options()).unwrap();
        adapter.start(id1, "absent", options()).unwrap();
        adapter.strategy().route(
            id1,
            StrategyOutcome::Ok {
                count: 0,
                ordinal: None,
                mutated: false,
            },
        );
        assert_eq!(
            completions.borrow().as_slice(),
            [FindCompletion::Reported {
                request_id: id1,
                result: FindResult {
                    match_count: 0,
                    current_match: None,
                },
                mutated: false,
            }]
        );
    }

    #[test]
    fn strategy_failure_reaches_the_sink_as_failed() {
        let (adapter, completions) = recording_adapter();

        let mut session = FindSession::default();
        let id1 = session.start("foo", options()).unwrap();
        adapter.start(id1, "foo", options()).unwrap();
        adapter.strategy().route(
            id1,
            StrategyOutcome::Failed {
                error: FindError::InvalidResult,
            },
        );
        assert_eq!(
            completions.borrow().as_slice(),
            [FindCompletion::Failed {
                request_id: id1,
                error: FindError::InvalidResult,
            }]
        );
    }

    #[test]
    fn mutated_flag_is_forwarded_with_the_result() {
        let (adapter, completions) = recording_adapter();

        let mut session = FindSession::default();
        let id1 = session.start("foo", options()).unwrap();
        adapter.start(id1, "foo", options()).unwrap();
        adapter.strategy().route(
            id1,
            StrategyOutcome::Ok {
                count: 2,
                ordinal: Some(1),
                mutated: true,
            },
        );
        assert_eq!(
            completions.borrow().as_slice(),
            [FindCompletion::Reported {
                request_id: id1,
                result: FindResult {
                    match_count: 2,
                    current_match: Some(1),
                },
                mutated: true,
            }]
        );
    }
}
