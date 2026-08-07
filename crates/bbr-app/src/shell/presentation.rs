//! Shared shell display helpers (DESIGN.md §11.8/§11.9, §4, §7.2).
//!
//! Title/URL fallbacks, deterministic favicon monograms, the status line,
//! and the empty-pane profile line. The separator-aware one-line truncation
//! (§4) lives here because both tab rows and space rows consume it.

use bbr_browser::ControllerSnapshot;

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

/// DESIGN.md §11.8 empty-pane profile line ("in Profile · Work").
pub fn profile_line(profile_title: &str) -> String {
    format!("in Profile · {profile_title}")
}

pub fn status_line(snapshot: &ControllerSnapshot) -> String {
    // A crash destroyed the session (and its nav state); the error surface
    // derives from the tab's persistent crash flag (§10.7), not `nav`. The
    // line carries no `⚠` glyph: §14 forbids emoji in chrome, and the error
    // state's non-color companions (§3.3) are the crash panel's drawn §7
    // lifering glyph, the error surface tint, and this copy.
    if snapshot
        .active_index
        .and_then(|index| snapshot.tabs.get(index))
        .is_some_and(|tab| tab.crashed)
    {
        return "Web content process terminated".into();
    }
    if snapshot.nav.loading {
        return "Loading…".into();
    }
    match (&snapshot.nav.url, &snapshot.nav.title) {
        (Some(_), Some(title)) if !title.is_empty() => title.clone(),
        (Some(url), _) => url.clone(),
        _ => snapshot
            .active_index
            .and_then(|index| snapshot.tabs.get(index))
            .map(|tab| {
                if tab.title.is_empty() {
                    tab.url.clone()
                } else {
                    tab.title.clone()
                }
            })
            .unwrap_or_else(|| "No page".into()),
    }
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

/// First character of the registrable hostname, uppercased — the §7.2
/// monogram tile letter. Falls back to the scheme's first letter.
pub fn monogram_letter(url: &str) -> char {
    let host = host_of(url);
    let source = if host.is_empty() {
        scheme_of(url)
    } else {
        host
    };
    source
        .chars()
        .next()
        .map(|c| c.to_ascii_uppercase())
        .unwrap_or('?')
}

/// FNV-1a of the hostname mod 6 — the deterministic slate-hue index for the
/// §7.2 monogram tile ramp. Stable per host.
pub fn monogram_hue(url: &str) -> usize {
    let host = host_of(url);
    let mut hash = 0x811c9dc5u32;
    for byte in host.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x01000193);
    }
    (hash as usize) % 6
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn monogram_is_deterministic_per_host() {
        assert_eq!(monogram_letter("https://example.com/x"), 'E');
        assert_eq!(monogram_letter("http://localhost:8080/x"), 'L');
        assert_eq!(monogram_letter("about:blank"), 'A');
        assert_eq!(
            monogram_letter("https://example.com/x"),
            monogram_letter("https://example.com/y")
        );
        assert_eq!(
            monogram_hue("https://example.com"),
            monogram_hue("https://example.com/other")
        );
        assert!((0..6).contains(&monogram_hue("https://example.com")));
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
        // Short titles with separators fit and are never cut.
        assert_eq!(truncate_title("GitHub – My Repo", 30), "GitHub – My Repo");
        assert_eq!(truncate_title("Short", 30), "Short");
        // Titles exactly at the budget stay whole.
        let at_budget: String = "x".repeat(30);
        assert_eq!(truncate_title(&at_budget, 30), at_budget);
    }

    #[test]
    fn truncate_title_falls_back_to_plain_when_head_does_not_fit() {
        // A separator so late that the pre-separator head itself overflows:
        // the full title is kept and GPUI's text_ellipsis does the visual cut.
        let late =
            "an extremely long first segment that by itself exceeds the row budget – with a suffix";
        assert_eq!(truncate_title(late, 30), late);
        // No separator at all: plain end-ellipsis (visual backstop).
        let plain =
            "just one very long unbroken descriptive phrase with no separator anywhere at all";
        assert_eq!(truncate_title(plain, 30), plain);
    }

    #[test]
    fn profile_line_names_the_owning_profile() {
        assert_eq!(profile_line("Default"), "in Profile · Default");
        assert_eq!(profile_line("Private"), "in Profile · Private");
    }

    #[test]
    fn status_line_never_embeds_emoji_glyphs() {
        // DESIGN.md §14: no emoji in chrome. The crash line names the state
        // in plain text; the error glyph is the drawn §7 lifering in the
        // crash panel and the row error surface, never a Unicode symbol.
        let mut snapshot = ControllerSnapshot {
            active_profile: bbr_web::ProfileId(1),
            active_profile_title: None,
            active_profile_hue: None,
            active_space: bbr_browser::SpaceId(1),
            tabs: vec![bbr_browser::TabSnapshot {
                index: 0,
                url: "https://example.com".into(),
                title: "Example".into(),
                pinned: false,
                loading: false,
                crashed: true,
                active: true,
                web_view: Some(bbr_web::WebViewId(1)),
                lifecycle: bbr_browser::TabLifecycle::Warm,
                last_used: 0,
            }],
            all_tabs: vec![],
            profiles: vec![],
            spaces: vec![],
            active_index: Some(0),
            nav: Default::default(),
            web_view_hosted: true,
            engine_sessions: 1,
            active_view: Some(bbr_web::WebViewId(1)),
            lifecycle_counts: Default::default(),
        };
        let line = status_line(&snapshot);
        assert_eq!(line, "Web content process terminated");
        assert!(
            line.is_ascii(),
            "status line carries no non-ASCII/emoji characters: {line:?}"
        );
        snapshot.tabs[0].crashed = false;
        snapshot.tabs[0].active = true;
        assert_eq!(status_line(&snapshot), "Example");
    }
}
