//! # Site identity — profile-scoped normalized origin keys
//!
//! A [`SiteKey`] is the canonical, validated identity of one site for
//! per-site settings. It is an **origin**, never a display URL: exactly the
//! scheme, host, and port, with the path/query/fragment ignored and
//! ambiguity rejected.
//!
//! ## Explicit semantics (no PSL)
//!
//! - **Scheme:** only `http` and `https` are accepted. `https` is not
//!   implied; a missing scheme is a hard error ([`SiteKeyError::MissingScheme`]).
//! - **Host:** lowercased ASCII. Domain names (labels of alphanumerics and
//!   hyphens, 1–63 chars, no leading/trailing hyphen, whole host ≤ 253),
//!   IPv4 (four decimal octets 0–255, no leading zeros), or bracketed IPv6.
//!   Non-ASCII/IDN hosts are rejected: punycode mapping is a native adapter
//!   concern, and accepting Unicode here would silently create duplicate keys
//!   for one site. Credentials (`user@host`) are rejected.
//! - **Port:** always stored explicitly. An omitted port defaults to the
//!   scheme default (`80` for `http`, `443` for `https`), so an explicit
//!   default port and an omitted one normalize to the same key. Port `0` is
//!   rejected.
//! - **Key equality** is exact normalized equality: `https://EXAMPLE.com:443`
//!   and `https://example.com/` are the same [`SiteKey`]; a different scheme,
//!   host, or port is a different key.
//!
//! [`ProfileSiteKey`] binds a [`SiteKey`] to a [`crate::ProfileId`]. Settings
//! are always addressed by profile + site; there is no cross-profile lookup.
//!
//! Serde (optional `serde` feature) serializes a [`SiteKey`] as its single
//! canonical string (for example `"https://example.com:443"`), so persisted
//! wire data cannot smuggle an invalid origin: deserialization re-runs
//! [`SiteKey::parse`] and rejects the whole value on failure.

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ProfileId;
use std::fmt;

/// URL scheme of a site key. Only unambiguous web schemes are supported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scheme {
    Http,
    Https,
}

impl Scheme {
    /// The canonical scheme text used by [`SiteKey`] display and wire form.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }

    /// The port implied when the origin omits one.
    pub const fn default_port(self) -> u16 {
        match self {
            Self::Http => 80,
            Self::Https => 443,
        }
    }
}

impl fmt::Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A validated, normalized site origin key.
///
/// Construct only via [`SiteKey::parse`]; the fields are private so an
/// invalid key cannot exist. Equality, ordering, and hashing are derived from
/// the normalized form.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SiteKey {
    scheme: Scheme,
    host: String,
    port: u16,
}

impl SiteKey {
    /// Parse and normalize a site origin.
    ///
    /// `input` may carry a path, query, or fragment; they are ignored. The
    /// input is trimmed, the scheme and host are lowercased, and the port is
    /// defaulted or validated. Invalid or empty origins reject the whole
    /// input with [`SiteKeyError`].
    pub fn parse(input: &str) -> Result<Self, SiteKeyError> {
        let raw = input.trim();
        if raw.is_empty() {
            return Err(SiteKeyError::Empty);
        }
        let (scheme_text, authority) = raw.split_once("://").ok_or(SiteKeyError::MissingScheme)?;
        if scheme_text.is_empty() {
            return Err(SiteKeyError::MissingScheme);
        }
        let scheme = match scheme_text.to_ascii_lowercase().as_str() {
            "http" => Scheme::Http,
            "https" => Scheme::Https,
            other => return Err(SiteKeyError::UnsupportedScheme(other.to_owned())),
        };
        // The origin is the authority only; everything after the first
        // path/query/fragment delimiter is not part of the site key.
        let authority = authority.split(['/', '?', '#']).next().unwrap_or_default();
        let (host, port) = parse_authority(authority, scheme)?;
        Ok(Self { scheme, host, port })
    }

    /// The normalized scheme.
    pub const fn scheme(&self) -> Scheme {
        self.scheme
    }

    /// The normalized, lowercased host (without IPv6 brackets).
    pub fn host(&self) -> &str {
        &self.host
    }

    /// The explicit port (always present; defaulted when omitted on input).
    pub const fn port(&self) -> u16 {
        self.port
    }
}

impl fmt::Display for SiteKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.host.contains(':') {
            write!(f, "{}://[{}]:{}", self.scheme, self.host, self.port)
        } else {
            write!(f, "{}://{}:{}", self.scheme, self.host, self.port)
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for SiteKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for SiteKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}

/// Typed failure of [`SiteKey::parse`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SiteKeyError {
    /// The input was empty or whitespace-only.
    Empty,
    /// The input had no `scheme://` authority prefix.
    MissingScheme,
    /// The input had a scheme other than `http` or `https`.
    UnsupportedScheme(String),
    /// The authority had no host.
    MissingHost,
    /// The host is not a valid normalized origin host (bad domain/IPv4/IPv6,
    /// credentials, or non-ASCII).
    InvalidHost,
    /// The port is not a valid `1..=65535` decimal port.
    InvalidPort,
}

impl fmt::Display for SiteKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("site origin is empty"),
            Self::MissingScheme => {
                f.write_str("site origin needs an explicit http:// or https:// scheme")
            }
            Self::UnsupportedScheme(scheme) => {
                write!(f, "site scheme {scheme:?} is not supported")
            }
            Self::MissingHost => f.write_str("site origin has no host"),
            Self::InvalidHost => f.write_str("site host is not a valid normalized origin host"),
            Self::InvalidPort => {
                f.write_str("site origin port is invalid (must be a decimal 1..=65535)")
            }
        }
    }
}

impl std::error::Error for SiteKeyError {}

/// A [`SiteKey`] owned by a [`ProfileId`]: the address of every per-site
/// setting. Profile identity is mandatory; there is no profile-less key.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProfileSiteKey {
    profile: ProfileId,
    site: SiteKey,
}

impl ProfileSiteKey {
    /// Bind a normalized site to a profile.
    pub const fn new(profile: ProfileId, site: SiteKey) -> Self {
        Self { profile, site }
    }

    /// The owning profile.
    pub const fn profile(&self) -> ProfileId {
        self.profile
    }

    /// The normalized site origin.
    pub fn site(&self) -> &SiteKey {
        &self.site
    }
}

impl fmt::Display for ProfileSiteKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "profile={} site={}", self.profile.0, self.site)
    }
}

/// Split an authority into its normalized (host, port) pair.
///
/// IPv6 must be bracketed; credentials are rejected; the port defaults to the
/// scheme default when omitted.
fn parse_authority(authority: &str, scheme: Scheme) -> Result<(String, u16), SiteKeyError> {
    if authority.contains('@') {
        return Err(SiteKeyError::InvalidHost);
    }
    if let Some(rest) = authority.strip_prefix('[') {
        let (host, after) = rest.split_once(']').ok_or(SiteKeyError::InvalidHost)?;
        let port = match after {
            "" => scheme.default_port(),
            port if port.starts_with(':') => parse_port(&port[1..])?,
            _ => return Err(SiteKeyError::InvalidHost),
        };
        return Ok((normalize_host(host, true)?, port));
    }
    let (host, port) = match authority.split_once(':') {
        None => (authority, scheme.default_port()),
        Some((host, port)) => (host, parse_port(port)?),
    };
    if host.is_empty() {
        return Err(SiteKeyError::MissingHost);
    }
    Ok((normalize_host(host, false)?, port))
}

/// Validate and lowercase a host. `bracketed_ipv6` marks a host extracted
/// from `[...]`, which must be a valid IPv6 address.
fn normalize_host(host: &str, bracketed_ipv6: bool) -> Result<String, SiteKeyError> {
    if host.is_empty() {
        return Err(SiteKeyError::MissingHost);
    }
    if !host.is_ascii() {
        // IDN/punycode mapping is a native adapter concern; rejecting keeps
        // the key a byte-exact canonical origin with no duplicate spellings.
        return Err(SiteKeyError::InvalidHost);
    }
    let lower = host.to_ascii_lowercase();
    if bracketed_ipv6 {
        return if is_valid_ipv6(&lower) {
            Ok(lower)
        } else {
            Err(SiteKeyError::InvalidHost)
        };
    }
    if looks_like_ipv4(&lower) {
        return if is_valid_ipv4(&lower) {
            Ok(lower)
        } else {
            Err(SiteKeyError::InvalidHost)
        };
    }
    if is_valid_domain(&lower) {
        Ok(lower)
    } else {
        Err(SiteKeyError::InvalidHost)
    }
}

/// Parse a decimal port string into `1..=65535`.
fn parse_port(raw: &str) -> Result<u16, SiteKeyError> {
    if raw.is_empty() || !raw.bytes().all(|b| b.is_ascii_digit()) {
        return Err(SiteKeyError::InvalidPort);
    }
    let port = raw.parse::<u16>().map_err(|_| SiteKeyError::InvalidPort)?;
    if port == 0 {
        return Err(SiteKeyError::InvalidPort);
    }
    Ok(port)
}

/// Four dot-separated, purely decimal parts (candidate IPv4 literal).
fn looks_like_ipv4(host: &str) -> bool {
    let parts: Vec<&str> = host.split('.').collect();
    parts.len() == 4
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

/// Four decimal octets `0..=255` with no leading zeros.
fn is_valid_ipv4(host: &str) -> bool {
    host.split('.').all(|octet| {
        !octet.is_empty()
            && octet.bytes().all(|b| b.is_ascii_digit())
            && !(octet.len() > 1 && octet.starts_with('0'))
            && octet.parse::<u8>().is_ok()
    })
}

/// Domain-name labels: 1–63 chars of ASCII alphanumerics and hyphens, no
/// leading/trailing hyphen, no empty labels, whole host ≤ 253 chars.
fn is_valid_domain(host: &str) -> bool {
    if host.is_empty() || host.len() > 253 {
        return false;
    }
    host.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && label
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            && !label.starts_with('-')
            && !label.ends_with('-')
    })
}

/// RFC 4291 IPv6 with at most one `::` compression; groups are 1–4 hex
/// digits and `::` must compress at least one group.
fn is_valid_ipv6(host: &str) -> bool {
    if host.is_empty() || !host.bytes().all(|b| b.is_ascii_hexdigit() || b == b':') {
        return false;
    }
    let mut parts = host.splitn(2, "::");
    let left = parts.next().unwrap_or_default();
    match parts.next() {
        None => {
            let groups: Vec<&str> = host.split(':').collect();
            groups.len() == 8 && groups.iter().all(|group| is_hex_group(group))
        }
        Some(right) => {
            if right.contains("::") {
                return false;
            }
            let left_groups: Vec<&str> = left.split(':').collect();
            let right_groups: Vec<&str> = right.split(':').collect();
            let left_ok = left.is_empty()
                || (!left.ends_with(':') && left_groups.iter().all(|g| is_hex_group(g)));
            let right_ok = right.is_empty()
                || (!right.starts_with(':') && right_groups.iter().all(|g| is_hex_group(g)));
            left_ok && right_ok && left_groups.len() + right_groups.len() < 8
        }
    }
}

fn is_hex_group(group: &str) -> bool {
    !group.is_empty() && group.len() <= 4 && group.bytes().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests;
