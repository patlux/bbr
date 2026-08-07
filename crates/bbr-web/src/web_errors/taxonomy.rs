//! Closed error categories/subreasons, typed severity, and retryability.
//!
//! This module is the taxonomy half of the Phase-8 web-error contract: it
//! fixes **what an incident is classified as**. The incident identity,
//! privacy-safe context, and the assembled [`Incident`] live in
//! [`super::identity`]; the recovery-action matrix lives in
//! [`super::recovery`]; the incident lifecycle (reported, superseded,
//! recovery requested, recovered, failed, dismissed, stale rejection) lives
//! in [`super::lifecycle`].
//!
//! ## Closed categories and subreasons
//!
//! Every incident belongs to exactly one of four closed categories
//! ([`IncidentCategory`]): TLS, navigation, offline, or process crash. Each
//! category owns a closed subreason set ([`TlsReason`], [`NavigationReason`],
//! [`OfflineReason`], [`CrashReason`]). The categories are distinct by
//! contract: an offline state is never modeled as a navigation failure, and a
//! process crash is never modeled as an ordinary retryable navigation.
//! Combined subreasons are represented by [`IncidentSubReason`].
//!
//! The adapter seam is a *pure validation*: an adapter translates an engine
//! error into a `(category, subreason)` candidate and the contract accepts
//! the pair only when it is consistent ([`super::validate_definition`]). A
//! contradictory pair (for example `Offline` combined with a TLS subreason)
//! is rejected with [`DefinitionError::CategoryMismatch`] before an incident
//! can exist. This keeps the closed taxonomy truthful even though
//! deserialization can craft arbitrary payloads: derived behavior always
//! reads the subreason, never the stored category copy.
//!
//! ## Derived properties
//!
//! [`SeverityLevel`] and [`Retryability`] are typed closed enums derived from
//! the subreason (`IncidentSubReason::severity` /
//! `IncidentSubReason::retryability`). Derivation (rather than caller input)
//! keeps severity and retry policy stable per taxonomy class and leaves no
//! way for two identical incidents to disagree.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::fmt;

use super::recovery::{legal_actions, RecoveryAction};
/// Closed top-level category of a web-error incident.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IncidentCategory {
    /// Transport/identity failures of the TLS handshake and certificate
    /// chain. A certificate exception (bypass) is **not** part of this
    /// contract.
    Tls,
    /// Ordinary navigation failures: unsupported scheme, DNS, connection,
    /// timeout, redirect loop.
    Navigation,
    /// The device or proxy cannot reach the network at all.
    Offline,
    /// The web content process (or a companion process terminated by the
    /// system) died; the page is gone.
    ProcessCrash,
}

impl IncidentCategory {
    /// All closed variants.
    pub const ALL: [IncidentCategory; 4] = [
        IncidentCategory::Tls,
        IncidentCategory::Navigation,
        IncidentCategory::Offline,
        IncidentCategory::ProcessCrash,
    ];
}

impl fmt::Display for IncidentCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Tls => "tls",
            Self::Navigation => "navigation",
            Self::Offline => "offline",
            Self::ProcessCrash => "process crash",
        };
        f.write_str(text)
    }
}

/// Closed subreasons of the TLS category.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TlsReason {
    /// The leaf certificate is past its `notAfter`.
    CertificateExpired,
    /// The leaf certificate is not valid yet (`notBefore` in the future).
    CertificateNotYetValid,
    /// The certificate chain cannot be trusted (self-signed or unknown root).
    CertificateUntrusted,
    /// The certificate is on a revocation list.
    CertificateRevoked,
    /// The certificate does not cover the requested host name.
    HostnameMismatch,
    /// The TLS handshake failed for another reason (protocol/version/cipher).
    HandshakeFailed,
}

impl TlsReason {
    pub const ALL: [TlsReason; 6] = [
        TlsReason::CertificateExpired,
        TlsReason::CertificateNotYetValid,
        TlsReason::CertificateUntrusted,
        TlsReason::CertificateRevoked,
        TlsReason::HostnameMismatch,
        TlsReason::HandshakeFailed,
    ];
}

impl fmt::Display for TlsReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::CertificateExpired => "certificate expired",
            Self::CertificateNotYetValid => "certificate not yet valid",
            Self::CertificateUntrusted => "certificate untrusted",
            Self::CertificateRevoked => "certificate revoked",
            Self::HostnameMismatch => "hostname mismatch",
            Self::HandshakeFailed => "handshake failed",
        };
        f.write_str(text)
    }
}

/// Closed subreasons of the navigation category.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NavigationReason {
    /// The URL scheme cannot be handled by the web engine.
    UnsupportedScheme,
    /// The host name did not resolve.
    ServerNotFound,
    /// The connection to the server was refused.
    ConnectionRefused,
    /// The request timed out.
    RequestTimedOut,
    /// Navigation was aborted after repeated redirects.
    RedirectLoop,
}

impl NavigationReason {
    pub const ALL: [NavigationReason; 5] = [
        NavigationReason::UnsupportedScheme,
        NavigationReason::ServerNotFound,
        NavigationReason::ConnectionRefused,
        NavigationReason::RequestTimedOut,
        NavigationReason::RedirectLoop,
    ];
}

impl fmt::Display for NavigationReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::UnsupportedScheme => "unsupported scheme",
            Self::ServerNotFound => "server not found",
            Self::ConnectionRefused => "connection refused",
            Self::RequestTimedOut => "request timed out",
            Self::RedirectLoop => "redirect loop",
        };
        f.write_str(text)
    }
}

/// Closed subreasons of the offline category.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OfflineReason {
    /// No network interface is available at all.
    NoNetworkConnection,
    /// The configured proxy is unreachable.
    ProxyUnreachable,
    /// A network route to the destination does not exist.
    NetworkUnreachable,
}

impl OfflineReason {
    pub const ALL: [OfflineReason; 3] = [
        OfflineReason::NoNetworkConnection,
        OfflineReason::ProxyUnreachable,
        OfflineReason::NetworkUnreachable,
    ];
}

impl fmt::Display for OfflineReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::NoNetworkConnection => "no network connection",
            Self::ProxyUnreachable => "proxy unreachable",
            Self::NetworkUnreachable => "network unreachable",
        };
        f.write_str(text)
    }
}

/// Closed subreasons of the process-crash category.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CrashReason {
    /// The web content process crashed while rendering.
    WebContentCrashed,
    /// The system terminated the web process (memory pressure, kill).
    ProcessTerminatedBySystem,
}

impl CrashReason {
    pub const ALL: [CrashReason; 2] = [
        CrashReason::WebContentCrashed,
        CrashReason::ProcessTerminatedBySystem,
    ];
}

impl fmt::Display for CrashReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::WebContentCrashed => "web content crashed",
            Self::ProcessTerminatedBySystem => "process terminated by system",
        };
        f.write_str(text)
    }
}

/// The combined, closed subreason taxonomy: one variant per category, each
/// wrapping that category's closed subreason set.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IncidentSubReason {
    Tls(TlsReason),
    Navigation(NavigationReason),
    Offline(OfflineReason),
    Crash(CrashReason),
}

impl IncidentSubReason {
    /// The single closed category this subreason belongs to. All derived
    /// behavior keys off this value.
    pub fn category(&self) -> IncidentCategory {
        match self {
            Self::Tls(_) => IncidentCategory::Tls,
            Self::Navigation(_) => IncidentCategory::Navigation,
            Self::Offline(_) => IncidentCategory::Offline,
            Self::Crash(_) => IncidentCategory::ProcessCrash,
        }
    }

    /// Typed severity derived from the taxonomy class.
    pub fn severity(&self) -> SeverityLevel {
        match self {
            Self::Tls(reason) => match reason {
                TlsReason::CertificateExpired
                | TlsReason::CertificateUntrusted
                | TlsReason::HandshakeFailed => SeverityLevel::High,
                TlsReason::CertificateNotYetValid => SeverityLevel::Medium,
                TlsReason::CertificateRevoked | TlsReason::HostnameMismatch => {
                    SeverityLevel::Critical
                }
            },
            Self::Navigation(reason) => match reason {
                NavigationReason::UnsupportedScheme => SeverityLevel::Low,
                NavigationReason::ServerNotFound
                | NavigationReason::ConnectionRefused
                | NavigationReason::RequestTimedOut => SeverityLevel::Medium,
                NavigationReason::RedirectLoop => SeverityLevel::High,
            },
            Self::Offline(reason) => match reason {
                OfflineReason::NoNetworkConnection | OfflineReason::ProxyUnreachable => {
                    SeverityLevel::Medium
                }
                OfflineReason::NetworkUnreachable => SeverityLevel::High,
            },
            Self::Crash(_) => SeverityLevel::High,
        }
    }

    /// Typed retryability derived from the taxonomy class.
    ///
    /// `Safe`: an immediate retry may succeed (transient failure). `Deferred`:
    /// retry is legal but only succeeds after an external condition changes
    /// (certificate renewed, network restored). `Never`: retrying this
    /// incident as-is can never succeed; use the category's recovery action
    /// (e.g. restore after a crash) or dismiss.
    pub fn retryability(&self) -> Retryability {
        match self {
            Self::Tls(reason) => match reason {
                TlsReason::CertificateExpired | TlsReason::CertificateNotYetValid => {
                    Retryability::Deferred
                }
                TlsReason::CertificateUntrusted
                | TlsReason::CertificateRevoked
                | TlsReason::HostnameMismatch => Retryability::Never,
                TlsReason::HandshakeFailed => Retryability::Safe,
            },
            Self::Navigation(reason) => match reason {
                NavigationReason::ServerNotFound | NavigationReason::RequestTimedOut => {
                    Retryability::Safe
                }
                NavigationReason::ConnectionRefused => Retryability::Deferred,
                NavigationReason::UnsupportedScheme | NavigationReason::RedirectLoop => {
                    Retryability::Never
                }
            },
            Self::Offline(_) => Retryability::Deferred,
            Self::Crash(_) => Retryability::Never,
        }
    }

    /// The closed recovery actions legal for this subreason's category.
    /// See [`super::recovery::legal_actions_for_category`].
    pub fn legal_actions(&self) -> Vec<RecoveryAction> {
        legal_actions(self)
    }
}

impl fmt::Display for IncidentSubReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = match self {
            Self::Tls(reason) => reason.to_string(),
            Self::Navigation(reason) => reason.to_string(),
            Self::Offline(reason) => reason.to_string(),
            Self::Crash(reason) => reason.to_string(),
        };
        write!(f, "{}: {inner}", self.category())
    }
}

/// Typed severity of an incident, ordered low → critical.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SeverityLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for SeverityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        };
        f.write_str(text)
    }
}

/// Typed retryability of an incident: whether a retry of the same navigation
/// is legal and what outcome it can have. See [`IncidentSubReason::retryability`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Retryability {
    /// An immediate retry may succeed without any external change.
    Safe,
    /// Retry is legal but only succeeds after an external condition changes.
    Deferred,
    /// Retrying this incident as-is is never legal.
    Never,
}

impl fmt::Display for Retryability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Safe => "safe",
            Self::Deferred => "deferred",
            Self::Never => "never",
        };
        f.write_str(text)
    }
}
