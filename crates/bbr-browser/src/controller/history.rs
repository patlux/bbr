//! Blueprint D14: the focused session's history for the Back/Forward
//! history menu. The engine's own list is the authority (read on demand,
//! never mirrored); the jump routes through the ordinary navigation seam.

use bbr_web::{SessionHistory, WebAction};

use crate::outcome::ControllerEvent;
use crate::ControllerError;

use super::BrowserController;

impl BrowserController {
    /// The session history of the focused navigation target (the active
    /// tab, the focused split pane, or the open preview) — `None` when it
    /// has no live session.
    pub fn session_history(&self) -> Option<SessionHistory> {
        let view = match &self.preview {
            Some(preview) => Some(preview.view),
            None => self
                .focused_tab()
                .and_then(|tab| self.registry.view_for_tab(tab)),
        }?;
        self.engine.borrow().session_history(view)
    }

    /// Jump `offset` entries through the focused session's history in one
    /// step (negative = back). Zero is rejected: it is not a navigation.
    pub fn go_to_history(&mut self, offset: i32) -> Result<ControllerEvent, ControllerError> {
        if offset == 0 {
            return Err(ControllerError::NoActiveTab);
        }
        self.navigate(WebAction::GoToHistory { offset })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use bbr_web::{HistoryEntry, SessionHistory, WebAction};

    use crate::fake::{EngineOp, FakeEngine};
    use crate::harness::seq_ids;
    use crate::{BrowserController, EngineRef};

    type Ops = Rc<RefCell<Vec<EngineOp>>>;

    /// A controller over a fake the test keeps concretely (to script the
    /// history the engine reports).
    fn plain_fake() -> (BrowserController, Rc<RefCell<FakeEngine>>, Ops) {
        let fake = Rc::new(RefCell::new(FakeEngine::new()));
        let ops = fake.borrow().ops().clone();
        let engine: EngineRef = fake.clone();
        (BrowserController::new(engine, seq_ids()), fake, ops)
    }

    fn entry(url: &str) -> HistoryEntry {
        HistoryEntry {
            url: url.into(),
            title: String::new(),
        }
    }

    #[test]
    fn history_reads_the_focused_session_and_jumps_through_navigate() {
        let (mut c, fake, ops) = plain_fake();
        c.open_tab("https://c.example").unwrap();
        let view = c.snapshot().active_view.unwrap();
        let history = SessionHistory {
            back: vec![entry("https://b.example"), entry("https://a.example")],
            forward: vec![],
        };
        fake.borrow_mut().set_history(view, history.clone());
        assert_eq!(c.session_history(), Some(history));
        c.go_to_history(-2).unwrap();
        assert!(ops.borrow().contains(&EngineOp::Action {
            view,
            action: WebAction::GoToHistory { offset: -2 },
        }));
        assert!(c.go_to_history(0).is_err(), "zero is not a navigation");
    }

    #[test]
    fn no_live_session_has_no_history() {
        let (c, _fake, _ops) = plain_fake();
        assert_eq!(c.session_history(), None);
    }
}
