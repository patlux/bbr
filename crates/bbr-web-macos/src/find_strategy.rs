//! The native strategy seam: what executes a scripted find op and routes its
//! outcome back to the adapter pipeline. Portable types only — no AppKit — so
//! contract tests can inject a deterministic fake instead of a display server.

use std::cell::RefCell;
use std::rc::Rc;

use bbr_web::find_in_page::FindRequestId;

use crate::find_mapping::{StrategyOp, StrategyOutcome};

/// Routes one strategy outcome (for `request_id`) into the adapter's
/// completion pipeline. Shared with the async completion block: the block
/// captures a clone and calls it on the main thread when WebKit completes.
pub type OutcomeRouter = Rc<RefCell<Option<Box<dyn FnMut(FindRequestId, StrategyOutcome)>>>>;

/// Executes scripted find operations against a native engine.
///
/// Implementations are main-thread-only. `execute` returns immediately; the
/// outcome is delivered later through `router`. When `request_id` is `None`
/// (clear ops from `finish`/`invalidate`) no completion is routed.
pub trait FindStrategy {
    fn execute(&self, op: StrategyOp, request_id: Option<FindRequestId>, router: &OutcomeRouter);
}
