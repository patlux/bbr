//! App-local reusable portal contract (task packet `native-portal-host`,
//! DESIGN.md §11.4/§13/§15).
//!
//! Ordinary chrome can escape local GPUI layout through a deferred GPUI host
//! ([`state::PortalHost::GpuiDeferred`], mirroring `gpui_component`'s
//! `deferred(anchored(...)).with_priority(...)`), while surfaces that cross
//! the native web boundary use the one implementation-owned native portal
//! host ([`state::PortalHost::NativeAboveWeb`]).
//!
//! - [`geometry`] — pure point-coordinate placement/clamping math (no GPUI,
//!   no AppKit): the eight-corner anchor model, edge-snap clamping, centered
//!   and anchored resolution, mirroring GPUI's `Anchored::prepaint`.
//! - [`state`] — stable `PortalId`/presentation identity, host policy,
//!   placement/anchor/priority/dismissal state, exactly one active
//!   presentation, and deterministic dismissal.
//! - [`probe`] — the production portal probe: one centered `NativeAboveWeb`
//!   presentation through the general Portal API (`bbr-portal-probe` /
//!   `-card` / `-backdrop`), the shared render state, and the portal
//!   window's GPUI root.
//! - [`native`] — the narrow app construction seam: exactly one transparent,
//!   non-activating portal window per browser window, created hidden and
//!   attached above the parent `NSWindow` by the bridge in `bbr-web-macos`.
//!
//! The auth sheet's card presentation moved to `bbr-ui::surfaces::auth`
//! (G2 Group 8); the probe builds the DTO and owns the geometry/backdrop.
//!
//! Consumers reference the submodules directly (`crate::portal::state::*`,
//! …); the app probe (`BbrApp::portal_present` / `portal_dismiss` /
//! `portal_outside_dismiss`) is the first production caller.

pub mod command;
pub mod context_menu;
pub mod geometry;
pub mod native;
pub mod peek;
pub mod probe;
pub mod state;
