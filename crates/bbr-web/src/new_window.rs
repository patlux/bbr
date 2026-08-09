//! Portable contract for new-window requests (`target="_blank"` / `window.open`).
//!
//! This is the narrow seam Phase 8's popup/new-window policy builds on. The
//! macOS adapter reports a typed [`NewWindowRequest`] extracted from WebKit;
//! portable orchestration answers with a [`NewWindowDecision`]; [`decide`]
//! translates that decision into the deterministic [`NewWindowOutcome`] the
//! adapter must execute.
//!
//! Invariants this module owns (unit-proven by `tests/new_window.rs`):
//!
//! - **Policy is portable.** The adapter never decides and never rejects on
//!   its own; a missing handler deterministically falls back to
//!   [`NewWindowDecision::Block`].
//! - **Profile isolation.** A created session always uses the request's
//!   `source_profile`; this contract never moves a request across profile
//!   boundaries and never copies cookies or credentials.
//! - **No invented navigation.** The URL passes through verbatim (no scheme
//!   defaulting) and the adapter never loads it into the source view.
//! - **No silent loss.** A blocked request is reported through the adapter's
//!   blocked-request channel instead of vanishing.
//!
//! Opener/security semantics are preserved by construction: the only session
//! this contract ever asks for is a fresh browsing context in the source
//! profile's website data store, which is exactly the relationship an ordinary
//! browser-created tab would have — never a stronger opener link.

use crate::{ProfileId, WebViewId};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Window features a page requested via `window.open` (WebKit metadata).
///
/// WebKit reports each attribute as *nil* when the page did not specify it,
/// so every field is optional: numbers are `None` when unspecified and the
/// boolean fields keep WebKit's three-state semantics (`None` = unspecified,
/// `Some(true/false)` = explicitly requested). The adapter fills this from
/// `WKWindowFeatures`; portable policy reads it (for example to keep
/// explicitly sized/tooled popups out of transient previews).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NewWindowFeatures {
    /// Requested window x position, points.
    pub x: Option<f64>,
    /// Requested window y position, points.
    pub y: Option<f64>,
    /// Requested window width, points.
    pub width: Option<f64>,
    /// Requested window height, points.
    pub height: Option<f64>,
    /// Whether the menu bar should be visible.
    pub menu_bar_visible: Option<bool>,
    /// Whether the status bar should be visible.
    pub status_bar_visible: Option<bool>,
    /// Whether toolbars should be visible.
    pub toolbar_visible: Option<bool>,
    /// Whether the window should be resizable.
    pub resizable: Option<bool>,
}

/// The target disposition the page requested.
///
/// WebKit only invokes the new-window delegate method for a fresh browsing
/// context, so every request is a new window/tab today; the enum is the
/// explicit extension point for later dispositions (named windows, download
/// handling).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewWindowTarget {
    /// A new browsing context (`target="_blank"` or `window.open`).
    NewWindow,
}

/// A typed new-window request, reported by the adapter, decided by portable
/// orchestration.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct NewWindowRequest {
    /// The session that issued the request.
    pub source: WebViewId,
    /// The profile the source session belongs to. A created window MUST use
    /// the same website data store; requests never cross profile boundaries.
    pub source_profile: ProfileId,
    /// The target URL exactly as requested by the page — never normalized,
    /// never scheme-defaulted, and never loaded by the adapter into `source`.
    pub url: String,
    /// WebKit's user-gesture report: `true` for an activation (click or
    /// keyboard), `false` for a script-initiated request.
    pub user_initiated: bool,
    /// The requested target disposition.
    pub target: NewWindowTarget,
    /// WebKit window-features metadata (sizes/toolbars); `None` per attribute
    /// when the page did not specify it.
    pub features: NewWindowFeatures,
}

/// Host decision for a reported request.
///
/// Only portable orchestration produces a decision; platform adapters execute
/// the corresponding [`NewWindowOutcome`] and never decide on their own.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewWindowDecision {
    /// Present the request as a transient preview (Peek) in the source profile.
    Preview,
    /// Open the request as a regular tab in the source profile.
    OpenTab,
    /// Hand the URL to the OS default handler; never create a web session.
    External,
    /// Reject the request. The adapter must report the refusal — the user's
    /// action is never silently discarded.
    Block,
}

/// Disposition preserved when a new browsing session is created.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewWindowDisposition {
    /// Present the session as a transient preview (Peek).
    Preview,
    /// Present the session as a regular browser tab.
    OpenTab,
}

/// A validated session-creation request.
///
/// The fields are intentionally private: callers cannot construct a session
/// for an arbitrary profile or lose the preview/tab disposition at the native
/// boundary. Only [`decide`] can create this value, and the profile is copied
/// from the request's `source_profile` there.
#[derive(Debug, Clone, PartialEq)]
pub struct NewWindowSession {
    disposition: NewWindowDisposition,
    source_profile: ProfileId,
    url: String,
}

impl NewWindowSession {
    /// The presentation disposition selected by portable policy.
    pub fn disposition(&self) -> NewWindowDisposition {
        self.disposition
    }

    /// The profile data store that must own the new session.
    pub fn source_profile(&self) -> ProfileId {
        self.source_profile
    }

    /// The URL exactly as requested by the page.
    pub fn url(&self) -> &str {
        &self.url
    }

    fn from_request(request: &NewWindowRequest, disposition: NewWindowDisposition) -> Self {
        Self {
            disposition,
            source_profile: request.source_profile,
            url: request.url.clone(),
        }
    }
}

/// The deterministic native action an adapter must execute for a decision.
#[derive(Debug, Clone, PartialEq)]
pub enum NewWindowOutcome {
    /// Create a fresh browsing context in the session's source-profile store.
    /// The opaque [`NewWindowSession`] preserves both the policy disposition
    /// and the validated profile; the source view is untouched and WebKit's
    /// ordinary opener relationship is never strengthened.
    CreateSession { session: NewWindowSession },
    /// Hand `url` to the OS default browser; never create a web session.
    External { url: String },
    /// The request was refused; the caller must surface it so the user's
    /// action is not silent.
    Blocked,
}

/// Synchronous request → decision seam.
///
/// WebKit requires the new-window delegate method to answer in the same turn,
/// so the handler runs on the main thread while the delegate method executes.
/// Deliberately not `Send`: the engine and its delegates are main-thread-only.
pub type NewWindowHandler = dyn FnMut(NewWindowRequest) -> NewWindowDecision;

/// One fully decided request delivered from a platform adapter to the app.
///
/// The adapter publishes this plain value after portable policy ran; the app
/// executes it through the browser controller. Keeping request and outcome
/// together lets execution validate source identity/profile without asking the
/// adapter to create a native session.
#[derive(Debug, Clone, PartialEq)]
pub struct NewWindowDelivery {
    pub request: NewWindowRequest,
    pub outcome: NewWindowOutcome,
}

/// MVP default policy for the productive new-window seam.
///
/// A direct user activation opens one regular tab. Script-created requests are
/// blocked and reported. Peek, external handling, background selection,
/// insertion policy, and opener provenance remain later bounded decisions.
pub fn default_decision(request: &NewWindowRequest) -> NewWindowDecision {
    if request.user_initiated {
        NewWindowDecision::OpenTab
    } else {
        NewWindowDecision::Block
    }
}

/// Translate a host decision into the native outcome for `request`.
///
/// This is the single pure decision-translation every platform adapter shares,
/// so the invariants are proven once in the portable crate:
///
/// - `Preview` and `OpenTab` create a session in `request.source_profile` —
///   a decision can never move the request across profile boundaries.
/// - `External` and `Block` never produce a session; `CreateSession` is the
///   only outcome that names a profile.
/// - The URL passes through verbatim: no scheme defaulting, no `normalize_url`.
pub fn decide(request: &NewWindowRequest, decision: NewWindowDecision) -> NewWindowOutcome {
    match decision {
        NewWindowDecision::Preview => NewWindowOutcome::CreateSession {
            session: NewWindowSession::from_request(request, NewWindowDisposition::Preview),
        },
        NewWindowDecision::OpenTab => NewWindowOutcome::CreateSession {
            session: NewWindowSession::from_request(request, NewWindowDisposition::OpenTab),
        },
        NewWindowDecision::External => NewWindowOutcome::External {
            url: request.url.clone(),
        },
        NewWindowDecision::Block => NewWindowOutcome::Blocked,
    }
}
