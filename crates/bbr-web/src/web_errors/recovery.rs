//! Closed recovery-action taxonomy and the category-legal action matrix.
//!
//! This module is the second half of the Phase-8 web-error contract: it fixes
//! **what a user or controller may do** about an incident. The actions are a
//! closed set ([`RecoveryAction`]); the matrix ([`legal_actions_for_category`])
//! derives which actions each category may expose, and [`validate_action`]
//! rejects everything else.
//!
//! ## The legal-action matrix
//!
//! ```text
//!                Retry Reload GoBack OpenSettings InspectCertificate Restore Dismiss
//! Tls              ✓     ✗     ✓        ✗               ✓              ✗       ✓
//! Navigation       ✓     ✓     ✓        ✗               ✗              ✗       ✓
//! Offline          ✓     ✓     ✓        ✓               ✗              ✗       ✓
//! ProcessCrash     ✗     ✗     ✗        ✗               ✗              ✓       ✓
//! ```
//!
//! Notes:
//!
//! - A process crash is **not** an ordinary navigation retry: neither
//!   `Retry` nor `Reload` is exposed; the only recovery path is `Restore`
//!   (recreate the web session) or `Dismiss`.
//! - `OpenSettings` exists only for the offline category (network settings).
//! - `InspectCertificate` exists only for TLS.
//! - **TLS bypass is not an action.** There is no variant that would bypass,
//!   downgrade, or ignore a certificate failure, for any category, and no
//!   future adapter can derive one from this closed set.
//!
//! Actions are derived from the validated taxonomy, never from arbitrary UI
//! input: the surface (or controller) asks [`legal_actions`] and must route
//! through [`validate_action`] before executing.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::fmt;

use super::taxonomy::{IncidentCategory, IncidentSubReason};

/// Closed set of recovery actions for a web-error incident.
///
/// Deliberately absent: any action that would bypass or downgrade TLS. Such
/// an action is not part of the taxonomy and cannot be derived for any
/// category.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RecoveryAction {
    /// Re-attempt the failed navigation to the same URL.
    Retry,
    /// Reload the current page content.
    Reload,
    /// Navigate back to the previous page.
    GoBack,
    /// Open the relevant system settings (network settings for offline).
    OpenSettings,
    /// Present the certificate details of the failing origin (TLS only).
    InspectCertificate,
    /// Recreate the web process/session after a crash (crash only).
    Restore,
    /// Dismiss the incident surface.
    Dismiss,
}

impl RecoveryAction {
    /// All closed variants in canonical (enum) order. The matrix and tests
    /// iterate this list, so a new variant must extend both the matrix and
    /// the exhaustive tests.
    pub const ALL: [RecoveryAction; 7] = [
        RecoveryAction::Retry,
        RecoveryAction::Reload,
        RecoveryAction::GoBack,
        RecoveryAction::OpenSettings,
        RecoveryAction::InspectCertificate,
        RecoveryAction::Restore,
        RecoveryAction::Dismiss,
    ];
}

impl fmt::Display for RecoveryAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Retry => "retry",
            Self::Reload => "reload",
            Self::GoBack => "go back",
            Self::OpenSettings => "open settings",
            Self::InspectCertificate => "inspect certificate",
            Self::Restore => "restore",
            Self::Dismiss => "dismiss",
        };
        f.write_str(text)
    }
}

/// Why [`validate_action`] rejected a recovery action for a category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionRejection {
    NotLegalForCategory {
        action: RecoveryAction,
        category: IncidentCategory,
    },
}

impl fmt::Display for ActionRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotLegalForCategory { action, category } => {
                write!(f, "action {action} is not legal for category {category}")
            }
        }
    }
}

/// The closed category-legal action matrix. Returns the actions in canonical
/// [`RecoveryAction::ALL`] order, filtered by category.
pub fn legal_actions_for_category(category: IncidentCategory) -> Vec<RecoveryAction> {
    use IncidentCategory::{Navigation, Offline, ProcessCrash, Tls};
    use RecoveryAction::{
        Dismiss, GoBack, InspectCertificate, OpenSettings, Reload, Restore, Retry,
    };

    match category {
        Tls => vec![Retry, GoBack, InspectCertificate, Dismiss],
        Navigation => vec![Retry, Reload, GoBack, Dismiss],
        Offline => vec![Retry, Reload, GoBack, OpenSettings, Dismiss],
        ProcessCrash => vec![Restore, Dismiss],
    }
}

/// Shorthand over [`legal_actions_for_category`] for a subreason.
pub fn legal_actions(reason: &IncidentSubReason) -> Vec<RecoveryAction> {
    legal_actions_for_category(reason.category())
}

/// Pure validation: only category-legal actions pass. Everything else is
/// rejected with the exact offending action and category.
pub fn validate_action(
    category: IncidentCategory,
    action: RecoveryAction,
) -> Result<(), ActionRejection> {
    if legal_actions_for_category(category).contains(&action) {
        Ok(())
    } else {
        Err(ActionRejection::NotLegalForCategory { action, category })
    }
}

/// Whether `action` is legal for `category`.
pub fn is_legal(category: IncidentCategory, action: RecoveryAction) -> bool {
    validate_action(category, action).is_ok()
}
