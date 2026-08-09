//! Portable Saved Page contract: stable identities, the one canonical-URL
//! equality seam, and the durable record shape.
//!
//! A Saved Page is a **global durable URL record** (product decision
//! 2026-08-08): it carries no [`TabId`](crate::TabId), `WebViewId`, native
//! session, Space, Profile, cookie, credential, history, snapshot, or
//! favicon blob. Saving a pinned/Favorite tab copies only the current
//! navigable URL and title; no identity link to that tab remains. It
//! survives closing tabs, spaces, windows, and restarting the browser.
//!
//! This module is the **portable contract only** — types, validation, the
//! canonical seam, and deterministic ordering. Durability is the
//! `bbr-persistence` repository's concern (schema v3), and B2 owns the app
//! composition that reads/writes through that repository. Nothing here
//! imports GPUI, SQLite, AppKit/WebKit, or native pointers
//! (`#![forbid(unsafe_code)]` is crate-wide).
//!
//! # The canonical-URL equality seam
//!
//! The settled B1 URL decision (Current delivery, standing decisions):
//! canonicalize through [`bbr_web::normalize_url`] (the existing
//! navigation authority's scheme defaulting), then ASCII-case-fold only the
//! scheme and host. Path, query, fragment, explicit port, trailing slash,
//! `www.`, and percent-encoding are preserved **byte-for-byte**. No default
//! port is elided and no bookmark-only rewrite is added, so two strings
//! that name the same navigable destination produce the same canonical key
//! exactly when they differ only in scheme/host casing.
//!
//! Unsupported, dangerous, or malformed navigable URLs reject whole:
//!
//! - the scheme must be one of the navigable set `http`, `https`, or
//!   `about` (BBR's product-navigation schemes); `javascript:`, `data:`,
//!   `file:`, `mailto:`, custom schemes, and every other scheme reject;
//! - userinfo credentials in the authority (`https://user:pass@host`)
//!   reject, so a Saved Page record is **credential-free by construction**
//!   and its `Debug` output is privacy-safe;
//! - protocol-relative input (`//host`), empty input, and any NUL,
//!   whitespace, or control byte reject.
//!
//! Because the key is deterministic, save/lookup/remove/search share exactly
//! this one equality authority; persistence stores the canonical key
//! opaquely and enforces uniqueness (at most one active record per key).

use std::fmt;

use bbr_web::normalize_url;

/// The navigable schemes a Saved Page record may carry (product navigation
/// authority: BBR navigates `http`/`https` pages and its own `about:`
/// surfaces such as `about:blank`). Everything else — including
/// `javascript:`, `data:`, `file:`, and `mailto:` — is unsupported or
/// dangerous for a durable global page record and rejects whole.
const SUPPORTED_SCHEMES: &[&str] = &["http", "https", "about"];

/// Stable global Saved Page identity (schema v3).
///
/// A distinct id space from profile/space/tab ids (the [`crate::FolderId`]
/// precedent): numeric equality with another kind is harmless because the
/// types never mix. The repository persists it as the `saved_pages` primary
/// key; B2's provider returns it to callers as the stable record identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BookmarkId(pub u64);

/// Stable flat-collection identity (schema v3).
///
/// Saved Pages belong to at most one flat collection; `CollectionId` names
/// that optional membership target. Collections are flat (no nesting, no
/// smart collections) per the B1/B3 product boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CollectionId(pub u64);

/// The one canonical-URL equality seam for Saved Pages (see module docs).
///
/// A `CanonicalUrl` is always navigable, scheme/host case-folded, and free
/// of credentials by construction: it validates through
/// [`bbr_web::normalize_url`] and the supported-scheme/userinfo rules in
/// [`canonical_url`]. `Display`/`as_str`/`into_string` yield the canonical
/// navigable string, which persistence stores opaquely as the duplicate-
/// rejection key. `Debug` prints that same string — privacy-safe because
/// userinfo is rejected at construction.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalUrl(String);

impl CanonicalUrl {
    /// The canonical navigable string (the persisted duplicate key).
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the value and returns the canonical navigable string.
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Debug for CanonicalUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The canonical form never embeds credentials (userinfo is rejected
        // at construction), so printing the string is privacy-safe.
        f.debug_tuple("CanonicalUrl").field(&self.0).finish()
    }
}

impl fmt::Display for CanonicalUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<&str> for CanonicalUrl {
    type Error = SavedPageError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        canonical_url(input).map(Self)
    }
}

/// A flat Saved Page collection (schema v3).
///
/// Titles are bounded and validated; membership is stored on the record
/// (at most one collection per Saved Page). The repository owns
/// create/rename/remove; this is the portable validated value shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collection {
    pub id: CollectionId,
    pub title: String,
}

/// The complete Saved Page record (B1 field contract, schema v3).
///
/// Every field is portable and validated; the record is global
/// installation data — no profile, space, tab, session, or native identity
/// is present by construction. Re-saving the same canonical URL is
/// idempotent and returns the existing [`BookmarkId`]; see the repository
/// contract for the exact duplicate semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedPage {
    /// Stable global identity; persisted as the primary key.
    pub id: BookmarkId,
    /// The canonical navigable URL — the duplicate-rejection key and the
    /// value B2 navigates when opening the page.
    pub url: CanonicalUrl,
    /// Display title captured at save time; may be empty.
    pub captured_title: String,
    /// Explicit user-provided label; `None` when the user never set one.
    pub user_label: Option<String>,
    /// Deterministic creation timestamp (persisted integer range).
    pub created_at: u64,
    /// Deterministic last-mutation timestamp; always `>= created_at`.
    pub updated_at: u64,
    /// Optional flat collection membership (at most one).
    pub collection: Option<CollectionId>,
}

/// Bounded length limits for Saved Page text fields (schema v3).
///
/// These are the contract bounds the domain validates and persistence
/// mirrors; they are never weakened later.
pub const MAX_CAPTURED_TITLE_LEN: usize = 1024;
pub const MAX_USER_LABEL_LEN: usize = 256;
pub const MAX_COLLECTION_TITLE_LEN: usize = 128;

impl SavedPage {
    /// Validates every field of the record against the portable contract.
    ///
    /// Cross-record invariants (unique canonical URLs, collection
    /// membership referencing a known collection) are store-level and are
    /// enforced by the repository, not here.
    pub fn validate(&self) -> Result<(), SavedPageError> {
        validate_bookmark_id(self.id)?;
        validate_timestamp(self.created_at, "created_at")?;
        validate_timestamp(self.updated_at, "updated_at")?;
        if self.updated_at < self.created_at {
            return Err(SavedPageError::InvalidTimestamp(
                "updated_at precedes created_at".into(),
            ));
        }
        validate_title(&self.captured_title)?;
        validate_label(self.user_label.as_deref())?;
        Ok(())
    }

    /// The deterministic sort key: `(updated_at, id)` ascending.
    ///
    /// Records with equal titles, URLs, timestamps, and collection
    /// membership still order deterministically through the `id` tiebreak;
    /// [`deterministic_order`] sorts a slice by this key. B2/B3 may present
    /// in the opposite direction but must never invent a second sort rule.
    pub fn sort_key(&self) -> (u64, u64) {
        (self.updated_at, self.id.0)
    }

    /// Case-insensitive substring match over title, label, and canonical
    /// URL — the deterministic in-memory search predicate B2's provider
    /// consumes over one loaded snapshot (no synchronous repository query
    /// from ranking or render).
    pub fn matches(&self, query: &str) -> bool {
        let needle = query.to_lowercase();
        if needle.is_empty() {
            return true;
        }
        self.captured_title.to_lowercase().contains(&needle)
            || self
                .user_label
                .as_deref()
                .is_some_and(|label| label.to_lowercase().contains(&needle))
            || self.url.0.to_lowercase().contains(&needle)
    }
}

/// Sorts a slice of Saved Pages into the deterministic contract order:
/// ascending `(updated_at, id)` ([`SavedPage::sort_key`]). The sort is
/// stable, so equal keys keep their prior relative order.
pub fn deterministic_order(pages: &mut [SavedPage]) {
    pages.sort_by_key(SavedPage::sort_key);
}

/// Errors returned by Saved Page value validation and the canonical seam.
///
/// Messages never embed the raw input URL (a malformed input could carry
/// credentials), so `Debug`/`Display` output stays privacy-safe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SavedPageError {
    /// The URL is empty, malformed, or not navigable (no scheme after
    /// defaulting, protocol-relative input, or control bytes).
    InvalidUrl(String),
    /// The URL's scheme is not navigable for a durable Saved Page.
    UnsupportedScheme(String),
    /// The URL embeds credentials in the authority (userinfo); records must
    /// be credential-free by construction.
    CredentialsNotAllowed,
    /// An identity is out of the persisted range (`1..=i64::MAX`).
    InvalidId(String),
    /// A title/label/collection title violates a bound.
    InvalidTitle(String),
    /// A timestamp violates the persisted-integer or monotonicity contract.
    InvalidTimestamp(String),
}

impl fmt::Display for SavedPageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUrl(detail) => write!(f, "invalid Saved Page URL: {detail}"),
            Self::UnsupportedScheme(scheme) => {
                write!(f, "unsupported Saved Page URL scheme `{scheme}`")
            }
            Self::CredentialsNotAllowed => {
                f.write_str("Saved Page URLs must not embed credentials (userinfo)")
            }
            Self::InvalidId(detail) => write!(f, "invalid Saved Page identity: {detail}"),
            Self::InvalidTitle(detail) => write!(f, "invalid Saved Page title/label: {detail}"),
            Self::InvalidTimestamp(detail) => write!(f, "invalid Saved Page timestamp: {detail}"),
        }
    }
}

impl std::error::Error for SavedPageError {}

/// The maximum persisted SQLite identity (`i64::MAX`).
pub const MAX_ID: u64 = i64::MAX as u64;

/// The canonicalization authority every Saved Page save/lookup/remove/search
/// shares. See the module docs for the exact settled semantics.
pub fn canonical_url(input: &str) -> Result<String, SavedPageError> {
    // The existing navigation authority defaults the scheme (trims, passes
    // through explicit schemes and `//`-prefixed input, otherwise prepends
    // `https://`).
    let normalized = normalize_url(input);
    if normalized.is_empty() {
        return Err(SavedPageError::InvalidUrl("empty URL".into()));
    }
    if normalized
        .bytes()
        .any(|byte| byte == 0 || byte.is_ascii_whitespace() || byte.is_ascii_control())
    {
        return Err(SavedPageError::InvalidUrl(
            "NUL, whitespace, or control bytes".into(),
        ));
    }
    let colon = normalized
        .find(':')
        .ok_or_else(|| SavedPageError::InvalidUrl("no scheme".into()))?;
    let scheme = &normalized[..colon];
    if scheme.is_empty()
        || !scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
    {
        return Err(SavedPageError::InvalidUrl("malformed scheme".into()));
    }
    let scheme = scheme.to_ascii_lowercase();
    if !SUPPORTED_SCHEMES.contains(&scheme.as_str()) {
        return Err(SavedPageError::UnsupportedScheme(scheme));
    }
    let rest = &normalized[colon + 1..];
    let mut canonical = String::with_capacity(normalized.len());
    canonical.push_str(&scheme);
    canonical.push(':');
    match rest.strip_prefix("//") {
        Some(authority_and_tail) => {
            canonical.push_str("//");
            // Authority ends at the first path/query/fragment separator.
            let authority_end = authority_and_tail
                .find(['/', '?', '#'])
                .unwrap_or(authority_and_tail.len());
            let (authority, tail) = authority_and_tail.split_at(authority_end);
            if authority.contains('@') {
                // Userinfo would embed credentials in the durable record.
                return Err(SavedPageError::CredentialsNotAllowed);
            }
            // ASCII-case-fold only the host. Port digits are unaffected;
            // percent-escape sequences are preserved byte-for-byte; a
            // bracketed IPv6 literal's hex letters are folded as host case.
            canonical.push_str(&fold_host(authority));
            canonical.push_str(tail);
        }
        None => {
            // Opaque scheme value (`about:blank`): no authority, preserved
            // byte-for-byte after the folded scheme.
            canonical.push_str(rest);
        }
    }
    Ok(canonical)
}

/// ASCII-case-folds a host while preserving `%XX` percent-escape sequences
/// byte-for-byte (the settled "preserve percent-encoding byte-for-byte"
/// rule applies to the host too).
fn fold_host(host: &str) -> String {
    let bytes = host.as_bytes();
    let mut folded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && bytes[index + 1].is_ascii_hexdigit()
            && bytes[index + 2].is_ascii_hexdigit()
        {
            folded.extend_from_slice(&bytes[index..index + 3]);
            index += 3;
        } else {
            folded.push(bytes[index].to_ascii_lowercase());
            index += 1;
        }
    }
    // Only ASCII bytes were replaced by ASCII bytes, so the output is the
    // same UTF-8 string the input was.
    String::from_utf8(folded).expect("ASCII case-folding preserves UTF-8")
}

pub(crate) fn validate_bookmark_id(id: BookmarkId) -> Result<(), SavedPageError> {
    if id.0 == 0 || id.0 > MAX_ID {
        Err(SavedPageError::InvalidId(format!("BookmarkId {}", id.0)))
    } else {
        Ok(())
    }
}

pub(crate) fn validate_collection_id(id: CollectionId) -> Result<(), SavedPageError> {
    if id.0 == 0 || id.0 > MAX_ID {
        Err(SavedPageError::InvalidId(format!("CollectionId {}", id.0)))
    } else {
        Ok(())
    }
}

/// Bounded title validation: may be empty, no NUL/control bytes, bounded
/// length. `MAX_TITLE` is the caller's chosen cap (captured titles may be
/// empty; collection titles are separately required non-empty).
fn validate_bounded_text(
    text: &str,
    max_len: usize,
    allow_empty: bool,
) -> Result<(), SavedPageError> {
    if !allow_empty && text.is_empty() {
        return Err(SavedPageError::InvalidTitle("empty text".into()));
    }
    if text.len() > max_len {
        return Err(SavedPageError::InvalidTitle(format!(
            "exceeds {max_len} bytes"
        )));
    }
    if text
        .bytes()
        .any(|byte| byte == 0 || byte.is_ascii_control())
    {
        return Err(SavedPageError::InvalidTitle("control bytes".into()));
    }
    Ok(())
}

fn validate_title(title: &str) -> Result<(), SavedPageError> {
    validate_bounded_text(title, MAX_CAPTURED_TITLE_LEN, true)
}

fn validate_label(label: Option<&str>) -> Result<(), SavedPageError> {
    match label {
        None => Ok(()),
        Some(label) => validate_bounded_text(label, MAX_USER_LABEL_LEN, false),
    }
}

/// Validates a collection record's id and bounded, non-empty title.
pub fn validate_collection(collection: &Collection) -> Result<(), SavedPageError> {
    validate_collection_id(collection.id)?;
    validate_bounded_text(&collection.title, MAX_COLLECTION_TITLE_LEN, false)
}

fn validate_timestamp(timestamp: u64, field: &str) -> Result<(), SavedPageError> {
    if timestamp > MAX_ID {
        Err(SavedPageError::InvalidTimestamp(format!(
            "{field} exceeds the SQLite integer range"
        )))
    } else {
        Ok(())
    }
}
