//! The semantic event stream the gateway broadcasts and replays.

use bbr_web::ProfileId;
use serde::{Deserialize, Serialize};

use super::{CommandBarMode, PresentationMeasurement, TestSpaceId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event {
    /// Fired once the window is open, the engine is attached and the
    /// gateway is accepting commands.
    AppReady { pid: u32, app_version: String },
    /// A navigation-state change reported by the native engine.
    Web(bbr_web::WebEvent),
    /// The GPUI chrome gained/lost keyboard focus.
    ChromeFocusChanged { chrome_focused: bool },
    /// The command bar's mode or text changed.
    CommandBarChanged { mode: CommandBarMode, text: String },
    /// One semantic browser-controller mutation completed.
    BrowserChanged { kind: BrowserChange },
    /// A startup or interaction mutation reached a semantically correct
    /// presentation frame inside the app process.
    PresentationCommitted(PresentationMeasurement),
    /// The app received a quit request (graceful shutdown started).
    Quitting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrowserChange {
    ProfileCreated {
        profile: ProfileId,
        space: TestSpaceId,
    },
    SpaceCreated {
        profile: ProfileId,
        space: TestSpaceId,
    },
    SpaceActivated {
        space: TestSpaceId,
    },
    /// Blueprint D05: a space moved within its profile group (`to` is its
    /// new position in the space order).
    SpaceMoved {
        space: TestSpaceId,
        to: usize,
    },
    /// Blueprint D05: a space was deleted; its tabs closed onto
    /// `fallback`'s recently-closed stack.
    SpaceDeleted {
        space: TestSpaceId,
        fallback: TestSpaceId,
        closed_tabs: usize,
        activated: bool,
    },
    TabOpened {
        index: usize,
    },
    NewWindowOpened {
        index: usize,
        space: TestSpaceId,
        profile: ProfileId,
        view: bbr_web::WebViewId,
    },
    TabSelected {
        index: usize,
    },
    TabClosed {
        index: usize,
    },
    TabPinned {
        index: usize,
    },
    TabUnpinned {
        index: usize,
    },
    TabReordered {
        from: usize,
        to: usize,
    },
    TabReopened {
        index: usize,
    },
    /// A two-pane split was opened over the two active-space tab indices.
    SplitOpened {
        primary: usize,
        secondary: usize,
    },
    /// The active space's split layout was closed (both tabs stay open).
    SplitClosed,
    TabMoved {
        to_space: TestSpaceId,
        recreated: bool,
    },
    TabSuspended {
        index: usize,
    },
    /// One committed logical content gesture dispatched exactly one real
    /// history action (Back/Forward) against the exact bound view — never a
    /// tab mutation. The gesture targets the pane under the pointer, which
    /// may differ from the focused command target in split mode.
    GestureHistoryRouted {
        view: bbr_web::WebViewId,
    },
    /// A committed logical Back gesture at a settled child-tab root closed
    /// the child through the normal close path and activated the exact
    /// opener: `child`/`opener` are the persistent `TabId`s, source is the
    /// child's Space/Profile, destination the opener's current
    /// Space/Profile, `closed_view` the destroyed child session,
    /// `presented_view` the opener session presented/focused, and
    /// `recreated` whether the opener's session had to be rebuilt
    /// (suspended/discarded lifecycle).
    OpenerReturn {
        child: u64,
        opener: u64,
        from_space: TestSpaceId,
        from_profile: ProfileId,
        to_space: TestSpaceId,
        to_profile: ProfileId,
        closed_view: bbr_web::WebViewId,
        presented_view: bbr_web::WebViewId,
        recreated: bool,
    },
}
