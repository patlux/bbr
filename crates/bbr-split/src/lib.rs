//! # bbr-split — portable two-pane split interaction
//!
//! The smallest portable layer that can describe the Phase-6 v1 split
//! invocation (see [Phase 6](../../docs/roadmap/phases/06-split-view.md) and
//! the [split-core task packet](../../docs/roadmap/tasks/split-core.md)):
//! exactly two ordered horizontal pane identities, a bounded normalized
//! divider position, a pure projection of a content rectangle into
//! primary/divider/secondary rects, and typed open/focus/close/divider
//! intents validated and evaluated **statelessly** against caller-supplied
//! authoritative values that follow the [browser interaction
//! contract](../../docs/product/browser-interactions.md) focus rules.
//!
//! ## Stateless authority model
//!
//! This crate owns **no stored pane, focus, or divider state**. The owning
//! `bbr-domain` split holds pane identities/order and the space active tab /
//! controller pane layout holds focus; this crate only validates requests and
//! evaluates them against the authoritative values the caller passes in:
//!
//! - [`plan::validate_open`] validates an open request into an ephemeral
//!   [`plan::OpenPlan`];
//! - [`plan::evaluate_focus`] evaluates a focus request against the caller's
//!   `primary`/`secondary`/`current` focus;
//! - [`plan::close_outcome`] produces the close outcome from the caller's
//!   authoritative values;
//! - [`plan::evaluate_resize`] evaluates a divider resize against only the
//!   caller-supplied current divider value.
//!
//! The crate is **portable with exactly one existing portable dependency**:
//! [`bbr_web`] for the established `f32` host-layout [`bbr_web::WebRect`]. No
//! GPUI, AppKit, WebKit, testkit, persistence, network, async, or `unsafe`.
//! Pane identity is a generic `T: Copy + Eq` so the later controller/app
//! composition can supply its own identity type (a `TabId`, `WebViewId`, or
//! plain value).
//!
//! ## Layout model
//!
//! - A split is exactly two horizontal panes ([`plan::SplitDirection`] has
//!   only `Horizontal`; there is no vertical behavior) with an ordered
//!   primary and secondary identity supplied by the caller.
//! - The divider position is a normalized fraction in `[0, 1]`
//!   ([`geometry::DividerPosition`]); non-finite requests are rejected and
//!   out-of-range finite values are clamped.
//! - [`geometry::project`] maps a content [`bbr_web::WebRect`] and divider
//!   width into three adjacent rects. The divider width is preserved exactly;
//!   invalid geometry (non-finite fields, non-positive dimensions, negative
//!   divider, overflowing endpoints, a width too small for two minimum
//!   panes plus the divider, or ordered `f32` boundaries that cannot advance
//!   representably at the content offset/scale) is rejected with a typed
//!   [`geometry::GeometryError`] and never silently rewritten.
//!
//! ## Contract invariants (proven by crate tests)
//!
//! 1. The two pane identities are never equal.
//! 2. The focused identity is always one of the two members: the pre-split
//!    selected pane stays focused on open and an outside identity cannot
//!    become split focus. A caller-supplied `current` focus that is itself
//!    outside the members is rejected before any outcome.
//! 3. Focusing an already-focused member is an explicit
//!    [`plan::FocusOutcome::Unchanged`]; only a real change reports
//!    [`plan::FocusOutcome::Changed`]. An invalid authoritative `current`
//!    never reports `Unchanged`.
//! 4. The divider position is always a bounded normalized value; an unchanged
//!    divider input reports `changed: false`.
//! 5. Every accepted projection keeps both panes strictly positive, preserves
//!    the divider width, keeps ordered adjacency with boundaries that advance
//!    representably, and lands the final endpoint on the validated content
//!    endpoint (`secondary.x + secondary.width ==
//!    content.x + content.width`) to `f32` rounding.
//!
//! The real-app split UI belongs to the later serial app-composition packet,
//! not this portable landing zone.
//!
//! # Example
//!
//! ```
//! use bbr_web::WebRect;
//! use bbr_split::geometry::{project, DividerPosition};
//! use bbr_split::plan::{
//!     evaluate_focus, evaluate_resize, validate_open, CloseOutcome, FocusIntent,
//!     FocusOutcome, OpenIntent, ResizeIntent, ResizeOutcome, SplitDirection,
//! };
//!
//! // Validate an open request into an ephemeral plan; the caller owns the
//! // authoritative pane identities, focus, and divider value.
//! let plan = validate_open(OpenIntent {
//!     primary: "left",
//!     secondary: "right",
//!     focused: "left",
//!     direction: SplitDirection::Horizontal,
//! })
//! .expect("distinct panes with a member focused");
//!
//! // Pure geometry projection from a content rect and divider width.
//! let panes = project(WebRect::new(0.0, 0.0, 100.0, 60.0), plan.initial_position, 8.0)
//!     .expect("valid geometry");
//! assert!(panes.primary.width > 0.0);
//! assert!(panes.secondary.width > 0.0);
//! assert_eq!(panes.divider.width, 8.0);
//!
//! // Focus evaluates against caller-supplied authoritative values.
//! assert_eq!(
//!     evaluate_focus("left", "right", "left", FocusIntent { pane: "right" }),
//!     FocusOutcome::Changed { focused: "right" },
//! );
//! assert_eq!(
//!     evaluate_focus("left", "right", "left", FocusIntent { pane: "left" }),
//!     FocusOutcome::Unchanged { focused: "left" },
//! );
//!
//! // Divider resize evaluates against only the current divider value.
//! let resized =
//!     evaluate_resize(plan.initial_position, ResizeIntent { position: 0.25 });
//! assert!(matches!(
//!     resized,
//!     ResizeOutcome::Resized { changed: true, .. },
//! ));
//!
//! // Closing keeps both identities and reports the focused survivor.
//! assert_eq!(
//!     CloseOutcome {
//!         primary: "left",
//!         secondary: "right",
//!         survivor: "left",
//!     },
//!     bbr_split::plan::close_outcome("left", "right", "left"),
//! );
//! ```

#![forbid(unsafe_code)]
#![deny(missing_debug_implementations)]

pub mod geometry;
pub mod plan;

pub use bbr_web::WebRect;
pub use geometry::{DividerPosition, GeometryError, Projection};
pub use plan::{
    close_outcome, evaluate_focus, evaluate_resize, validate_open, CloseOutcome, FocusIntent,
    FocusOutcome, OpenIntent, OpenPlan, OpenRejection, ResizeIntent, ResizeOutcome, SplitDirection,
};

#[cfg(test)]
mod tests_geometry;
#[cfg(test)]
mod tests_geometry_invalid;
#[cfg(test)]
mod tests_plan;
