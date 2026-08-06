//! The testkit semantic snapshot and the §15 visible-id derivation.
//!
//! `TestSnapshot` is the app-local typed state the E2E protocol converts to
//! the wire `Snapshot`; `visible_ids` mirrors the current render so the id
//! assertions cannot drift from the shell. Both are testkit-only.

#[cfg(feature = "testkit")]
use bbr_browser::ControllerSnapshot;

#[cfg(feature = "testkit")]
#[derive(Debug, Clone, PartialEq)]
pub struct TestSnapshot {
    pub url: Option<String>,
    pub title: Option<String>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub web_view_hosted: bool,
    pub active_profile: bbr_web::ProfileId,
    pub active_profile_title: Option<String>,
    /// The active profile's identity hue name (DESIGN.md §12).
    pub active_profile_hue: Option<String>,
    pub active_space: bbr_testkit::protocol::TestSpaceId,
    pub tabs: Vec<bbr_browser::TabSnapshot>,
    /// Complete profile/space collections in creation order.
    pub profiles: Vec<bbr_browser::ProfileSnapshot>,
    pub spaces: Vec<bbr_browser::SpaceSnapshot>,
    pub active_tab_index: Option<usize>,
    pub active_web_view: Option<bbr_web::WebViewId>,
    pub engine_sessions: usize,
    pub engine_kind: &'static str,
    pub chrome_focused: bool,
    pub shell_modes: bbr_testkit::protocol::ShellModes,
    pub tab_states: Vec<bbr_testkit::protocol::TabVisualState>,
    pub row_titles: Vec<String>,
    /// The pinned/regular boundary (DESIGN.md §11.2): leading `pinned_count`
    /// tabs of the active space are pinned.
    pub pinned_count: usize,
    /// A command-bar destination hint (DESIGN.md §11.3), e.g. the
    /// destination profile of a staged move.
    pub command_bar_hint: Option<String>,
    /// The §11.3 cross-context transient: after a cross-profile space
    /// activation the destination profile appears at the top of the command
    /// field for one interaction ("in Profile · <title>"). `None` at rest;
    /// same-profile switching never sets it; the next chrome context
    /// mutation clears it.
    pub command_profile_hint: Option<String>,
    /// The dragged tab row's active-space index (§10.9). `None` when no drag
    /// is in flight; the dragged row renders `dragging`.
    pub drag_source: Option<usize>,
    /// The current drag target: the tab insertion slot or the targeted
    /// space's stable id, plus the drop's legality.
    pub drag_slot: Option<usize>,
    pub drag_space: Option<bbr_testkit::protocol::TestSpaceId>,
    pub drag_valid: bool,
    /// §11.3 space states in render order (selected for the active space).
    pub space_states: Vec<bbr_testkit::protocol::SpaceVisualState>,
    /// §11.3 space row titles in render order, after §4 truncation.
    pub space_titles: Vec<String>,
    pub visible_ids: Vec<String>,
    /// The typed semantic tree of the current render (DESIGN.md §15 /
    /// accessibility track): roles, names, values, enabled/selected/focused
    /// state, and hierarchy. Internal model — GPUI 0.2.2 has no platform
    /// accessibility layer, so these are not macOS Accessibility nodes; the
    /// platform bridge is Phase 9.
    pub semantics: Vec<bbr_testkit::protocol::SemanticNode>,
    pub command_bar_mode: bbr_testkit::protocol::CommandBarMode,
    pub command_bar_text: String,
    /// Lifecycle population counts of the active space (dense-list evidence).
    pub lifecycle_counts: bbr_testkit::protocol::LifecycleCounts,
    /// The dense-list render contract (§11.6): bounded rendered window,
    /// deterministic scroll anchor, active-row visibility.
    pub dense: bbr_testkit::protocol::DenseListSnapshot,
    pub sidebar_visible: bool,
    /// The effective sidebar rail width: the DESIGN §5 adjustable width
    /// clamped to the window (0 when collapsed — the rail is gone).
    pub sidebar_width: f32,
    /// The DESIGN §5 adjustable-range contract, exposed so E2E asserts the
    /// same constants the render enforces.
    pub sidebar_min: f32,
    pub sidebar_default: f32,
    pub sidebar_max: f32,
    /// The native-content floor (DESIGN §5/§13): the WKWebView slot is never
    /// laid out narrower, so the rail yields below the requested width in
    /// compact windows.
    pub content_min: f32,
    /// The resize divider drag state (idle vs dragging).
    pub divider_dragging: bool,
    /// The stable divider label (future Accessibility; §15 shared identity).
    pub divider_label: String,
    /// The last native slot rect handed to the engine (WKWebView frame).
    pub content_rect: [f32; 4],
    /// The authoritative pane layout (primary/secondary/focused ids), or
    /// `None` when no live pane is rendered.
    pub pane_layout: Option<bbr_browser::PaneLayout>,
    /// Keyed per-pane native geometry in pane-layout order (the split shared
    /// contract evidence: pane ids and their real WKWebView frames).
    pub pane_rects: Vec<bbr_testkit::protocol::PaneRectSnapshot>,
    pub window_size: [f32; 2],
}

/// The stable `bbr-*` ids present in the current shell render (DESIGN.md
/// §15). Derived from the same structures the renderer consumes — rows via
/// the row-visual indices, the content-pane branch mirroring `Render::render`
/// (native slot / crash panel / empty pane), the command cursor mirroring the
/// command-bar component — so the E2E id assertion cannot drift from the
/// shell: if a spec row's ids are not present here, the E2E fails.
#[cfg(feature = "testkit")]
#[allow(clippy::too_many_arguments)]
pub fn visible_ids(
    browser: &ControllerSnapshot,
    chrome_focused: bool,
    command_bar_mode: crate::command_bar::CommandBarMode,
    sidebar_visible: bool,
    command_bar_hint: Option<&str>,
    command_profile_hint: Option<&str>,
    drag_slot: Option<usize>,
    dense: crate::dense::DenseRenderState,
    pane_layout: Option<bbr_browser::PaneLayout>,
) -> Vec<String> {
    let mut ids: Vec<String> = vec!["bbr-root".into()];
    if sidebar_visible {
        for id in [
            "bbr-sidebar",
            "bbr-brand",
            "bbr-command-bar",
            "bbr-command-text",
            "bbr-status",
            "bbr-sidebar-toggle",
            "bbr-space-list",
            "bbr-space-new",
            "bbr-tab-list",
            "bbr-tab-scroll",
            "bbr-sidebar-divider",
        ] {
            ids.push(id.into());
        }
        if command_bar_hint.is_some() {
            ids.push("bbr-command-hint".into());
        }
        if command_profile_hint.is_some() {
            ids.push("bbr-command-profile-hint".into());
        }
        // The §11.2 section labels the render shows (hidden when empty).
        let total = browser.tabs.len();
        let pinned = crate::shell::pinned_count(&browser.tabs);
        if pinned > 0 {
            ids.push("bbr-section-pinned".into());
        }
        if total > pinned {
            ids.push("bbr-section-tabs".into());
        }
        for space in &browser.spaces {
            ids.push(format!("bbr-space-row-{}", space.index));
            ids.push(format!("bbr-space-title-{}", space.index));
            ids.push(format!("bbr-space-count-{}", space.index));
        }
        for profile in &browser.profiles {
            ids.push(crate::components::profile_chip_id(profile.index).to_string());
        }
    }
    if chrome_focused && command_bar_mode.is_active() {
        ids.push("bbr-command-cursor".into());
    }
    // The native content slots render whenever a pane layout exists (the
    // focused/active view hosts a live native session). Every rendered slot
    // id is keyed by its `WebViewId`; `visible_ids` must not advertise an
    // unkeyed element that GPUI did not render.
    if let Some(layout) = pane_layout {
        ids.push(format!("bbr-native-web-slot-{}", layout.primary.0));
        if let Some(secondary) = layout.secondary {
            ids.push(format!("bbr-native-web-slot-{}", secondary.0));
        }
    }
    // Rendered rows only: every pinned row (the fixed block) plus the
    // bounded regular window the shell actually built (§11.6) — a dense
    // collection never exposes ids for rows it did not render. The window is
    // the last frame's render state, which can lag the model by a frame
    // (snapshots are taken on demand, right after a mutation, before GPUI
    // draws the next frame), so the exposed rows are intersected with the
    // current model: a row the mutation just removed (e.g. the final close
    // into the empty pane) is never exposed by a stale window.
    let total = browser.tabs.len();
    let pinned = crate::shell::pinned_count(&browser.tabs);
    let mut rendered: Vec<usize> = (0..pinned.min(total)).collect();
    rendered.extend(
        (dense.first_rendered..dense.first_rendered + dense.rendered_rows)
            .filter(|&index| index < total),
    );
    for index in rendered {
        ids.push(format!("bbr-tab-row-{index}"));
        ids.push(format!("bbr-tab-title-{index}"));
        ids.push(format!("bbr-tab-favicon-{index}"));
        ids.push(format!("bbr-tab-state-{index}"));
        ids.push(format!("bbr-tab-close-{index}"));
        ids.push(format!("bbr-tab-pin-{index}"));
    }
    if let Some(slot) = drag_slot {
        ids.push(format!("bbr-drag-tick-{slot}"));
    }
    let active_tab = browser
        .active_index
        .and_then(|index| browser.tabs.get(index));
    if browser.active_view.is_none() {
        if active_tab.is_some_and(|tab| tab.crashed) {
            ids.extend(
                ["bbr-crash-panel", "bbr-crash-reload", "bbr-crash-close"].map(String::from),
            );
        } else {
            ids.extend(["bbr-empty-new-tab", "bbr-empty-new-tab-button"].map(String::from));
        }
    }
    ids
}

#[cfg(all(test, feature = "testkit"))]
mod tests {
    use super::*;

    /// Build a snapshot with two profiles and three spaces (two in the first
    /// profile, one in the second) for the id-derivation tests.
    fn space_snapshot() -> ControllerSnapshot {
        let space =
            |index: usize, id: u64, profile: u64, title: &str, active: bool, tabs: usize| {
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
}
