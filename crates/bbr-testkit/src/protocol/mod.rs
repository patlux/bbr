//! Protocol types shared by the gateway server and the E2E client.
//!
//! All types are plain data with `serde` derives so they round-trip through
//! JSON without leaking any framework type onto the wire. `protocol_version`
//! is part of the handshake; bump it when a type here changes in a breaking
//! way (see `docs/e2e-protocol.md` in E2E_ARCHITECTURE.md).
//!
//! The wire types are split by concern so no module outgrows the ~500-line
//! budget:
//!
//! - [`snapshot`] — the typed `Snapshot` and its row/space/profile parts
//!   plus the §15 visual-state vocabulary.
//! - [`benchmark`] — the benchmark kinds and presentation measurements.
//! - [`request`] — the gateway request enum.
//! - [`event`] — the semantic event stream.
//! - [`dsl`] — context-free request helpers for harness code.
//!
//! Everything callers reference keeps the single `bbr_testkit::protocol::*`
//! path through the re-exports below (additive protocol growth stays on the
//! same wire version).

use serde::{Deserialize, Serialize};

mod benchmark;
pub mod dsl;
mod event;
mod request;
mod snapshot;

pub use benchmark::{BenchmarkKind, PresentationMeasurement};
pub use event::{BrowserChange, Event};
pub use request::Request;
pub use snapshot::{
    DenseListSnapshot, LifecycleCounts, ProfileSnapshot, SemanticNode, SemanticRole, Snapshot,
    SpaceSnapshot, SpaceVisualState, TabSnapshot, TabVisualState,
};

/// Current protocol version understood by this crate.
pub const PROTOCOL_VERSION: u32 = 6;

/// Observable command-bar intent, independent from native focus ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandBarMode {
    Inactive,
    Location,
    NewTab,
    /// `Cmd-Shift-N`: stage a new space name in the active profile.
    NewSpace,
    /// `Cmd-Shift-M`: stage a move of the active tab into a destination
    /// space (typed by name); Enter commits, Escape cancels.
    MoveTab,
}

/// The shell's resolved §15 mode fields (theme/contrast/density).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ShellModes {
    pub theme: String,
    pub contrast: String,
    pub density: String,
}

/// Stable space identity on the portable test wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestSpaceId(pub u64);

/// Harness-assigned identity for one measured production-path mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchmarkOperationId(pub u64);

// ---------------------------------------------------------------------------
// Indirection layer: bbr-web already carries WebViewId/ProfileId/WebAction
// with serde support; re-export the ids used by harnesses so they don't
// need to reach into bbr-web themselves for constructing requests.
// ---------------------------------------------------------------------------

pub use bbr_web::{ProfileId as TestProfileId, WebViewId as TestWebViewId};

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

// The `Snapshot` payload grew with the §15 mode fields and per-row states;
// boxing it would churn the wire format for a lint that only measures enum
// size on the harness side.
#[allow(clippy::large_enum_variant)]
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
