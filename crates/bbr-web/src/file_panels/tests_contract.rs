//! Request-layer contract tests: identifiers, metadata, pure validation
//! (accept filters, cardinality, capability, suggested names, content
//! types), and opaque-token separation — all without filesystem or native
//! types.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::file_panels::{
    is_valid_content_type, is_valid_file_extension, is_valid_mime_filter, is_valid_suggested_name,
    validate_save_request, validate_upload_request, AcceptFilter, CaptureHint, FileRequest,
    FileRequestId, FileRequestKind, FileRequestRejection, InvalidAcceptFilter, ReplacementPolicy,
    RequestContext, SaveDestinationToken, SaveRejection, SaveRequest, SelectionCapability,
    SelectionCardinality, UploadRejection, UploadRequest, UploadSelectionToken,
};
use crate::{ProfileId, WebViewId};

fn id(n: u64) -> FileRequestId {
    FileRequestId(n)
}

fn profile(n: u64) -> ProfileId {
    ProfileId(n)
}

fn view(n: u64) -> WebViewId {
    WebViewId(n)
}

fn context() -> RequestContext {
    RequestContext::new(Some("https://example.com".into()), true)
}

/// Canonical well-formed single-file upload request.
fn upload_request() -> UploadRequest {
    UploadRequest {
        id: id(1),
        profile: profile(1),
        web_view: view(1),
        context: context(),
        accept: AcceptFilter::any(),
        cardinality: SelectionCardinality::Single,
        capability: SelectionCapability::default(),
        capture: CaptureHint::Any,
    }
}

fn save_request() -> SaveRequest {
    SaveRequest {
        id: id(2),
        profile: profile(1),
        web_view: view(1),
        context: context(),
        suggested_name: Some("report.pdf".into()),
        content_type: Some("application/pdf".into()),
        replacement: ReplacementPolicy::Uniquify,
    }
}

/// Identifiers are plain ordered values usable as `BTreeMap`/`BTreeSet`
/// keys and `HashSet` elements, mirroring the `ProfileId`/`WebViewId`
/// guard in `lib.rs`.
#[test]
fn file_request_ids_are_ordered_and_hashable() {
    let mut requests = BTreeMap::new();
    requests.insert(id(1), "upload");
    requests.insert(id(2), "save");
    assert_eq!(requests.get(&id(2)), Some(&"save"));

    let mut done = BTreeSet::new();
    done.insert(id(3));
    assert!(done.contains(&id(3)));
    assert!(!done.contains(&id(4)));

    let mut seen = HashSet::new();
    seen.insert(id(9));
    assert!(seen.contains(&id(9)));
    assert_eq!(id(1), id(1));
    assert_ne!(id(1), id(2));
    assert!(id(1) < id(2));
}

/// Every request carries its kind, stable id, and owning profile/view, so
/// later completion routing never needs to guess ownership.
#[test]
fn file_request_kind_identity_and_ownership() {
    let upload = FileRequest::Upload(upload_request());
    assert_eq!(upload.kind(), FileRequestKind::Upload);
    assert_eq!(upload.id(), id(1));
    assert_eq!(upload.profile(), profile(1));
    assert_eq!(upload.web_view(), view(1));
    assert!(matches!(upload.validate(), Ok(())));

    let save = FileRequest::Save(save_request());
    assert_eq!(save.kind(), FileRequestKind::Save);
    assert_eq!(save.id(), id(2));
    assert_eq!(save.profile(), profile(1));
    assert_eq!(save.web_view(), view(1));
    assert!(matches!(save.validate(), Ok(())));
}

/// The request context carries the optional origin and user-gesture flag
/// as informational metadata.
#[test]
fn request_context_carries_origin_and_gesture() {
    let gesture = context();
    assert_eq!(gesture.origin.as_deref(), Some("https://example.com"));
    assert!(gesture.user_gesture);

    let synthetic = RequestContext::new(None, false);
    assert_eq!(synthetic.origin, None);
    assert!(!synthetic.user_gesture);
    assert_ne!(gesture, synthetic);
}

/// Accept-attribute MIME filters follow the `*` / `type/*` /
/// `type/subtype` grammar; malformed entries are rejected.
#[test]
fn mime_filter_rules() {
    for valid in [
        "*",
        "*/*",
        "image/png",
        "text/*",
        "TEXT/HTML",
        "application/vnd.apple.keynote",
        "a-b/c_d",
    ] {
        assert!(is_valid_mime_filter(valid), "{valid:?} should be legal");
    }
    for invalid in [
        "",
        "image",
        "image/",
        "/png",
        "*/png",
        "image/pn g",
        "image/png;charset=utf-8",
        "image//png",
        "text /plain",
        "/",
    ] {
        assert!(
            !is_valid_mime_filter(invalid),
            "{invalid:?} should be rejected"
        );
    }
}

/// Extensions must be dot-prefixed basename tails without path separators,
/// whitespace, or control characters; `.tar.gz` is legal, bare names are
/// not.
#[test]
fn file_extension_rules() {
    for valid in [".png", ".jpeg", ".tar.gz", ".x-archive", "._data"] {
        assert!(is_valid_file_extension(valid), "{valid:?} should be legal");
    }
    for invalid in [
        "png", ".", "..", "...", "", ".exe ", ".a/b", ".p\nng", ".so\0",
    ] {
        assert!(
            !is_valid_file_extension(invalid),
            "{invalid:?} should be rejected"
        );
    }
}

/// `AcceptFilter` construction validates every entry, so an invalid filter
/// is unrepresentable; an empty filter list accepts anything.
#[test]
fn accept_filter_new_validates_entries() {
    let filter = AcceptFilter::new(
        vec!["image/png".into(), "text/*".into()],
        vec![".pdf".into()],
    )
    .expect("valid filters");
    assert_eq!(
        filter.mime_types(),
        &["image/png".to_string(), "text/*".to_string()]
    );
    assert_eq!(filter.extensions(), &[".pdf".to_string()]);
    assert!(!filter.accepts_anything());

    assert_eq!(
        AcceptFilter::new(vec!["not-a-mime".into()], vec![]),
        Err(InvalidAcceptFilter::InvalidMimeType("not-a-mime".into()))
    );
    assert_eq!(
        AcceptFilter::new(vec![], vec!["pdf".into()]),
        Err(InvalidAcceptFilter::InvalidExtension("pdf".into()))
    );
    assert_eq!(AcceptFilter::new(vec![], vec![]), Ok(AcceptFilter::any()));
    assert!(AcceptFilter::any().accepts_anything());
}

/// The canonical single-file upload passes pure validation.
#[test]
fn upload_validation_accepts_canonical_request() {
    assert_eq!(validate_upload_request(&upload_request()), Ok(()));
}

/// A picker where neither files nor directories are selectable is
/// nonsense and rejected.
#[test]
fn upload_validation_rejects_nothing_selectable() {
    let mut request = upload_request();
    request.capability = SelectionCapability {
        files: false,
        directories: false,
    };
    assert_eq!(
        validate_upload_request(&request),
        Err(UploadRejection::NothingSelectable)
    );
}

/// Capture targets a single camera/microphone item, so directory selection
/// is incompatible with a capture hint.
#[test]
fn upload_validation_rejects_capture_with_directories() {
    let mut request = upload_request();
    request.capability = SelectionCapability {
        files: true,
        directories: true,
    };
    request.capture = CaptureHint::Environment;
    assert_eq!(
        validate_upload_request(&request),
        Err(UploadRejection::CaptureWithDirectories)
    );
}

/// A capture produces exactly one item, so multiple selection is rejected
/// when a capture hint is present.
#[test]
fn upload_validation_rejects_capture_with_multiple_selection() {
    let mut request = upload_request();
    request.cardinality = SelectionCardinality::Multiple;
    request.capture = CaptureHint::User;
    assert_eq!(
        validate_upload_request(&request),
        Err(UploadRejection::CaptureRequiresSingle {
            cardinality: SelectionCardinality::Multiple,
            capture: CaptureHint::User,
        })
    );
}

/// A single-directory picker (folder upload) is legal: directories only,
/// single selection, no capture.
#[test]
fn upload_validation_allows_single_directory_selection() {
    let mut request = upload_request();
    request.capability = SelectionCapability {
        files: false,
        directories: true,
    };
    assert_eq!(validate_upload_request(&request), Ok(()));
}

/// A multiple-selection file upload without capture is legal.
#[test]
fn upload_validation_allows_multiple_file_selection() {
    let mut request = upload_request();
    request.cardinality = SelectionCardinality::Multiple;
    request.capture = CaptureHint::Any;
    assert_eq!(validate_upload_request(&request), Ok(()));
}

/// The canonical save request passes pure validation.
#[test]
fn save_validation_accepts_canonical_request() {
    assert_eq!(validate_save_request(&save_request()), Ok(()));
}

/// Suggested names must be legal basenames: non-empty after trimming, not
/// `.`/`..`, and free of path separators and control characters.
#[test]
fn save_validation_rejects_illegal_suggested_names() {
    for name in [
        "",
        "   ",
        ".",
        "..",
        "a/b.pdf",
        "a\\b.pdf",
        "report\n.pdf",
        "tab\t.pdf",
    ] {
        let mut request = save_request();
        request.suggested_name = Some(name.into());
        assert_eq!(
            validate_save_request(&request),
            Err(SaveRejection::InvalidSuggestedName(name.into())),
            "{name:?} should be rejected as a suggested name"
        );
    }
}

/// Internal spaces are legal in macOS filenames and must not be rejected.
#[test]
fn save_validation_accepts_names_with_internal_spaces() {
    let mut request = save_request();
    request.suggested_name = Some("my report.pdf".into());
    assert_eq!(validate_save_request(&request), Ok(()));
}

/// A save request without a suggestion or content type lets the adapter
/// derive both.
#[test]
fn save_validation_accepts_missing_metadata() {
    let mut request = save_request();
    request.suggested_name = None;
    request.content_type = None;
    assert_eq!(validate_save_request(&request), Ok(()));
}

/// Save content types must be concrete `type/subtype` MIME types;
/// wildcards and malformed values are rejected.
#[test]
fn save_validation_rejects_invalid_content_types() {
    for content_type in [
        "",
        "text",
        "text/",
        "/plain",
        "text/*",
        "*",
        "*/*",
        "text/pla in",
    ] {
        let mut request = save_request();
        request.content_type = Some(content_type.into());
        assert_eq!(
            validate_save_request(&request),
            Err(SaveRejection::InvalidContentType(content_type.into())),
            "{content_type:?} should be rejected as a save content type"
        );
    }
}

#[test]
fn content_type_accepts_concrete_types() {
    for content_type in [
        "application/pdf",
        "text/plain",
        "application/json",
        "image/svg+xml",
    ] {
        assert!(
            is_valid_content_type(content_type),
            "{content_type:?} should be legal"
        );
    }
    assert!(!is_valid_content_type("text/*"));
    assert!(!is_valid_content_type("*/*"));
    assert!(!is_valid_content_type("*"));
}

/// The standalone suggested-name validator mirrors the request-level rule.
#[test]
fn suggested_name_rules() {
    for valid in ["report.pdf", "my report.pdf", "日本語.pdf"] {
        assert!(is_valid_suggested_name(valid), "{valid:?} should be legal");
    }
    for invalid in ["", "   ", ".", "..", "a/b", "a\\b", "a\nb", "a\x00b"] {
        assert!(
            !is_valid_suggested_name(invalid),
            "{invalid:?} should be rejected"
        );
    }
}

/// `FileRequest::validate` dispatches kind-typed rejections.
#[test]
fn file_request_validate_dispatches_kind_typed_rejections() {
    let mut save = save_request();
    save.suggested_name = Some("a/b.pdf".into());
    let rejected = FileRequest::Save(save).validate();
    assert!(matches!(
        rejected,
        Err(FileRequestRejection::Save(
            SaveRejection::InvalidSuggestedName(_)
        ))
    ));

    let mut upload = upload_request();
    upload.capability = SelectionCapability {
        files: false,
        directories: false,
    };
    assert!(matches!(
        FileRequest::Upload(upload).validate(),
        Err(FileRequestRejection::Upload(
            UploadRejection::NothingSelectable
        ))
    ));
}

/// Upload-selection tokens are opaque ordered handles: comparable,
/// hashable, and distinct per handle value.
#[test]
fn upload_selection_tokens_are_opaque_ordered_handles() {
    let first = UploadSelectionToken::new(1);
    let second = UploadSelectionToken::new(2);
    assert_eq!(UploadSelectionToken::new(1), first);
    assert_ne!(first, second);
    assert!(first < second);

    let mut set = BTreeSet::new();
    set.insert(first);
    set.insert(second);
    assert_eq!(set.len(), 2);

    let mut map = BTreeMap::new();
    map.insert(second, "second");
    assert_eq!(map.get(&UploadSelectionToken::new(2)), Some(&"second"));
}

/// Save-destination tokens are likewise opaque, ordered, and hashable.
#[test]
fn save_destination_tokens_are_opaque_ordered_handles() {
    let token = SaveDestinationToken::new(9);
    let mut seen = HashSet::new();
    seen.insert(token);
    assert!(seen.contains(&token));
    assert!(!seen.contains(&SaveDestinationToken::new(10)));
    assert!(SaveDestinationToken::new(9) == SaveDestinationToken::new(9));
    assert!(SaveDestinationToken::new(9) < SaveDestinationToken::new(10));
}

/// The two token kinds are distinct types: an upload selection can never be
/// supplied where a save destination is required (cross-kind use is a
/// compile-time error, guarded structurally here by both remaining bare
/// u64-sized opaque values with no payload-bearing fields).
#[test]
fn upload_and_save_tokens_are_kind_separated() {
    use std::mem::size_of;
    assert_eq!(size_of::<UploadSelectionToken>(), size_of::<u64>());
    assert_eq!(size_of::<SaveDestinationToken>(), size_of::<u64>());

    fn takes_upload(_: UploadSelectionToken) {}
    takes_upload(UploadSelectionToken::new(1));
    // takes_upload(SaveDestinationToken::new(1)); // compile-time rejection
}

/// The request layer carries no filesystem or native payload: every string
/// is a URL origin, MIME type, or basename, and tokens are bare handles.
/// The serde suite proves the wire shapes (bare integers for tokens).
#[test]
fn request_layer_has_no_filesystem_or_native_payload() {
    let upload = upload_request();
    assert!(upload.accept.mime_types().is_empty());
    assert!(upload.accept.extensions().is_empty());
    assert_eq!(
        upload.context.origin.as_deref(),
        Some("https://example.com")
    );

    let save = save_request();
    assert_eq!(save.suggested_name.as_deref(), Some("report.pdf"));
    assert_eq!(save.content_type.as_deref(), Some("application/pdf"));
}
