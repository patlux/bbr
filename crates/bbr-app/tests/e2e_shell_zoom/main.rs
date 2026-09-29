//! Real-app per-site zoom E2E (queue item 11, part A).
//!
//! Launches the real `bbr` app with real WKWebViews and proves the zoom
//! wiring end to end through the typed testkit protocol (task packet
//! `site-settings-zoom-contract` evidence section):
//!
//! - `Cmd-Plus` / `Cmd-Minus` / `Cmd-0` (the real GPUI key bindings)
//!   drive the merged contract's integer-percent ladder (25% steps,
//!   25..=400) on the active web view, applied natively (`pageZoom`);
//! - the ladder clamps deterministically at both ends and `Cmd-0` resets
//!   to the documented default and removes the override record;
//! - per-site persistence: a navigation within the same origin keeps the
//!   level, a navigation to a different origin uses its own stored level,
//!   and returning restores the stored level — all through the portable
//!   `SiteZoomLifecycle` + `SiteSettingsStore` port;
//! - zooming never changes tab, session, `WebViewId`, profile, space, or
//!   session identity, and never composites or samples web content;
//! - switching between tabs on different sites re-applies each site's
//!   level to the freshly shown view.
//!
//! No sleeps/pixels: every step synchronizes on typed snapshot state or
//! semantic events under a deadline. The suite is split into focused
//! scenario modules (`ladder`, `per_site`) so every file stays near the
//! ~500 line budget; `just e2e-shell-zoom` gates the one `e2e_shell_zoom`
//! target.

#![cfg(target_os = "macos")]

#[path = "../e2e_common/mod.rs"]
mod e2e_common;

mod accessory;
mod identity;
mod ladder;
mod overlays;
mod per_site;
mod support;
