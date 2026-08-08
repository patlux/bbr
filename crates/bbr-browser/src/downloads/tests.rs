//! Contract tests for the portable download controller (no display server,
//! no filesystem, no wall clock). Every L6 event/rejection path, ordering,
//! cancel races, retry/restart linkage, reveal eligibility, the action
//! sink, and the settled post-MVP history seams (clear / delete / restore /
//! terminal-only persistence). Split by responsibility into
//! [`lifecycle`] (machine fidelity), [`ordering`] (deterministic ordering,
//! reveal, sink edge cases), and [`history`] (persistence seams) so no test
//! module outgrows the ~500-line structure gate.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use bbr_web::downloads::{
    DestinationDecision, DownloadAction, DownloadEvent, DownloadId, DownloadProgress,
};

use super::{ApplyOutcome, DownloadController};

mod history;
mod lifecycle;
mod ordering;

fn id(n: u64) -> DownloadId {
    DownloadId(n)
}

/// A controller with a deterministic advancing clock.
fn controller() -> (DownloadController, Rc<Cell<u64>>) {
    let clock = Rc::new(Cell::new(1_000));
    let tick = clock.clone();
    (DownloadController::with_clock(move || tick.get()), clock)
}

/// A controller with a recorded action sink.
fn wired() -> (DownloadController, Rc<RefCell<Vec<DownloadAction>>>) {
    let (mut controller, _) = controller();
    let actions: Rc<RefCell<Vec<DownloadAction>>> = Rc::new(RefCell::new(Vec::new()));
    let recorded = actions.clone();
    controller.set_action_sink(Box::new(move |action| recorded.borrow_mut().push(action)));
    (controller, actions)
}

fn suggested() -> DestinationDecision {
    DestinationDecision::Suggested("file.bin".into())
}

/// Drive a fresh start to `Running { progress }` with a resolved path.
fn run_attempt(controller: &mut DownloadController, url: &str) -> DownloadId {
    let attempt = controller.start(url, suggested()).unwrap();
    assert_eq!(
        controller.apply(DownloadEvent::DestinationResolved {
            id: attempt,
            destination: format!("/Downloads/{url}"),
        }),
        ApplyOutcome::Applied
    );
    assert_eq!(
        controller.apply(DownloadEvent::Progress {
            id: attempt,
            progress: DownloadProgress::new(10, Some(100)).unwrap(),
        }),
        ApplyOutcome::Applied
    );
    attempt
}

/// Drive a fresh start to a fully-received success (legal L6 transition:
/// `Succeeded` requires every declared byte).
fn succeed_attempt(controller: &mut DownloadController, url: &str) -> DownloadId {
    let attempt = run_attempt(controller, url);
    assert_eq!(
        controller.apply(DownloadEvent::Progress {
            id: attempt,
            progress: DownloadProgress::new(100, Some(100)).unwrap(),
        }),
        ApplyOutcome::Applied
    );
    assert_eq!(
        controller.apply(DownloadEvent::Succeeded { id: attempt }),
        ApplyOutcome::Applied
    );
    attempt
}
