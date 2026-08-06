//! The semantic event stream the gateway broadcasts and replays.

use bbr_web::{ProfileId, WebEvent};
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
    TabOpened {
        index: usize,
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
    TabReopened {
        index: usize,
    },
    TabMoved {
        to_space: TestSpaceId,
        recreated: bool,
    },
    TabSuspended {
        index: usize,
    },
}
