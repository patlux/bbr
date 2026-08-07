//! Validated web-origin identity for permission policy keys.
//!
//! [`PermissionOrigin`] is the security-relevant part of every durable
//! permission key: a normalized `scheme://host[:port]` triple. The fields are
//! private and every construction path ([`PermissionOrigin::parse`],
//! [`PermissionOrigin::try_new`], and the `serde` deserializer) re-runs the
//! same validation, so an empty or cross-origin-conflating key is
//! unrepresentable without importing a URL crate.

use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A validated web origin: scheme + host + optional port, normalized.
///
/// Normalization collapses equivalent spellings into one identity:
///
/// - scheme and host are lowercased (`HTTPS://Example.COM` ≡ `https://example.com`);
/// - a path, query, or fragment is stripped (`https://example.com/a?b` ≡
///   `https://example.com`);
/// - a default port for the scheme collapses (`https://example.com:443` ≡
///   `https://example.com`).
///
/// The host is an opaque non-empty string. WebKit reports a `WKWebView`
/// security origin as separate scheme/host/port parts, so no URL parsing
/// library is needed here; userinfo (`user@`) and whitespace are rejected
/// because they are not part of a web origin.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PermissionOrigin {
    scheme: String,
    host: String,
    port: Option<u16>,
}

impl PermissionOrigin {
    /// Parse and normalize an origin from a URL string.
    ///
    /// The input may carry a path, query, or fragment — only the origin part
    /// is retained. See [`OriginError`] for the rejected shapes.
    pub fn parse(raw: &str) -> Result<Self, OriginError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(OriginError::Empty);
        }
        let (scheme, rest) = trimmed
            .split_once("://")
            .ok_or(OriginError::MissingSchemeSeparator)?;
        if !is_valid_scheme(scheme) {
            return Err(OriginError::InvalidScheme);
        }
        let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
        let (host, port) = split_host_port(authority)?;
        Self::try_new(scheme, host, port)
    }

    /// Build an origin from WebKit-style parts.
    ///
    /// A `WKWebView` security origin reports scheme/host/port separately; this
    /// is the adapter-facing construction path. It applies the same
    /// validation and normalization as [`parse`](Self::parse).
    pub fn try_new(scheme: &str, host: &str, port: Option<u16>) -> Result<Self, OriginError> {
        if !is_valid_scheme(scheme) {
            return Err(OriginError::InvalidScheme);
        }
        let host = host.trim();
        if host.is_empty() {
            return Err(OriginError::EmptyHost);
        }
        if host.chars().any(char::is_whitespace) || host.contains('@') {
            return Err(OriginError::InvalidHost);
        }
        let port = match port {
            Some(0) => return Err(OriginError::InvalidPort),
            Some(p) if is_default_port(scheme, p) => None,
            other => other,
        };
        Ok(Self {
            scheme: scheme.to_ascii_lowercase(),
            host: host.to_ascii_lowercase(),
            port,
        })
    }

    /// The normalized scheme (`https`).
    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    /// The normalized host (`example.com`).
    pub fn host(&self) -> &str {
        &self.host
    }

    /// The explicit non-default port, or `None` when the scheme's default
    /// port applies.
    pub fn port(&self) -> Option<u16> {
        self.port
    }
}

impl fmt::Display for PermissionOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.port {
            None => write!(f, "{}://{}", self.scheme, self.host),
            Some(port) => write!(f, "{}://{}:{}", self.scheme, self.host, port),
        }
    }
}

/// Why an origin was rejected.
///
/// Deserializing an origin from the wire re-runs the same validation, so a
/// corrupt or malicious policy record is rejected at the boundary instead of
/// being stored as an empty or cross-origin key.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginError {
    /// The input is empty or whitespace only.
    Empty,
    /// The input has no `scheme://` separator.
    MissingSchemeSeparator,
    /// The scheme is missing, does not start with an ASCII letter, or
    /// contains characters outside `[a-zA-Z0-9+-.]`.
    InvalidScheme,
    /// The host part is empty.
    EmptyHost,
    /// The host contains userinfo (`@`) or whitespace, which are not part of
    /// a web origin.
    InvalidHost,
    /// The port is missing, not a number, zero, or out of `u16` range.
    InvalidPort,
}

impl fmt::Display for OriginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Empty => "origin is empty (or whitespace only)",
            Self::MissingSchemeSeparator => "origin is missing a `scheme://` separator",
            Self::InvalidScheme => "origin scheme is missing or invalid",
            Self::EmptyHost => "origin host is empty",
            Self::InvalidHost => "origin host contains userinfo or whitespace",
            Self::InvalidPort => "origin port is missing or out of range",
        };
        f.write_str(message)
    }
}

impl std::error::Error for OriginError {}

/// The `serde` boundary re-validates: a stored or wire-provided origin is
/// deserialized into raw parts and run through [`try_new`](Self::try_new), so
/// normalization and rejection behave identically to the string boundary.
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for PermissionOrigin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawOrigin {
            scheme: String,
            host: String,
            #[serde(default)]
            port: Option<u16>,
        }

        let raw = RawOrigin::deserialize(deserializer)?;
        PermissionOrigin::try_new(&raw.scheme, &raw.host, raw.port).map_err(|error| {
            serde::de::Error::custom(format!("invalid permission origin: {error}"))
        })
    }
}

/// A scheme: an ASCII letter followed by ASCII alphanumerics, `+`, `-`, `.`.
fn is_valid_scheme(scheme: &str) -> bool {
    let mut chars = scheme.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

/// Split `host[:port]`, validating the port when present.
///
/// A bracketed host (`[::1]:8443`) keeps everything up to the closing `]`
/// as the host; only the optional `:` after that is the port separator. For
/// an unbracketed host the first `:` separates the port. An empty host,
/// userinfo, or whitespace is rejected here; a zero port is rejected later in
/// [`try_new`](PermissionOrigin::try_new), and default ports collapse there
/// via [`is_default_port`].
fn split_host_port(authority: &str) -> Result<(&str, Option<u16>), OriginError> {
    let (host, port) = if let Some(rest) = authority.strip_prefix('[') {
        let end = rest.find(']').ok_or(OriginError::InvalidHost)?;
        let host = &authority[..=end + 1];
        let after = &authority[end + 2..];
        if after.is_empty() {
            (host, None)
        } else if let Some(port) = after.strip_prefix(':') {
            (host, Some(parse_port(port)?))
        } else {
            return Err(OriginError::InvalidHost);
        }
    } else {
        match authority.split_once(':') {
            None => (authority, None),
            Some((host, port)) => (host, Some(parse_port(port)?)),
        }
    };
    if host.is_empty() {
        return Err(OriginError::EmptyHost);
    }
    if host.contains('@') || host.chars().any(char::is_whitespace) {
        return Err(OriginError::InvalidHost);
    }
    Ok((host, port))
}

/// Parse a port string: non-empty ASCII digits fitting `u16`.
fn parse_port(port: &str) -> Result<u16, OriginError> {
    if port.is_empty() {
        return Err(OriginError::InvalidPort);
    }
    port.parse::<u16>().map_err(|_| OriginError::InvalidPort)
}

/// Whether `port` is the scheme's default port and therefore collapses away
/// during normalization.
fn is_default_port(scheme: &str, port: u16) -> bool {
    match scheme.to_ascii_lowercase().as_str() {
        "http" => port == 80,
        "https" => port == 443,
        _ => false,
    }
}
