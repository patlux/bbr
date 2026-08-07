//! Incident identity, privacy-safe context, the assembled incident, and
//! definition validation.
//!
//! This module is the identity half of the Phase-8 web-error contract: it
//! fixes **which incident** a report belongs to and **what display context**
//! it may carry. The closed categories/subreasons and the severity/
//! retryability derivations live in [`super::taxonomy`]; the recovery-action
//! matrix lives in [`super::recovery`].
//!
//! ## Stable identity
//!
//! [`IncidentId`] is scoped to the owning [`ProfileId`] and [`WebViewId`] —
//! the same pair that keys live web sessions — and disambiguated by a
//! per-web-view `sequence` counter. Successive incidents on the same web
//! view carry increasing sequences, so a stale event holding an older id can
//! never mutate a newer incident (the lifecycle step rejects it).
//!
//! ## Privacy-safe context
//!
//! [`IncidentContext`] carries only the failing origin (`scheme://host[:port]`,
//! lower-cased, userinfo stripped) or nothing — never the path, query,
//! fragment, cookies, or page content. `IncidentContext::from_url` derives
//! that reduced origin from any URL by construction. Authority-less scheme
//! URLs (`javascript:…`, `mailto:…`, `data:…`) are reduced to their
//! lower-cased scheme — never their payload — and the well-known blank page
//! `about:blank` is kept verbatim.
//!
//! ## Assembled incident and definition validation
//!
//! [`Incident`] bundles id, category, subreason, and context. `category` is
//! stored for a stable wire shape and must equal the subreason's category;
//! [`Incident::new`] enforces that through [`validate_definition`]. Derived
//! behavior (severity, retryability, legal actions) always reads the
//! subreason, never the stored category copy, so a tampered deserialized
//! payload cannot stretch the taxonomy.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::fmt;

use crate::{ProfileId, WebViewId};

use super::recovery::{is_legal, legal_actions, RecoveryAction};
use super::taxonomy::{IncidentCategory, IncidentSubReason, Retryability, SeverityLevel};

/// Stable identity of one web-error incident.
///
/// Identity is scoped to the owning [`ProfileId`] and [`WebViewId`] — the
/// same pair that keys live web sessions — and disambiguated by a per-web-view
/// `sequence` counter. Successive incidents on the same web view carry
/// increasing sequences, so a stale event holding an older id can never
/// mutate a newer incident (the lifecycle step rejects it).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IncidentId {
    pub profile: ProfileId,
    pub web_view: WebViewId,
    /// Monotonic per-web-view discriminator; the controller allocates
    /// increasing values for successive incidents on the same web view.
    pub sequence: u64,
}

impl IncidentId {
    pub fn new(profile: ProfileId, web_view: WebViewId, sequence: u64) -> Self {
        Self {
            profile,
            web_view,
            sequence,
        }
    }
}

impl fmt::Display for IncidentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "profile {} web-view {} incident {}",
            self.profile.0, self.web_view.0, self.sequence
        )
    }
}

/// Carries only the failing **origin** (`scheme://host[:port]`, lower-cased,
/// any URL userinfo stripped) — never the path, query, fragment, cookies, or
/// page content. Authority-less scheme URLs (`javascript:…`, `mailto:…`,
/// `data:…`) are reduced to their lower-cased scheme, and the blank page
/// `about:blank` stays verbatim. `None` means the origin is unknown (for
/// example a process crash before a committed navigation).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct IncidentContext {
    origin: Option<String>,
}

impl IncidentContext {
    /// A reduced origin, or `None` when the origin is unknown. The value is
    /// trimmed; empty input becomes `None`.
    pub fn new(origin: Option<&str>) -> Self {
        let origin = origin
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from);
        Self { origin }
    }

    /// Derive a privacy-safe context from a URL: scheme + host + port only,
    /// lower-cased, userinfo stripped, path/query/fragment dropped;
    /// authority-less scheme URLs reduce to their scheme. A blank URL yields
    /// an unknown origin.
    pub fn from_url(url: &str) -> Self {
        let trimmed = url.trim();
        if trimmed.is_empty() {
            return Self { origin: None };
        }
        Self {
            origin: Some(origin_of(trimmed)),
        }
    }

    /// The reduced origin, if known.
    pub fn origin(&self) -> Option<&str> {
        self.origin.as_deref()
    }
}

impl fmt::Display for IncidentContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.origin {
            Some(origin) => f.write_str(origin),
            None => f.write_str("unknown origin"),
        }
    }
}

/// Reduce a URL to `scheme://host[:port]`, lower-cased, userinfo stripped.
///
/// Non-hierarchical URLs without an authority (`javascript:…`, `mailto:…`,
/// `data:…`) have no host to keep; they are reduced to their lower-cased
/// scheme so embedded payloads (code, addresses, data) can never survive.
/// The well-known blank page `about:blank` is the single verbatim exception.
/// Scheme-less typed hosts (`example.com/path`) keep the host portion up to
/// the first delimiter.
fn origin_of(url: &str) -> String {
    let Some(scheme_end) = url.find("://") else {
        // Non-hierarchical URL: no authority to keep.
        let end = url.find(['/', '?', '#']).unwrap_or(url.len());
        let prefix = &url[..end];
        let Some(colon) = prefix.find(':') else {
            // Scheme-less typed host: keep the host portion only.
            return prefix.to_lowercase();
        };
        // Scheme-bearing but authority-less: keep the scheme, drop the payload.
        let scheme = prefix[..colon].to_lowercase();
        if scheme == "about" && prefix.eq_ignore_ascii_case("about:blank") {
            return "about:blank".to_string();
        }
        return format!("{scheme}:");
    };
    let authority_start = scheme_end + 3;
    let authority_end = url[authority_start..]
        .find(['/', '?', '#'])
        .map(|i| authority_start + i)
        .unwrap_or(url.len());
    let authority = &url[authority_start..authority_end];
    // Strip userinfo ("user:pass@host" -> "host") for privacy.
    let host = authority
        .rfind('@')
        .map(|at| &authority[at + 1..])
        .unwrap_or(authority);
    format!("{}://{host}", &url[..scheme_end]).to_lowercase()
}

/// One reported web-error incident: identity, category, subreason, and
/// privacy-safe context.
///
/// `category` is stored for a stable wire shape and must equal
/// [`IncidentSubReason::category`] (enforced by [`Incident::new`] through
/// [`validate_definition`]). All derived behavior — severity, retryability,
/// and legal recovery actions — is derived from the **subreason**, which
/// stays the single source of truth even for a tampered deserialized
/// payload.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Incident {
    pub id: IncidentId,
    pub category: IncidentCategory,
    pub reason: IncidentSubReason,
    pub context: IncidentContext,
}

impl Incident {
    /// Create an incident, rejecting a category/reason pair that contradicts
    /// the closed taxonomy.
    pub fn new(
        id: IncidentId,
        category: IncidentCategory,
        reason: IncidentSubReason,
        context: IncidentContext,
    ) -> Result<Self, DefinitionError> {
        validate_definition(category, &reason)?;
        Ok(Self {
            id,
            category,
            reason,
            context,
        })
    }

    /// Typed severity derived from the subreason.
    pub fn severity(&self) -> SeverityLevel {
        self.reason.severity()
    }

    /// Typed retryability derived from the subreason.
    pub fn retryability(&self) -> Retryability {
        self.reason.retryability()
    }

    /// The closed recovery actions legal for this incident's category.
    pub fn legal_actions(&self) -> Vec<RecoveryAction> {
        legal_actions(&self.reason)
    }

    /// Whether a recovery action is legal for this incident.
    pub fn can_recover_with(&self, action: RecoveryAction) -> bool {
        is_legal(self.reason.category(), action)
    }
}

impl fmt::Display for Incident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} ({})", self.id, self.reason, self.context)
    }
}

/// Why a proposed incident definition was rejected by [`validate_definition`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionError {
    /// The supplied category does not match the subreason's own category.
    /// The adapter mapping seam reports a `(category, subreason)` pair and
    /// the contract rejects contradictory pairs before an incident exists.
    CategoryMismatch {
        category: IncidentCategory,
        reason: IncidentSubReason,
    },
}

impl fmt::Display for DefinitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CategoryMismatch { category, reason } => {
                write!(f, "category {category} does not match reason {reason}")
            }
        }
    }
}

/// Pure validation of an incident definition: the category must equal the
/// category the subreason belongs to. Contradictory pairs are rejected so a
/// closed taxonomy cannot be stretched by an adapter mapping.
pub fn validate_definition(
    category: IncidentCategory,
    reason: &IncidentSubReason,
) -> Result<(), DefinitionError> {
    if reason.category() == category {
        Ok(())
    } else {
        Err(DefinitionError::CategoryMismatch {
            category,
            reason: *reason,
        })
    }
}
