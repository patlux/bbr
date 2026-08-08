//! Command-bar modes, restoration state, and the §11.5 address-domain
//! derivation (DESIGN.md §11.4/§11.5).
//!
//! The sidebar command slot owns two presentations of the same §11.5 engine
//! (the centered Cmd-T palette and the field-anchored Cmd-L/click overlay);
//! the sidebar address control itself is a display-only pill (§11.5) that
//! shows the derived domain. This module owns the mode enum, the
//! Escape-restoration state, and the pure display derivation the pill
//! renders — scheme/path/query dropping, `www.` elision, IDN decoding,
//! port handling, internal-page placeholders, and the mid-ellipsis budget
//! that shares the pill's reserved accessory slot.

use gpui::Pixels;

use crate::theme::Theme;

/// The command bar either edits the current tab or stages a future tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBarMode {
    /// Browser chrome may be focused, but the command bar is not editing.
    Inactive,
    /// `Cmd-L`: edit or replace the active tab's location.
    Location,
    /// `Cmd-T`: collect a destination without creating a tab or web session.
    NewTab,
    /// `Cmd-Shift-N`: collect a new space name without creating anything
    /// until Enter commits it (the pointer half is the New Space row).
    NewSpace,
    /// `Cmd-Shift-M`: collect a destination space name for moving the active
    /// tab; Enter commits the move, Escape cancels it.
    MoveTab,
}

impl CommandBarMode {
    pub fn is_active(self) -> bool {
        !matches!(self, Self::Inactive)
    }
}

#[cfg(feature = "testkit")]
impl From<CommandBarMode> for bbr_testkit::protocol::CommandBarMode {
    fn from(mode: CommandBarMode) -> Self {
        match mode {
            CommandBarMode::Inactive => Self::Inactive,
            CommandBarMode::Location => Self::Location,
            CommandBarMode::NewTab => Self::NewTab,
            CommandBarMode::NewSpace => Self::NewSpace,
            CommandBarMode::MoveTab => Self::MoveTab,
        }
    }
}

/// State restored when a staged new-tab command is cancelled.
#[derive(Debug, Clone)]
pub struct CommandBarReturn {
    pub mode: CommandBarMode,
    pub text: String,
    pub chrome_focused: bool,
}

/// §11.5 address-control display derivation: the **domain only** — host
/// without scheme, path or query; a leading `www.` elided; IDN punycode
/// decoded to the human-readable form; an explicit port dropped (the pill
/// shows the domain; the full address lives in the palette). Non-http(s)
/// schemes (internal pages) render their scheme's short name as the §17.13
/// placeholder — the exact internal-page naming is an open decision and
/// this slice does not invent it. Empty or unparsable input yields the
/// empty label.
pub(crate) fn address_domain(url: &str) -> String {
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

/// §11.5 address-pill display budget: the domain text area in `type:title`
/// characters, derived from geometry tokens only (the settled S1 scale
/// table, DESIGN.md §5/§8 — no invented sizes): the sidebar width minus the
/// pill's outside `mx` margin, its inner `px` padding, the domain↔accessory
/// gap, and — when the trailing accessory slot is occupied — the slot's
/// reserved `density:hit-min` width. The render always passes `true` for
/// the accessory: the slot is reserved even while empty, so a future §14
/// glyph changes no layout (DESIGN.md §11.5).
pub(crate) fn address_domain_budget_chars(theme: &Theme, accessory: bool) -> usize {
    let density = theme.density;
    let mx = f32::from(crate::theme::Spacing::sp3()) * 2.0;
    let px = f32::from(crate::theme::Spacing::sp3()) * 2.0;
    let gap = f32::from(crate::theme::Spacing::sp2());
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
pub(crate) fn mid_ellipsis(text: &str, budget: usize) -> String {
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
pub(crate) fn accessory_slot_width(density: crate::theme::Density) -> Pixels {
    density.hit_min()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ContrastMode, Density, Theme, ThemeMode};
    use gpui::px;

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
        // Plain ASCII labels stay untouched.
        assert_eq!(address_domain("https://github.com/user/repo"), "github.com");
    }

    #[test]
    fn address_domain_internal_pages_show_the_scheme_placeholder() {
        // §17.13 open decision: the scheme's short name is the placeholder,
        // never an invented internal-page name.
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
        let theme = Theme::new(ThemeMode::Dark, ContrastMode::Standard, Density::Normal);
        let without = address_domain_budget_chars(&theme, false);
        let with_slot = address_domain_budget_chars(&theme, true);
        assert!(
            without > with_slot,
            "the reserved accessory slot consumes display width"
        );
        let long = "verylongsubdomainname.example.com";
        let clipped = mid_ellipsis(long, with_slot);
        assert!(clipped.chars().count() < long.chars().count());
        assert!(
            clipped.ends_with(".com"),
            "the TLD side stays visible: {clipped}"
        );
        assert!(clipped.contains('…'));
        assert!(!clipped.ends_with('…'), "the ellipsis never cuts the tail");
        // A domain that fits the budget is returned unchanged.
        assert_eq!(mid_ellipsis("example.com", with_slot), "example.com");
        // Degenerate budgets never panic or fabricate.
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
