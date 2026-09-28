//! Real-app error-surface E2E (task packet `error-surface-contract`).
//!
//! Launches the real `bbr` app with real WKWebViews and proves the §11.8
//! error surface end to end over the typed testkit protocol:
//!
//! - native failure classification: a TLS handshake failure (HTTPS against
//!   the plaintext fixture server) presents the TLS surface, an unreachable
//!   host (a closed loopback port) presents the navigation surface, the
//!   redirect-loop fixture (an HTTP error condition WebKit actually fails)
//!   presents `Navigation/RedirectLoop`, and the truncated-500 fixture (a
//!   chunked response closed mid-chunk, which WebKit fails with
//!   `NSURLErrorNetworkConnectionLost`) presents the navigation surface —
//!   while a plain 500 with a delivered body renders the server page and
//!   never invents an incident;
//! - the offline class: the classified offline incident enters through the
//!   exact production `on_incident` seam (a genuinely-offline host cannot be
//!   toggled deterministically; the native-code → taxonomy mapping stays
//!   unit-proven in `bbr-web-macos::error_mapping`);
//! - recovery: the surface's primary action retries the failing URL through
//!   the production controller path — on the flaky fixture the retried
//!   navigation succeeds, the incident recovers, and the surface clears;
//!   on the redirect-loop fixture the retry fails again and a fresh
//!   incident supersedes the old one; Dismiss clears the surface;
//! - a renderer crash is recoverable: the §11.8 crash panel's Reload
//!   rebuilds the session;
//! - identity: showing, dismissing, retrying, or recovering never changes
//!   tab, session, profile, or space identity.
//!
//! No sleeps/pixels: every step synchronizes on typed snapshot state under a
//! deadline. The shared harness lives in `e2e_common`; scenario modules stay
//! near the ~500 line budget; `just e2e-shell-errors` gates the one
//! `e2e_shell_errors` target.

#![cfg(target_os = "macos")]

#[path = "../e2e_common/mod.rs"]
mod e2e_common;

mod illustrations;
mod recovery;
mod support;
mod surface;
