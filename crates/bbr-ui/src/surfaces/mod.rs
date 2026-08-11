//! Full-surface presentation components (DESIGN.md §11) — the classified
//! web-error surface ([`error`]). Each surface is a plain function taking a
//! presentation DTO + callbacks; the app keeps the controller, lifecycle,
//! and DTO building. The glass panes (crash / empty / unresponsive) live in
//! [`crate::components::pane`]; this module owns the surfaces that carry
//! model-driven copy.

pub mod error;

pub use error::{error_surface, ErrorSurfaceModel};
