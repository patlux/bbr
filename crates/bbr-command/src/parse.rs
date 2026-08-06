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
    /// The lowercase projection is computed once here so ranking never
    /// repeats case folding.
    pub fn new(mode: Mode, text: &str) -> Self {
        let text = text.trim().to_owned();
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
        }
    }
}

/// Classifies raw text as a navigation destination or a search query.
///
/// The rules are deliberately small and deterministic:
///
/// 1. Empty or whitespace-only text is [`Destination::Empty`].
/// 2. A scheme prefix (`[A-Za-z][A-Za-z0-9+.-]*:`) names a URL. This covers
///    `http://`, `https://`, `about:blank`, `file:///...`, `mailto:`, and
///    custom schemes.
/// 3. Raw whitespace inside the text means it cannot be a single URL, so it
///    is a search.
/// 4. `localhost` (with an optional port) names a URL.
/// 5. An IPv4 literal (four dotted decimal groups, optional port) names a
///    URL.
/// 6. A domain-like host (two or more dotted labels, final label at least
///    two characters, at least one letter anywhere, no port surprises) names
///    a URL. Internationalized domain names without a scheme are treated as
///    search text because the punycode conversion is an app concern.
/// 7. Anything else is a search.
///
/// No rule consults the network, DNS, or any external table, so the outcome
/// is identical on every platform.
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
    if text.contains(char::is_whitespace) {
        return false;
    }
    if has_scheme(text) {
        return true;
    }
    let Some(host) = host_candidate(text) else {
        return false;
    };
    is_local_host(host) || is_ipv4(host) || looks_like_domain(host)
}

/// True when the text starts with `scheme:` where the scheme is an ASCII
/// letter followed by letters, digits, `+`, `.`, or `-`.
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
            return i > 0;
        }
        if !(ch.is_ascii_alphanumeric() || ch == '+' || ch == '.' || ch == '-') {
            return false;
        }
    }
    false
}

/// The host-like prefix up to the first path/query/fragment/port separator.
fn host_candidate(text: &str) -> Option<&str> {
    let end = text.find(['/', '?', '#']).unwrap_or(text.len());
    let segment = &text[..end];
    let host = segment.split(':').next().unwrap_or(segment);
    if host.is_empty() {
        None
    } else {
        Some(host)
    }
}

fn is_local_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
}

/// True when the host is four dotted decimal groups, optionally followed by
/// a `:port` that `host_candidate` has already stripped.
fn is_ipv4(host: &str) -> bool {
    let parts: Vec<&str> = host.split('.').collect();
    parts.len() == 4
        && parts.iter().all(|part| {
            !part.is_empty() && part.len() <= 3 && part.bytes().all(|b| b.is_ascii_digit())
        })
}

/// True when the host looks like a registered domain: at least two dotted
/// labels, every label non-empty and ASCII alphanumeric/hyphen, the final
/// label at least two characters, and at least one letter somewhere (so
/// version strings like `12.34` stay searches). A single trailing root dot
/// is accepted (`example.com.`).
fn looks_like_domain(host: &str) -> bool {
    let host = host.strip_suffix('.').unwrap_or(host);
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() < 2 {
        return false;
    }
    let mut has_letter = false;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            return false;
        }
        for b in part.bytes() {
            if b.is_ascii_alphanumeric() || b == b'-' {
                if b.is_ascii_alphabetic() {
                    has_letter = true;
                }
            } else {
                return false;
            }
        }
        if i == parts.len() - 1 && part.len() < 2 {
            return false;
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
