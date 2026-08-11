//! The app-portal surface catalog (DESIGN.md §11.4/§13/§15) — plain
//! presentation models + callbacks for the surfaces the one `NativeAboveWeb`
//! portal window renders (extracted per the `ui-library-anchor` extraction
//! table, G2 Browser Components groups).
//!
//! Every surface is a plain function taking `(model: &Model, callbacks…)`
//! where `Model` is a `bbr-ui` DTO carrying the resolved [`crate::theme::Theme`]
//! and the callbacks are the same GPUI handler closures `bbr-app` passes
//! today. `bbr-ui` holds no portal/engine/app authority: the caller (the
//! app's portal probe) owns the geometry, the backdrop policy, the
//! lifecycle, and the `cx.listener` closures; it builds the DTO and wires
//! the callbacks. The gallery stories drive the same symbols with fixture
//! models and recorder callbacks — never a gallery-local reimplementation
//! of anatomy.
//!
//! This catalog mirrors the anchor table's browser-component surface rows:
//! command palette, find bar, error/crash, downloads, context menu, and the
//! auth sheet. Each row is extracted serially by its own G2 group; the auth
//! sheet ([`auth`]) is the first extracted surface (Group 8).

pub mod auth;

pub use auth::{
    auth_sheet, AuthField, AuthSheetModel, AUTH_CANCEL_ID, AUTH_HEADER_ID, AUTH_PASSWORD_ID,
    AUTH_SHEET_ID, AUTH_SUBMIT_ID, AUTH_USERNAME_ID,
};
