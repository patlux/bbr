//! Browser surfaces (DESIGN.md §11/§13/§15) — model-driven presentation
//! functions taking plain DTOs plus typed callbacks. The app keeps
//! controllers, lifecycle, action policy, DTO construction, native Portal
//! hosting, credential provenance, and browser/session authority.
//!
//! Gallery stories drive these same production symbols with fixture models
//! and recorder callbacks, never gallery-local presentation copies.

pub mod auth;
pub mod client_certificate;
pub mod command;
pub mod downloads;
pub mod error;
pub mod find;
pub mod floating;
pub mod history;
pub mod menu;
pub mod open_app;
pub mod peek_wash;
pub mod permissions;
pub mod settings;
pub mod space_chooser;
pub mod space_config;
pub mod space_delete;
pub mod tab_menu;
pub mod toast;
pub mod welcome;

pub use auth::{
    auth_sheet, AuthField, AuthSheetModel, AUTH_CANCEL_ID, AUTH_HEADER_ID, AUTH_PASSWORD_ID,
    AUTH_SHEET_ID, AUTH_SUBMIT_ID, AUTH_USERNAME_ID,
};
pub use command::{
    command_card, CommandModel, CommandPresentation, CommandRect, CommandResultRow,
    CommandResultState, OVERLAY_CURSOR_ID, OVERLAY_FIELD_ID, OVERLAY_ID, OVERLAY_STATE_EMPTY,
    OVERLAY_STATE_ERROR, OVERLAY_STATE_LOADING, OVERLAY_TEXT_ID, PALETTE_CURSOR_ID,
    PALETTE_FIELD_ID, PALETTE_ID, PALETTE_MODE_ID, PALETTE_RESULT_PREFIX, PALETTE_STATE_EMPTY,
    PALETTE_STATE_ERROR, PALETTE_STATE_LOADING, PALETTE_TEXT_ID,
};
pub use error::{error_surface, ErrorSurfaceModel};
pub use find::{count_label, find_bar, FindBarRect, FindSurface};
