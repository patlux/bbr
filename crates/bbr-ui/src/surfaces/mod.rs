//! Browser surfaces (DESIGN.md §11/§15) — model-driven presentation
//! functions taking plain DTOs plus typed callbacks. The app keeps
//! controllers, lifecycle, action policy, DTO construction, native Portal
//! hosting, and browser/session authority.

pub mod downloads;
pub mod error;
pub mod find;

pub use error::{error_surface, ErrorSurfaceModel};
pub use find::{count_label, find_bar, FindBarRect, FindSurface};
