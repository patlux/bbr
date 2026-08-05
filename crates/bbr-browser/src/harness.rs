//! Shared controller-test harness: builds a [`BrowserController`] over the
//! deterministic fake engine, optionally wiring the engine's event sink into
//! `apply_event` exactly like the composition root will.

use std::cell::RefCell;
use std::rc::Rc;

use crate::fake::{FakeEngine, FakeHarness};
use crate::{BrowserController, ControllerEvent, ViewIdAllocator};

/// A deterministic 1, 2, 3, … view-id allocator (E2E-style injection).
pub(crate) fn seq_ids() -> ViewIdAllocator {
    let mut next: u64 = 0;
    Box::new(move || {
        next += 1;
        next
    })
}

/// Build a controller over the fake engine with events wired into
/// `apply_event` exactly like the composition root does, and return the
/// observation handle.
pub(crate) fn wired() -> (Rc<RefCell<BrowserController>>, FakeHarness) {
    let harness = FakeEngine::new().into_harness();
    let controller = Rc::new(RefCell::new(BrowserController::new(
        harness.engine.clone(),
        seq_ids(),
    )));
    harness.engine.borrow_mut().set_events({
        let controller = controller.clone();
        Box::new(move |event| {
            controller.borrow_mut().apply_event(event);
        })
    });
    (controller, harness)
}

/// A fresh controller over the fake (no event wiring) plus the harness.
pub(crate) fn plain() -> (BrowserController, FakeHarness) {
    let harness = FakeEngine::new().into_harness();
    let controller = BrowserController::new(harness.engine.clone(), seq_ids());
    (controller, harness)
}

/// Open a tab, expecting success.
pub(crate) fn open(c: &mut BrowserController, url: &str) -> ControllerEvent {
    c.open_tab(url).expect("open tab succeeds")
}
