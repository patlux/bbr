//! Classification contract tests: the exact scheme rules, web-URL shape
//! validation, and the privacy reduction rule — all without any native or
//! external input.

use crate::external_urls::{classify, MalformedUrl, RejectionReason, UrlClass, WebScheme, WebUrl};

/// Assert a URL classifies as a supported web URL and return the parse.
fn web(url: &str) -> WebUrl {
    match classify(url) {
        UrlClass::Web(url) => url,
        other => panic!("expected web URL for {url:?}, got {other:?}"),
    }
}

/// Assert a URL classifies as malformed with the exact reason.
fn malformed(url: &str, expected: MalformedUrl) {
    match classify(url) {
        UrlClass::Malformed(reason) => assert_eq!(reason, expected, "for {url:?}"),
        other => panic!("expected malformed for {url:?}, got {other:?}"),
    }
}

/// A well-formed https URL keeps its full shape: scheme, lower-cased host,
/// and the untransformed remainder.
#[test]
fn https_url_keeps_full_shape() {
    let url = web("https://Example.COM/a/b?q=1#frag");
    assert_eq!(url.scheme, WebScheme::Https);
    assert_eq!(url.host, "example.com");
    assert_eq!(url.port, None);
    assert_eq!(url.remainder, "/a/b?q=1#frag");
}

/// http and https are the only supported web schemes; both are matched
/// case-insensitively and the host is lower-cased for routing.
#[test]
fn http_and_https_schemes_are_case_insensitive() {
    let url = web("HTTP://EXAMPLE.COM");
    assert_eq!(url.scheme, WebScheme::Http);
    assert_eq!(url.host, "example.com");
    assert_eq!(url.remainder, "");

    let url = web("HtTpS://Example.com/");
    assert_eq!(url.scheme, WebScheme::Https);
    assert_eq!(url.remainder, "/");
}

/// An explicit port is parsed, validated, and preserved in the reduction.
#[test]
fn explicit_port_is_parsed_and_validated() {
    let url = web("https://example.com:8443/x");
    assert_eq!(url.port, Some(8443));
    assert_eq!(url.reduced_origin(), "https://example.com:8443");
}

/// Userinfo is stripped from the parsed host (the navigated URL itself stays
/// untouched in the request), so routing never carries credentials.
#[test]
fn userinfo_is_stripped_from_host() {
    let url = web("https://user:secret@example.com/x");
    assert_eq!(url.host, "example.com");
    assert_eq!(url.port, None);
    assert_eq!(url.reduced_origin(), "https://example.com");
}

/// Bracketed IPv6 hosts parse; the port after the closing bracket is
/// optional.
#[test]
fn bracketed_ipv6_hosts_parse() {
    let url = web("http://[::1]:8080/x");
    assert_eq!(url.host, "::1");
    assert_eq!(url.port, Some(8080));

    let url = web("http://[::1]");
    assert_eq!(url.host, "::1");
    assert_eq!(url.port, None);
}

/// A web URL with no path, query, or fragment has an empty remainder.
#[test]
fn bare_origin_has_empty_remainder() {
    let url = web("https://example.com");
    assert_eq!(url.remainder, "");
    assert_eq!(url.reduced_origin(), "https://example.com");
}

/// The dangerous schemes are classified explicitly, case-insensitively, and
/// their payloads are never inspected or retained.
#[test]
fn dangerous_schemes_are_classified_and_never_opened() {
    for (raw, scheme) in [
        ("javascript:alert(1)", "javascript"),
        ("JAVASCRIPT:alert(1)", "javascript"),
        ("data:text/html,<b>hi</b>", "data"),
        ("file:///etc/passwd", "file"),
        ("about:blank", "about"),
        ("about:config", "about"),
        ("vbscript:msgbox(1)", "vbscript"),
    ] {
        match classify(raw) {
            UrlClass::DangerousScheme { scheme: found } => {
                assert_eq!(found, scheme, "for {raw:?}");
            }
            other => panic!("expected dangerous {scheme}: for {raw:?}, got {other:?}"),
        }
    }
}

/// Valid non-web schemes are unsupported: the browser cannot render them and
/// the OS should have dispatched them to a registered handler.
#[test]
fn unsupported_schemes_are_classified() {
    for (raw, scheme) in [
        ("mailto:user@example.com", "mailto"),
        ("tel:+49123456789", "tel"),
        ("ftp://files.example.com/x", "ftp"),
        ("magnet:?xt=urn:btih:abc", "magnet"),
        ("itms-services://?action=download", "itms-services"),
        ("bbr://internal/command", "bbr"),
        ("webcal://calendar.example.com/cal.ics", "webcal"),
    ] {
        match classify(raw) {
            UrlClass::UnsupportedScheme { scheme: found } => {
                assert_eq!(found, scheme, "for {raw:?}");
            }
            other => panic!("expected unsupported {scheme}: for {raw:?}, got {other:?}"),
        }
    }
}

/// Empty input is malformed.
#[test]
fn empty_input_is_malformed() {
    malformed("", MalformedUrl::Empty);
    malformed("   ", MalformedUrl::Empty);
    malformed("\t\n", MalformedUrl::Empty);
}

/// Scheme-less input is malformed: the OS delivers absolute URLs and this
/// contract never applies `normalize_url`'s scheme defaulting.
#[test]
fn scheme_less_input_is_malformed() {
    malformed("example.com", MalformedUrl::MissingScheme);
    malformed("//example.com", MalformedUrl::MissingScheme);
    malformed("example.com/path", MalformedUrl::MissingScheme);
}

/// A token that could be a scheme is read as a host:port when the suffix is
/// digits only (the same judgment `normalize_url` applies), so such input is
/// scheme-less and rejected rather than guessed.
#[test]
fn port_lookalike_is_not_a_scheme() {
    malformed("example.com:8080", MalformedUrl::MissingScheme);
    malformed("example.com:8080/path", MalformedUrl::MissingScheme);
    malformed("localhost:3000", MalformedUrl::MissingScheme);
    // A digit-only payload after a real scheme is read the same way; it is
    // still a rejection either way, never an open.
    malformed("tel:0123456789", MalformedUrl::MissingScheme);
}

/// A scheme position whose name violates RFC 3986 (empty, digit-first, or
/// illegal characters) is malformed.
#[test]
fn invalid_scheme_names_are_malformed() {
    malformed(":8080", MalformedUrl::InvalidScheme);
    malformed("1http://x", MalformedUrl::InvalidScheme);
    malformed("ht~tp://x", MalformedUrl::InvalidScheme);
}

/// Whitespace or control characters anywhere in the URL are malformed, even
/// inside an otherwise plausible web URL.
#[test]
fn internal_whitespace_and_control_are_malformed() {
    malformed("https://exa mple.com", MalformedUrl::InvalidCharacters);
    malformed("https://example.com/a b", MalformedUrl::InvalidCharacters);
    malformed("https://example.com/a\nb", MalformedUrl::InvalidCharacters);
    // Non-ASCII whitespace is whitespace too.
    malformed(
        "https://exa\u{00a0}mple.com",
        MalformedUrl::InvalidCharacters,
    );
}

/// Web URLs without an authority or with an empty host are malformed.
#[test]
fn web_url_without_host_is_malformed() {
    malformed("https://", MalformedUrl::MissingHost);
    malformed("https:///path", MalformedUrl::MissingHost);
    malformed("https:", MalformedUrl::MissingHost);
    malformed("https:example.com", MalformedUrl::MissingHost);
    malformed("https://user@", MalformedUrl::MissingHost);
    malformed("https://:8080", MalformedUrl::MissingHost);
}

/// Ports must be non-empty ASCII digits and at most 65535.
#[test]
fn illegal_ports_are_malformed() {
    malformed("https://host:", MalformedUrl::InvalidPort);
    malformed("https://host:abc", MalformedUrl::InvalidPort);
    malformed("https://host:99999", MalformedUrl::InvalidPort);
    malformed("https://host:12a34", MalformedUrl::InvalidPort);
    // Overflowing digit strings parse-fail instead of panicking.
    malformed(
        "https://host:99999999999999999999999999999",
        MalformedUrl::InvalidPort,
    );
}

/// A bracketed host must be closed; junk after the closing bracket is not an
/// accepted authority shape.
#[test]
fn malformed_bracketed_hosts() {
    malformed("http://[::1", MalformedUrl::UnclosedBracketedHost);
    malformed("http://[::1]x", MalformedUrl::MissingHost);
    malformed("http://[]", MalformedUrl::MissingHost);
}

/// The privacy reduction matches the sibling web-error rule: authority
/// reduced to `scheme://host[:port]`, lower-cased, userinfo stripped;
/// authority-less schemes reduce to the scheme only; malformed URLs carry no
/// content.
#[test]
fn privacy_reduction_never_exposes_payloads() {
    assert_eq!(
        web("https://user:pass@Example.com:8443/secret?token=abc#frag").reduced_origin(),
        "https://example.com:8443"
    );
    assert_eq!(
        classify("javascript:alert(document.cookie)")
            .reduced()
            .as_str(),
        "javascript:"
    );
    assert_eq!(
        classify("mailto:user@example.com").reduced().as_str(),
        "mailto:"
    );
    assert_eq!(
        classify("https://example.com/a?token=secret")
            .reduced()
            .as_str(),
        "https://example.com"
    );
    assert_eq!(classify("not a url").reduced().as_str(), "<malformed url>");
    assert_eq!(classify("").reduced().as_str(), "<malformed url>");
}

/// Rejection reasons carry only the lower-cased scheme (or the malformed
/// reason) — never the payload — and render as short technical diagnostics.
#[test]
fn rejection_reasons_are_payload_free_and_displayable() {
    let reason = RejectionReason::DangerousScheme {
        scheme: "javascript".into(),
    };
    assert_eq!(reason.to_string(), "dangerous scheme javascript:");

    let reason = RejectionReason::UnsupportedScheme {
        scheme: "mailto".into(),
    };
    assert_eq!(reason.to_string(), "unsupported scheme mailto:");

    let reason = RejectionReason::Malformed(MalformedUrl::MissingHost);
    assert_eq!(reason.to_string(), "malformed url (missing host)");
}
