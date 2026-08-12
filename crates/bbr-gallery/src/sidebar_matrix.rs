//! The Sidebar composition matrix authority (packet
//! `component-gallery-compositions.md`): **one declaration source** for
//! every valid §11.4 rail family and every rejected combination. The
//! gallery story renders exactly the valid families through the production
//! `bbr_ui::sidebar::sidebar_surface` composition, and the fixture builder
//! refuses (`None`) any case the validity authority rejects — so the story,
//! the fixture builder, the matrix table, and the tests cannot drift.
//!
//! Matrix rows come straight from the packet's Sidebar row: compact
//! (narrow-window clamp → the 64 pt content floor) / wide (full rail
//! 280–400 pt); empty (no tabs) / dense (500 tabs, S density, windowed
//! render); long-title/missing-icon (truncation + monogram); cross-profile
//! (profile chips, command-profile-hint, space strip identity);
//! reduced-motion; contrast standard/increased; plus the committed
//! interaction boundaries (open/collapsed, plus menu, command
//! staging/results child, downloads panel child, hover/pressed/focus/drag/
//! key paths). Rejected: a sidebar with zero native-content floor (content
//! stays strictly positive, §5/§11.4); a collapsed rail showing chrome
//! content; two space strips.
//!
//! The plain [`bbr_ui::sidebar::SidebarModel`] construction lives in the
//! focused [`crate::sidebar_fixture`] module (a bounded split so this
//! authority stays under the ~500-line module budget); the fixture builds
//! the model only — every visual decision stays in `bbr-ui`/DESIGN, and
//! the dense window is pre-windowed there (the app keeps the windowing
//! authority — no second list authority).

use bbr_ui::theme::{ContrastMode, Density};

/// The §11.5 command-slot kind the family renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
    /// The §11.5 address pill at rest.
    Address,
    /// The `bbr-command-bar` staging input (New Space / Move Tab / the
    /// §11.8 empty New Tab surface).
    Staging,
}

/// One matrix row: a family label, the production model dimensions, and the
/// matrix-only claims that name the rejected combinations in the same case
/// shape.
#[derive(Debug, Clone, Copy)]
pub struct SidebarCase {
    pub label: &'static str,
    pub density: Density,
    pub contrast: ContrastMode,
    /// The requested rail width (the production clamp derives the effective
    /// width from the window).
    pub requested_width: f32,
    /// The window width the effective-width clamp runs against.
    pub window_width: f32,
    /// §11.4: the collapsed rail renders at width 0 (the app builds the
    /// rail but the root only adds it while visible).
    pub collapsed: bool,
    /// The pinned-region row count (a small, stable block above the scroll
    /// view, §11.2).
    pub pinned_count: usize,
    /// Regular-region tab count (the scrollable collection size).
    pub tab_count: usize,
    /// §11.10 favorites tile count.
    pub favorite_count: usize,
    /// §11.11 folder row count.
    pub folder_count: usize,
    /// Nested member rows of the expanded folder.
    pub folder_members: usize,
    /// §11.3 space count (one chip per space in the permanent strip).
    pub space_count: usize,
    /// §11.4 whether the plus menu is open.
    pub plus_menu: bool,
    /// The plus menu's keyboard selection index.
    pub plus_menu_selected: usize,
    /// §11.4 the `[Downloads]` `tnum` count while any active attempt
    /// exists.
    pub downloads_count: Option<usize>,
    /// The §11.5 command slot kind.
    pub command: CommandKind,
    /// §11.5: the in-rail command-results child slot is present.
    pub command_results: bool,
    /// §11.12: the anchored downloads-panel child slot is present.
    pub downloads_panel: bool,
    /// §10.2/§10.3: the hovered/pressed row (the row renders the hover
    /// wash / pressed wash + close reveal).
    pub hovered_row: Option<usize>,
    pub pressed_row: Option<usize>,
    /// §9: whether the tab list owns chrome focus (the ring).
    pub list_focused: bool,
    /// §9: whether the folders region owns chrome focus.
    pub folders_focused: bool,
    /// §9: whether the resize divider owns keyboard focus.
    pub divider_focused: bool,
    /// §10.9: the drag insertion tick's target slot (`None` at rest).
    pub drag_tick: Option<usize>,
    /// §10.9: whether the tick is legal.
    pub tick_valid: bool,
    /// §11.10: a tab drag is in flight and the empty favorites region
    /// renders as the compact drop band (label + §10.9 drop wash) so the
    /// first favorite can be created by dragging a concrete tab into the
    /// grid. With tiles the region renders normally; the band flag is only
    /// meaningful on the empty-grid family.
    pub favorites_band: bool,
    /// §4: long one-line titles (truncation).
    pub long_titles: bool,
    /// §7.2: rows without a favicon (monogram fallback).
    pub missing_icons: bool,
    /// §11.3: cross-profile (the address pill carries the profile hint and
    /// the strip shows distinct space identities).
    pub cross_profile: bool,
    /// §15: the reduced-motion mode field.
    pub reduced_motion: bool,
    // Matrix-only claims (the rejected combinations).
    pub claims_zero_content_floor: bool,
    pub claims_collapsed_chrome: bool,
    pub claims_two_strips: bool,
}

impl SidebarCase {
    /// The valid-family constructor: the production fields with the
    /// matrix-only flags defaulting to false.
    #[allow(clippy::too_many_arguments)]
    const fn new(
        label: &'static str,
        density: Density,
        requested_width: f32,
        window_width: f32,
        tab_count: usize,
    ) -> Self {
        SidebarCase {
            label,
            density,
            contrast: ContrastMode::Standard,
            requested_width,
            window_width,
            collapsed: false,
            pinned_count: 0,
            tab_count,
            favorite_count: 0,
            folder_count: 0,
            folder_members: 0,
            space_count: 1,
            plus_menu: false,
            plus_menu_selected: 0,
            downloads_count: None,
            command: CommandKind::Address,
            command_results: false,
            downloads_panel: false,
            hovered_row: None,
            pressed_row: None,
            list_focused: false,
            folders_focused: false,
            divider_focused: false,
            drag_tick: None,
            tick_valid: true,
            favorites_band: false,
            long_titles: false,
            missing_icons: false,
            cross_profile: false,
            reduced_motion: false,
            claims_zero_content_floor: false,
            claims_collapsed_chrome: false,
            claims_two_strips: false,
        }
    }
}

/// Every valid sidebar family — the single declaration the story renders.
/// The families cover the packet's variant rows (compact/wide, empty/dense,
/// long-title/missing-icon, cross-profile, reduced-motion, contrast) plus
/// the committed interaction boundaries (open/collapsed, plus menu, command
/// staging/results child, downloads panel child, hover/pressed/focus/drag/
/// key paths).
#[rustfmt::skip]
pub const SIDEBAR_VALID: &[SidebarCase] = &[
    // §11.4 full composition at N density: brand, address pill, favorites,
    // pinned + regular tabs, expanded folder, status, toggle, footer with
    // the permanent strip, divider.
    SidebarCase {
        pinned_count: 1,
        tab_count: 3,
        favorite_count: 3,
        folder_count: 1,
        folder_members: 2,
        space_count: 2,
        ..SidebarCase::new("N open rail — full composition", Density::Normal, 280.0, 900.0, 3)
    },
    // §11.6 dense 500-tab windowed rail at S density: the model carries a
    // bounded 64-row window plus spacers — the rail never lays out 500 rows.
    SidebarCase {
        pinned_count: 10,
        tab_count: 500,
        favorite_count: 2,
        folder_count: 1,
        space_count: 2,
        ..SidebarCase::new("S dense 500-tab windowed rail", Density::Small, 280.0, 900.0, 500)
    },
    // §5 compact window: the rail yields below the requested width so the
    // native content keeps exactly the 64 pt floor.
    SidebarCase {
        tab_count: 3,
        ..SidebarCase::new("compact window — content floor clamp", Density::Normal, 400.0, 300.0, 3)
    },
    // §5 wide rail at the max.
    SidebarCase {
        tab_count: 3,
        ..SidebarCase::new("wide rail — full 400 pt", Density::Normal, 400.0, 900.0, 3)
    },
    // §11.4 collapsed: the rail renders at width 0 (chrome laid out, not
    // visible — the app builds the rail but the root only adds it while
    // visible).
    SidebarCase {
        collapsed: true,
        tab_count: 3,
        ..SidebarCase::new("collapsed rail — width 0", Density::Normal, 0.0, 900.0, 3)
    },
    // §11.4 empty rail: no tabs/favorites/folders — the region labels hide.
    SidebarCase {
        ..SidebarCase::new("empty rail — no tabs/favorites/folders", Density::Normal, 280.0, 900.0, 0)
    },
    // §4 long-title + §7.2 missing-icon rows (truncation + monogram
    // fallback).
    SidebarCase {
        tab_count: 4,
        long_titles: true,
        missing_icons: true,
        ..SidebarCase::new("long-title + missing-icon rows", Density::Normal, 280.0, 900.0, 4)
    },
    // §11.3 cross-profile: the address pill carries the profile hint and
    // the strip shows two spaces with distinct identities.
    SidebarCase {
        tab_count: 2,
        space_count: 2,
        cross_profile: true,
        ..SidebarCase::new("cross-profile — profile hint + strip identity", Density::Normal, 280.0, 900.0, 2)
    },
    // §11.4 plus menu open with the keyboard selection on New Folder.
    SidebarCase {
        tab_count: 2,
        plus_menu: true,
        plus_menu_selected: 1,
        ..SidebarCase::new("plus menu open — selection 1", Density::Normal, 280.0, 900.0, 2)
    },
    // §11.5 command staging + the in-rail results child slot.
    SidebarCase {
        tab_count: 2,
        command: CommandKind::Staging,
        command_results: true,
        ..SidebarCase::new("command staging + results child", Density::Normal, 280.0, 900.0, 2)
    },
    // §11.12 downloads panel child + the running tnum count.
    SidebarCase {
        tab_count: 2,
        downloads_panel: true,
        downloads_count: Some(2),
        ..SidebarCase::new("downloads panel child + running count", Density::Normal, 280.0, 900.0, 2)
    },
    // §10 hover/pressed/focus/drag/key paths: a hovered+pressed row, the
    // focused tab list/folders/divider, and a legal drag tick.
    SidebarCase {
        tab_count: 4,
        folder_count: 1,
        hovered_row: Some(1),
        pressed_row: Some(1),
        list_focused: true,
        folders_focused: true,
        divider_focused: true,
        drag_tick: Some(2),
        tick_valid: true,
        ..SidebarCase::new("hover/pressed/focus/drag/key paths", Density::Normal, 280.0, 900.0, 4)
    },
    // §15 reduced-motion + increased contrast: the same full composition
    // under the mode fields the canvas can switch.
    SidebarCase {
        pinned_count: 1,
        tab_count: 3,
        favorite_count: 2,
        folder_count: 1,
        space_count: 2,
        contrast: ContrastMode::Increased,
        reduced_motion: true,
        ..SidebarCase::new("reduced-motion + increased contrast", Density::Normal, 280.0, 900.0, 3)
    },
    // §11.10 favorites drop band: a tab drag is in flight and the empty
    // grid renders as the compact drop band (label + §10.9 drop wash) so
    // the first favorite can be created by dragging a concrete tab into
    // the grid (daily-driver Sidebar slice).
    SidebarCase {
        tab_count: 2,
        favorites_band: true,
        ..SidebarCase::new("favorites drop band — empty grid during tab drag", Density::Normal, 280.0, 900.0, 2)
    },
];

/// A rejected combination with the exact DESIGN/packet reason (the matrix
/// "rejected" column). Same [`SidebarCase`] shape as the valid rows, so one
/// validity authority decides both sides and the table cannot drift.
#[derive(Debug, Clone, Copy)]
pub struct RejectedSidebar {
    pub label: &'static str,
    pub case: SidebarCase,
    pub reason: &'static str,
}

/// The matrix "rejected" column. Every entry must fail
/// [`sidebar_case_valid`] and thus refuse fixture construction.
#[rustfmt::skip]
pub const SIDEBAR_REJECTED: &[RejectedSidebar] = &[
    RejectedSidebar {
        label: "zero native-content floor",
        case: SidebarCase {
            claims_zero_content_floor: true,
            ..SidebarCase::new("zero native-content floor", Density::Normal, 400.0, 900.0, 3)
        },
        reason: "the §5/§11.4 geometry contract keeps the native content strictly positive — the effective width never exceeds window − CONTENT_MIN (64 pt); a case claiming the floor can be zero/negative refuses construction",
    },
    RejectedSidebar {
        label: "collapsed rail showing chrome content",
        case: SidebarCase {
            collapsed: true,
            requested_width: 280.0,
            claims_collapsed_chrome: true,
            ..SidebarCase::new("collapsed rail showing chrome content", Density::Normal, 280.0, 900.0, 3)
        },
        reason: "a collapsed rail renders at width 0 (the app builds the rail but the root only adds it while visible) — chrome content at a positive width while collapsed is the rejected combination",
    },
    RejectedSidebar {
        label: "two space strips",
        case: SidebarCase {
            claims_two_strips: true,
            ..SidebarCase::new("two space strips", Density::Normal, 280.0, 900.0, 3)
        },
        reason: "the §11.4 footer is exactly [Downloads] [one permanent strip] [+] — the model carries one StripModel; a second strip would duplicate the §11.3 identity authority",
    },
    RejectedSidebar {
        label: "favorites band flag on a non-empty grid",
        case: SidebarCase {
            favorite_count: 2,
            favorites_band: true,
            ..SidebarCase::new("favorites band flag on a non-empty grid", Density::Normal, 280.0, 900.0, 3)
        },
        reason: "the §11.10 drop band is the empty grid during a tab drag (the region renders its label + §10.9 wash so the first favorite can be created); with tiles the region renders the normal grid and the wash is the targeted flag — claiming the empty-band form on a non-empty grid is the rejected combination",
    },
];

/// The one validity authority: a case is valid only when no matrix-only
/// claim is set and the collapsed/geometry invariants hold. The fixture
/// builder refuses every invalid case.
pub fn sidebar_case_valid(case: &SidebarCase) -> bool {
    if case.claims_zero_content_floor || case.claims_collapsed_chrome || case.claims_two_strips {
        return false;
    }
    // §11.10: the drop band is the empty grid during a tab drag; with tiles
    // the region renders the normal grid (the wash is the targeted flag).
    if case.favorites_band && case.favorite_count > 0 {
        return false;
    }
    // A collapsed rail renders at width 0 — chrome content at a positive
    // width while collapsed is the rejected combination.
    if case.collapsed && case.requested_width > 0.0 {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sidebar_fixture::{build_sidebar_fixture, SidebarFixtureHandles, DENSE_WINDOW_ROWS};
    use bbr_ui::sidebar::geometry::SidebarGeometry;
    use gpui::ScrollHandle;

    /// Build the fixture handles through a test context (GPUI handles
    /// cannot be created without one).
    fn test_handles() -> SidebarFixtureHandles {
        let cx = gpui::TestAppContext::single();
        cx.update(|cx| SidebarFixtureHandles {
            tab_list: cx.focus_handle(),
            folders: cx.focus_handle(),
            plus_menu: cx.focus_handle(),
            space_strip: cx.focus_handle(),
            divider: cx.focus_handle(),
            downloads: cx.focus_handle(),
            dense_scroll: ScrollHandle::new(),
            space_strip_scroll: ScrollHandle::new(),
        })
    }

    #[test]
    fn every_valid_family_builds_a_model() {
        let handles = test_handles();
        for case in SIDEBAR_VALID {
            assert!(
                sidebar_case_valid(case),
                "valid family {} must pass the validity authority",
                case.label
            );
            let model = build_sidebar_fixture(case, &handles)
                .unwrap_or_else(|| panic!("valid family {} builds", case.label));
            // The effective width never exceeds the window minus the floor.
            if !case.collapsed {
                let effective =
                    SidebarGeometry::effective_width_for(case.requested_width, case.window_width);
                assert_eq!(model.sidebar_width, effective);
                assert!(
                    case.window_width - model.sidebar_width >= 0.0,
                    "content never negative"
                );
            } else {
                assert_eq!(model.sidebar_width, 0.0, "collapsed rail renders at 0");
            }
        }
    }

    #[test]
    fn every_rejected_family_refuses_construction() {
        let handles = test_handles();
        for rejected in SIDEBAR_REJECTED {
            assert!(
                !sidebar_case_valid(&rejected.case),
                "rejected sidebar {} must fail the validity authority",
                rejected.label
            );
            assert!(
                build_sidebar_fixture(&rejected.case, &handles).is_none(),
                "rejected sidebar {} refuses fixture construction",
                rejected.label
            );
        }
    }

    #[test]
    fn the_dense_family_renders_a_bounded_window() {
        let handles = test_handles();
        let dense = SIDEBAR_VALID
            .iter()
            .find(|c| c.tab_count == 500)
            .expect("dense family declared");
        let model = build_sidebar_fixture(dense, &handles).expect("dense family builds");
        assert_eq!(model.tabs.pinned.len(), 10);
        assert_eq!(model.tabs.window.len(), DENSE_WINDOW_ROWS);
        assert!(
            model.tabs.scroll_bottom > 0.0,
            "the spacer keeps the container height equal to the full collection"
        );
        // The window rows carry the global indices (pinned block first).
        assert_eq!(model.tabs.window[0].index, 10);
        assert_eq!(model.tabs.window[63].index, 73);
        // The model never carries 500 rows.
        assert!(model.tabs.pinned.len() + model.tabs.window.len() < 500);
    }

    #[test]
    fn the_compact_family_keeps_the_content_floor() {
        let handles = test_handles();
        let compact = SIDEBAR_VALID
            .iter()
            .find(|c| c.window_width == 300.0)
            .expect("compact family declared");
        let model = build_sidebar_fixture(compact, &handles).expect("compact family builds");
        // 400 requested in a 300 pt window yields 236 — the content keeps
        // exactly the 64 pt floor.
        assert_eq!(model.sidebar_width, 300.0 - 64.0);
        assert_eq!(300.0 - model.sidebar_width, 64.0);
    }
}
