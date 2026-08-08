//! The production strategy: executes the scripted op through the public
//! `evaluateJavaScript` bridge on a real WKWebView. Main-thread only.
//!
//! ## Content-world isolation
//!
//! The strategy runs in [`WKContentWorld::defaultClientWorld`] — BBR's own
//! client world — never the page's main content world
//! (`evaluateJavaScript:completionHandler:` executes in the page world and
//! is deliberately not used). The client world shares the DOM with the page
//! (the strategy can read text runs and move the selection) but has its own
//! JavaScript global object, so BBR's find machinery — the `__bbr*` globals
//! and the MutationObserver installed by `js/find_strategy.js` — is
//! unobservable by and cannot collide with page scripts. The real-app E2E
//! proves this with a fixture page that records every unexpected global it
//! can see and asserts none after a find run.
//!
//! The strategy stays non-destructive: it may read the DOM and move the
//! selection, but never persistently mutates page structure, attributes,
//! styles, or globals. Highlights (CSS Custom Highlight registry) and the
//! selection are transient and cleared on finish/invalidate/zero-match.
//!
//! # Safety
//!
//! `evaluateJavaScript` completions are delivered on the main thread, so the
//! completion block only ever runs there. The block captures a clone of the
//! [`OutcomeRouter`] (`Rc<RefCell<...>>`) — never the web view — so a
//! completion arriving after the adapter was dropped is harmless: it borrows
//! the router (kept alive by the pending block) and routes into whatever sink
//! is still installed, or does nothing.

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::MainThreadMarker;
use objc2_foundation::{NSError, NSString};
use objc2_web_kit::{WKContentWorld, WKWebView};

use bbr_web::find_in_page::{FindError, FindRequestId};

use crate::find_mapping::{parse_outcome, StrategyOp, StrategyOutcome};
use crate::find_strategy::{FindStrategy, OutcomeRouter};

/// Strategy that runs [`StrategyOp`] scripts in a specific live WKWebView.
pub struct WebKitFindStrategy {
    web_view: Retained<WKWebView>,
}

impl WebKitFindStrategy {
    /// Create a strategy bound to `web_view` (a retained clone of the live
    /// session's view — the find adapter never creates or moves sessions).
    pub fn new(mtm: MainThreadMarker, web_view: Retained<WKWebView>) -> Self {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "WebKitFindStrategy constructed off the main thread"
        );
        let _ = mtm;
        Self { web_view }
    }

    fn assert_main_thread(&self) {
        debug_assert!(
            MainThreadMarker::new().is_some(),
            "WebKitFindStrategy used off the main thread"
        );
    }

    fn execute_script(
        &self,
        script: &str,
        request_id: Option<FindRequestId>,
        router: &OutcomeRouter,
    ) {
        self.assert_main_thread();
        let mtm = MainThreadMarker::new().expect("find strategy on the main thread");
        let ns_script = NSString::from_str(script);
        // The strategy executes in BBR's own client content world
        // (`defaultClientWorld`), never the page's main content world: the
        // find machinery's `__bbr*` globals and MutationObserver live in a
        // separate JS global object and are unobservable by page scripts,
        // while the shared DOM still allows full read/selection access.
        // `frame: None` targets the main frame (the strategy walks
        // same-origin iframes itself).
        let world = unsafe { WKContentWorld::defaultClientWorld(mtm) };
        let router = router.clone();
        let block = block2::RcBlock::new(move |result: *mut AnyObject, error: *mut NSError| {
            let outcome = js_result_to_outcome(result, error);
            if let Some(request_id) = request_id {
                if let Some(router) = router.borrow_mut().as_mut() {
                    router(request_id, outcome);
                }
            }
        });
        // SAFETY: main thread; `self.web_view` is retained and alive for the
        // duration of this call; `world` is the process-wide default client
        // world (retained by WebKit); the completion block only touches
        // `router`.
        unsafe {
            self.web_view
                .evaluateJavaScript_inFrame_inContentWorld_completionHandler(
                    &ns_script,
                    None,
                    &world,
                    Some(&block),
                );
        }
    }
}

impl FindStrategy for WebKitFindStrategy {
    fn execute(&self, op: StrategyOp, request_id: Option<FindRequestId>, router: &OutcomeRouter) {
        let script = match op {
            StrategyOp::Start(script) | StrategyOp::Step(script) | StrategyOp::Clear(script) => {
                script
            }
        };
        self.execute_script(&script, request_id, router);
    }
}

/// Convert the `evaluateJavaScript` completion into a strategy outcome.
///
/// A non-null error (JS exception, crashed process) or a non-string return
/// value means the engine produced no usable result; the reply parser never
/// sees fabricated data. Completions from a client world run on the main
/// thread exactly like the page-world bridge, so the parsing contract is
/// unchanged by the isolation fix.
fn js_result_to_outcome(result: *mut AnyObject, error: *mut NSError) -> StrategyOutcome {
    let failed = || StrategyOutcome::Failed {
        error: FindError::InvalidResult,
    };
    // SAFETY: the error pointer is a valid `NSError*` or null; we only test it.
    if !error.is_null() {
        log::warn!("find strategy: evaluateJavaScript failed with an error");
        return failed();
    }
    // SAFETY: `result` is the JS return value (an NSString for our string
    // script) or null; downcast_ref verifies the class before dereferencing.
    let Some(object) = (unsafe { result.as_ref() }) else {
        return failed();
    };
    let Some(ns_string) = object.downcast_ref::<NSString>() else {
        log::warn!("find strategy: evaluateJavaScript returned a non-string value");
        return failed();
    };
    parse_outcome(&ns_string.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_result_is_a_failed_outcome() {
        assert!(matches!(
            js_result_to_outcome(std::ptr::null_mut(), std::ptr::null_mut()),
            StrategyOutcome::Failed { .. }
        ));
    }
}
