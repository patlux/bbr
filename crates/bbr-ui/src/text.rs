//! Pure text/presentation helpers (DESIGN.md §4, §7.2, §11.5): title/URL
//! fallbacks, separator-aware truncation, the §11.5 address-domain
//! derivation (scheme/path/query dropping, `www.` elision, IDN decoding,
//! port handling, internal-page placeholders) and its mid-ellipsis budget,
//! plus the deterministic monogram host/scheme extraction. No GPUI types
//! beyond the `Theme`/`Density` tokens the budgets read — these are plain
//! string helpers shared by `bbr-ui` components and the app's
//! presentation derivation.

use crate::theme::{Density, Spacing, Theme};

/// DESIGN.md §4 truncation preference: one line, end-ellipsis; when the full
/// title exceeds the row budget and contains a separator (`" – "`, `" - "`,
/// or `" | "`), cut at the first separator so the site name survives;
/// otherwise keep the full title and let GPUI's `text_ellipsis` do the
/// pixel-exact cut. The separator cut fires only when the pre-separator head
/// plus the ellipsis fits within the budget, so short titles are never
/// truncated.
pub fn truncate_title(title: &str, budget: usize) -> String {
    if title.chars().count() <= budget {
        return title.to_string();
    }
    for sep in [" – ", " - ", " | "] {
        if let Some(index) = title.find(sep) {
            let head = &title[..index];
            if head.chars().count() < budget {
                return format!("{head}…");
            }
        }
    }
    title.to_string()
}

/// The registrable host of a URL (`https://github.com/a/b` → `github.com`),
/// used for monogram tiles and title fallbacks. Empty when unparsable.
pub fn host_of(url: &str) -> String {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    // Drop an explicit port for display/monogram purposes.
    let host = host.split(':').next().unwrap_or(host);
    host.trim().to_string()
}

/// The §11.4 palette's shortened URL: scheme and `www.` dropped, a deep
/// path collapsed to `host/…/last` (`https://zed.dev/docs/gpui` →
/// `zed.dev/…/gpui`), a query kept after a root path.
pub fn short_url(url: &str) -> String {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    let rest = rest.strip_prefix("www.").unwrap_or(rest);
    let (location, query) = match rest.split_once('?') {
        Some((location, query)) => (location, Some(query)),
        None => (rest, None),
    };
    let mut parts = location.split('/').filter(|part| !part.is_empty());
    let host = parts.next().unwrap_or("");
    let segments: Vec<&str> = parts.collect();
    let mut short = match segments.as_slice() {
        [] => host.to_string(),
        [only] => format!("{host}/{only}"),
        [.., last] => format!("{host}/…/{last}"),
    };
    if let Some(query) = query.filter(|_| segments.is_empty()) {
        short.push_str("/?");
        short.push_str(query);
    }
    short
}

/// The URL scheme (`https`, `http`, `about`, `data`), used as the last title
/// fallback when even the host is empty.
pub fn scheme_of(url: &str) -> String {
    url.split(':').next().unwrap_or("").trim().to_string()
}

/// DESIGN.md §11.9 title fallback: a title consisting only of whitespace is
/// empty; the empty title falls back to the hostname; an empty host falls
/// back to the URL scheme.
pub fn display_title(url: &str, title: &str) -> String {
    let title = title.trim();
    if !title.is_empty() {
        return title.to_string();
    }
    let host = host_of(url);
    if !host.is_empty() {
        return host;
    }
    scheme_of(url)
}

/// §11.5 address-control display derivation: the **domain only** — host
/// without scheme, path or query; a leading `www.` elided; IDN punycode
/// decoded to the human-readable form; an explicit port dropped (the pill
/// shows the domain; the full address lives in the palette). Non-http(s)
/// schemes (internal pages) render their scheme's short name as the §17.13
/// placeholder — the exact internal-page naming is an open decision and
/// this slice does not invent it. Empty or unparsable input yields the
/// empty label.
pub fn address_domain(url: &str) -> String {
    let url = url.trim();
    if url.is_empty() {
        return String::new();
    }
    let Some((scheme, rest)) = url.split_once(':') else {
        // No scheme at all: not a parsable URL, nothing to display.
        return String::new();
    };
    let scheme = scheme.trim();
    if scheme != "http" && scheme != "https" {
        // Internal pages (§17.13 open): the scheme is the short-name
        // placeholder, never an invented name.
        return scheme.to_string();
    }
    // http(s): the host is the authority without the scheme prefix, cut at
    // the first path/query/fragment delimiter.
    let authority = rest.strip_prefix("//").unwrap_or(rest);
    let host = authority
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    // Drop an explicit port: the pill shows the registrable host, the
    // palette carries the full address (including any port).
    let host = host.split(':').next().unwrap_or(&host);
    let mut labels: Vec<String> = host.split('.').map(decode_idn_label).collect();
    // A leading `www.` is elided (case-insensitive; other `www*` labels and
    // internal `www` are untouched).
    if labels.first().is_some_and(|label| label == "www") {
        labels.remove(0);
    }
    labels.join(".")
}

/// §11.5 address-pill display budget: the domain text area in `type:title`
/// characters, derived from geometry tokens only (the settled S1 scale
/// table, DESIGN.md §5/§8 — no invented sizes): the sidebar width minus the
/// pill's outside `mx` margin, its inner `px` padding, the domain↔accessory
/// gap, and — when the trailing accessory slot is occupied — the slot's
/// reserved `density:hit-min` width. The render always passes `true` for
/// the accessory: the slot is reserved even while empty, so a future §14
/// glyph changes no layout (DESIGN.md §11.5).
pub fn address_domain_budget_chars(theme: &Theme, accessory: bool) -> usize {
    let density = theme.density;
    let mx = f32::from(Spacing::sp3()) * 2.0;
    let px = f32::from(Spacing::sp3()) * 2.0;
    let gap = f32::from(Spacing::sp2());
    let accessory = if accessory {
        f32::from(density.hit_min())
    } else {
        0.0
    };
    let fixed = mx + px + gap + accessory;
    let area = f32::from(density.sidebar_width()) - fixed;
    let per_char = f32::from(theme.type_.title.size) * 0.45;
    (area / per_char).floor().max(1.0) as usize
}

/// DESIGN.md §11.5 mid-ellipsis: when the domain exceeds the display
/// budget, keep the head and the tail (the TLD side) visible — the classic
/// URL mid-ellipsis (`verylongsub.example.com` → `verylon…ple.com`), so the
/// domain text yields width instead of pushing the reserved accessory slot
/// out of the pill. A domain within the budget is returned unchanged;
/// GPUI's `text_ellipsis` on the rendered element is the pixel-exact
/// backstop below this char budget.
pub fn mid_ellipsis(text: &str, budget: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= budget || budget <= 1 {
        return text.to_string();
    }
    // Keep the tail (≈40% of the budget) so the TLD stays readable.
    let tail = ((budget as f32) * 0.4).floor() as usize;
    let head = budget - tail - 1; // one char for the ellipsis
    let mut out: String = chars[..head].iter().collect();
    out.push('…');
    out.extend(chars[chars.len() - tail..].iter());
    out
}

/// The pill's reserved accessory-slot width: one §14 glyph at
/// `density:hit-min` (DESIGN.md §11.5 — "reserves space for one to three
/// §14 glyphs at density:hit-min"; v1 reserves one glyph's width and the
/// §17.14 follow-up slices widen it on top of this geometry).
pub fn accessory_slot_width(density: Density) -> gpui::Pixels {
    density.hit_min()
}

/// Decode one IDN label: an `xn--` (ACE) label is decoded to its Unicode
/// form; any other label is returned unchanged. A malformed `xn--` label is
/// kept verbatim (the decoder never fabricates output).
fn decode_idn_label(label: &str) -> String {
    match label.strip_prefix("xn--") {
        Some(encoded) => punycode_decode(encoded).unwrap_or_else(|| label.to_string()),
        None => label.to_string(),
    }
}

/// RFC 3492 punycode decoding (the payload of an `xn--` IDN label).
/// `None` on any malformed input; the caller keeps the original label.
fn punycode_decode(input: &str) -> Option<String> {
    const BASE: u32 = 36;
    const TMIN: u32 = 1;
    const TMAX: u32 = 26;
    const INITIAL_BIAS: u32 = 72;
    const INITIAL_N: u32 = 128;

    let bytes = input.as_bytes();
    let mut output: Vec<char> = Vec::new();
    // Basic code points before the last delimiter are copied verbatim.
    let last_delim = bytes.iter().rposition(|&b| b == b'-');
    let mut pos = 0;
    if let Some(delim) = last_delim {
        for &b in &bytes[..delim] {
            if b < 0x80 {
                output.push(char::from(b));
            } else {
                return None; // a non-ASCII basic code point is malformed
            }
        }
        pos = delim + 1;
    }
    let mut n = INITIAL_N;
    let mut i = 0u32;
    let mut bias = INITIAL_BIAS;
    while pos < bytes.len() {
        let old_i = i;
        let mut w = 1u32;
        let mut k = BASE;
        loop {
            if pos >= bytes.len() {
                return None; // truncated delta
            }
            let digit = punycode_digit(bytes[pos])?;
            pos += 1;
            i = i.checked_add(digit.checked_mul(w)?)?;
            let t = if k <= bias {
                TMIN
            } else if k >= bias + TMAX {
                TMAX
            } else {
                k - bias
            };
            if digit < t {
                break;
            }
            w = w.checked_mul(BASE - t)?;
            k += BASE;
        }
        let out_len = output.len() as u32 + 1;
        bias = adapt(i - old_i, out_len, old_i == 0);
        n = n.checked_add(i / out_len)?;
        i %= out_len;
        output.insert(i as usize, char::from_u32(n)?);
        i += 1;
    }
    Some(output.into_iter().collect())
}

/// IDNA punycode basic code point → digit: case-folded letters `a`–`z` /
/// `A`–`Z` map to 0–25 and `0`–`9` to 26–35 (RFC 3492 §7.2 case folding —
/// the ACE form browsers use, not the raw RFC §5 table).
fn punycode_digit(b: u8) -> Option<u32> {
    match b {
        b'a'..=b'z' => Some(u32::from(b - b'a')),
        b'A'..=b'Z' => Some(u32::from(b - b'A')),
        b'0'..=b'9' => Some(u32::from(b - b'0') + 26),
        _ => None,
    }
}

/// RFC 3492 bias adaptation.
fn adapt(mut delta: u32, numpoints: u32, first_time: bool) -> u32 {
    const BASE: u32 = 36;
    const TMIN: u32 = 1;
    const TMAX: u32 = 26;
    const SKEW: u32 = 38;
    const DAMP: u32 = 700;
    delta = if first_time { delta / DAMP } else { delta / 2 };
    delta += delta / numpoints.max(1);
    let mut k = 0;
    while delta > ((BASE - TMIN) * TMAX) / 2 {
        delta /= BASE - TMIN;
        k += BASE;
    }
    k + (((BASE - TMIN + 1) * delta) / (delta + SKEW))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_url_keeps_the_host_and_the_last_segment() {
        assert_eq!(
            short_url("https://zed.dev/docs/render/gpui"),
            "zed.dev/…/gpui"
        );
        assert_eq!(short_url("https://www.youtube.com/"), "youtube.com");
        assert_eq!(short_url("http://example.test/simd"), "example.test/simd");
        assert_eq!(
            short_url("https://duckduckgo.com/?q=gpui+render"),
            "duckduckgo.com/?q=gpui+render"
        );
    }
    use crate::theme::{ContrastMode, Theme, ThemeMode};
    use gpui::px;

    #[test]
    fn display_title_falls_back_host_then_scheme() {
        assert_eq!(display_title("https://example.com/a", "  "), "example.com");
        assert_eq!(display_title("https://example.com/a", ""), "example.com");
        assert_eq!(
            display_title("https://example.com/a", "BBR Fixture"),
            "BBR Fixture"
        );
        assert_eq!(display_title("about:blank", ""), "about");
        assert_eq!(display_title("", ""), "");
    }

    #[test]
    fn truncate_title_cuts_at_separator_when_long() {
        // Long title with an early separator: the site name survives.
        let long = "Site Name – A long descriptive suffix that surely pushes the row past its one-line budget";
        assert_eq!(truncate_title(long, 30), "Site Name…");
        // Hyphen and pipe separators are recognized.
        assert_eq!(
            truncate_title(
                "gitlab - merge request details go on and on beyond the budget",
                30
            ),
            "gitlab…"
        );
        assert_eq!(
            truncate_title(
                "News | an article headline that keeps going well past the budget",
                30
            ),
            "News…"
        );
    }

    #[test]
    fn truncate_title_leaves_short_or_fitting_titles_alone() {
        assert_eq!(truncate_title("GitHub – My Repo", 30), "GitHub – My Repo");
        assert_eq!(truncate_title("Short", 30), "Short");
        let at_budget: String = "x".repeat(30);
        assert_eq!(truncate_title(&at_budget, 30), at_budget);
    }

    #[test]
    fn truncate_title_falls_back_to_plain_when_head_does_not_fit() {
        let late =
            "an extremely long first segment that by itself exceeds the row budget – with a suffix";
        assert_eq!(truncate_title(late, 30), late);
        let plain =
            "just one very long unbroken descriptive phrase with no separator anywhere at all";
        assert_eq!(truncate_title(plain, 30), plain);
    }

    #[test]
    fn address_domain_drops_scheme_path_and_query() {
        assert_eq!(address_domain("https://example.com/path"), "example.com");
        assert_eq!(
            address_domain("http://example.com/path?q=1#frag"),
            "example.com"
        );
        assert_eq!(
            address_domain("https://sub.example.com/a/b?x=1"),
            "sub.example.com"
        );
        assert_eq!(address_domain("https://example.com/"), "example.com");
    }

    #[test]
    fn address_domain_elides_only_a_leading_www() {
        assert_eq!(address_domain("https://www.example.com/"), "example.com");
        assert_eq!(address_domain("http://WWW.example.com/x"), "example.com");
        assert_eq!(
            address_domain("https://www2.example.com/x"),
            "www2.example.com"
        );
        assert_eq!(
            address_domain("https://mywww.example.com/x"),
            "mywww.example.com"
        );
    }

    #[test]
    fn address_domain_handles_ports() {
        assert_eq!(address_domain("http://localhost:8080/x"), "localhost");
        assert_eq!(address_domain("https://example.com:8443/x"), "example.com");
        assert_eq!(address_domain("https://example.com:443/x"), "example.com");
    }

    #[test]
    fn address_domain_decodes_idn_to_human_readable() {
        assert_eq!(
            address_domain("https://xn--bcher-kva.example/x"),
            "bücher.example"
        );
        assert_eq!(
            address_domain("https://www.xn--bcher-kva.example/x"),
            "bücher.example"
        );
        assert_eq!(address_domain("https://xn--mnchen-3ya.de/x"), "münchen.de");
        assert_eq!(address_domain("https://xn--wgv71a119e.jp/x"), "日本語.jp");
        assert_eq!(address_domain("https://xn--fiqs8s.cn/x"), "中国.cn");
        assert_eq!(
            address_domain("https://xn--h2brj9c.example/x"),
            "भारत.example"
        );
        assert_eq!(address_domain("https://github.com/user/repo"), "github.com");
    }

    #[test]
    fn address_domain_internal_pages_show_the_scheme_placeholder() {
        assert_eq!(address_domain("about:blank"), "about");
        assert_eq!(address_domain("data:text/html,hi"), "data");
        assert_eq!(address_domain("file:///tmp/x.html"), "file");
    }

    #[test]
    fn address_domain_empty_and_unparsable_input() {
        assert_eq!(address_domain(""), "");
        assert_eq!(address_domain("   "), "");
        assert_eq!(address_domain("not a url"), "");
        assert_eq!(address_domain("https:///path"), "");
    }

    #[test]
    fn punycode_decodes_the_rfc3492_vectors() {
        assert_eq!(punycode_decode("bcher-kva").as_deref(), Some("bücher"));
        assert_eq!(punycode_decode("mnchen-3ya").as_deref(), Some("münchen"));
        assert_eq!(punycode_decode("wgv71a119e").as_deref(), Some("日本語"));
        assert_eq!(punycode_decode("fiqs8s").as_deref(), Some("中国"));
        assert_eq!(punycode_decode("h2brj9c").as_deref(), Some("भारत"));
    }

    #[test]
    fn punycode_rejects_malformed_input() {
        assert_eq!(punycode_decode(""), Some(String::new()));
        assert_eq!(punycode_decode("a-"), Some("a".to_string()));
        assert_eq!(punycode_decode("a-!"), None);
        assert_eq!(punycode_decode("bcher-kv!"), None);
    }

    #[test]
    fn mid_ellipsis_keeps_the_tld_side_when_the_accessory_slot_is_reserved() {
        let theme = Theme::new(
            ThemeMode::Dark,
            ContrastMode::Standard,
            crate::theme::Density::Normal,
        );
        let without = address_domain_budget_chars(&theme, false);
        let with_slot = address_domain_budget_chars(&theme, true);
        assert!(
            without > with_slot,
            "the reserved accessory slot consumes display width"
        );
        let long = "verylongsubdomainname.with-another-long-segment.example.com";
        let clipped = mid_ellipsis(long, with_slot);
        assert!(clipped.chars().count() < long.chars().count());
        assert!(
            clipped.ends_with(".com"),
            "the TLD side stays visible: {clipped}"
        );
        assert!(clipped.contains('…'));
        assert!(!clipped.ends_with('…'), "the ellipsis never cuts the tail");
        assert_eq!(mid_ellipsis("example.com", with_slot), "example.com");
        assert_eq!(mid_ellipsis("example.com", 0), "example.com");
        assert_eq!(mid_ellipsis("example.com", 1), "example.com");
    }

    #[test]
    fn accessory_slot_uses_the_hit_min_token_per_density() {
        assert_eq!(accessory_slot_width(Density::Small), px(28.0));
        assert_eq!(accessory_slot_width(Density::Normal), px(32.0));
        assert_eq!(accessory_slot_width(Density::ExtraLarge), px(40.0));
    }
}
