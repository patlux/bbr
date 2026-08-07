//! Portable web-error taxonomy, incident lifecycle, and recovery-action
//! contracts.
//!
//! The bounded Phase-8 implementation is owned by
//! `docs/roadmap/tasks/error-surface-contract.md`.
//!
//! ## What lives here
//!
//! - [`identity`]: stable incident identity ([`IncidentId`] over
//!   `ProfileId`/`WebViewId` plus a per-web-view sequence), privacy-safe
//!   display context, the assembled [`Incident`], and the pure definition
//!   validation that rejects contradictory category/reason pairs.
//! - [`taxonomy`]: the closed error categories/subreasons for TLS,
//!   navigation, offline, and process crash, plus typed severity and
//!   retryability.
//! - [`recovery`]: the closed recovery-action set ([`RecoveryAction`]) and
//!   the category-legal action matrix ([`legal_actions_for_category`],
//!   [`validate_action`]).
//! - [`lifecycle`]: the pure incident lifecycle ([`LifecycleState`],
//!   [`LifecycleEvent`], [`apply_event`]) from reported → superseded /
//!   recovery requested → recovered/failed / dismissed, plus the
//!   per-web-view [`Lifecycle`] tracker that rejects duplicate, stale, and
//!   mismatched reports and callbacks. Recovery requests are validated
//!   against the incident-derived requestable set (category-legal actions
//!   minus `Dismiss`).
//!
//! ## Adapter mapping boundary
//!
//! A native adapter maps an engine failure into a `(category, subreason)`
//! candidate and feeds it through [`validate_definition`] before an incident
//! exists. This module deliberately contains **no native error codes** (no
//! `NSError`/WKError codes, no `CFNetwork` constants) and **no user-facing
//! copy**: [`Display`] strings are short technical diagnostics for logs and
//! contract tests, not UI text. Translating a native code into a category
//! still happens in the adapter; the contract only validates the result and
//! derives the legal actions and lifecycle transitions.
//!
//! Security boundary: there is no generic TLS bypass anywhere in this
//! contract. No category exposes a bypass/downgrade action, and the closed
//! action set cannot express one.

pub mod identity;
pub mod lifecycle;
pub mod recovery;
pub mod taxonomy;

#[cfg(test)]
mod tests_lifecycle;
#[cfg(test)]
mod tests_lifecycle_table;
#[cfg(test)]
mod tests_recovery;
#[cfg(test)]
mod tests_taxonomy;

pub use identity::{validate_definition, DefinitionError, Incident, IncidentContext, IncidentId};
pub use lifecycle::{
    apply_event, lifecycle_requestable_actions, Lifecycle, LifecycleError, LifecycleEvent,
    LifecycleState,
};
pub use recovery::{
    legal_actions, legal_actions_for_category, validate_action, ActionRejection, RecoveryAction,
};
pub use taxonomy::{
    CrashReason, IncidentCategory, IncidentSubReason, NavigationReason, OfflineReason,
    Retryability, SeverityLevel, TlsReason,
};
