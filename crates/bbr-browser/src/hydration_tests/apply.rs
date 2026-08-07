//! Apply/execution contract tests.
//!
//! These prove the execution path through the real [`HydrationPolicy::apply`]
//! against the recording target and the fake engine: plan-before-execute,
//! identities-then-sessions ordering, exact engine session counts (the
//! dense no-explosion bound), and stop-on-error propagation with the
//! non-transactional contract pinned for the serial facade.

use bbr_domain::{ProfileHue, SpaceId, TabId};
use bbr_web::ProfileId;

use super::support::{dense_500, intent, profile, space, tab, RecordingTarget, TargetOp};
use crate::fake::{EngineOp, FakeEngine};
use crate::{
    HydrationError, HydrationPolicy, IntentSplit, RestoreTarget, RestoredClosedTab, RestoredSplit,
};

#[test]
fn apply_restores_500_identities_then_creates_exactly_one_engine_session() {
    let harness = FakeEngine::new().into_harness();
    let mut target = RecordingTarget::new(harness.engine.clone());

    let plan = HydrationPolicy::apply(&dense_500(), SpaceId(10), &mut target)
        .expect("valid intent applies");

    // Phase 1 executed fully before any session: every identity op precedes
    // the single session op in the combined log.
    let first_session = target
        .ops
        .iter()
        .position(|op| matches!(op, TargetOp::Session { .. }))
        .expect("exactly one session op");
    assert_eq!(first_session, target.identity_ops());
    assert_eq!(target.identity_ops(), 2 + 3 + 500 + 3 * 3);
    let sessions = target.session_ops();
    assert_eq!(sessions.len(), 1);

    // The engine saw exactly one create + one show — never 500.
    let ops = harness.ops();
    let creates: Vec<&EngineOp> = ops
        .iter()
        .filter(|op| matches!(op, EngineOp::Create { .. }))
        .collect();
    assert_eq!(creates.len(), 1, "no eager session explosion");
    match creates[0] {
        EngineOp::Create { profile, url, .. } => {
            assert_eq!(
                *profile,
                ProfileId(1),
                "session in the active space's profile"
            );
            assert_eq!(url, "https://dense100.example");
        }
        _ => unreachable!("filtered to Create"),
    }
    assert_eq!(harness.engine.borrow().live_session_count(), 1);

    // The plan returned by apply matches plan().
    assert_eq!(
        plan,
        HydrationPolicy::plan(&dense_500(), SpaceId(10)).expect("plan again")
    );
}

#[test]
fn apply_hydrates_split_panes_exactly_once_each() {
    let mut split_space = space(
        10,
        vec![tab(100, "https://a.example"), tab(101, "https://b.example")],
    );
    split_space.active_tab = Some(100);
    split_space.split = Some(IntentSplit {
        primary_tab: 100,
        secondary_tab: 101,
    });
    let intent = intent(vec![profile(1, vec![split_space])]);
    let harness = FakeEngine::new().into_harness();
    let mut target = RecordingTarget::new(harness.engine.clone());

    HydrationPolicy::apply(&intent, SpaceId(10), &mut target).expect("valid intent applies");

    let ops = harness.ops();
    let creates: Vec<&EngineOp> = ops
        .iter()
        .filter(|op| matches!(op, EngineOp::Create { .. }))
        .collect();
    assert_eq!(creates.len(), 2, "both panes hydrate exactly once");
    for op in creates {
        match op {
            EngineOp::Create { profile, .. } => assert_eq!(*profile, ProfileId(1)),
            _ => unreachable!("filtered to Create"),
        }
    }
    assert_eq!(harness.engine.borrow().live_session_count(), 2);
    let sessions = target.session_ops();
    assert_eq!(
        sessions
            .iter()
            .map(|op| match op {
                TargetOp::Session { tab, .. } => *tab,
                _ => unreachable!("filtered to Session"),
            })
            .collect::<Vec<_>>(),
        vec![TabId(100), TabId(101)],
        "split panes hydrate in primary/secondary order"
    );
}

#[test]
fn rejected_target_errors_propagate() {
    struct RejectingTarget;
    impl RestoreTarget for RejectingTarget {
        fn restore_profile(
            &mut self,
            _id: ProfileId,
            _title: &str,
            _hue: ProfileHue,
        ) -> Result<(), HydrationError> {
            Err(HydrationError::Target("profile store unavailable".into()))
        }
        fn restore_space(
            &mut self,
            _id: SpaceId,
            _profile: ProfileId,
            _title: &str,
        ) -> Result<(), HydrationError> {
            Ok(())
        }
        fn restore_tab(
            &mut self,
            _id: TabId,
            _space: SpaceId,
            _title: &str,
            _url: &str,
            _pinned: bool,
        ) -> Result<(), HydrationError> {
            Ok(())
        }
        fn set_active_tab(
            &mut self,
            _space: SpaceId,
            _tab: Option<TabId>,
        ) -> Result<(), HydrationError> {
            Ok(())
        }
        fn set_split(
            &mut self,
            _space: SpaceId,
            _split: Option<RestoredSplit>,
        ) -> Result<(), HydrationError> {
            Ok(())
        }
        fn restore_recently_closed(
            &mut self,
            _space: SpaceId,
            _entries: &[RestoredClosedTab],
        ) -> Result<(), HydrationError> {
            Ok(())
        }
        fn hydrate_session(
            &mut self,
            _tab: TabId,
            _profile: ProfileId,
            _url: &str,
        ) -> Result<(), HydrationError> {
            Ok(())
        }
    }

    let intent = intent(vec![profile(
        1,
        vec![space(10, vec![tab(100, "https://a.example")])],
    )]);
    let mut target = RejectingTarget;
    assert_eq!(
        HydrationPolicy::apply(&intent, SpaceId(10), &mut target),
        Err(HydrationError::Target("profile store unavailable".into()))
    );
}

#[test]
fn apply_rejects_an_invalid_intent_before_touching_the_target() {
    let harness = FakeEngine::new().into_harness();
    let mut target = RecordingTarget::new(harness.engine.clone());

    // Duplicate tab id — planning fails before a single target op executes.
    let invalid = intent(vec![profile(
        1,
        vec![space(
            10,
            vec![tab(100, "https://a.example"), tab(100, "https://b.example")],
        )],
    )]);
    assert_eq!(
        HydrationPolicy::apply(&invalid, SpaceId(10), &mut target),
        Err(HydrationError::DuplicateIdentity(100))
    );
    assert!(
        target.ops.is_empty(),
        "an invalid intent must not execute a single target op"
    );
    assert_eq!(harness.engine.borrow().live_session_count(), 0);
}

#[test]
fn apply_stops_at_the_first_target_error_leaving_earlier_ops_applied() {
    struct SessionFailingTarget {
        identity_calls: usize,
        session_calls: usize,
    }
    impl RestoreTarget for SessionFailingTarget {
        fn restore_profile(
            &mut self,
            _id: ProfileId,
            _title: &str,
            _hue: ProfileHue,
        ) -> Result<(), HydrationError> {
            self.identity_calls += 1;
            Ok(())
        }
        fn restore_space(
            &mut self,
            _id: SpaceId,
            _profile: ProfileId,
            _title: &str,
        ) -> Result<(), HydrationError> {
            self.identity_calls += 1;
            Ok(())
        }
        fn restore_tab(
            &mut self,
            _id: TabId,
            _space: SpaceId,
            _title: &str,
            _url: &str,
            _pinned: bool,
        ) -> Result<(), HydrationError> {
            self.identity_calls += 1;
            Ok(())
        }
        fn set_active_tab(
            &mut self,
            _space: SpaceId,
            _tab: Option<TabId>,
        ) -> Result<(), HydrationError> {
            self.identity_calls += 1;
            Ok(())
        }
        fn set_split(
            &mut self,
            _space: SpaceId,
            _split: Option<RestoredSplit>,
        ) -> Result<(), HydrationError> {
            self.identity_calls += 1;
            Ok(())
        }
        fn restore_recently_closed(
            &mut self,
            _space: SpaceId,
            _entries: &[RestoredClosedTab],
        ) -> Result<(), HydrationError> {
            self.identity_calls += 1;
            Ok(())
        }
        fn hydrate_session(
            &mut self,
            _tab: TabId,
            _profile: ProfileId,
            _url: &str,
        ) -> Result<(), HydrationError> {
            self.session_calls += 1;
            Err(HydrationError::Target("engine store unavailable".into()))
        }
    }

    let mut active = space(10, vec![tab(100, "https://a.example")]);
    active.active_tab = Some(100);
    let intent = intent(vec![profile(1, vec![active])]);
    let mut target = SessionFailingTarget {
        identity_calls: 0,
        session_calls: 0,
    };

    assert_eq!(
        HydrationPolicy::apply(&intent, SpaceId(10), &mut target),
        Err(HydrationError::Target("engine store unavailable".into()))
    );
    // Non-transactional contract, pinned for the serial facade: every
    // identity op ran before the failing session op, no rollback is
    // attempted, and no further op is issued.
    assert_eq!(target.identity_calls, 6);
    assert_eq!(target.session_calls, 1);
}
