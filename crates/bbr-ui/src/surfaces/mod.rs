//! Browser surfaces (DESIGN.md §11/§15) — higher-level `bbr-ui`
//! presentation surfaces that take plain model DTOs plus typed callback
//! bundles. Each surface owns anatomy only: controller, lifecycle, DTO
//! building, native Portal hosting, and browser/session policy stay in
//! `bbr-app`.

pub mod downloads;
pub mod find;

pub use find::{count_label, find_bar, FindBarRect, FindSurface};
