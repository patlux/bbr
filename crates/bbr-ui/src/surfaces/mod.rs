//! BBR portal-surface presentations (DESIGN.md §11.4/§11.13/§13/§15) —
//! the portable presentation halves of the app's one-per-window
//! `NativeAboveWeb` portal surfaces. The app remains the sole surface
//! authority (controller state, dismissal/activation rules, focus
//! choreography, native geometry); each module here owns the card anatomy
//! only, as a plain model + render function per the anchor's
//! model/callback contract (G2 browser-component cutovers from the anchor's
//! extraction table).
//!
//! Current surface: the BBR page context menu ([`menu`]). The remaining
//! portal surfaces (command, find, auth, downloads, error) follow in their
//! own bounded cutover groups.
pub mod menu;
