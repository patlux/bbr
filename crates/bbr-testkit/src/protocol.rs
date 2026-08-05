//! Protocol types shared by the gateway server and the E2E client.
//!
//! All types are plain data with `serde` derives so they round-trip through
//! JSON without leaking any framework type onto the wire. `protocol_version`
//! is part of the handshake; bump it when a type here changes in a breaking
//! way (see `docs/e2e-protocol.md` in E2E_ARCHITECTURE.md).

use bbr_web::{ProfileId, WebAction, WebViewId};
use serde::{Deserialize, Serialize};

/// Current protocol version understood by this crate.
pub const PROTOCOL_VERSION: u32 = 1;

/// A snapshot of the whole app, returned on demand by `GetSnapshot`.
///
/// Deliberately typed (no free-form JSON) so the harness can assert on
/// fields without stringly-typed marshalling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    /// Active web view state (None until the first navigation event).
    pub url: Option<String>,
    pub title: Option<String>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    /// A live native web view is hosted in the chrome slot.
    pub web_view_hosted: bool,
    /// Engine implementation tag, e.g. "native-wkwebview" or "fake".
    pub engine_kind: String,
    /// GPUI chrome focus (command bar focused\).
    pub chrome_focused: bool,
    pub command_bar_text: String,
    pub sidebar_visible: bool,
    /// Window content size in points (diagnostics only).
    pub window_size: [f32; 2],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event {
    /// Fired once the window is open, the engine is attached and the
    /// gateway is accepting commands.
    AppReady { pid: u32, app_version: String },
    /// A navigation-state change reported by the native engine.
    Web(bbr_web::WebEvent),
    /// The GPUI chrome gained/lost keyboard focus.
    ChromeFocusChanged { chrome_focused: bool },
    /// The command bar's text changed.
    CommandBarChanged { text: String },
    /// The app received a quit request (graceful shutdown started).
    Quitting,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Request {
    /// Handshake: authenticate and exchange capabilities.
    Hello {
        protocol_version: u32,
        token: String,
        capabilities: Vec<String>,
    },
    /// Subscribe to the event stream; `since` replays all events with
    /// seq > `since` from the server ring (live delivery continues after).
    Attach { since: Option<u64> },
    /// Navigate the active web view to a URL.
    Navigate { url: String },
    /// Drive a [`WebAction`] on the active web view (Back/Forward/Reload/…).
    WebAction(WebAction),
    /// Focus the GPUI command bar (chrome) — the E2E reader of Cmd-L.
    ActivateCommandBar,
    /// Return keyboard focus to the web view — the E2E reader of Escape.
    FocusWeb,
    /// Set the command bar text directly (avoids per-keystroke round trips).
    SetCommandBarText { text: String },
    /// Dispatch a synthetic keystroke through GPUI's real input path
    /// (e.g. "cmd-l", "escape"). Zero permissions, zero OS event posting.
    DispatchKeystroke { keystroke: String },
    /// Full app snapshot.
    GetSnapshot,
    /// Liveness probe.
    Ping,
    /// Graceful app shutdown (`quit()`, window closes, process exits clean).
    Quit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Response {
    /// Reply to [`Request::Hello`].
    Welcome {
        protocol_version: u32,
        app_version: String,
        pid: u32,
        capabilities: Vec<String>,
        next_seq: u64,
    },
    /// Plain success.
    Ok,
    /// Reply to [`Request::Attach`].
    Replay {
        events: Vec<(u64, Event)>,
        next_seq: u64,
    },
    /// Reply to [`Request::GetSnapshot`].
    Snapshot(Snapshot),
    /// Failure carrying a human-readable reason.
    Error(Error),
}

/// Structured error payload. Kept as data (not `anyhow`) so the client can
/// match on it without importing the app's error types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Error {
    pub code: String,
    pub message: String,
}

impl Error {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for Error {}

// ---------------------------------------------------------------------------
// Indirection layer: bbr-web already carries WebViewId/ProfileId/WebAction
// with serde support; re-export the ids used by harnesses so they don't
// need to reach into bbr-web themselves for constructing requests.
// ---------------------------------------------------------------------------

pub use bbr_web::{ProfileId as TestProfileId, WebViewId as TestWebViewId};

/// Context-free request helpers, so harness code reads as a script.
pub mod dsl {
    use super::*;

    pub fn hello(token: &str, capabilities: &[&str]) -> Request {
        Request::Hello {
            protocol_version: PROTOCOL_VERSION,
            token: token.to_string(),
            capabilities: capabilities.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn navigate(url: impl Into<String>) -> Request {
        Request::Navigate { url: url.into() }
    }

    pub fn back() -> Request {
        Request::WebAction(WebAction::Back)
    }

    pub fn forward() -> Request {
        Request::WebAction(WebAction::Forward)
    }

    pub fn reload() -> Request {
        Request::WebAction(WebAction::Reload)
    }

    pub fn stop() -> Request {
        Request::WebAction(WebAction::Stop)
    }

    /// The canonical ids used by the Phase-1 spike (one profile, one view).
    pub fn test_ids() -> (ProfileId, WebViewId) {
        (ProfileId(1), WebViewId(1))
    }
}
