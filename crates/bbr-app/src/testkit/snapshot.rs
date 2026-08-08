//! Testkit wire conversions: controller outcomes → typed `BrowserChange`
//! events, and the app-local typed snapshot → the protocol `Snapshot`.
//!
//! Kept separate from the gateway plumbing so the two responsibilities stay
//! under their size budgets and the wire mapping is testable in one place.

use bbr_browser::ControllerEvent;
use bbr_testkit::protocol::{BrowserChange, GlobalTabSnapshot, Snapshot, TabSnapshot, TestSpaceId};

pub(crate) fn browser_change(outcome: &ControllerEvent) -> Option<BrowserChange> {
    match outcome {
        ControllerEvent::ProfileCreated { profile, space } => Some(BrowserChange::ProfileCreated {
            profile: *profile,
            space: bbr_testkit::protocol::TestSpaceId(space.0),
        }),
        ControllerEvent::SpaceCreated { profile, space } => Some(BrowserChange::SpaceCreated {
            profile: *profile,
            space: bbr_testkit::protocol::TestSpaceId(space.0),
        }),
        ControllerEvent::SpaceActivated { space } => Some(BrowserChange::SpaceActivated {
            space: bbr_testkit::protocol::TestSpaceId(space.0),
        }),
        ControllerEvent::TabOpened { index, .. } => {
            Some(BrowserChange::TabOpened { index: *index })
        }
        ControllerEvent::TabSelected { index } => {
            Some(BrowserChange::TabSelected { index: *index })
        }
        ControllerEvent::TabClosed { index } => Some(BrowserChange::TabClosed { index: *index }),
        ControllerEvent::TabPinned { index } => Some(BrowserChange::TabPinned { index: *index }),
        ControllerEvent::TabUnpinned { index } => {
            Some(BrowserChange::TabUnpinned { index: *index })
        }
        ControllerEvent::TabReordered { from, to } => Some(BrowserChange::TabReordered {
            from: *from,
            to: *to,
        }),
        ControllerEvent::TabReopened { index, .. } => {
            Some(BrowserChange::TabReopened { index: *index })
        }
        ControllerEvent::TabMoved {
            to_space,
            recreated,
            ..
        } => Some(BrowserChange::TabMoved {
            to_space: bbr_testkit::protocol::TestSpaceId(to_space.0),
            recreated: *recreated,
        }),
        ControllerEvent::TabSuspended { index } => {
            Some(BrowserChange::TabSuspended { index: *index })
        }
        ControllerEvent::SplitOpened { primary, secondary } => Some(BrowserChange::SplitOpened {
            primary: *primary,
            secondary: *secondary,
        }),
        ControllerEvent::SplitClosed => Some(BrowserChange::SplitClosed),
        _ => None,
    }
}

/// Conversion from the app's local snapshot type to the wire type.
pub(crate) trait SnapshotExt {
    fn to_protocol(&self) -> Snapshot;
}

impl SnapshotExt for crate::shell::TestSnapshot {
    fn to_protocol(&self) -> Snapshot {
        Snapshot {
            url: self.url.clone(),
            title: self.title.clone(),
            loading: self.loading,
            can_go_back: self.can_go_back,
            can_go_forward: self.can_go_forward,
            web_view_hosted: self.web_view_hosted,
            active_profile: self.active_profile,
            active_profile_title: self.active_profile_title.clone(),
            active_profile_hue: self.active_profile_hue.clone(),
            active_space: self.active_space,
            tabs: self
                .tabs
                .iter()
                .map(|tab| TabSnapshot {
                    index: tab.index,
                    url: tab.url.clone(),
                    title: tab.title.clone(),
                    pinned: tab.pinned,
                    loading: tab.loading,
                    crashed: tab.crashed,
                    active: tab.active,
                    web_view: tab.web_view,
                    lifecycle: match tab.lifecycle {
                        bbr_browser::TabLifecycle::Active => "active".into(),
                        bbr_browser::TabLifecycle::Warm => "warm".into(),
                        bbr_browser::TabLifecycle::Suspended => "suspended".into(),
                        bbr_browser::TabLifecycle::Discarded => "discarded".into(),
                    },
                    last_used: tab.last_used,
                })
                .collect(),
            profiles: self
                .profiles
                .iter()
                .map(|p| bbr_testkit::protocol::ProfileSnapshot {
                    index: p.index,
                    id: p.id,
                    title: p.title.clone(),
                    hue: p.hue.name().to_string(),
                })
                .collect(),
            spaces: self
                .spaces
                .iter()
                .map(|s| bbr_testkit::protocol::SpaceSnapshot {
                    index: s.index,
                    space: bbr_testkit::protocol::TestSpaceId(s.id.0),
                    profile: s.profile,
                    title: s.title.clone(),
                    active_tab: s.active_tab,
                    active_view: s.active_view,
                    active: s.active,
                    tab_count: s.tab_count,
                })
                .collect(),
            active_tab_index: self.active_tab_index,
            active_web_view: self.active_web_view,
            engine_sessions: self.engine_sessions,
            lifecycle_counts: self.lifecycle_counts,
            dense: self.dense,
            engine_kind: self.engine_kind.to_string(),
            chrome_focused: self.chrome_focused,
            shell_modes: self.shell_modes.clone(),
            tab_states: self.tab_states.clone(),
            row_titles: self.row_titles.clone(),
            pinned_count: self.pinned_count,
            command_bar_hint: self.command_bar_hint.clone(),
            command_profile_hint: self.command_profile_hint.clone(),
            drag_source: self.drag_source,
            drag_slot: self.drag_slot,
            drag_valid: self.drag_valid,
            space_chips: self.space_chips.clone(),
            visible_ids: self.visible_ids.clone(),
            semantics: self.semantics.clone(),
            command_bar_mode: self.command_bar_mode,
            command_bar_text: self.command_bar_text.clone(),
            address_domain: self.address_domain.clone(),
            address_hovered: self.address_hovered,
            address_rect: self.address_rect,
            command_surface: self.command_surface,
            command_result_state: self.command_result_state,
            command_results: self.command_results.clone(),
            command_result_total: self.command_result_total,
            command_cursor: self.command_cursor,
            command_selection: self.command_selection,
            all_tabs: self
                .all_tabs
                .iter()
                .map(|tab| GlobalTabSnapshot {
                    id: tab.id.0,
                    space: TestSpaceId(tab.space.0),
                    profile: tab.profile.0,
                    index: tab.index,
                    url: tab.url.clone(),
                    title: tab.title.clone(),
                    pinned: tab.pinned,
                    web_view: tab.web_view,
                    lifecycle: match tab.lifecycle {
                        bbr_browser::TabLifecycle::Active => "active".into(),
                        bbr_browser::TabLifecycle::Warm => "warm".into(),
                        bbr_browser::TabLifecycle::Suspended => "suspended".into(),
                        bbr_browser::TabLifecycle::Discarded => "discarded".into(),
                    },
                    last_used: tab.last_used,
                })
                .collect(),
            sidebar_visible: self.sidebar_visible,
            sidebar_width: self.sidebar_width,
            sidebar_min: self.sidebar_min,
            sidebar_default: self.sidebar_default,
            sidebar_max: self.sidebar_max,
            content_min: self.content_min,
            divider_dragging: self.divider_dragging,
            divider_label: self.divider_label.clone(),
            content_rect: self.content_rect,
            pane_layout: self
                .pane_layout
                .map(|layout| bbr_testkit::protocol::PaneLayoutSnapshot {
                    primary: layout.primary,
                    secondary: layout.secondary,
                    focused: layout.focused,
                }),
            pane_rects: self.pane_rects.clone(),
            window_size: self.window_size,
            window_content_size: self.window_content_size,
            portal: self.portal.clone(),
            find: self.find.clone(),
            zoom: self.zoom.clone(),
            context_menu: self.context_menu.clone(),
            auth: self.auth.clone(),
            plus_menu_open: self.plus_menu_open,
            plus_menu_selection: self.plus_menu_selection,
            favorites: self.favorites.clone(),
            folders: self.folders.clone(),
            download_rows: self.download_rows.clone(),
            downloads: self.downloads,
            #[cfg(all(target_os = "macos", feature = "testkit"))]
            web_content_probe: self.web_content_probe.clone(),
            #[cfg(not(all(target_os = "macos", feature = "testkit")))]
            web_content_probe: None,
            error_surface: self.error_surface.clone(),
            store: self.store.clone(),
            restore: self.restore,
        }
    }
}
