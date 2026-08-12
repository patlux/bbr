//! BBR portable presentation crate — the single design-system authority
//! (DESIGN.md §2–§15).
//!
//! `bbr-ui` owns every BBR visual surface's shared foundation: the Chassis
//! design tokens ([`theme`] + [`palette`]), the §15 state vocabulary
//! ([`state`]), the §7 drawn icon catalog ([`icons`]), the plain text
//! helpers ([`text`]), and the honest repeated presentation primitives
//! ([`components`]). The app composition and the dev-only gallery import
//! the exact same production symbols — there are no copied or rebuilt
//! components and no second token authority anywhere.
//!
//! # Dependency boundary (audited, not assumed)
//!
//! The crate declares **exactly two direct normal dependencies** — `gpui`
//! (the pinned GPUI-CE source the workspace patches every `gpui` consumer
//! to) and `bbr-domain` for the single hue/appearance authority
//! ([`bbr_domain::ProfileHue`], [`bbr_domain::SpaceTheme`],
//! [`bbr_domain::SpaceId`]). Its only permitted first-party transitive edge
//! is the existing `bbr-domain → bbr-web` `ProfileId` path documented in
//! ARCHITECTURE.md; `bbr-ui` never declares, imports, or uses the web crate
//! itself and never drags in the browser controller, the app composition,
//! the testkit gateway, persistence, the Portal host, or any native
//! bridge. The
//! [`dependency_contract`] unit test re-derives the allowlist from this
//! crate's manifest; `scripts/check_ui_dependencies.py` (the proposed
//! `just ui-deps` gate) audits the direct set, the first-party closure, and
//! the forbidden-source-import guard from `cargo metadata`. It deliberately
//! does **not** flat-blacklist GPUI's own audited third-party platform
//! closure (raw-window-handle, objc2, serde, …).
//!
//! # Component contract
//!
//! Every component is a plain function taking `(model: &Model, theme:
//! &Theme, callbacks…)` where `Model` is a `bbr-ui` DTO (plain data,
//! `Clone`/`Debug`, never a GPUI entity, never a controller
//! snapshot) and the callbacks are the same GPUI handler closures the app
//! passes today. `bbr-ui` holds no browser/session/profile/space authority:
//! no controller, no snapshot type, no engine, no portal host, no
//! persistence, no testkit.
//!
//! # Cutover provenance
//!
//! Every symbol here was extracted from the app crate in the G1a
//! Foundations/Primitives cutover group (the packet
//! `docs/roadmap/tasks/ui-library-anchor.md` extraction table) and the old
//! definitions were deleted in the same coherent group — no compatibility
//! re-export, no duplicate, no second authority. Token names, `bbr-*` ids,
//! the §10/§15 state vocabulary, and the §15 snapshot semantics are
//! byte-identical to the pre-cutover shell.

#![forbid(unsafe_code)]

pub mod components;
pub mod icons;
pub mod palette;
pub mod sidebar;
pub mod state;
pub mod surfaces;
pub mod text;
pub mod theme;
pub mod titlebar;

#[cfg(test)]
mod dependency_contract;

// The exact module layout mirrors the extraction table:
// - `theme` + `theme::density` + `theme::space_tint` + `theme::elevation`
//   own the Chassis foundations (DESIGN.md §2–§8).
// - `palette` owns the color system and the WCAG contrast gate (§3).
// - `state` owns the §15 vocabulary (VisualState, DownloadVisual, the
//   permanent-strip chip DTOs).
// - `icons` owns the §7 drawn glyph catalog and its geometry.
// - `text` owns the §4/§11.5 pure string helpers (truncation, domains,
//   mid-ellipsis).
// - `surfaces` owns the higher-level browser surfaces (Downloads panel,
//   DESIGN.md §11.12) that take a plain model DTO plus a typed callback
//   bundle.
// - `components` owns the honest repeated primitives (§11).
// - `surfaces` owns the model-driven browser surfaces (§11): Downloads,
//   Find, Error, Auth, and BBR-owned Context Menu are integrated.
