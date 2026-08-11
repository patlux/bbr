//! Browser surfaces (DESIGN.md §11/§13/§15) — model-driven presentation
//! functions taking plain DTOs plus typed callbacks. The app keeps
//! controllers, lifecycle, action policy, DTO construction, native Portal
//! hosting, credential provenance, and browser/session authority.
//!
//! Gallery stories drive these same production symbols with fixture models
//! and recorder callbacks, never gallery-local presentation copies.

pub mod auth;
pub mod downloads;
pub mod error;
pub mod find;
pub mod menu;

pub use auth::{
    auth_sheet, AuthField, AuthSheetModel, AUTH_CANCEL_ID, AUTH_HEADER_ID, AUTH_PASSWORD_ID,
    AUTH_SHEET_ID, AUTH_SUBMIT_ID, AUTH_USERNAME_ID,
};
pub use error::{error_surface, ErrorSurfaceModel};
pub use find::{count_label, find_bar, FindBarRect, FindSurface};
