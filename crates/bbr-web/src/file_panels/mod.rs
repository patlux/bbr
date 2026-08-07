//! # File upload / save-panel — portable request contract
//!
//! Portable request vocabulary for web-engine file uploads (`<input
//! type="file">`) and save destinations, owned by
//! `docs/roadmap/tasks/file-panels-contract.md`. Pure `std`: no frameworks,
//! GPUI, `unsafe`, file I/O, network, or new dependencies. AppKit/WebKit
//! panels, paths, sandbox access, and bookmarks are later adapter work.
//!
//! ## Requests and metadata
//!
//! A request is a plain value: a stable [`FileRequestId`] owned by a
//! [`ProfileId`]/[`WebViewId`], a [`FileRequestKind`] (upload or save), a
//! [`RequestContext`] (optional page origin and user-gesture flag), and
//! typed metadata: upload carries [`AcceptFilter`], [`SelectionCardinality`],
//! [`SelectionCapability`] (files versus directories), and a
//! [`CaptureHint`]; save carries basename, content type, and [`ReplacementPolicy`].
//!
//! [`validate_upload_request`] / [`validate_save_request`] are **pure**:
//! they reject nonsensical capability/cardinality/capture combinations and
//! illegal suggested names without touching a filesystem or the platform.
//! Malformed accept filters are unrepresentable: [`AcceptFilter::new`] and
//! its serde `Deserialize` impl reject them at construction.
//!
//! ## Opaque adapter-owned tokens
//!
//! [`UploadSelectionToken`] and [`SaveDestinationToken`] are the values an
//! adapter returns when the user picks files or a destination. They are
//! **opaque handles**: portable code may create, compare, order, hash,
//! serialize, and return them, but they expose no path, `file://` URL,
//! bookmark, file descriptor, native pointer, or platform payload. The two
//! kinds are distinct types: cross-kind use is a compile-time error. The
//! **adapter owns sandbox lifetime**: holding a token grants portable code
//! no sandbox-extension right and no ability to open, read, or write.
//!
//! ## Lifecycle and port
//!
//! The deterministic completion lifecycle (pending, selection requested,
//! selected, cancelled, failed, superseded) and the object-safe
//! [`FilePanelPort`] live in [`lifecycle`]; non-goals stay: no paths,
//! panel presentation, selection persistence, or [`crate::WebEngine`] edits.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::fmt;

use crate::{ProfileId, WebViewId};

pub mod lifecycle;
pub use lifecycle::*;

/// Stable identity of one file-panel request (upload or save). Never reused
/// once its lifecycle resolves.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileRequestId(pub u64);

/// Which panel a request asks the adapter to present.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileRequestKind {
    /// An `<input type="file">`-style open selection.
    Upload,
    /// A save-destination selection.
    Save,
}

/// Engine-provided context attached to every request.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestContext {
    /// Page origin (`scheme://host[:port]`) that initiated the request,
    /// when known. Informational only; never a filesystem path.
    pub origin: Option<String>,
    /// Whether the request was initiated by a user gesture.
    pub user_gesture: bool,
}

impl RequestContext {
    pub const fn new(origin: Option<String>, user_gesture: bool) -> Self {
        Self {
            origin,
            user_gesture,
        }
    }
}

/// Accepted content for an upload: MIME filters and/or filename extensions,
/// mirroring the HTML `accept` attribute grammar.
///
/// Invariant-holding: every entry is validated by [`AcceptFilter::new`] and
/// the serde `Deserialize` impl, so invalid filters are unrepresentable; an
/// empty list accepts any file.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AcceptFilter {
    mime_types: Vec<String>,
    extensions: Vec<String>,
}

/// Why [`AcceptFilter::new`] rejected an entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidAcceptFilter {
    /// Not a legal accept-attribute MIME filter (`*`, `type/*`, or
    /// `type/subtype`).
    InvalidMimeType(String),
    /// Not a legal filename extension: must start with `.`, be non-empty
    /// after it, and contain no path separator, whitespace, or control
    /// character.
    InvalidExtension(String),
}

impl fmt::Display for InvalidAcceptFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMimeType(value) => write!(f, "invalid accept MIME filter {value:?}"),
            Self::InvalidExtension(value) => write!(f, "invalid accept extension {value:?}"),
        }
    }
}

impl std::error::Error for InvalidAcceptFilter {}

impl AcceptFilter {
    /// Accept any file (empty filter lists).
    pub fn any() -> Self {
        Self::default()
    }

    /// Validate and store the filters; returns the first invalid entry.
    pub fn new(
        mime_types: Vec<String>,
        extensions: Vec<String>,
    ) -> Result<Self, InvalidAcceptFilter> {
        for mime in &mime_types {
            if !is_valid_mime_filter(mime) {
                return Err(InvalidAcceptFilter::InvalidMimeType(mime.clone()));
            }
        }
        for extension in &extensions {
            if !is_valid_file_extension(extension) {
                return Err(InvalidAcceptFilter::InvalidExtension(extension.clone()));
            }
        }
        Ok(Self {
            mime_types,
            extensions,
        })
    }

    /// Accepted MIME filters, in request order.
    pub fn mime_types(&self) -> &[String] {
        &self.mime_types
    }

    /// Accepted filename extensions, in request order.
    pub fn extensions(&self) -> &[String] {
        &self.extensions
    }

    /// True when nothing is filtered (every file is accepted).
    pub fn accepts_anything(&self) -> bool {
        self.mime_types.is_empty() && self.extensions.is_empty()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for AcceptFilter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            mime_types: Vec<String>,
            extensions: Vec<String>,
        }
        let raw = Raw::deserialize(deserializer)?;
        AcceptFilter::new(raw.mime_types, raw.extensions).map_err(serde::de::Error::custom)
    }
}

/// Whether a string is a legal accept-attribute MIME filter: `*`, `*/*`,
/// `type/*`, or `type/subtype` (case-insensitive tokens, no parameters).
pub fn is_valid_mime_filter(value: &str) -> bool {
    if value == "*" {
        return true;
    }
    let Some((kind, subtype)) = value.split_once('/') else {
        return false;
    };
    if kind == "*" {
        return subtype == "*";
    }
    is_mime_token(kind) && (subtype == "*" || is_mime_token(subtype))
}

/// Legal accept filename extension: dot-prefixed body of ASCII
/// alphanumerics/`.`/`-`/`_` with no path separator, whitespace, or
/// control character; `.tar.gz` is legal, bare names and `/` are not.
pub fn is_valid_file_extension(value: &str) -> bool {
    let Some(body) = value.strip_prefix('.') else {
        return false;
    };
    if body.is_empty() || body == "." || body == ".." {
        return false;
    }
    body.bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
}

/// RFC 6838 MIME token: non-empty ASCII alphanumerics plus `!#$&^_.+-`.
fn is_mime_token(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|b| {
            b.is_ascii_alphanumeric()
                || matches!(
                    b,
                    b'!' | b'#' | b'$' | b'&' | b'^' | b'_' | b'.' | b'+' | b'-'
                )
        })
}

/// How many items one upload may select.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionCardinality {
    #[default]
    Single,
    Multiple,
}

/// What kinds of items an upload may select; at least one flag must be
/// true ([`UploadRejection::NothingSelectable`]).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionCapability {
    pub files: bool,
    pub directories: bool,
}

impl Default for SelectionCapability {
    fn default() -> Self {
        Self {
            files: true,
            directories: false,
        }
    }
}

/// Capture intent for an upload (`<input capture>`), where representable.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CaptureHint {
    /// No capture intent; a normal file selection.
    #[default]
    Any,
    /// Capture from the user-facing camera/microphone.
    User,
    /// Capture from the outward-facing camera.
    Environment,
}

/// A request to present an upload (open) selection panel.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadRequest {
    pub id: FileRequestId,
    /// Owning browsing profile.
    pub profile: ProfileId,
    /// Owning web view.
    pub web_view: WebViewId,
    pub context: RequestContext,
    pub accept: AcceptFilter,
    pub cardinality: SelectionCardinality,
    pub capability: SelectionCapability,
    pub capture: CaptureHint,
}

/// Why [`validate_upload_request`] rejected the request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UploadRejection {
    /// Neither files nor directories are selectable.
    NothingSelectable,
    /// A capture hint combined with multi-selection: a capture produces
    /// exactly one item.
    CaptureRequiresSingle {
        cardinality: SelectionCardinality,
        capture: CaptureHint,
    },
    /// A capture hint combined with directory selection.
    CaptureWithDirectories,
}

/// Pure validation of an upload request; no filesystem or native behavior.
pub fn validate_upload_request(request: &UploadRequest) -> Result<(), UploadRejection> {
    if !request.capability.files && !request.capability.directories {
        return Err(UploadRejection::NothingSelectable);
    }
    if matches!(
        request.capture,
        CaptureHint::User | CaptureHint::Environment
    ) {
        if request.capability.directories {
            return Err(UploadRejection::CaptureWithDirectories);
        }
        if request.cardinality == SelectionCardinality::Multiple {
            return Err(UploadRejection::CaptureRequiresSingle {
                cardinality: request.cardinality,
                capture: request.capture,
            });
        }
    }
    Ok(())
}

/// What may happen when the chosen save destination already exists.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReplacementPolicy {
    /// Overwrite the existing file.
    AllowOverwrite,
    /// Do not overwrite; surface the conflict to the user.
    RejectExisting,
    /// Resolve the conflict (for example by uniquifying) and report the final destination.
    #[default]
    Uniquify,
}

/// A request to present a save-destination selection. A save panel is
/// single-selection, so there is no cardinality field; a save cannot
/// complete without a [`SaveDestinationToken`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveRequest {
    pub id: FileRequestId,
    pub profile: ProfileId,
    pub web_view: WebViewId,
    pub context: RequestContext,
    /// Suggested basename for the file, if the engine has one. `None` lets
    /// the adapter derive a name. Never a path: separators are rejected by
    /// [`is_valid_suggested_name`].
    pub suggested_name: Option<String>,
    /// Suggested concrete content type, if known.
    pub content_type: Option<String>,
    pub replacement: ReplacementPolicy,
}

/// Why [`validate_save_request`] rejected the request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveRejection {
    /// `suggested_name` is present but is not a legal basename (empty,
    /// whitespace-only, `.`/`..`, a path separator, or a control char).
    InvalidSuggestedName(String),
    /// `content_type` is present but is not a concrete `type/subtype`
    /// MIME type (wildcards are not meaningful for a save).
    InvalidContentType(String),
}

/// Legal suggested save basename: non-empty, not `.`/`..`, free of path
/// separators and control characters; internal spaces are fine.
pub fn is_valid_suggested_name(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
        return false;
    }
    !trimmed
        .bytes()
        .any(|b| b == b'/' || b == b'\\' || b.is_ascii_control())
}

/// Concrete MIME type: exactly `type/subtype` with two non-wildcard RFC
/// 6838 tokens; wildcards and bare `*` are invalid here.
pub fn is_valid_content_type(value: &str) -> bool {
    let Some((kind, subtype)) = value.split_once('/') else {
        return false;
    };
    kind != "*" && subtype != "*" && is_mime_token(kind) && is_mime_token(subtype)
}

/// Pure validation of a save request: no filesystem or native behavior.
pub fn validate_save_request(request: &SaveRequest) -> Result<(), SaveRejection> {
    if let Some(name) = &request.suggested_name {
        if !is_valid_suggested_name(name) {
            return Err(SaveRejection::InvalidSuggestedName(name.clone()));
        }
    }
    if let Some(content_type) = &request.content_type {
        if !is_valid_content_type(content_type) {
            return Err(SaveRejection::InvalidContentType(content_type.clone()));
        }
    }
    Ok(())
}

/// One typed file-panel request together with its kind and owning
/// profile/view.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileRequest {
    Upload(UploadRequest),
    Save(SaveRequest),
}

impl FileRequest {
    pub fn id(&self) -> FileRequestId {
        match self {
            FileRequest::Upload(request) => request.id,
            FileRequest::Save(request) => request.id,
        }
    }

    pub fn kind(&self) -> FileRequestKind {
        match self {
            FileRequest::Upload(_) => FileRequestKind::Upload,
            FileRequest::Save(_) => FileRequestKind::Save,
        }
    }

    pub fn profile(&self) -> ProfileId {
        match self {
            FileRequest::Upload(request) => request.profile,
            FileRequest::Save(request) => request.profile,
        }
    }

    pub fn web_view(&self) -> WebViewId {
        match self {
            FileRequest::Upload(request) => request.web_view,
            FileRequest::Save(request) => request.web_view,
        }
    }

    /// Pure validation; see [`validate_upload_request`] and
    /// [`validate_save_request`].
    pub fn validate(&self) -> Result<(), FileRequestRejection> {
        match self {
            FileRequest::Upload(request) => {
                validate_upload_request(request).map_err(FileRequestRejection::Upload)
            }
            FileRequest::Save(request) => {
                validate_save_request(request).map_err(FileRequestRejection::Save)
            }
        }
    }
}

/// Kind-typed rejection of a [`FileRequest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileRequestRejection {
    Upload(UploadRejection),
    Save(SaveRejection),
}

/// Opaque adapter-issued handle identifying one completed upload selection.
///
/// [`new`](Self::new) wraps an adapter-private handle value; the serialized
/// form is a bare integer, so no path or platform payload can ride along.
/// The **adapter owns sandbox lifetime**: portable code holds no
/// sandbox-extension right through this token.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UploadSelectionToken(u64);

impl UploadSelectionToken {
    /// Wrap an adapter-owned opaque handle. Not a path, filesystem index,
    /// or native pointer.
    pub const fn new(handle: u64) -> Self {
        Self(handle)
    }
}

/// Opaque adapter-issued handle identifying one chosen save destination.
///
/// Same contract as [`UploadSelectionToken`], as a **distinct type**: an
/// upload selection can never complete a save request.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SaveDestinationToken(u64);

impl SaveDestinationToken {
    /// Wrap an adapter-owned opaque handle. Not a path, filesystem index,
    /// or native pointer.
    pub const fn new(handle: u64) -> Self {
        Self(handle)
    }
}

#[cfg(test)]
mod tests_contract;

#[cfg(test)]
mod tests_lifecycle;

#[cfg(test)]
mod tests_port;

#[cfg(all(test, feature = "serde"))]
mod tests_serde;
