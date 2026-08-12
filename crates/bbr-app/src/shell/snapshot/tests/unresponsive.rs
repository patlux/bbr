//! §11.8a unresponsive-pane visible-id regressions: while the active tab's
//! renderer is unresponsive the pane replaces exactly its native slot (the
//! slot it replaced is never advertised); a responsive view keeps its slot,
//! and in a split only the active member's slot is replaced. Split from
//! `tests.rs` into the `u1`/`u2`-style sibling module so the parent test
//! file stays under the ~500-line budget.

use super::*;

#[test]
fn visible_ids_mirror_the_unresponsive_pane_for_the_active_slot() {
    // §11.8a/§15: while the active tab's renderer is unresponsive the pane
    // replaces exactly its native slot (the slot it replaced is never
    // advertised); a responsive view keeps its slot, and in a split only
    // the active member's slot is replaced.
    let tab = |index: usize, unresponsive: bool| bbr_browser::TabSnapshot {
        index,
        url: "https://example.com".into(),
        title: format!("tab {index}"),
        pinned: false,
        loading: false,
        crashed: false,
        unresponsive,
        active: index == 0,
        web_view: Some(bbr_web::WebViewId(index as u64 + 7)),
        lifecycle: bbr_browser::TabLifecycle::Warm,
        last_used: 0,
        favicon: bbr_browser::TabFavicon::Monogram,
    };
    let browser = |tabs: Vec<bbr_browser::TabSnapshot>| ControllerSnapshot {
        active_profile: bbr_web::ProfileId(1),
        active_profile_title: Some("Default".into()),
        active_profile_hue: Some(bbr_browser::ProfileHue::Cobalt),
        active_space: bbr_browser::SpaceId(1),
        tabs: tabs.clone(),
        all_tabs: Vec::new(),
        profiles: Vec::new(),
        spaces: Vec::new(),
        active_index: Some(0),
        nav: Default::default(),
        web_view_hosted: true,
        engine_sessions: tabs.len(),
        active_view: tabs.first().and_then(|tab| tab.web_view),
        lifecycle_counts: bbr_browser::LifecycleCounts::default(),
        preview: None,
    };
    let ids_for = |browser: &ControllerSnapshot, layout: Option<bbr_browser::PaneLayout>| {
        visible_ids(
            browser,
            false,
            crate::command_bar::CommandBarMode::Inactive,
            true,
            None,
            None,
            closed(),
            crate::app::command_rank::CommandResultState::Inactive,
            &[],
            &[],
            None,
            None,
            dense_all(browser.tabs.len()),
            layout,
        )
    };
    let unresponsive_ids = [
        "bbr-unresponsive-panel",
        "bbr-unresponsive-wait",
        "bbr-unresponsive-reload",
        "bbr-unresponsive-close",
    ];

    // Single pane: the hung active tab's slot is replaced by the pane.
    let hung = browser(vec![tab(0, true)]);
    let ids = ids_for(
        &hung,
        Some(bbr_browser::PaneLayout {
            primary: bbr_web::WebViewId(7),
            secondary: None,
            focused: bbr_web::WebViewId(7),
        }),
    );
    for required in unresponsive_ids {
        assert!(
            ids.contains(&required.to_string()),
            "missing {required}: {ids:?}"
        );
    }
    assert!(
        !ids.iter().any(|id| id.starts_with("bbr-native-web-slot")),
        "the replaced slot is never advertised: {ids:?}"
    );
    assert!(!ids.iter().any(|id| id.starts_with("bbr-error")));

    // Responsive again: the slot id returns and the pane ids vanish.
    let alive = browser(vec![tab(0, false)]);
    let ids = ids_for(
        &alive,
        Some(bbr_browser::PaneLayout {
            primary: bbr_web::WebViewId(7),
            secondary: None,
            focused: bbr_web::WebViewId(7),
        }),
    );
    assert!(ids.contains(&"bbr-native-web-slot-7".to_string()));
    assert!(!ids.iter().any(|id| id.starts_with("bbr-unresponsive")));

    // Split, active member hung: only the active member's slot is replaced;
    // the responsive sibling keeps its slot.
    let hung_split = browser(vec![tab(0, true), tab(1, false)]);
    let ids = ids_for(
        &hung_split,
        Some(bbr_browser::PaneLayout {
            primary: bbr_web::WebViewId(7),
            secondary: Some(bbr_web::WebViewId(8)),
            focused: bbr_web::WebViewId(7),
        }),
    );
    assert!(ids.contains(&"bbr-unresponsive-panel".to_string()));
    assert!(ids.contains(&"bbr-native-web-slot-8".to_string()));
    assert!(!ids.contains(&"bbr-native-web-slot-7".to_string()));
    // The hung non-active sibling never replaces its own slot: the pane is
    // drawn only over the exact active view's slot (the sibling keeps its
    // native slot and stays warm).
    let sibling_hung = browser(vec![tab(0, false), tab(1, true)]);
    let ids = ids_for(
        &sibling_hung,
        Some(bbr_browser::PaneLayout {
            primary: bbr_web::WebViewId(7),
            secondary: Some(bbr_web::WebViewId(8)),
            focused: bbr_web::WebViewId(7),
        }),
    );
    assert!(ids.contains(&"bbr-native-web-slot-7".to_string()));
    assert!(ids.contains(&"bbr-native-web-slot-8".to_string()));
    assert!(!ids.iter().any(|id| id.starts_with("bbr-unresponsive")));
}
