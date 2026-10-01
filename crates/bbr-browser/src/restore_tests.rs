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
                                pinned_page: None,
                                closed: false,
                            },
                            IntentTab {
                                id: 101,
                                title: "One".into(),
                                url: "https://b.example".into(),
                                pinned: false,
                                pinned_page: None,
                                closed: false,
                            },
                            IntentTab {
                                id: 102,
                                title: "Two".into(),
                                url: "https://c.example".into(),
                                pinned: false,
                                pinned_page: None,
                                closed: false,
                            },
                        ],
                        active_tab: Some(101),
                        recently_closed: vec![IntentClosedTab {
                            tab_id: 900,
                            title: String::new(),
                            url: "https://closed.example".into(),
                            pinned: false,
                            former_position: 2,
                            in_place: false,
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
                            pinned_page: None,
                            closed: false,
                        }],
                        active_tab: Some(110),
                        recently_closed: vec![IntentClosedTab {
                            tab_id: 901,
                            title: String::new(),
                            url: "https://e.example".into(),
                            pinned: false,
                            former_position: 0,
                            in_place: false,
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
                            pinned_page: None,
                            closed: false,
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
fn activating_a_restored_space_hydrates_its_visible_set() {
    // Regression (S12): activating a space whose restored active tab is
    // session-less left it Suspended, so the app presented the empty-space
    // pane over a space that has tabs. Activation wakes exactly the
    // destination's visible set.
    let intent = dense_intent();
    let harness = crate::fake::FakeEngine::new().into_harness();
    let (mut controller, _plan) = BrowserController::restored(
        harness.engine.clone(),
        seq_ids(),
        &intent,
        bbr_domain::SpaceId(20),
    )
    .unwrap();
    assert_eq!(controller.snapshot().engine_sessions, 1);
    controller.activate_space(bbr_domain::SpaceId(11)).unwrap();
    let snap = controller.snapshot();
    assert_eq!(snap.engine_sessions, 2, "exactly the destination's tab 110");
    assert!(
        snap.active_view.is_some(),
        "the destination presents its tab"
    );
    let solo = snap
        .all_tabs
        .iter()
        .find(|t| t.id == bbr_domain::TabId(110))
        .expect("tab 110");
    assert_eq!(solo.lifecycle, crate::TabLifecycle::Active);
    // A split destination wakes both panes and nothing else.
    controller.activate_space(bbr_domain::SpaceId(10)).unwrap();
    let snap = controller.snapshot();
    assert_eq!(snap.engine_sessions, 4, "panes 101/102 added");
    let suspended = snap
        .all_tabs
        .iter()
        .filter(|t| t.lifecycle == crate::TabLifecycle::Suspended)
        .count();
    assert_eq!(
        suspended, 500,
        "the pinned 100 and 499 dense tabs stay deferred"
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
    // Blueprint D23: split groups persist beside the store (the two-pane
    // table cannot hold a tree), so the exported intent carries none; the
    // restored legacy split lives on as a group.
    // A restored pinned tab exports the page it is pinned at (its URL when
    // the intent named none).
    let mut expected = dense_intent();
    for profile in &mut expected.profiles {
        for space in &mut profile.spaces {
            space.split = None;
            for tab in space.tabs.iter_mut().filter(|tab| tab.pinned) {
                tab.pinned_page = Some(bbr_domain::PinnedPage {
                    url: tab.url.clone(),
                    title: tab.title.clone(),
                });
            }
        }
    }
    assert_eq!(exported, expected);
    assert_eq!(controller.split_groups().len(), 1);
    let plan = HydrationPolicy::plan(&dense_intent(), bbr_domain::SpaceId(10)).unwrap();
    assert_eq!(plan.sessions.len(), 2);
    assert_eq!(plan.deferred.len(), 502);
}

/// One space: a closed pinned row 100 (pinned at mail.example, closed on
/// inbox/42 — its in-place record), an open pinned row 101 navigated away
/// from its pinned page, and the selected regular tab 102.
fn kept_places_intent() -> RestoreIntent {
    let page = |url: &str, title: &str| {
        Some(bbr_domain::PinnedPage {
            url: url.into(),
            title: title.into(),
        })
    };
    RestoreIntent {
        profiles: vec![IntentProfile {
            id: 1,
            title: "Work".into(),
            hue: crate::ProfileHue::Cobalt,
            spaces: vec![IntentSpace {
                id: 10,
                title: "Main".into(),
                tabs: vec![
                    IntentTab {
                        id: 100,
                        title: "Mail".into(),
                        url: "https://mail.example".into(),
                        pinned: true,
                        pinned_page: page("https://mail.example", "Mail"),
                        closed: true,
                    },
                    IntentTab {
                        id: 101,
                        title: "Thread".into(),
                        url: "https://chat.example/thread".into(),
                        pinned: true,
                        pinned_page: page("https://chat.example", "Chat"),
                        closed: false,
                    },
                    IntentTab {
                        id: 102,
                        title: "News".into(),
                        url: "https://news.example".into(),
                        pinned: false,
                        pinned_page: None,
                        closed: false,
                    },
                ],
                active_tab: Some(102),
                recently_closed: vec![IntentClosedTab {
                    tab_id: 100,
                    title: "Re: lunch".into(),
                    url: "https://mail.example/inbox/42".into(),
                    pinned: true,
                    former_position: 0,
                    in_place: true,
                }],
                split: None,
            }],
        }],
    }
}

#[test]
fn restore_keeps_closed_pinned_rows_their_pages_and_in_place_records() {
    let harness = crate::fake::FakeEngine::new().into_harness();
    let (mut controller, plan) = BrowserController::restored(
        harness.engine.clone(),
        seq_ids(),
        &kept_places_intent(),
        bbr_domain::SpaceId(10),
    )
    .unwrap();
    // Only the selected regular tab hydrates; the closed row owns nothing.
    assert_eq!(plan.sessions.len(), 1);
    assert_eq!(harness.engine.borrow().live_session_count(), 1);
    assert_eq!(controller.intent(), kept_places_intent());
    let row = controller.browser.tab(bbr_domain::TabId(100)).unwrap();
    assert_eq!(row.lifecycle, bbr_domain::TabLifecycle::Closed);
    // ⇧⌘T after the relaunch brings back the page the row closed on.
    controller.reopen_tab().unwrap();
    let row = controller.browser.tab(bbr_domain::TabId(100)).unwrap();
    assert_eq!(row.lifecycle, bbr_domain::TabLifecycle::Active);
    assert_eq!(row.url, "https://mail.example/inbox/42");
    assert_eq!(harness.engine.borrow().live_session_count(), 2);
}

#[test]
fn restore_rejects_a_shown_closed_row_or_a_stray_in_place_record() {
    // A closed row cannot be the selection.
    let mut shown = kept_places_intent();
    shown.profiles[0].spaces[0].active_tab = Some(100);
    assert_eq!(
        HydrationPolicy::plan(&shown, bbr_domain::SpaceId(10)),
        Err(crate::HydrationError::InvalidActiveTab {
            space_id: 10,
            tab_id: 100
        })
    );
    // An in-place record must name a closed row.
    let mut stray = kept_places_intent();
    stray.profiles[0].spaces[0].recently_closed[0].tab_id = 101;
    assert_eq!(
        HydrationPolicy::plan(&stray, bbr_domain::SpaceId(10)),
        Err(crate::HydrationError::InvalidKeptPlace { tab_id: 101 })
    );
    // Only a pinned tab keeps a place.
    let mut regular = kept_places_intent();
    regular.profiles[0].spaces[0].tabs[2].closed = true;
    assert_eq!(
        HydrationPolicy::plan(&regular, bbr_domain::SpaceId(10)),
        Err(crate::HydrationError::InvalidKeptPlace { tab_id: 102 })
    );
}
