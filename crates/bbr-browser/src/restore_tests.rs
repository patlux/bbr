//! Restore-facade contract tests (the `restore` module's test side): the
//! chrome-first visible-set hydration, the pinned non-transactional
//! rollback, allocator advancement past restored ids, and the persist-
//! direction intent export — exercised against the fake engine.

use bbr_web::{ProfileId, WebViewId};

use crate::fake::EngineOp;
use crate::harness::seq_ids;
use crate::hydration::{
    HydrationPolicy, IntentClosedTab, IntentProfile, IntentSpace, IntentSplit, IntentTab,
    RestoreIntent,
};
use crate::{BrowserController, SessionRequest};

/// The dense persisted fixture (mirrors `hydration_tests::support`):
/// profile 1 with space 10 (tabs 100..102, pinned prefix [100], active
/// 101, one recently-closed entry, a split over 101/102) plus space 11
/// (tab 110, active 110, one recently-closed entry) and profile 2 with
/// space 20 (500 tabs, pinned prefix, active 200). The active space is
/// pinned by the caller.
pub(crate) fn dense_intent() -> RestoreIntent {
    RestoreIntent {
        profiles: vec![
            IntentProfile {
                id: 1,
                title: "Work".into(),
                hue: crate::ProfileHue::Cobalt,
                spaces: vec![
                    IntentSpace {
                        id: 10,
                        title: "Main".into(),
                        tabs: vec![
                            IntentTab {
                                id: 100,
                                title: "Pinned".into(),
                                url: "https://a.example".into(),
                                pinned: true,
                            },
                            IntentTab {
                                id: 101,
                                title: "One".into(),
                                url: "https://b.example".into(),
                                pinned: false,
                            },
                            IntentTab {
                                id: 102,
                                title: "Two".into(),
                                url: "https://c.example".into(),
                                pinned: false,
                            },
                        ],
                        active_tab: Some(101),
                        recently_closed: vec![IntentClosedTab {
                            tab_id: 900,
                            title: String::new(),
                            url: "https://closed.example".into(),
                            pinned: false,
                            former_position: 2,
                        }],
                        split: Some(IntentSplit {
                            primary_tab: 101,
                            secondary_tab: 102,
                        }),
                    },
                    IntentSpace {
                        id: 11,
                        title: "Second".into(),
                        tabs: vec![IntentTab {
                            id: 110,
                            title: "Solo".into(),
                            url: "https://d.example".into(),
                            pinned: false,
                        }],
                        active_tab: Some(110),
                        recently_closed: vec![IntentClosedTab {
                            tab_id: 901,
                            title: String::new(),
                            url: "https://e.example".into(),
                            pinned: false,
                            former_position: 0,
                        }],
                        split: None,
                    },
                ],
            },
            IntentProfile {
                id: 2,
                title: "Dense".into(),
                hue: crate::ProfileHue::Violet,
                spaces: vec![IntentSpace {
                    id: 20,
                    title: "Huge".into(),
                    tabs: (0..500)
                        .map(|index| IntentTab {
                            id: 1000 + index,
                            title: format!("Tab {index}"),
                            url: format!("https://dense{index}.example/"),
                            pinned: index < 3,
                        })
                        .collect(),
                    active_tab: Some(1000),
                    recently_closed: Vec::new(),
                    split: None,
                }],
            },
        ],
    }
}

#[test]
fn restored_chrome_first_hydrates_only_the_visible_set() {
    let intent = dense_intent();
    let harness = crate::fake::FakeEngine::new().into_harness();
    let (controller, plan) = BrowserController::restored(
        harness.engine.clone(),
        seq_ids(),
        &intent,
        bbr_domain::SpaceId(10),
    )
    .unwrap();

    // Chrome first: every identity restored with its persisted id.
    let snap = controller.snapshot();
    assert_eq!(snap.active_space, bbr_domain::SpaceId(10));
    assert_eq!(snap.profiles.len(), 2);
    assert_eq!(snap.spaces.len(), 3);
    assert_eq!(snap.spaces[0].id, bbr_domain::SpaceId(10));
    assert_eq!(snap.spaces[0].tab_count, 3);
    assert_eq!(snap.spaces[1].tab_count, 1);
    assert_eq!(snap.spaces[2].tab_count, 500);
    assert_eq!(snap.all_tabs.len(), 504, "503 live + 1 recently closed");

    // Exactly the visible set hydrates: the active space 10 has a split
    // (panes 101/102) → two sessions, never 504.
    assert_eq!(plan.sessions.len(), 2);
    assert_eq!(plan.deferred.len(), 502);
    assert_eq!(
        plan.sessions,
        vec![
            SessionRequest {
                tab: bbr_domain::TabId(101),
                profile: ProfileId(1),
                url: "https://b.example".into()
            },
            SessionRequest {
                tab: bbr_domain::TabId(102),
                profile: ProfileId(1),
                url: "https://c.example".into()
            },
        ]
    );
    assert_eq!(harness.engine.borrow().live_session_count(), 2);
    assert_eq!(snap.engine_sessions, 2);
    // The two hydrated panes are shown; nothing else was created.
    assert!(harness.recorded(&[
        EngineOp::Create {
            view: WebViewId(1),
            profile: ProfileId(1),
            url: "https://b.example".into()
        },
        EngineOp::Show {
            view: WebViewId(1),
            visible: true
        },
        EngineOp::Create {
            view: WebViewId(2),
            profile: ProfileId(1),
            url: "https://c.example".into()
        },
        EngineOp::Show {
            view: WebViewId(2),
            visible: true
        },
    ]));
    // Deferred tabs are session-less (Suspended).
    assert_eq!(
        snap.all_tabs
            .iter()
            .filter(|t| t.lifecycle == crate::TabLifecycle::Suspended)
            .count(),
        502
    );
}

#[test]
fn restored_single_pane_active_space_hydrates_exactly_one_session() {
    let intent = dense_intent();
    let harness = crate::fake::FakeEngine::new().into_harness();
    let (controller, plan) = BrowserController::restored(
        harness.engine.clone(),
        seq_ids(),
        &intent,
        bbr_domain::SpaceId(20),
    )
    .unwrap();
    // The dense space 20 is single-pane (active tab 1000): exactly one
    // session from 500 identities — the canonical warm-startup shape.
    assert_eq!(plan.sessions.len(), 1);
    assert_eq!(plan.deferred.len(), 503);
    assert_eq!(controller.snapshot().engine_sessions, 1);
    assert_eq!(controller.snapshot().active_view, Some(WebViewId(1)));
    // Session-less until activation: selecting a suspended tab recreates
    // exactly one new session (the old one is destroyed first).
    let mut controller = controller;
    controller.select_tab(1).unwrap();
    assert_eq!(
        controller.snapshot().engine_sessions,
        2,
        "deferred tab 1001 activated"
    );
}

#[test]
fn restore_failure_rolls_back_to_fresh_default() {
    let harness = crate::fake::FakeEngine::new().into_harness();
    // A duplicate identity fails validation before any op.
    let mut intent = dense_intent();
    intent.profiles[0].spaces[0].tabs[0].id = 10; // collides with the space id
    let error = BrowserController::restored(
        harness.engine.clone(),
        seq_ids(),
        &intent,
        bbr_domain::SpaceId(10),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        crate::HydrationError::DuplicateIdentity(10)
    ));
    assert_eq!(
        harness.engine.borrow().live_session_count(),
        0,
        "no leaked sessions"
    );
}

#[test]
fn restore_advances_the_allocator_past_restored_ids() {
    let intent = dense_intent();
    let harness = crate::fake::FakeEngine::new().into_harness();
    let (mut controller, _) = BrowserController::restored(
        harness.engine.clone(),
        seq_ids(),
        &intent,
        bbr_domain::SpaceId(20),
    )
    .unwrap();
    // Fresh profile/space/tab ids stay above every restored id.
    let profile = controller.create_profile("Fresh");
    let space = controller.create_space(profile, "Fresh Space").unwrap();
    controller.activate_space(space).unwrap();
    let event = controller.open_tab("https://fresh.example").unwrap();
    let _ = event;
    let snap = controller.snapshot();
    // The fresh tab id stays above every restored id (max restored 1499);
    // the restored identities keep their persisted ids.
    let fresh = snap
        .all_tabs
        .iter()
        .find(|t| t.url == "https://fresh.example")
        .expect("fresh tab");
    assert!(
        fresh.id.0 > 1499,
        "fresh id {} above restored max",
        fresh.id.0
    );
    assert!(
        snap.all_tabs.iter().any(|t| t.id.0 == 1000),
        "restored dense identity 1000 kept"
    );
}

#[test]
fn intent_export_round_trips_the_live_model() {
    let harness = crate::fake::FakeEngine::new().into_harness();
    let (controller, _) = BrowserController::restored(
        harness.engine.clone(),
        seq_ids(),
        &dense_intent(),
        bbr_domain::SpaceId(10),
    )
    .unwrap();
    // Export the live model and re-plan it: same intent order, same
    // session/deferred split (the persist → restore direction is one
    // shape). The live intent carries the hydrated tabs' urls.
    let exported = controller.intent();
    assert_eq!(exported, dense_intent());
    let plan = HydrationPolicy::plan(&exported, bbr_domain::SpaceId(10)).unwrap();
    assert_eq!(plan.sessions.len(), 2);
    assert_eq!(plan.deferred.len(), 502);
}
