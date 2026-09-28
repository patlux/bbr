//! Typed command-bar input parsing: surface mode and URL-vs-search intent.
//!
//! The parser is the pure, table-tested core of Phase-5 input handling. It
//! decides whether raw text names a navigation destination, a web search, or
//! nothing, and exposes the caller-selected [`Mode`] that later app
//! composition maps onto `Cmd-L`, `Cmd-T`, the switcher surface, and an
//! action shortcut.

use crate::id::{ProfileId, SpaceId};

/// The command-bar surface the user invoked (Phase 5).
///
/// The mode is caller-selected, never guessed from text. Ranking behaves
/// identically for [`Mode::Location`], [`Mode::NewTab`], and
/// [`Mode::Switcher`]; [`Mode::Action`] restricts results to browser
/// commands and never appends a navigate/search fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// `Cmd-L`: location entry; the app pre-selects the active tab's URL.
    Location,
    /// `Cmd-T`: staged new-tab destination; nothing is created until the
    /// user activates the result.
    NewTab,
    /// Filtering open tabs, spaces, history, and commands.
    Switcher,
    /// Action shortcut: only registered browser commands are ranked.
    Action,
}

/// The prefix that switches the command surface to actions (blueprint D04).
pub const ACTIONS_PREFIX: char = '>';

/// Whether raw command text asks for the actions mode (leading `>`).
pub fn is_actions_text(text: &str) -> bool {
    text.trim_start().starts_with(ACTIONS_PREFIX)
}

/// Parsed intent of the raw text inside one command-bar mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    /// A concrete destination address. The text is returned as typed
    /// (trimmed); scheme completion such as `https://` is an app-composition
    /// concern, not part of the portable core.
    Navigate(String),
    /// A web-search fallback for text that is not a URL.
    Search(String),
    /// Empty input; no destination intent.
    Empty,
}

/// A typed, parsed command-bar query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    /// The surface the query was entered on.
    pub mode: Mode,
    /// Trimmed raw text exactly as the user typed it.
    pub text: String,
    /// Parsed destination intent for location/new-tab semantics.
    pub destination: Destination,
    lower: String,
}

impl Query {
    /// Parses trimmed text into a typed query for `mode`.
    ///
    /// Blueprint D04: text that starts with `>` switches any surface to
    /// [`Mode::Action`] and ranks the remainder against the browser
    /// commands only. The lowercase projection is computed once here so
    /// ranking never repeats case folding.
    pub fn new(mode: Mode, text: &str) -> Self {
        let text = text.trim();
        let (mode, text) = match text.strip_prefix(ACTIONS_PREFIX) {
            Some(rest) => (Mode::Action, rest.trim()),
            None => (mode, text),
        };
        let text = text.to_owned();
        let destination = parse_destination(&text);
        let lower = text.to_lowercase();
        Query {
            mode,
            text,
            destination,
            lower,
        }
    }

    /// True when the user typed nothing.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Precomputed lowercase text used for deterministic matching.
    pub fn lower(&self) -> &str {
        &self.lower
    }
}

/// The command context the ranking is resolved against.
///
/// Both fields are optional because the corpus is provider-neutral: a caller
/// that has no notion of a current space or profile passes `None` and
/// cross-space/cross-profile results simply label every space.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct QueryContext {
    /// The space the user is currently viewing.
    pub current_space: Option<SpaceId>,
    /// The profile the current space belongs to.
    pub current_profile: Option<ProfileId>,
    /// The tab the user is looking at (the empty ⌘T recents skip it).
    pub current_tab: Option<crate::id::TabId>,
}

impl QueryContext {
    /// A context without a known current space or profile.
    pub fn new() -> Self {
        Self::default()
    }

    /// A context with an explicit current space and profile.
    pub fn with_current(space: SpaceId, profile: ProfileId) -> Self {
        Self {
            current_space: Some(space),
            current_profile: Some(profile),
            current_tab: None,
        }
    }

    /// The same context with the tab the user is looking at.
    pub fn with_current_tab(mut self, tab: Option<crate::id::TabId>) -> Self {
        self.current_tab = tab;
        self
    }
}

/// Classifies raw text as a navigation destination or a search query.
///
/// Explicit `scheme://` and opaque undotted `scheme:value` input navigates
/// unchanged. A dotted or `localhost` prefix before a bare colon is instead an
/// authority candidate and must pass host and port validation. Otherwise, the
/// parser validates only the authority before the first path, query, or
/// fragment separator. Accepted authorities are `localhost`, an IPv4 literal,
/// a bracketed IPv6 literal, or a strict ASCII domain, each with an optional
/// numeric port in `1..=65535`. Unbracketed IPv6 and Unicode IDNs remain
/// searches; callers may navigate them by supplying an explicit scheme.
///
/// No rule consults the network, DNS, or an external suffix table, so the
/// outcome is identical on every platform.
pub fn parse_destination(text: &str) -> Destination {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Destination::Empty;
    }
    if is_url_candidate(trimmed) {
        Destination::Navigate(trimmed.to_owned())
    } else {
        Destination::Search(trimmed.to_owned())
    }
}

fn is_url_candidate(text: &str) -> bool {
    if text.parse::<std::net::Ipv6Addr>().is_ok() {
        return false;
    }
    if has_scheme(text) {
        return true;
    }
    if text.contains(char::is_whitespace) {
        return false;
    }
    let end = text.find(['/', '?', '#']).unwrap_or(text.len());
    let authority = &text[..end];
    is_bracketed_ipv6_authority(authority) || is_host_authority(authority)
}

/// True when the text has an explicit `scheme://` prefix or an opaque,
/// undotted `scheme:value` prefix. Dotted and `localhost` bare-colon prefixes
/// are authority candidates and must be validated as hosts with ports.
fn has_scheme(text: &str) -> bool {
    let mut chars = text.char_indices();
    let Some((_, first)) = chars.next() else {
        return false;
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    for (i, ch) in chars {
        if ch == ':' {
            let prefix = &text[..i];
            let suffix = &text[i + 1..];
            // A domain-like or localhost prefix followed by a bare colon is
            // an authority with a port, not an opaque custom scheme. `://`
            // remains an explicit scheme even when its name contains dots.
            return suffix.starts_with("//") || (!is_local_host(prefix) && !prefix.contains('.'));
        }
        if !(ch.is_ascii_alphanumeric() || ch == '+' || ch == '.' || ch == '-') {
            return false;
        }
    }
    false
}

fn is_bracketed_ipv6_authority(authority: &str) -> bool {
    let Some(literal_and_suffix) = authority.strip_prefix('[') else {
        return false;
    };
    let Some(close) = literal_and_suffix.find(']') else {
        return false;
    };
    let literal = &literal_and_suffix[..close];
    let suffix = &literal_and_suffix[close + 1..];
    if literal.parse::<std::net::Ipv6Addr>().is_err() {
        return false;
    }
    suffix.is_empty() || suffix.strip_prefix(':').is_some_and(valid_port)
}

fn is_host_authority(authority: &str) -> bool {
    let mut parts = authority.split(':');
    let Some(host) = parts.next() else {
        return false;
    };
    let port = parts.next();
    if host.is_empty() || parts.next().is_some() || port.is_some_and(|value| !valid_port(value)) {
        return false;
    }
    is_local_host(host) || host.parse::<std::net::Ipv4Addr>().is_ok() || looks_like_domain(host)
}

fn valid_port(port: &str) -> bool {
    !port.is_empty()
        && port.bytes().all(|byte| byte.is_ascii_digit())
        && port.parse::<u16>().is_ok_and(|value| value > 0)
}

fn is_local_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
}

/// True when the host is a strict ASCII domain: no more than 253 bytes after
/// removing one optional trailing root dot, at least two labels, labels of
/// 1..=63 alphanumeric/hyphen bytes without edge hyphens, a final label of at
/// least two bytes, and at least one ASCII letter across the name. Valid
/// punycode labels therefore remain accepted while raw Unicode IDNs do not.
fn looks_like_domain(host: &str) -> bool {
    if !host.is_ascii() {
        return false;
    }
    let host = host.strip_suffix('.').unwrap_or(host);
    if host.is_empty() || host.len() > 253 {
        return false;
    }
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() < 2 || parts.last().is_none_or(|part| part.len() < 2) {
        return false;
    }
    let mut has_letter = false;
    for part in parts {
        if part.is_empty() || part.len() > 63 || part.starts_with('-') || part.ends_with('-') {
            return false;
        }
        for byte in part.bytes() {
            if !byte.is_ascii_alphanumeric() && byte != b'-' {
                return false;
            }
            has_letter |= byte.is_ascii_alphabetic();
        }
    }
    has_letter
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nav(text: &str) -> Destination {
        Destination::Navigate(text.to_owned())
    }

    fn search(text: &str) -> Destination {
        Destination::Search(text.to_owned())
    }

    #[test]
    fn empty_and_whitespace_are_empty() {
        for text in ["", "   ", "\t\n"] {
            assert_eq!(parse_destination(text), Destination::Empty, "{text:?}");
        }
    }

    #[test]
    fn scheme_urls_are_navigation() {
        for text in [
            "https://example.com",
            "http://example.com",
            "HTTPS://EXAMPLE.COM",
            "about:blank",
            "file:///tmp/x.html",
            "mailto:user@example.com",
            "bbr:settings",
            "custom+scheme:x",
            "https://",
        ] {
            assert_eq!(parse_destination(text), nav(text), "{text:?}");
        }
    }

    #[test]
    fn dotted_bare_colon_prefixes_require_valid_authorities() {
        for text in [
            "bad-.com:443",
            "foo..com:443",
            "bad-.com:abc",
            "com.example:settings",
        ] {
            assert_eq!(parse_destination(text), search(text), "{text:?}");
        }
        assert_eq!(
            parse_destination("com.example://settings"),
            nav("com.example://settings")
        );
    }

    #[test]
    fn local_hosts_are_navigation() {
        for text in ["localhost", "localhost:8080", "LOCALHOST"] {
            assert_eq!(parse_destination(text), nav(text), "{text:?}");
        }
    }

    #[test]
    fn ipv4_literals_are_navigation() {
        for text in [
            "127.0.0.1",
            "127.0.0.1:3000",
            "192.168.1.1",
            "192.168.1.1/admin",
            "10.0.0.1:8080/path?q=1",
        ] {
            assert_eq!(parse_destination(text), nav(text), "{text:?}");
        }
    }

    #[test]
    fn domain_like_hosts_are_navigation() {
        for text in [
            "example.com",
            "www.example.com",
            "sub.example.co.uk/path?q=1#frag",
            "news.ycombinator.com",
            "golang.org/pkg/fmt",
            "example.com:8080/x",
            "example.com.",
            "a-b.example-site.io",
        ] {
            assert_eq!(parse_destination(text), nav(text), "{text:?}");
        }
    }

    #[test]
    fn non_urls_are_searches() {
        for text in [
            "hello world",
            "how to write rust",
            "example",
            "12.34",
            "example .com",
            "foo_bar",
            "a.b",
            "example.c",
            "кто.рф",
            "héllo wörld",
            "в поисках",
            "what is 12.34 about",
        ] {
            assert_eq!(parse_destination(text), search(text), "{text:?}");
        }
    }

    #[test]
    fn query_trims_and_lowercases() {
        let q = Query::new(Mode::Location, "  Example.COM  ");
        assert_eq!(q.text, "Example.COM");
        assert_eq!(q.lower(), "example.com");
        assert_eq!(q.destination, nav("Example.COM"));
        assert!(!q.is_empty());

        let empty = Query::new(Mode::Switcher, " ");
        assert!(empty.is_empty());
        assert_eq!(empty.destination, Destination::Empty);
    }

    #[test]
    fn query_keeps_mode() {
        assert_eq!(Query::new(Mode::NewTab, "x").mode, Mode::NewTab);
        assert_eq!(Query::new(Mode::Action, "x").mode, Mode::Action);
    }
}
