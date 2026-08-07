//! Tests for typed shell snapshots and visible-id derivation.

use super::*;

/// Build a snapshot with two profiles and three spaces (two in the first
/// profile, one in the second) for the id-derivation tests.
fn space_snapshot() -> ControllerSnapshot {
    let space = |index: usize, id: u64, profile: u64, title: &str, active: bool, tabs: usize| {
        bbr_browser::SpaceSnapshot {
            index,
            id: bbr_browser::SpaceId(id),
            profile: bbr_web::ProfileId(profile),
            title: title.into(),
            active_tab: Some(0),
            active_view: None,
            active,
            tab_count: tabs,
        }
    };
    ControllerSnapshot {
        active_profile: bbr_web::ProfileId(1),
        active_profile_title: Some("Default".into()),
        active_profile_hue: Some(bbr_browser::ProfileHue::Cobalt),
        active_space: bbr_browser::SpaceId(2),
        tabs: Vec::new(),
        all_tabs: Vec::new(),
        profiles: vec![
            bbr_browser::ProfileSnapshot {
                index: 0,
                id: bbr_web::ProfileId(1),
                title: "Default".into(),
                hue: bbr_browser::ProfileHue::Cobalt,
            },
            bbr_browser::ProfileSnapshot {
                index: 1,
                id: bbr_web::ProfileId(3),
                title: "Private".into(),
                hue: bbr_browser::ProfileHue::Violet,
            },
        ],
        spaces: vec![
            space(0, 2, 1, "Default", true, 3),
            space(1, 4, 3, "Private", false, 1),
            space(2, 5, 1, "Reading", false, 0),
        ],
        active_index: Some(0),
        nav: Default::default(),
        web_view_hosted: false,
        engine_sessions: 0,
        active_view: None,
        lifecycle_counts: bbr_browser::LifecycleCounts::default(),
    }
}

/// A dense render state that covers every row (small-list default).
fn dense_all(len: usize) -> crate::dense::DenseRenderState {
    crate::dense::DenseRenderState {
        first_rendered: 0,
        rendered_rows: len,
        item_count: len,
        scroll_anchor: 0,
    }
}

/// No palette/overlay presentation open (the default surface state).
fn closed() -> crate::app::palette::CommandSurface {
    crate::app::palette::CommandSurface::None
}

#[test]
fn visible_ids_mirror_the_render_for_each_pane_branch() {
    let snap = |index: usize, crashed: bool| bbr_browser::TabSnapshot {
        index,
        url: "https://example.com".into(),
        title: format!("tab {index}"),
        pinned: false,
        loading: false,
        crashed,
        active: index == 0,
        web_view: None,
        lifecycle: bbr_browser::TabLifecycle::Warm,
        last_used: 0,
    };
    let tabs = vec![snap(0, false), snap(1, false)];
    let browser = ControllerSnapshot {
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
        web_view_hosted: false,
        engine_sessions: 0,
        active_view: None,
        lifecycle_counts: bbr_browser::LifecycleCounts::default(),
    };
    let ids = visible_ids(
        &browser,
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
        dense_all(tabs.len()),
        None,
    );
    for required in [
        "bbr-root",
        "bbr-sidebar",
        "bbr-tab-list",
        "bbr-tab-scroll",
        "bbr-tab-row-0",
        "bbr-tab-row-1",
        "bbr-tab-title-1",
        "bbr-tab-favicon-1",
        "bbr-tab-state-1",
        "bbr-tab-close-1",
        "bbr-tab-pin-1",
        "bbr-empty-new-tab",
    ] {
        assert!(
            ids.contains(&required.to_string()),
            "missing {required}: {ids:?}"
        );
    }
    assert!(!ids.iter().any(|id| id.starts_with("bbr-crash")));
    let mut crashed = tabs;
    crashed[0].crashed = true;
    let browser = ControllerSnapshot {
        active_profile: bbr_web::ProfileId(1),
        active_profile_title: Some("Default".into()),
        active_profile_hue: Some(bbr_browser::ProfileHue::Cobalt),
        active_space: bbr_browser::SpaceId(1),
        tabs: crashed,
        all_tabs: Vec::new(),
        profiles: Vec::new(),
        spaces: Vec::new(),
        active_index: Some(0),
        nav: Default::default(),
        web_view_hosted: false,
        engine_sessions: 0,
        active_view: None,
        lifecycle_counts: bbr_browser::LifecycleCounts::default(),
    };
    let ids = visible_ids(
        &browser,
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
        dense_all(1),
        None,
    );
    for required in ["bbr-crash-panel", "bbr-crash-reload", "bbr-crash-close"] {
        assert!(
            ids.contains(&required.to_string()),
            "missing {required}: {ids:?}"
        );
    }
    assert!(!ids.iter().any(|id| id.starts_with("bbr-empty")));

    // Active command bar in NewTab mode: the cursor id appears.
    let ids = visible_ids(
        &browser,
        true,
        crate::command_bar::CommandBarMode::NewTab,
        true,
        None,
        None,
        closed(),
        crate::app::command_rank::CommandResultState::Inactive,
        &[],
        &[],
        None,
        dense_all(1),
        None,
    );
    assert!(ids.contains(&"bbr-command-cursor".to_string()));
}

#[test]
fn space_row_chips_key_to_the_owning_profile_index() {
    // DESIGN.md §12/§15: the chip id follows the profile's creation-order
    // index, never the space index. In `space_snapshot` profile 0 owns
    // spaces 0 and 2 while profile 1 owns space 1, so space indices and
    // chip ids diverge. Regression: the render previously keyed the chip
    // to the space index, attaching `bbr-profile-chip-2` to a row the
    // contract identifies as `bbr-profile-chip-0`.
    let browser = space_snapshot();
    let ids = visible_ids(
        &browser,
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
        dense_all(0),
        None,
    );
    let chips = crate::shell::space::space_groups(&browser, usize::MAX)
        .into_iter()
        .flat_map(|group| group.rows)
        .map(|row| crate::components::profile_chip_id(row.profile_index).to_string())
        .collect::<Vec<_>>();
    // Render order: Default's spaces (indices 0, 2), then Private's space
    // (index 1). Both Default rows attach the same profile-keyed chip.
    assert_eq!(
        chips,
        vec![
            "bbr-profile-chip-0".to_string(),
            "bbr-profile-chip-0".to_string(),
            "bbr-profile-chip-1".to_string(),
        ]
    );
    for chip in &chips {
        assert!(
            ids.contains(chip),
            "rendered chip {chip} missing from the semantic snapshot"
        );
    }
    // The divergent space index 2 must never leak into a chip id.
    assert!(!ids.iter().any(|id| id == "bbr-profile-chip-2"));
}

#[test]
fn visible_ids_include_space_rows_and_profile_chips() {
    let browser = space_snapshot();
    let ids = visible_ids(
        &browser,
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
        dense_all(0),
        None,
    );
    for required in [
        "bbr-space-list",
        "bbr-space-new",
        "bbr-space-row-0",
        "bbr-space-row-1",
        "bbr-space-row-2",
        "bbr-space-title-0",
        "bbr-space-count-1",
        "bbr-profile-chip-0",
        "bbr-profile-chip-1",
    ] {
        assert!(
            ids.contains(&required.to_string()),
            "missing {required}: {ids:?}"
        );
    }
    // Sidebar hidden: space rows and chips disappear with the rail.
    let hidden = visible_ids(
        &browser,
        false,
        crate::command_bar::CommandBarMode::Inactive,
        false,
        None,
        None,
        closed(),
        crate::app::command_rank::CommandResultState::Inactive,
        &[],
        &[],
        None,
        dense_all(0),
        None,
    );
    assert!(!hidden.iter().any(|id| id.starts_with("bbr-space")));
}

#[test]
fn visible_ids_expose_the_cross_context_profile_hint_when_shown() {
    // §11.3/§15: the command-field cross-context transient exposes its
    // stable id only while the hint is shown (one interaction after a
    // cross-profile switch); at rest it is absent.
    let browser = space_snapshot();
    let with_hint = visible_ids(
        &browser,
        false,
        crate::command_bar::CommandBarMode::Inactive,
        true,
        None,
        Some("in Profile · Private"),
        closed(),
        crate::app::command_rank::CommandResultState::Inactive,
        &[],
        &[],
        None,
        dense_all(0),
        None,
    );
    assert!(with_hint.contains(&"bbr-command-profile-hint".to_string()));
    let without = visible_ids(
        &browser,
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
        dense_all(0),
        None,
    );
    assert!(!without.contains(&"bbr-command-profile-hint".to_string()));
}

#[test]
fn stale_dense_window_never_exposes_rows_the_model_removed() {
    // Regression (dense slice): closing the final tab into the empty pane
    // can leave the last frame's dense window non-empty (snapshots are
    // taken on demand, right after the mutation, before GPUI draws the
    // next frame). The empty space must not expose row ids from the
    // stale window — the exposed set is the rendered window intersected
    // with the current model.
    let mut browser = space_snapshot();
    browser.active_index = None;
    browser.active_view = None;
    browser.lifecycle_counts = bbr_browser::LifecycleCounts::default();
    let stale = crate::dense::DenseRenderState {
        first_rendered: 0,
        rendered_rows: 2,
        item_count: 2,
        scroll_anchor: 0,
    };
    let ids = visible_ids(
        &browser,
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
        stale,
        None,
    );
    assert!(
        !ids.iter().any(|id| id.starts_with("bbr-tab-row")),
        "a stale window must not expose rows the model removed: {ids:?}"
    );
    assert!(
        ids.contains(&"bbr-empty-new-tab".to_string()),
        "the empty pane replaces the rows: {ids:?}"
    );
}

mod u1;
