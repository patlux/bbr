//! Contract tests for the portable new-window request/decision contract.
//!
//! These run on every platform and never touch WebKit: they pin the request
//! shape (source session, profile, URL, user-gesture state, disposition,
//! WebKit metadata) and the pure `decide` invariants every platform adapter
//! depends on (profile preservation, no invented navigation, no silent block).

use bbr_web::new_window::{
    decide, decision_for_intent, default_decision, peek_eligible, NewWindowDecision,
    NewWindowDisposition, NewWindowFeatures, NewWindowInput, NewWindowIntent, NewWindowOutcome,
    NewWindowRequest, NewWindowTarget,
};
use bbr_web::{ProfileId, WebViewId};

fn request(url: &str) -> NewWindowRequest {
    NewWindowRequest {
        source: WebViewId(7),
        source_profile: ProfileId(3),
        url: url.into(),
        user_initiated: true,
        target: NewWindowTarget::NewWindow,
        input: NewWindowInput::Primary,

        features: NewWindowFeatures::default(),
    }
}

#[test]
fn request_carries_source_session_and_profile() {
    let r = request("https://example.com");
    assert_eq!(r.source, WebViewId(7));
    assert_eq!(r.source_profile, ProfileId(3));
    assert_eq!(r.target, NewWindowTarget::NewWindow);
}

#[test]
fn request_carries_user_gesture_state() {
    assert!(request("https://example.com").user_initiated);
    let mut r = request("https://example.com");
    r.user_initiated = false;
    assert!(!r.user_initiated);
}

#[test]
fn default_policy_opens_peek_for_eligible_direct_activations_and_blocks_scripts() {
    let direct = request("https://example.com/direct");
    assert_eq!(default_decision(&direct), NewWindowDecision::Preview);

    let mut script = request("https://example.com/script");
    script.user_initiated = false;
    assert_eq!(default_decision(&script), NewWindowDecision::Block);
}

#[test]
fn peek_eligibility_requires_direct_activation() {
    let mut script = request("https://example.com");
    script.user_initiated = false;
    assert!(!peek_eligible(&script));
    assert!(peek_eligible(&request("https://example.com")));
}

#[test]
fn peek_eligibility_requires_a_previewable_scheme() {
    for (url, eligible) in [
        ("https://example.com/a", true),
        ("http://example.com/a", true),
        ("about:blank", true),
        ("mailto:hello@example.com", false),
        ("data:text/html,hi", false),
        ("file:///tmp/x.html", false),
        ("ftp://example.com/x", false),
        ("example.com/no-scheme", false),
    ] {
        assert_eq!(peek_eligible(&request(url)), eligible, "for {url}");
    }
}

#[test]
fn peek_eligibility_excludes_sized_and_tool_style_popups() {
    let mut sized = request("https://example.com");
    sized.features.width = Some(600.0);
    sized.features.height = Some(400.0);
    assert!(!peek_eligible(&sized), "explicitly sized popup");

    let mut tool = request("https://example.com");
    tool.features.toolbar_visible = Some(true);
    assert!(!peek_eligible(&tool), "tool-style popup");

    // Zero/unspecified sizes and explicitly-false chrome stay eligible:
    // only an explicit positive size or requested chrome marks a popup.
    let mut safe = request("https://example.com");
    safe.features.width = Some(0.0);
    safe.features.toolbar_visible = Some(false);
    assert!(peek_eligible(&safe));
}

#[test]
fn explicit_open_tab_intent_opens_a_regular_tab() {
    let direct = request("https://example.com/direct");
    assert_eq!(
        decision_for_intent(&direct, NewWindowIntent::OpenTab),
        NewWindowDecision::OpenTab
    );
    // An explicit OpenTab intent still never promotes a script request.
    let mut script = request("https://example.com/script");
    script.user_initiated = false;
    assert_eq!(
        decision_for_intent(&script, NewWindowIntent::OpenTab),
        NewWindowDecision::Block
    );
}

#[test]
fn explicit_peek_intent_never_forces_an_unsafe_preview() {
    // Eligible normal link + explicit Peek → Preview.
    assert_eq!(
        decision_for_intent(&request("https://example.com"), NewWindowIntent::Peek),
        NewWindowDecision::Preview
    );
    // Ineligible (non-web scheme) + explicit Peek → ordinary OpenTab, never
    // a forced preview and never a silent drop.
    assert_eq!(
        decision_for_intent(&request("mailto:hello@example.com"), NewWindowIntent::Peek),
        NewWindowDecision::OpenTab
    );
    // Ineligible (sized popup) + explicit Peek → ordinary OpenTab.
    let mut sized = request("https://example.com");
    sized.features.width = Some(600.0);
    assert_eq!(
        decision_for_intent(&sized, NewWindowIntent::Peek),
        NewWindowDecision::OpenTab
    );
    // Script + explicit Peek → reported Block.
    let mut script = request("https://example.com");
    script.user_initiated = false;
    assert_eq!(
        decision_for_intent(&script, NewWindowIntent::Peek),
        NewWindowDecision::Block
    );
}

#[test]
fn default_policy_falls_back_to_ordinary_open_tab_for_ineligible_direct_activation() {
    // A non-web scheme direct activation must never vanish: it opens a
    // regular tab rather than a preview.
    assert_eq!(
        default_decision(&request("mailto:hello@example.com")),
        NewWindowDecision::OpenTab
    );
    let mut sized = request("https://example.com");
    sized.features.width = Some(600.0);
    assert_eq!(default_decision(&sized), NewWindowDecision::OpenTab);
    let mut tool = request("https://example.com");
    tool.features.toolbar_visible = Some(true);
    assert_eq!(default_decision(&tool), NewWindowDecision::OpenTab);
}

#[test]
fn request_carries_webkit_feature_metadata_verbatim() {
    let mut r = request("https://example.com");
    r.features = NewWindowFeatures {
        x: Some(12.0),
        y: Some(34.0),
        width: Some(600.0),
        height: Some(400.0),
        menu_bar_visible: Some(false),
        status_bar_visible: Some(true),
        toolbar_visible: None,
        resizable: Some(false),
    };
    assert_eq!(r.features.x, Some(12.0));
    assert_eq!(r.features.y, Some(34.0));
    assert_eq!(r.features.width, Some(600.0));
    assert_eq!(r.features.height, Some(400.0));
    assert_eq!(r.features.menu_bar_visible, Some(false));
    assert_eq!(r.features.status_bar_visible, Some(true));
    assert_eq!(r.features.toolbar_visible, None);
    assert_eq!(r.features.resizable, Some(false));
}

#[test]
fn features_default_is_all_unspecified() {
    assert_eq!(
        NewWindowFeatures::default(),
        NewWindowFeatures {
            x: None,
            y: None,
            width: None,
            height: None,
            menu_bar_visible: None,
            status_bar_visible: None,
            toolbar_visible: None,
            resizable: None,
        }
    );
}

#[test]
fn decide_preview_creates_session_in_source_profile() {
    let r = request("https://example.com");
    let NewWindowOutcome::CreateSession { session } = decide(&r, NewWindowDecision::Preview) else {
        panic!("expected session");
    };
    assert_eq!(session.source_profile(), ProfileId(3));
    assert_eq!(session.disposition(), NewWindowDisposition::Preview);
    assert_eq!(session.url(), "https://example.com");
}

#[test]
fn decide_open_tab_creates_session_in_source_profile() {
    let r = request("https://example.com");
    let NewWindowOutcome::CreateSession { session } = decide(&r, NewWindowDecision::OpenTab) else {
        panic!("expected session");
    };
    assert_eq!(session.source_profile(), ProfileId(3));
    assert_eq!(session.disposition(), NewWindowDisposition::OpenTab);
    assert_eq!(session.url(), "https://example.com");
}

#[test]
fn decide_never_moves_a_request_across_profiles() {
    // The outcome profile is structurally the request's source_profile: a
    // decision carries no profile of its own and cannot relocate the request.
    for decision in [NewWindowDecision::Preview, NewWindowDecision::OpenTab] {
        let r = request("https://example.com");
        match decide(&r, decision) {
            NewWindowOutcome::CreateSession { session } => {
                assert_eq!(
                    session.source_profile(),
                    r.source_profile,
                    "for {decision:?}"
                );
                assert_eq!(session.url(), r.url, "for {decision:?}");
            }
            other => panic!("expected a session for {decision:?}, got {other:?}"),
        }
    }
}

#[test]
fn decide_does_not_normalize_the_url() {
    // A scheme-less URL passes through verbatim: no auto-navigation, no
    // scheme defaulting by the adapter or the contract.
    let r = request("example.com/foo");
    let NewWindowOutcome::CreateSession { session } = decide(&r, NewWindowDecision::OpenTab) else {
        panic!("expected session");
    };
    assert_eq!(session.source_profile(), ProfileId(3));
    assert_eq!(session.url(), "example.com/foo");
}

#[test]
fn decide_external_never_creates_a_session() {
    let r = request("mailto:hello@example.com");
    assert_eq!(
        decide(&r, NewWindowDecision::External),
        NewWindowOutcome::External {
            url: "mailto:hello@example.com".into(),
        }
    );
}

#[test]
fn decide_block_never_navigates_and_never_creates() {
    let r = request("https://example.com");
    assert_eq!(
        decide(&r, NewWindowDecision::Block),
        NewWindowOutcome::Blocked
    );
}

#[test]
fn only_session_naming_outcome_is_create_session() {
    // Structural guard: External and Block never carry a session, so only the
    // create path can ever touch a website data store.
    for decision in [NewWindowDecision::External, NewWindowDecision::Block] {
        assert!(
            !matches!(
                decide(&request("https://example.com"), decision),
                NewWindowOutcome::CreateSession { .. }
            ),
            "for {decision:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Background input disposition (background link tabs packet)
// ---------------------------------------------------------------------------

fn background_request(url: &str) -> NewWindowRequest {
    let mut r = request(url);
    r.input = NewWindowInput::Background;
    r
}

#[test]
fn background_input_routes_direct_activation_to_background_child() {
    // Command-primary / middle click on ANY direct link (even a peek-eligible
    // one) is the explicit user gesture for a background child: the default
    // policy must choose the background disposition over the preview set.
    let r = background_request("https://example.com/link");
    assert_eq!(default_decision(&r), NewWindowDecision::OpenBackgroundTab);
}

#[test]
fn background_input_never_fabricates_a_script_gesture() {
    // Input metadata is evidence, not authority: a script-created request
    // claiming a background modifier still blocks (the adapter's
    // user-activation report gates every disposition).
    let mut r = background_request("https://example.com/script");
    r.user_initiated = false;
    assert_eq!(default_decision(&r), NewWindowDecision::Block);
}

#[test]
fn background_decision_creates_one_background_session_in_source_profile() {
    let r = background_request("https://example.com/child");
    let outcome = decide(&r, default_decision(&r));
    let NewWindowOutcome::CreateSession { session } = outcome else {
        panic!("background disposition must create a session");
    };
    assert_eq!(session.source_profile(), ProfileId(3));
    assert_eq!(
        session.disposition(),
        NewWindowDisposition::OpenBackgroundTab
    );
    assert_eq!(session.url(), "https://example.com/child");
}

#[test]
fn background_input_never_crosses_profile_boundaries() {
    let mut r = background_request("https://example.com/child");
    r.source_profile = ProfileId(9);
    let NewWindowOutcome::CreateSession { session } =
        decide(&r, NewWindowDecision::OpenBackgroundTab)
    else {
        panic!("expected a session");
    };
    assert_eq!(session.source_profile(), ProfileId(9));
}

#[test]
fn explicit_intents_keep_their_plain_click_semantics() {
    // Explicit UI intents are authoritative plain-click actions: even with a
    // background modifier they must not silently change the user-visible
    // Peek/OpenTab behavior (the preview UI and command surfaces never send
    // a real Command-click).
    let r = background_request("https://example.com");
    assert_eq!(
        decision_for_intent(&r, NewWindowIntent::OpenTab),
        NewWindowDecision::OpenTab
    );
    assert_eq!(
        decision_for_intent(&r, NewWindowIntent::Peek),
        NewWindowDecision::Preview
    );
}

#[test]
fn background_input_does_not_change_plain_or_script_routing() {
    // The presence of the input field must not disturb the settled Primary
    // matrix: eligible direct → Preview, ineligible direct → OpenTab,
    // script → Block.
    let eligible = request("https://example.com");
    assert_eq!(default_decision(&eligible), NewWindowDecision::Preview);
    let mut ineligible = request("mailto:hello@example.com");
    assert_eq!(default_decision(&ineligible), NewWindowDecision::OpenTab);
    ineligible.user_initiated = false;
    assert_eq!(default_decision(&ineligible), NewWindowDecision::Block);
}

#[test]
fn primary_is_the_default_input() {
    assert_eq!(NewWindowInput::default(), NewWindowInput::Primary);
    let r = request("https://example.com");
    assert_eq!(r.input, NewWindowInput::Primary);
}
