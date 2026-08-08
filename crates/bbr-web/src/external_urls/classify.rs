//! URL classification: supported web URL, dangerous scheme, unsupported
//! scheme, or malformed, with the exact scheme rules documented here.
//!
//! ## Scheme rules
//!
//! A URL is classified by its **scheme**, matched case-insensitively (RFC
//! 3986 §3.1: `ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )`, required to start
//! with an ASCII letter):
//!
//! - `http` / `https` — the only **supported web schemes**. A web URL must
//!   carry an authority (`://` followed by a non-empty host); without one it
//!   is malformed, never opened.
//! - `javascript`, `data`, `file`, `about`, `vbscript` — **dangerous
//!   schemes**, classified explicitly and never opened: they can execute
//!   code (`javascript:`, `data:`, `vbscript:`), expose the local
//!   filesystem (`file:`), or address browser-internal pages (`about:`,
//!   including `about:blank` — an external open never asks for it).
//! - any other grammar-valid scheme (`mailto:`, `tel:`, `ftp:`,
//!   `itms-services:`, `magnet:`, custom app schemes, …) — **unsupported**:
//!   the browser cannot render them, and the OS is the dispatcher for
//!   registered schemes, so one reaching the browser is a misregistration.
//!   Rejected, never opened.
//! - everything else — **malformed**.
//!
//! ## Judgment notes (reject rather than guess)
//!
//! - A scheme-less string (`example.com`, `//example.com`) is malformed: the
//!   OS delivers absolute URLs, and this contract never applies
//!   [`crate::normalize_url`]'s scheme defaulting.
//! - A token that could be a scheme is read as a **host:port** when the
//!   remainder before the first `/`, `?`, or `#` is non-empty ASCII digits —
//!   the same judgment [`crate::normalize_url`] applies to user-typed input.
//!   Such input is scheme-less here and therefore rejected
//!   (`example.com:8080`; `tel:0123456789` is still a rejection either way —
//!   never an open).
//! - Leading/trailing whitespace is trimmed before classification; any
//!   whitespace (including non-ASCII) or control character inside the URL is
//!   malformed. Trimming can never alter a valid URL, because RFC 3986
//!   forbids spaces entirely.
//! - Ports must be non-empty ASCII digits and at most 65535; an empty port
//!   (`https://host:`) is malformed.
//! - A bracketed host (`[::1]`) must be closed; no further IPv6 shape is
//!   validated here — the engine owns that.
//!
//! Classification only parses what routing needs (scheme, host, port). The
//! navigated URL is the request's own string; nothing is reconstructed.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::fmt;

/// The only schemes BBR opens for external URLs.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebScheme {
    Http,
    Https,
}

impl WebScheme {
    fn as_str(self) -> &'static str {
        match self {
            WebScheme::Http => "http",
            WebScheme::Https => "https",
        }
    }
}

impl fmt::Display for WebScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A parsed supported web URL.
///
/// Only the fields routing needs are parsed: the scheme, a lower-cased host
/// with any userinfo stripped, and the port. `remainder` is the raw
/// path+query+fragment exactly as delivered (empty when absent), kept
/// untransformed so burst deduplication is byte-exact and the navigated URL
/// is never reconstructed from parts.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebUrl {
    pub scheme: WebScheme,
    /// Lower-cased host with any userinfo stripped.
    pub host: String,
    /// Validated port, when the URL carries one.
    pub port: Option<u16>,
    /// The raw path+query+fragment exactly as delivered; `""` when absent.
    pub remainder: String,
}

impl WebUrl {
    /// The privacy-reduced origin: `scheme://host[:port]`, lower-cased,
    /// userinfo already stripped at parse time. Never the path, query,
    /// fragment, or credentials — the same reduction the sibling web-error
    /// contract establishes.
    pub fn reduced_origin(&self) -> String {
        let mut origin = String::from(self.scheme.as_str());
        origin.push_str("://");
        origin.push_str(&self.host);
        if let Some(port) = self.port {
            origin.push(':');
            origin.push_str(&port.to_string());
        }
        origin
    }
}

/// Why a URL is not a supported web URL. Carried by the wire-able
/// [`ExternalUrlOutcome::Rejected`] variant, so it derives serde like the
/// outcome; only the control-path errors stay non-serde.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectionReason {
    Malformed(MalformedUrl),
    DangerousScheme {
        /// Lower-cased scheme name; never the payload.
        scheme: String,
    },
    UnsupportedScheme {
        /// Lower-cased scheme name; never the payload.
        scheme: String,
    },
}

impl fmt::Display for RejectionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RejectionReason::Malformed(reason) => write!(f, "malformed url ({reason})"),
            RejectionReason::DangerousScheme { scheme } => {
                write!(f, "dangerous scheme {scheme}:")
            }
            RejectionReason::UnsupportedScheme { scheme } => {
                write!(f, "unsupported scheme {scheme}:")
            }
        }
    }
}

/// Why a URL is malformed. Content-free by construction: a malformed string
/// carries no safe diagnostic part.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MalformedUrl {
    /// Empty after trimming.
    Empty,
    /// No usable absolute scheme: no `:`, a `:` that is a host:port
    /// look-alike (`example.com:8080`), or no scheme at all.
    MissingScheme,
    /// A `:`-prefixed token that is not a legal RFC 3986 scheme name (empty,
    /// starts with a digit, or contains illegal characters).
    InvalidScheme,
    /// `http`/`https` without an authority or with an empty host
    /// (`https://`, `https:///path`, `https:example.com`, `https://user@`).
    MissingHost,
    /// A bracketed host that never closes (`http://[::1`).
    UnclosedBracketedHost,
    /// A port that is empty, not all ASCII digits, or above 65535.
    InvalidPort,
    /// Whitespace or control characters inside the URL.
    InvalidCharacters,
}

impl fmt::Display for MalformedUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            MalformedUrl::Empty => "empty",
            MalformedUrl::MissingScheme => "missing scheme",
            MalformedUrl::InvalidScheme => "invalid scheme",
            MalformedUrl::MissingHost => "missing host",
            MalformedUrl::UnclosedBracketedHost => "unclosed bracketed host",
            MalformedUrl::InvalidPort => "invalid port",
            MalformedUrl::InvalidCharacters => "invalid characters",
        };
        f.write_str(text)
    }
}

/// The typed classification of one URL string.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlClass {
    /// A supported `http`/`https` URL that may be routed.
    Web(WebUrl),
    /// A scheme that can execute code or expose local files; never opened.
    DangerousScheme {
        /// Lower-cased scheme name; never the payload.
        scheme: String,
    },
    /// A valid scheme the browser cannot render; the OS should have
    /// dispatched it to a registered handler. Never opened.
    UnsupportedScheme {
        /// Lower-cased scheme name; never the payload.
        scheme: String,
    },
    /// Not a routable URL at all.
    Malformed(MalformedUrl),
}

impl UrlClass {
    /// The privacy-reduced diagnostic form of this class: the reduced origin
    /// for a web URL, the lower-cased scheme for a dangerous/unsupported
    /// scheme (never the payload), and the fixed placeholder
    /// `"<malformed url>"` for malformed input.
    pub fn reduced(&self) -> ReducedUrl {
        match self {
            UrlClass::Web(url) => ReducedUrl(url.reduced_origin()),
            UrlClass::DangerousScheme { scheme } | UrlClass::UnsupportedScheme { scheme } => {
                ReducedUrl(format!("{scheme}:"))
            }
            UrlClass::Malformed(_) => ReducedUrl("<malformed url>".to_string()),
        }
    }
}

/// Privacy-reduced URL form for diagnostics: never the path, query,
/// fragment, or credentials.
///
/// Authority-bearing web URLs reduce to `scheme://host[:port]` (lower-cased,
/// userinfo stripped); authority-less scheme URLs reduce to their lower-cased
/// scheme (`javascript:`, `mailto:`, …); malformed URLs reduce to the fixed
/// placeholder `"<malformed url>"`. This is the same reduction rule the
/// sibling web-error contract's incident context establishes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReducedUrl(String);

impl ReducedUrl {
    /// The reduced form as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ReducedUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Classify a raw URL string into a typed [`UrlClass`]. See the module docs
/// for the exact scheme rules and judgment notes.
pub fn classify(raw: &str) -> UrlClass {
    let url = raw.trim();
    if url.is_empty() {
        return UrlClass::Malformed(MalformedUrl::Empty);
    }
    if url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return UrlClass::Malformed(MalformedUrl::InvalidCharacters);
    }
    let Some(colon) = url.find(':') else {
        return UrlClass::Malformed(MalformedUrl::MissingScheme);
    };
    let scheme = &url[..colon];
    if colon == 0 || !is_valid_scheme_name(scheme) {
        return UrlClass::Malformed(MalformedUrl::InvalidScheme);
    }
    let rest = &url[colon + 1..];
    let suffix = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    if !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit()) {
        // Host:port look-alike (`example.com:8080`), not a scheme — the same
        // judgment `normalize_url` applies; scheme-less input is rejected.
        return UrlClass::Malformed(MalformedUrl::MissingScheme);
    }
    let scheme_lower = scheme.to_ascii_lowercase();
    match scheme_lower.as_str() {
        "http" | "https" => match parse_web_url(&scheme_lower, rest) {
            Ok(url) => UrlClass::Web(url),
            Err(malformed) => UrlClass::Malformed(malformed),
        },
        "javascript" | "data" | "file" | "about" | "vbscript" => UrlClass::DangerousScheme {
            scheme: scheme_lower,
        },
        _ => UrlClass::UnsupportedScheme {
            scheme: scheme_lower,
        },
    }
}

/// RFC 3986 §3.1 scheme name: an ASCII letter followed by ASCII
/// alphanumerics, `+`, `-`, or `.`.
fn is_valid_scheme_name(scheme: &str) -> bool {
    let mut chars = scheme.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

/// Parse the part after `scheme:` for a web URL. The authority is required
/// (`://` plus a non-empty host); `remainder` is everything after the
/// authority, byte-exact.
fn parse_web_url(scheme: &str, rest: &str) -> Result<WebUrl, MalformedUrl> {
    let Some(rest) = rest.strip_prefix("//") else {
        return Err(MalformedUrl::MissingHost);
    };
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    let remainder = &rest[authority_end..];
    if authority.is_empty() {
        return Err(MalformedUrl::MissingHost);
    }
    let (host, port) = split_host_port(authority)?;
    Ok(WebUrl {
        scheme: if scheme == "http" {
            WebScheme::Http
        } else {
            WebScheme::Https
        },
        host,
        port,
        remainder: remainder.to_string(),
    })
}

/// Split an authority into `(host, port)`, stripping userinfo and handling
/// bracketed IPv6 hosts. Returns the malformed reason when the authority has
/// no host or an illegal port.
fn split_host_port(authority: &str) -> Result<(String, Option<u16>), MalformedUrl> {
    // Userinfo ends at the last '@' (RFC 3986); strip it for the host.
    let authority = authority.rsplit('@').next().unwrap_or(authority);
    if authority.is_empty() {
        return Err(MalformedUrl::MissingHost);
    }
    if let Some(rest) = authority.strip_prefix('[') {
        let Some(close) = rest.find(']') else {
            return Err(MalformedUrl::UnclosedBracketedHost);
        };
        let host = &rest[..close];
        if host.is_empty() {
            return Err(MalformedUrl::MissingHost);
        }
        let after = &rest[close + 1..];
        let port = if after.is_empty() {
            None
        } else {
            let Some(port_text) = after.strip_prefix(':') else {
                // Junk after the closing bracket (`[::1]x`) is not an
                // authority shape this contract accepts.
                return Err(MalformedUrl::MissingHost);
            };
            Some(parse_port(port_text)?)
        };
        return Ok((host.to_lowercase(), port));
    }
    let (host, port) = match authority.find(':') {
        Some(colon) => (&authority[..colon], Some(&authority[colon + 1..])),
        None => (authority, None),
    };
    if host.is_empty() {
        return Err(MalformedUrl::MissingHost);
    }
    let port = match port {
        Some(text) => Some(parse_port(text)?),
        None => None,
    };
    Ok((host.to_lowercase(), port))
}

/// A port must be non-empty ASCII digits and at most 65535. Overflowing
/// digit strings parse-fail instead of panicking.
fn parse_port(text: &str) -> Result<u16, MalformedUrl> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(MalformedUrl::InvalidPort);
    }
    match text.parse::<u64>() {
        Ok(value) if value <= u64::from(u16::MAX) => Ok(value as u16),
        _ => Err(MalformedUrl::InvalidPort),
    }
}
