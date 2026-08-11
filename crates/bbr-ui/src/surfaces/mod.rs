//! Browser surfaces (DESIGN.md §11/§15) — the higher-level `bbr-ui`
//! presentation surfaces that take a plain model DTO plus a typed callback
//! bundle (per the anchor's model/callback contract in
//! `docs/roadmap/tasks/ui-library-anchor.md`). Each surface owns the
//! anatomy only: the controller, lifecycle, DTO building, and native
//! adapters stay in `bbr-app`.

pub mod downloads;
