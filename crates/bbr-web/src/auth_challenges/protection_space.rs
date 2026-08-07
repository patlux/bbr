//! Validated protection-space identity for authentication challenges.
//!
//! A protection space names the authority a challenge belongs to (origin or
//! proxy) together with the normalized scheme, host, port, and realm. Spaces
//! are value-keyed: equal spaces are interchangeable, and the authority is
//! part of the key, so an origin space and a proxy space can never collide
//! even when host/port/realm are identical.
//!
//! All constructors validate and normalize, so an invalid or ambiguous space
//! is unrepresentable. Hosts are trimmed and lowercased (DNS names are
//! case-insensitive); the scheme's default port is canonicalized to `0`
//! ("scheme default") so a URL spelling the default port and one omitting it
//! produce the same key.

use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Maximum length of a normalized host, mirroring the RFC 1035 full DNS name
/// length of 253 characters.
pub const MAX_HOST_LEN: usize = 253;

/// Maximum length of a realm label. A realm is a server-chosen label; it
/// must never become an unbounded diagnostic or storage key.
pub const MAX_REALM_LEN: usize = 512;

/// Why a host, realm, or protection space could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectionSpaceError {
    /// The host is empty (or only whitespace).
    EmptyHost,
    /// The host contains characters that would make the protection-space key
    /// ambiguous (whitespace, control bytes, or URL delimiters such as
    /// `: / ? # @`), or is not a representable name.
    InvalidHost,
    /// The host exceeds [`MAX_HOST_LEN`].
    HostTooLong,
    /// The realm is empty (or only whitespace).
    EmptyRealm,
    /// The realm exceeds [`MAX_REALM_LEN`].
    RealmTooLong,
}

impl fmt::Display for ProtectionSpaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtectionSpaceError::EmptyHost => write!(f, "host must not be empty"),
            ProtectionSpaceError::InvalidHost => {
                write!(f, "host contains ambiguous or unsupported characters")
            }
            ProtectionSpaceError::HostTooLong => {
                write!(f, "host exceeds the maximum length of {MAX_HOST_LEN}")
            }
            ProtectionSpaceError::EmptyRealm => write!(f, "realm must not be empty"),
            ProtectionSpaceError::RealmTooLong => {
                write!(f, "realm exceeds the maximum length of {MAX_REALM_LEN}")
            }
        }
    }
}

impl std::error::Error for ProtectionSpaceError {}

/// HTTP scheme of a protection space. Only schemes with defined HTTP
/// authentication behavior are representable; the adapter maps every engine
/// challenge to `http` or `https`, and anything else is reported through the
/// typed unsupported taxonomy instead of an opaque payload.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpScheme {
    /// Plain HTTP (`http://`). No transport security.
    Http,
    /// HTTP over TLS (`https://`). Required for client-certificate challenges.
    Https,
}

impl HttpScheme {
    /// The scheme's default port: 80 for HTTP, 443 for HTTPS. An explicitly
    /// spelled default port is canonicalized to `0` ("scheme default") when a
    /// protection space is constructed.
    pub const fn default_port(self) -> u16 {
        match self {
            HttpScheme::Http => 80,
            HttpScheme::Https => 443,
        }
    }
}

/// A normalized host name (or bracketed IPv6 literal) of a protection space.
///
/// [`Host::new`] validates and normalizes:
///
/// - surrounding ASCII whitespace is trimmed;
/// - the host is lowercased, so `Example.COM` and `example.com` name the same
///   authority;
/// - only `[a-z0-9._-]` remain; URL delimiters and whitespace that would
///   make the key ambiguous with a port, path, query, fragment, userinfo, or
///   encoding are rejected;
/// - the length is bounded by [`MAX_HOST_LEN`].
///
/// Bracketed IPv6 literals (`[::1]`, `[fe80::1%eth0]`) are accepted
/// verbatim: colons, dots, percent, underscore, and hyphen are legal **only**
/// inside the brackets, where they cannot be mistaken for a port or URL
/// delimiter. The adapter passes the host exactly as the engine reports it.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Host(String);

impl Host {
    /// Validate and normalize `raw` into a canonical host.
    pub fn new(raw: &str) -> Result<Self, ProtectionSpaceError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(ProtectionSpaceError::EmptyHost);
        }
        if trimmed.len() > MAX_HOST_LEN {
            return Err(ProtectionSpaceError::HostTooLong);
        }
        validate_host(trimmed)?;
        Ok(Host(trimmed.to_ascii_lowercase()))
    }

    /// The normalized host string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Characters allowed in a DNS-style host: ASCII alphanumerics and DNS
/// punctuation. Everything else (whitespace, controls, `: / ? # @ % [ ]`)
/// is rejected so the host can never be ambiguous with a port, path, query,
/// fragment, userinfo, or percent-encoding.
fn is_host_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_')
}

/// Characters allowed inside a bracketed IPv6 literal: hex digits plus the
/// IPv6 separators and zone-id characters. Bracket context is what keeps
/// these unambiguous with port or URL delimiters.
fn is_ipv6_inner_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b':' | b'.' | b'%' | b'_' | b'-')
}

/// Validate a trimmed host: either a DNS-style name, or a bracketed IPv6
/// literal (`[…]`) whose inner characters are restricted to the IPv6 set.
fn validate_host(host: &str) -> Result<(), ProtectionSpaceError> {
    let bytes = host.as_bytes();
    if bytes.first() == Some(&b'[') {
        // Bracketed IPv6 literal: `[` at the start, `]` at the end, nothing
        // but IPv6 characters in between.
        if bytes.last() != Some(&b']') {
            return Err(ProtectionSpaceError::InvalidHost);
        }
        let inner = &bytes[1..bytes.len() - 1];
        if inner.is_empty() || inner.iter().any(|b| !is_ipv6_inner_byte(*b)) {
            return Err(ProtectionSpaceError::InvalidHost);
        }
        Ok(())
    } else if bytes.iter().any(|b| !is_host_byte(*b)) {
        Err(ProtectionSpaceError::InvalidHost)
    } else {
        Ok(())
    }
}

impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Host {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct HostVisitor;

        impl<'de> serde::de::Visitor<'de> for HostVisitor {
            type Value = Host;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "a validated host string")
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Host, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let raw = String::deserialize(deserializer)?;
                Host::new(&raw).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_newtype_struct("Host", HostVisitor)
    }
}

/// A protection-space realm label (the `realm` parameter of the
/// `WWW-Authenticate` / `Proxy-Authenticate` header).
///
/// Realms are opaque, case-sensitive labels per RFC 7617: the contract only
/// trims surrounding whitespace, requires non-emptiness, and bounds the
/// length. Case is preserved.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Realm(String);

impl Realm {
    /// Validate a realm label: trim, require non-emptiness, bound the length.
    pub fn new(raw: &str) -> Result<Self, ProtectionSpaceError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(ProtectionSpaceError::EmptyRealm);
        }
        if trimmed.len() > MAX_REALM_LEN {
            return Err(ProtectionSpaceError::RealmTooLong);
        }
        Ok(Realm(trimmed.to_owned()))
    }

    /// The trimmed realm label, with case preserved.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Realm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Realm {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct RealmVisitor;

        impl<'de> serde::de::Visitor<'de> for RealmVisitor {
            type Value = Realm;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "a validated realm label")
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Realm, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let raw = String::deserialize(deserializer)?;
                Realm::new(&raw).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_newtype_struct("Realm", RealmVisitor)
    }
}

/// Which server issued the challenge: the origin that served the request, or
/// an intermediary proxy. Part of the protection-space key, so an origin and
/// a proxy never share a space.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthAuthority {
    /// The origin server that served the request.
    Origin,
    /// An intermediary proxy that intercepted the request.
    Proxy,
}

/// The validated protection space of one challenge: scheme, host, port,
/// realm, and authority.
///
/// Constructed via [`ProtectionSpace::new`], which normalizes the host,
/// canonicalizes the default port, and validates the realm. Deserialization
/// re-runs the same validation **and** normalization, so an invalid or
/// non-normalized space (for example `https` with an explicitly spelled
/// `port: 443`) can never enter the contract through the wire as a key
/// different from the constructor's canonical space.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProtectionSpace {
    scheme: HttpScheme,
    host: Host,
    port: u16,
    realm: Option<Realm>,
    authority: AuthAuthority,
}

impl ProtectionSpace {
    /// Build a validated protection space from raw values.
    ///
    /// - `host` is trimmed, lowercased, and checked ([`Host::new`]);
    /// - `realm` is trimmed and checked ([`Realm::new`]) when present;
    /// - `port` is canonicalized: the scheme's default port (80 for HTTP,
    ///   443 for HTTPS) is stored as `0` ("scheme default"), so a URL that
    ///   spells the default port and one that omits it produce the same key.
    ///   `0` is always accepted and means the scheme default.
    pub fn new(
        scheme: HttpScheme,
        host: &str,
        port: u16,
        realm: Option<&str>,
        authority: AuthAuthority,
    ) -> Result<Self, ProtectionSpaceError> {
        let host = Host::new(host)?;
        let realm = match realm {
            Some(raw) => Some(Realm::new(raw)?),
            None => None,
        };
        let port = if port == scheme.default_port() {
            0
        } else {
            port
        };
        Ok(Self {
            scheme,
            host,
            port,
            realm,
            authority,
        })
    }

    /// The HTTP scheme of the space.
    pub fn scheme(&self) -> HttpScheme {
        self.scheme
    }

    /// The normalized host.
    pub fn host(&self) -> &Host {
        &self.host
    }

    /// The port, with the scheme's default port canonicalized to `0`
    /// ("scheme default").
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The realm label, when the challenge carried one.
    pub fn realm(&self) -> Option<&Realm> {
        self.realm.as_ref()
    }

    /// Which authority issued the challenge: origin or proxy.
    pub fn authority(&self) -> AuthAuthority {
        self.authority
    }

    /// Whether this space belongs to an intermediary proxy.
    pub fn is_proxy(&self) -> bool {
        self.authority == AuthAuthority::Proxy
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for ProtectionSpace {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawProtectionSpace {
            scheme: HttpScheme,
            host: Host,
            port: u16,
            realm: Option<Realm>,
            authority: AuthAuthority,
        }

        // A plain field-wise derive would accept a non-normalized space
        // (e.g. https with an explicitly spelled `port: 443`), producing a
        // key unequal to `ProtectionSpace::new`'s canonical `port: 0` for
        // the same authority. Route through the constructor so validation
        // *and* normalization are re-applied on the wire.
        let raw = RawProtectionSpace::deserialize(deserializer)?;
        ProtectionSpace::new(
            raw.scheme,
            raw.host.as_str(),
            raw.port,
            raw.realm.as_ref().map(Realm::as_str),
            raw.authority,
        )
        .map_err(serde::de::Error::custom)
    }
}
