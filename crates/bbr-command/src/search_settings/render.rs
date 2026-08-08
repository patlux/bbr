//! Deterministic rendering of search terms into engine URLs.
//!
//! [`render_search_url`] replaces the engine template's single `%s`
//! placeholder with the RFC 3986 percent-encoding of the UTF-8 bytes of the
//! terms (uppercase hex). This reproduces the settled command-bar semantics
//! exactly: the app's `command_activate` fallback renders
//! `https://duckduckgo.com/?q=<percent-encoded terms>` with the same
//! unreserved-preserving encoding, so first-run rendering is identical to
//! today's shipping core.

use crate::search_settings::engine::SearchEngine;

/// RFC 3986 unreserved-preserving percent encoding of UTF-8 bytes, uppercase
/// hex. Space becomes `%20`, never `+`; multibyte code points are encoded
/// byte by byte.
pub fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 3);
    for byte in input.bytes() {
        if is_unreserved(byte) {
            out.push(byte as char);
        } else {
            out.push('%');
            push_hex(&mut out, byte);
        }
    }
    out
}

/// The RFC 3986 unreserved set: ALPHA / DIGIT / `-` / `.` / `_` / `~`.
fn is_unreserved(byte: u8) -> bool {
    matches!(
        byte,
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~'
    )
}

fn push_hex(out: &mut String, byte: u8) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    out.push(HEX[(byte >> 4) as usize] as char);
    out.push(HEX[(byte & 0x0F) as usize] as char);
}

/// Renders `terms` into the engine's search URL.
///
/// The single validated `%s` placeholder is replaced with [`percent_encode`]
/// of `terms`; all other template bytes — including valid `%XX` sequences
/// such as a literal `%25` — are preserved verbatim. Rendering is total for
/// every constructed [`SearchEngine`]: curated templates are constants and
/// custom templates are validated at construction.
pub fn render_search_url(engine: &SearchEngine, terms: &str) -> String {
    let encoded = percent_encode(terms);
    engine.template().replacen("%s", &encoded, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curated_endpoints_render_deterministically() {
        let cases: &[(&str, SearchEngine, &str)] = &[
            (
                "hello world",
                SearchEngine::DuckDuckGo,
                "https://duckduckgo.com/?q=hello%20world",
            ),
            (
                "hello world",
                SearchEngine::Google,
                "https://www.google.com/search?q=hello%20world",
            ),
            (
                "hello world",
                SearchEngine::Bing,
                "https://www.bing.com/search?q=hello%20world",
            ),
            (
                "hello world",
                SearchEngine::Ecosia,
                "https://www.ecosia.org/search?q=hello%20world",
            ),
            (
                "hello world",
                SearchEngine::Kagi,
                "https://kagi.com/search?q=hello%20world",
            ),
            ("", SearchEngine::DuckDuckGo, "https://duckduckgo.com/?q="),
        ];
        for (terms, engine, expected) in cases {
            assert_eq!(&render_search_url(engine, terms), expected);
            // Determinism: repeated calls are byte-identical.
            assert_eq!(&render_search_url(engine, terms), expected);
        }
    }

    #[test]
    fn settled_command_boundary_parity() {
        // The E2E-settled intent `search:q` renders the DuckDuckGo endpoint
        // exactly as the app's command_activate fallback documents it.
        assert_eq!(
            render_search_url(&SearchEngine::DuckDuckGo, "q"),
            "https://duckduckgo.com/?q=q"
        );
        for terms in ["q", "bbr browser", "rust lang", "héllo wörld", ""] {
            assert_eq!(
                render_search_url(&SearchEngine::DuckDuckGo, terms),
                format!("https://duckduckgo.com/?q={}", percent_encode(terms))
            );
        }
    }

    #[test]
    fn percent_encode_table_is_deterministic() {
        let cases: &[(&str, &str)] = &[
            ("", ""),
            ("abcXYZ012-_.~", "abcXYZ012-_.~"),
            ("hello world", "hello%20world"),
            ("a/b?c#d&e=f+g", "a%2Fb%3Fc%23d%26e%3Df%2Bg"),
            ("héllo", "h%C3%A9llo"),
            ("100% done", "100%25%20done"),
            ("\n\t", "%0A%09"),
            ("€", "%E2%82%AC"),
            (
                "в поисках",
                "%D0%B2%20%D0%BF%D0%BE%D0%B8%D1%81%D0%BA%D0%B0%D1%85",
            ),
        ];
        for (input, expected) in cases {
            assert_eq!(&percent_encode(input), expected, "input {input:?}");
            assert_eq!(&percent_encode(input), expected, "must be deterministic");
        }
    }

    #[test]
    fn unreserved_set_passes_through_exactly() {
        let unreserved = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~";
        assert_eq!(percent_encode(unreserved), unreserved);
        // Every byte outside the unreserved set is encoded as uppercase hex.
        for byte in 0u8..=255 {
            let ch = byte as char;
            if ch.is_ascii()
                && !matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~')
            {
                let input = String::from(ch);
                assert_eq!(
                    percent_encode(&input),
                    format!("%{byte:02X}"),
                    "byte {byte} must be encoded uppercase"
                );
            }
        }
    }

    #[test]
    fn custom_template_renders_in_place() {
        let custom = SearchEngine::Custom(
            crate::search_settings::engine::CustomUrlTemplate::new(
                "https://example.com/search?q=%s&lang=en",
            )
            .unwrap(),
        );
        assert_eq!(
            render_search_url(&custom, "rust lang"),
            "https://example.com/search?q=rust%20lang&lang=en"
        );

        let path = SearchEngine::Custom(
            crate::search_settings::engine::CustomUrlTemplate::new("https://example.com/%s")
                .unwrap(),
        );
        assert_eq!(
            render_search_url(&path, "foo bar"),
            "https://example.com/foo%20bar"
        );
    }

    #[test]
    fn custom_template_preserves_literal_percent_encodings() {
        // A valid `%25` stays literal; only the `%s` placeholder is replaced.
        let custom = SearchEngine::Custom(
            crate::search_settings::engine::CustomUrlTemplate::new(
                "https://example.com/search?q=%s&ref=%25",
            )
            .unwrap(),
        );
        assert_eq!(
            render_search_url(&custom, "hi"),
            "https://example.com/search?q=hi&ref=%25"
        );
    }

    #[test]
    fn terms_never_trigger_template_placeholder_semantics() {
        // A `%s` inside the terms is encoded as `%25s` and cannot collide
        // with the template placeholder.
        let custom = SearchEngine::Custom(
            crate::search_settings::engine::CustomUrlTemplate::new("https://example.com/?q=%s")
                .unwrap(),
        );
        assert_eq!(
            render_search_url(&custom, "search %s here"),
            "https://example.com/?q=search%20%25s%20here"
        );
    }

    #[test]
    fn unicode_terms_are_utf8_percent_encoded() {
        assert_eq!(
            render_search_url(&SearchEngine::Google, "в поисках"),
            "https://www.google.com/search?q=%D0%B2%20%D0%BF%D0%BE%D0%B8%D1%81%D0%BA%D0%B0%D1%85"
        );
    }
}
