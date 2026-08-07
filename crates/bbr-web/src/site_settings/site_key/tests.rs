use super::*;
use std::collections::BTreeSet;
use std::hash::{DefaultHasher, Hash, Hasher};

fn parse(input: &str) -> SiteKey {
    SiteKey::parse(input).unwrap_or_else(|e| panic!("parse({input:?}) failed: {e}"))
}

#[test]
fn normalizes_scheme_host_and_default_port() {
    let key = parse("https://example.com");
    assert_eq!(key.scheme(), Scheme::Https);
    assert_eq!(key.host(), "example.com");
    assert_eq!(key.port(), 443);
    assert_eq!(key.to_string(), "https://example.com:443");
}

#[test]
fn normalizes_uppercase_and_trailing_slash() {
    assert_eq!(parse("HTTPS://EXAMPLE.COM"), parse("https://example.com"));
    assert_eq!(
        parse("https://Example.COM:443/"),
        parse("https://example.com")
    );
}

#[test]
fn trims_whitespace_and_ignores_path_query_fragment() {
    let key = parse("  https://example.com:8443/path?q=1#frag  ");
    assert_eq!(key.to_string(), "https://example.com:8443");
    assert_eq!(
        parse("https://example.com?q=1"),
        parse("https://example.com")
    );
}

#[test]
fn http_uses_default_port_80() {
    let key = parse("http://example.com");
    assert_eq!(key.scheme(), Scheme::Http);
    assert_eq!(key.port(), 80);
    assert_eq!(key.to_string(), "http://example.com:80");
}

#[test]
fn explicit_default_port_collapses() {
    assert_eq!(
        parse("https://example.com:443"),
        parse("https://example.com")
    );
    assert_eq!(parse("http://example.com:80"), parse("http://example.com"));
}

#[test]
fn parses_ipv4_and_ipv6_hosts() {
    assert_eq!(
        parse("http://127.0.0.1:8080").to_string(),
        "http://127.0.0.1:8080"
    );
    assert_eq!(parse("http://[::1]:8080").to_string(), "http://[::1]:8080");
    assert_eq!(
        parse("http://[2001:db8::1]").to_string(),
        "http://[2001:db8::1]:80"
    );
    assert_eq!(
        parse("https://[::ffff:1:2]").to_string(),
        "https://[::ffff:1:2]:443"
    );
    assert_eq!(
        parse("https://[2001:DB8::1]"),
        parse("https://[2001:db8::1]")
    );
}

#[test]
fn parses_localhost() {
    assert_eq!(
        parse("https://localhost:4443").to_string(),
        "https://localhost:4443"
    );
}

#[test]
fn ipv6_display_round_trips_through_parse() {
    for raw in [
        "http://[::1]:8080",
        "https://[2001:db8::1]",
        "http://[::ffff:1:2]:443",
    ] {
        let key = parse(raw);
        assert_eq!(parse(&key.to_string()), key, "round trip of {raw}");
    }
}

#[test]
fn rejects_empty_and_whitespace_only() {
    assert_eq!(SiteKey::parse(""), Err(SiteKeyError::Empty));
    assert_eq!(SiteKey::parse("   "), Err(SiteKeyError::Empty));
}

#[test]
fn rejects_missing_or_unsupported_scheme() {
    assert_eq!(
        SiteKey::parse("example.com"),
        Err(SiteKeyError::MissingScheme)
    );
    assert_eq!(
        SiteKey::parse("about:blank"),
        Err(SiteKeyError::MissingScheme)
    );
    assert_eq!(
        SiteKey::parse("ftp://example.com"),
        Err(SiteKeyError::UnsupportedScheme("ftp".into()))
    );
    assert_eq!(
        SiteKey::parse("file:///tmp/x"),
        Err(SiteKeyError::UnsupportedScheme("file".into()))
    );
}

#[test]
fn rejects_missing_host() {
    assert_eq!(SiteKey::parse("https://"), Err(SiteKeyError::MissingHost));
    assert_eq!(
        SiteKey::parse("https:///path"),
        Err(SiteKeyError::MissingHost)
    );
    assert_eq!(SiteKey::parse("https://[]"), Err(SiteKeyError::MissingHost));
}

#[test]
fn rejects_invalid_ports() {
    assert_eq!(
        SiteKey::parse("https://example.com:0"),
        Err(SiteKeyError::InvalidPort)
    );
    assert_eq!(
        SiteKey::parse("https://example.com:65536"),
        Err(SiteKeyError::InvalidPort)
    );
    assert_eq!(
        SiteKey::parse("https://example.com:abc"),
        Err(SiteKeyError::InvalidPort)
    );
    assert_eq!(
        SiteKey::parse("https://example.com:"),
        Err(SiteKeyError::InvalidPort)
    );
    assert_eq!(
        SiteKey::parse("https://[::1]:99999"),
        Err(SiteKeyError::InvalidPort)
    );
}

#[test]
fn rejects_invalid_hosts() {
    for input in [
        "https://exa mple.com",
        "https://example..com",
        "https://.example.com",
        "https://example.com.",
        "https://-example.com",
        "https://example-.com",
        "https://exa_mple.com",
        "https://user@example.com",
        "https://exämple.com",
        "https://256.1.1.1",
        "https://01.2.3.4",
        "https://[::1",
        "https://[gg::1]",
        "https://[::1]junk",
    ] {
        assert_eq!(
            SiteKey::parse(input),
            Err(SiteKeyError::InvalidHost),
            "expected {input:?} to be rejected"
        );
    }
}

#[test]
fn rejects_malformed_ipv6() {
    for input in [
        "https://[1:2:3:4:5:6:7:8:9]",
        "https://[1:2:3:4:5:6:7:8::]",
        "https://[1::2::3]",
        "https://[:::]",
    ] {
        assert_eq!(
            SiteKey::parse(input),
            Err(SiteKeyError::InvalidHost),
            "expected {input:?} to be rejected"
        );
    }
}

#[test]
fn scheme_and_port_distinguish_keys() {
    assert_ne!(
        parse("https://example.com:8443"),
        parse("https://example.com")
    );
    assert_ne!(parse("https://example.com"), parse("http://example.com"));
    assert_ne!(parse("http://example.com:81"), parse("http://example.com"));
}

#[test]
fn profile_owns_site_keys() {
    let site = parse("https://example.com");
    let a1 = ProfileSiteKey::new(ProfileId(1), site.clone());
    let a2 = ProfileSiteKey::new(ProfileId(1), site.clone());
    let b = ProfileSiteKey::new(ProfileId(2), site.clone());
    assert_eq!(a1, a2);
    assert_ne!(a1, b);
    assert_eq!(a1.profile(), ProfileId(1));
    assert_eq!(a1.site(), &site);
    assert_eq!(a1.to_string(), "profile=1 site=https://example.com:443");
}

#[test]
fn equality_and_hash_are_stable() {
    let a = parse("https://EXAMPLE.com:443");
    let b = parse("https://example.com");
    assert_eq!(a, b);
    let mut ha = DefaultHasher::new();
    let mut hb = DefaultHasher::new();
    a.hash(&mut ha);
    b.hash(&mut hb);
    assert_eq!(ha.finish(), hb.finish());
}

#[test]
fn site_keys_are_ordered_collection_keys() {
    let mut set = BTreeSet::new();
    set.insert(parse("https://example.com:8443"));
    set.insert(parse("https://example.com"));
    set.insert(parse("http://example.com"));
    let ordered: Vec<String> = set.iter().map(|key| key.to_string()).collect();
    assert_eq!(
        ordered,
        vec![
            "http://example.com:80",
            "https://example.com:443",
            "https://example.com:8443",
        ]
    );
}

#[test]
fn debug_shape_is_exact() {
    let key = parse("https://example.com");
    assert_eq!(
        format!("{key:?}"),
        "SiteKey { scheme: Https, host: \"example.com\", port: 443 }"
    );
    let psk = ProfileSiteKey::new(ProfileId(7), key);
    assert_eq!(
            format!("{psk:?}"),
            "ProfileSiteKey { profile: ProfileId(7), site: SiteKey { scheme: Https, host: \"example.com\", port: 443 } }"
        );
}

#[test]
fn error_display_messages_are_explicit() {
    assert_eq!(SiteKeyError::Empty.to_string(), "site origin is empty");
    assert_eq!(
        SiteKeyError::MissingScheme.to_string(),
        "site origin needs an explicit http:// or https:// scheme"
    );
    assert_eq!(
        SiteKeyError::UnsupportedScheme("ftp".into()).to_string(),
        "site scheme \"ftp\" is not supported"
    );
    assert_eq!(
        SiteKeyError::InvalidPort.to_string(),
        "site origin port is invalid (must be a decimal 1..=65535)"
    );
}
