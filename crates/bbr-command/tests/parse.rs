//! Table-driven public-API tests for command-bar input parsing.
//!
//! Covers the URL-vs-search ambiguity matrix, mode preservation, trimming,
//! and empty input across the public [`bbr_command::parse`] surface.

use bbr_command::parse::parse_destination;
use bbr_command::parse::{Destination, Mode, Query};

/// Expected classification of one input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Navigate,
    Search,
    Empty,
}

fn kind(text: &str) -> Kind {
    match parse_destination(text) {
        Destination::Navigate(_) => Kind::Navigate,
        Destination::Search(_) => Kind::Search,
        Destination::Empty => Kind::Empty,
    }
}

#[test]
fn url_search_ambiguity_matrix() {
    // (input, expected classification, why)
    let cases: &[(&str, Kind, &str)] = &[
        // Empty input has no intent.
        ("", Kind::Empty, "empty string"),
        ("   ", Kind::Empty, "whitespace only"),
        ("\t\n", Kind::Empty, "control whitespace"),
        // Scheme prefixes always navigate.
        ("https://example.com", Kind::Navigate, "https scheme"),
        ("http://example.com", Kind::Navigate, "http scheme"),
        ("HTTPS://EXAMPLE.COM", Kind::Navigate, "uppercase scheme"),
        ("about:blank", Kind::Navigate, "about scheme"),
        ("file:///tmp/x.html", Kind::Navigate, "file scheme"),
        ("mailto:user@example.com", Kind::Navigate, "mailto scheme"),
        ("bbr:settings", Kind::Navigate, "custom scheme"),
        ("custom+scheme:x", Kind::Navigate, "scheme with plus/dash"),
        ("https://", Kind::Navigate, "bare scheme still navigates"),
        // Local hosts.
        ("localhost", Kind::Navigate, "localhost"),
        ("localhost:8080", Kind::Navigate, "localhost with port"),
        (
            "localhost:65535/path?q=1",
            Kind::Navigate,
            "localhost max port",
        ),
        ("LOCALHOST", Kind::Navigate, "localhost case-insensitive"),
        // IPv4 literals.
        ("127.0.0.1", Kind::Navigate, "loopback IPv4"),
        ("127.0.0.1:3000", Kind::Navigate, "IPv4 with port"),
        ("127.0.0.1:1/path?x=1", Kind::Navigate, "IPv4 min port"),
        ("192.168.1.1", Kind::Navigate, "private IPv4"),
        ("192.168.1.1/admin", Kind::Navigate, "IPv4 with path"),
        (
            "10.0.0.1:8080/path?q=1",
            Kind::Navigate,
            "IPv4 port path query",
        ),
        // Domain-like hosts.
        ("example.com", Kind::Navigate, "bare domain"),
        ("www.example.com", Kind::Navigate, "www domain"),
        (
            "sub.example.co.uk/path?q=1#frag",
            Kind::Navigate,
            "multi-label with path",
        ),
        ("news.ycombinator.com", Kind::Navigate, "known domain"),
        ("golang.org/pkg/fmt", Kind::Navigate, "domain with path"),
        ("example.com:8080/x", Kind::Navigate, "domain port path"),
        (
            "example.com:443/path?q=1",
            Kind::Navigate,
            "domain port query",
        ),
        ("example.com.", Kind::Navigate, "trailing root dot"),
        ("a-b.example-site.io", Kind::Navigate, "hyphenated labels"),
        ("xn--bcher-kva.example", Kind::Navigate, "punycode label"),
        // Bracketed IPv6 authorities.
        ("[::1]", Kind::Navigate, "bracketed IPv6 loopback"),
        (
            "[2001:db8::1]:443/path?q=1",
            Kind::Navigate,
            "IPv6 port path",
        ),
        // Non-URLs stay searches.
        ("hello world", Kind::Search, "plain sentence"),
        ("how to write rust", Kind::Search, "question-style query"),
        ("example", Kind::Search, "single word without dot"),
        ("12.34", Kind::Search, "version number without letter"),
        ("example .com", Kind::Search, "whitespace splits domain"),
        ("foo_bar", Kind::Search, "underscore-only token"),
        ("a.b", Kind::Search, "one-character final label"),
        ("example.c", Kind::Search, "one-character final label"),
        ("999.999.999.999", Kind::Search, "invalid IPv4 octets"),
        ("256.1.1.1", Kind::Search, "IPv4 octet too large"),
        ("example.com:abc", Kind::Search, "alphabetic domain port"),
        ("example.com:0", Kind::Search, "zero domain port"),
        ("example.com:65536", Kind::Search, "oversized domain port"),
        ("example.com:", Kind::Search, "empty domain port"),
        ("example.com:80:90", Kind::Search, "multiple colons"),
        ("localhost:abc", Kind::Search, "alphabetic localhost port"),
        ("-bad.com", Kind::Search, "leading label hyphen"),
        ("bad-.com", Kind::Search, "trailing label hyphen"),
        ("[::1", Kind::Search, "missing IPv6 closing bracket"),
        ("::1]", Kind::Search, "missing IPv6 opening bracket"),
        ("[not-ipv6]", Kind::Search, "invalid IPv6 literal"),
        ("[::1]:0", Kind::Search, "zero IPv6 port"),
        ("[::1]:65536", Kind::Search, "oversized IPv6 port"),
        ("[::1]:abc", Kind::Search, "alphabetic IPv6 port"),
        ("[::1]extra", Kind::Search, "suffix after IPv6 bracket"),
        ("2001:db8::1", Kind::Search, "unbracketed IPv6"),
        ("кто.рф", Kind::Search, "IDN without scheme"),
        ("héllo wörld", Kind::Search, "unicode sentence"),
        ("в поисках", Kind::Search, "unicode query"),
        (
            "what is 12.34 about",
            Kind::Search,
            "version number inside query",
        ),
    ];
    for &(input, expected, why) in cases {
        assert_eq!(kind(input), expected, "input {input:?} ({why})");
    }
}

#[test]
fn navigate_preserves_trimmed_text() {
    let cases = [
        "  example.com  ",
        " https://example.com/x ",
        "localhost:8080",
    ];
    for input in cases {
        match parse_destination(input) {
            Destination::Navigate(url) => assert_eq!(url, input.trim(), "input {input:?}"),
            other => panic!("expected Navigate for {input:?}, got {other:?}"),
        }
    }
}

#[test]
fn search_preserves_trimmed_text() {
    match parse_destination("  rust async runtimes  ") {
        Destination::Search(terms) => assert_eq!(terms, "rust async runtimes"),
        other => panic!("expected Search, got {other:?}"),
    }
}

#[test]
fn query_round_trips_modes_and_text() {
    let q = Query::new(Mode::NewTab, "  Example.COM  ");
    assert_eq!(q.mode, Mode::NewTab);
    assert_eq!(q.text, "Example.COM");
    assert_eq!(q.lower(), "example.com");
    assert_eq!(
        q.destination,
        Destination::Navigate("Example.COM".to_owned())
    );

    let q = Query::new(Mode::Action, "pin");
    assert_eq!(q.mode, Mode::Action);
    assert_eq!(q.destination, Destination::Search("pin".to_owned()));
}

#[test]
fn empty_query_is_empty_in_every_mode() {
    for mode in [Mode::Location, Mode::NewTab, Mode::Switcher, Mode::Action] {
        let q = Query::new(mode, " \t ");
        assert!(q.is_empty(), "{mode:?}");
        assert_eq!(q.destination, Destination::Empty, "{mode:?}");
    }
}
