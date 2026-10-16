//! The Sidebar composition fixture builder (packet
//! `component-gallery-compositions.md`): a bounded split of
//! `sidebar_matrix.rs` so the matrix authority stays under the ~500-line
//! module budget. This module owns the plain [`bbr_ui::sidebar::SidebarModel`]
//! construction for a matrix family — the fixture builds the model only;
//! every visual decision stays in `bbr-ui`/DESIGN, and the validity
//! authority stays in [`crate::sidebar_matrix::sidebar_case_valid`] (the
//! fixture refuses every case the matrix rejects).
//!
//! The dense window is **pre-windowed** (the fixture mirrors the app's
//! bounded window; the app keeps the windowing authority — no second list
//! authority): a 500-tab collection renders at most [`DENSE_WINDOW_ROWS`]
//! regular rows plus the spacers that keep the container height equal to
//! the full collection.

use bbr_domain::{FolderId, ProfileHue, SpaceId, SpaceTheme};
use bbr_ui::components::{
    AddressModel, AddressProfileHint, FavoriteTileSpec, FolderMemberVisual, FolderRowSpec,
    ProfileChipSpec, RowVisuals, StatusModel,
};
use bbr_ui::sidebar::geometry::SidebarGeometry;
use bbr_ui::sidebar::model::{
    CommandSlot, FavoritesModel, FoldersModel, FooterModel, PlusMenuModel, SidebarModel,
    StripModel, TabsModel,
};
use bbr_ui::state::{SpaceChipVisuals, TabVisualState};
#[cfg(test)]
use bbr_ui::theme::ThemeMode;
use bbr_ui::theme::{ContrastMode, MotionMode, Theme};
use gpui::{FocusHandle, ScrollHandle, SharedString};

use crate::sidebar_matrix::{sidebar_case_valid, CommandKind, SidebarCase};

/// The production dense-window bound the fixture mirrors (the app's
/// `crate::dense::DENSE_WINDOW_ROWS`; the app keeps the windowing
/// authority — the fixture only builds a pre-windowed model).
pub const DENSE_WINDOW_ROWS: usize = 64;

/// The fixture handles the model needs (GPUI handles cannot be created
/// without a context; the app state creates them once and shares them
/// across every family).
pub struct SidebarFixtureHandles {
    pub tab_list: FocusHandle,
    pub folders: FocusHandle,
    pub plus_menu: FocusHandle,
    pub space_strip: FocusHandle,
    pub divider: FocusHandle,
    pub downloads: FocusHandle,
    pub dense_scroll: ScrollHandle,
    pub space_strip_scroll: ScrollHandle,
}

impl SidebarFixtureHandles {
    /// An owned clone of the focus handles (the E2E focuses them to drive
    /// the production keyboard paths through real keystrokes).
    pub fn clone_handles(&self) -> SidebarFixtureHandles {
        SidebarFixtureHandles {
            tab_list: self.tab_list.clone(),
            folders: self.folders.clone(),
            plus_menu: self.plus_menu.clone(),
            space_strip: self.space_strip.clone(),
            divider: self.divider.clone(),
            downloads: self.downloads.clone(),
            dense_scroll: self.dense_scroll.clone(),
            space_strip_scroll: self.space_strip_scroll.clone(),
        }
    }
}

/// Family density and named accessibility requirements stay authoritative;
/// canvas appearance and stronger accessibility preferences are inherited.
pub fn fixture_theme(case: &SidebarCase, canvas: &Theme) -> Theme {
    Theme::new(
        canvas.mode,
        if case.contrast == ContrastMode::Increased {
            case.contrast
        } else {
            canvas.contrast
        },
        case.density,
    )
    .with_motion_mode(if case.reduced_motion {
        MotionMode::Reduced
    } else {
        canvas.motion_mode
    })
}

/// Build the plain §11.4 rail model for a matrix family. `None` for any
/// case the validity authority rejects. The model is plain data (the
/// handles travel in the fixture bundle); the rail renders it through the
/// production `bbr-ui` components.
#[cfg(test)]
pub fn build_sidebar_fixture(
    case: &SidebarCase,
    handles: &SidebarFixtureHandles,
) -> Option<SidebarModel> {
    if !sidebar_case_valid(case) {
        return None;
    }
    let theme = Theme::new(ThemeMode::Dark, case.contrast, case.density);
    build_sidebar_fixture_with_theme(case, handles, &theme)
}

/// Resolve the canvas appearance and family constraints consistently for both
/// model geometry/tint and the production Sidebar renderer.
pub fn build_sidebar_fixture_with_theme(
    case: &SidebarCase,
    handles: &SidebarFixtureHandles,
    theme: &Theme,
) -> Option<SidebarModel> {
    if !sidebar_case_valid(case) {
        return None;
    }
    let theme = fixture_theme(case, theme);
    let sidebar_width = if case.collapsed {
        0.0
    } else {
        SidebarGeometry::effective_width_for(case.requested_width, case.window_width)
    };
    let chrome_tint = theme.palette.surface_sidebar;
    let command = match case.command {
        CommandKind::Address => CommandSlot::Address(AddressModel {
            domain: SharedString::from(if case.cross_profile {
                "example.com"
            } else {
                "bbr.dev"
            }),
            hovered: false,
            profile_hint: case.cross_profile.then(|| AddressProfileHint {
                profile_index: 1,
                spec: ProfileChipSpec {
                    letter: "PR".into(),
                    hue: ProfileHue::Violet,
                },
                label: "Private".into(),
            }),
            zoom: case.zoom,
            progress: bbr_ui::components::AddressProgress::Idle,
            mark: None,
            site: Default::default(),
        }),
        CommandKind::Staging => CommandSlot::Staging(bbr_ui::components::StagingModel {
            focused: true,
            text: "New Space".into(),
            hint: Some("Name the space".into()),
            show_cursor: true,
            edit: bbr_ui::components::EditState::end_of("New Space"),
            line_sink: None,
            profile_hint: None,
        }),
    };
    let favorites = if case.favorite_count > 0 {
        Some(FavoritesModel {
            tiles: (0..case.favorite_count)
                .map(|i| FavoriteTileSpec {
                    index: i,
                    tab: (100 + i) as u64,
                    space: 1,
                    profile: 0,
                    title: format!("Favorite {}", i + 1),
                    url: format!("https://fav{i}.example"),
                    letter: (b'A' + i as u8) as char,
                    hue: i % 6,
                    image: None,
                    closed: false,
                    selected: false,
                })
                .collect(),
            targeted: false,
            offer: case.drag_tick.is_some(),
            pop: None,
        })
    } else if case.favorites_band {
        // §11.10: a tab drag is in flight and the empty grid renders as
        // the compact drop band (label + §10.9 drop wash) so the first
        // favorite can be created by dragging a concrete tab into the
        // grid (daily-driver Sidebar slice).
        Some(FavoritesModel {
            tiles: Vec::new(),
            targeted: true,
            offer: true,
            pop: None,
        })
    } else {
        None
    };
    let folders = if case.folder_count > 0 {
        Some(FoldersModel {
            rows: (0..case.folder_count)
                .map(|i| FolderRowSpec {
                    index: i,
                    id: FolderId(i as u64 + 1),
                    title: format!("Folder {}", i + 1),
                    count: case.folder_members,
                    expanded: case.folder_members > 0,
                    reveal_count: case.folders_focused,
                    rename: None,
                    drop_target: false,
                })
                .collect(),
            members: (0..case.folder_members)
                .map(|m| FolderMemberVisual {
                    folder: FolderId(1),
                    folder_index: 0,
                    member: m,
                    title: format!("Member {}", m + 1),
                    letter: (b'X' + m as u8) as char,
                    hue: m % 6,
                    image: None,
                    selected: false,
                    closed: false,
                })
                .collect(),
            focused: case.folders_focused,
        })
    } else {
        None
    };
    // §8 pitch: rows step on surface + gap (`density:pitch`), so the
    // fixture mirrors the app's scroll extent in exact pitch units — the
    // same unit the app's dense scroll math uses (§11.6).
    let pitch = theme.density.pitch();
    let pinned: Vec<RowVisuals> = (0..case.pinned_count)
        .map(|i| row_visual(case, i))
        .collect();
    // The bounded dense window: the fixture mirrors the app's windowing
    // (the app keeps the authority) — a 500-tab collection renders at most
    // DENSE_WINDOW_ROWS regular rows plus the spacers that keep the
    // container height equal to the full collection.
    let window_len = case.tab_count.min(DENSE_WINDOW_ROWS);
    let window: Vec<RowVisuals> = (0..window_len)
        .map(|i| row_visual(case, case.pinned_count + i))
        .collect();
    let scroll_top = 0.0;
    let scroll_bottom = (case.tab_count.saturating_sub(window_len)) as f32 * f32::from(pitch);
    let tabs = TabsModel {
        pinned,
        pinned_end: case.pinned_count,
        window,
        scroll_top,
        scroll_bottom,
        tick: case.drag_tick,
        tick_valid: case.tick_valid,
        // The fixture's drag is a regular row's reorder.
        tick_regular: case.drag_tick.is_some_and(|slot| slot >= case.pinned_count),
        list_focused: case.list_focused,
        // §11.2: an in-flight drag of an unpinned row offers the ghost row.
        pin_offer: case.drag_tick.is_some() || case.favorites_band,
        pin_offer_targeted: false,
    };
    let status = StatusModel {
        text: SharedString::from(if case.tab_count == 0 {
            "No page"
        } else {
            "bbr.dev"
        }),
    };
    // §11.14: the titlebar row (the rail's top strip) — the safe-area
    // geometry, the sidebar collapse state, and the genuine
    // Back/Forward/Reload availability of the focused pane. Plain
    // presentation data; the row renders it through the production
    // components.
    let titlebar = bbr_ui::titlebar::TitlebarModel {
        safe_area: bbr_ui::titlebar::SafeArea {
            height: case.safe_area_height,
            traffic_lights: case.traffic_lights,
        },
        sidebar_collapsed: case.titlebar_collapsed,
        can_go_back: case.nav_back,
        can_go_forward: case.nav_forward,
        can_reload: case.nav_reload,
        hovered: None,
        pressed: None,
    };
    let plus_menu = case.plus_menu.then_some(PlusMenuModel {
        selected: case.plus_menu_selected,
    });
    let footer = FooterModel {
        downloads_count: case.downloads_count,
        downloads_open: false,
        hovered: None,
        downloads_focused: false,
        downloads_focus: handles.downloads.clone(),
    };
    let strip = StripModel {
        chips: (0..case.space_count)
            .map(|i| SpaceChipVisuals {
                space: SpaceId(i as u64 + 1),
                title: if i == 0 {
                    "Default".into()
                } else {
                    "Private".into()
                },
                profile_title: if i == 0 {
                    "Default".into()
                } else {
                    "Private".into()
                },
                identity: if i == 0 { "code".into() } else { "lock".into() },
                theme: if i == 0 {
                    SpaceTheme::Cobalt
                } else {
                    SpaceTheme::Violet
                },
                intensity: bbr_domain::TintIntensity::Standard,
                selected: i == 0,
                drag_target: false,
                companion: None,
            })
            .collect(),
        swipe_tint: None,
        scroll: handles.space_strip_scroll.clone(),
        focus: handles.space_strip.clone(),
        focused: false,
    };
    Some(SidebarModel {
        sidebar_width,
        chrome_tint,
        background_in_parent: false,
        tint_fade: None,
        grain: false,
        floating: false,
        active_space_title: SharedString::from("Default"),
        command,
        favorites,
        folders,
        tabs,
        // The proxy is window-positioned; the rail story shows the in-list
        // `dragging` row instead.
        drag_proxy: None,
        drag_destination: None,
        status,
        status_visible: false,
        media: None,
        titlebar,
        plus_menu,
        footer,
        strip,
        divider_focused: case.divider_focused,
        tab_list_handle: handles.tab_list.clone(),
        dense_scroll: handles.dense_scroll.clone(),
        folders_handle: handles.folders.clone(),
        plus_menu_handle: handles.plus_menu.clone(),
        space_strip_handle: handles.space_strip.clone(),
        space_strip_scroll: handles.space_strip_scroll.clone(),
        divider_handle: handles.divider.clone(),
    })
}

/// One §11.1 row visual for the fixture: the deterministic index, the
/// §4-truncated title (long-title family), the §10 state (pressed → Active,
/// hovered → Hover, else Default — the same precedence the app's
/// `row_state` uses), the §7.2 monogram favicon (missing-icon family), and
/// the §10.2 close reveal.
fn row_visual(case: &SidebarCase, index: usize) -> RowVisuals {
    let title = if case.long_titles {
        format!("A very long tab title that must truncate at the row budget — tab {index}")
    } else {
        format!("Tab {index}")
    };
    let state = if case.pressed_row == Some(index) {
        TabVisualState::Active
    } else if case.hovered_row == Some(index) {
        TabVisualState::Hover
    } else {
        TabVisualState::Default
    };
    RowVisuals {
        index,
        title: SharedString::from(title),
        state,
        favicon: if case.missing_icons {
            None
        } else {
            Some(bbr_ui::components::FaviconSpec {
                image: None,
                letter: (b'a' + (index % 26) as u8) as char,
                hue: index % 6,
            })
        },
        pinned: index < case.pinned_count,
        closed: false,
        show_close: true,
        reveal_close: case.hovered_row == Some(index)
            || case.pressed_row == Some(index)
            || index == 0,
        split: None,
    }
}
