//! Testkit wire conversions: controller outcomes → typed `BrowserChange`
//! events, and the app-local typed snapshot → the protocol `Snapshot`.
//!
//! Kept separate from the gateway plumbing so the two responsibilities stay
//! under their size budgets and the wire mapping is testable in one place.

use bbr_browser::ControllerEvent;
use bbr_testkit::protocol::{BrowserChange, Snapshot, TabSnapshot};

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
            engine_kind: self.engine_kind.to_string(),
            chrome_focused: self.chrome_focused,
            shell_modes: self.shell_modes.clone(),
            tab_states: self.tab_states.clone(),
            row_titles: self.row_titles.clone(),
            space_states: self.space_states.clone(),
            space_titles: self.space_titles.clone(),
            visible_ids: self.visible_ids.clone(),
            command_bar_mode: self.command_bar_mode,
            command_bar_text: self.command_bar_text.clone(),
            sidebar_visible: self.sidebar_visible,
            sidebar_width: self.sidebar_width,
            sidebar_min: self.sidebar_min,
            sidebar_default: self.sidebar_default,
            sidebar_max: self.sidebar_max,
            content_min: self.content_min,
            divider_dragging: self.divider_dragging,
            divider_label: self.divider_label.clone(),
            content_rect: self.content_rect,
            window_size: self.window_size,
        }
    }
}
