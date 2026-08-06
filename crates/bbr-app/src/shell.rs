//! App-local shell formatting and state mapping.
//!
//! Product state lives in `bbr-browser::BrowserController`; this module owns
//! presentation-only derivation (title/URL fallbacks per DESIGN.md §11.9,
//! deterministic favicon monograms per §7.2, per-row visual states per §10)
//! and the app-local typed E2E snapshot adapter.

use bbr_browser::ControllerSnapshot;

pub fn status_line(snapshot: &ControllerSnapshot) -> String {
    // A crash destroyed the session (and its nav state); the error surface
    // derives from the tab's persistent crash flag (§10.7), not `nav`.
    if snapshot
        .active_index
        .and_then(|index| snapshot.tabs.get(index))
        .is_some_and(|tab| tab.crashed)
    {
        return "⚠ Web content process terminated".into();
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

/// DESIGN.md §10 row state for the E2E snapshot's `tab_states` field:
/// error (crash) wins over loading; loading wins over selected; the active
/// row renders `selected`; everything else is `default`.
pub fn row_state(active: bool, loading: bool, crashed: bool) -> crate::components::TabVisualState {
    if crashed {
        return crate::components::TabVisualState::Error;
    }
    if loading {
        return crate::components::TabVisualState::Loading;
    }
    if active {
        return crate::components::TabVisualState::Selected;
    }
    crate::components::TabVisualState::Default
}

/// Index of the first non-pinned tab — the pinned/regular region boundary
/// (DESIGN.md §11.2). The pinned region renders only when it has rows.
pub fn pinned_count(tabs: &[bbr_browser::TabSnapshot]) -> usize {
    tabs.iter().position(|t| !t.pinned).unwrap_or(tabs.len())
}

#[cfg(feature = "testkit")]
#[derive(Debug, Clone, PartialEq)]
pub struct TestSnapshot {
    pub url: Option<String>,
    pub title: Option<String>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub web_view_hosted: bool,
    pub active_profile: bbr_web::ProfileId,
    pub active_space: bbr_testkit::protocol::TestSpaceId,
    pub tabs: Vec<bbr_browser::TabSnapshot>,
    pub active_tab_index: Option<usize>,
    pub active_web_view: Option<bbr_web::WebViewId>,
    pub engine_sessions: usize,
    pub engine_kind: &'static str,
    pub chrome_focused: bool,
    pub shell_modes: bbr_testkit::protocol::ShellModes,
    pub tab_states: Vec<bbr_testkit::protocol::TabVisualState>,
    pub command_bar_mode: bbr_testkit::protocol::CommandBarMode,
    pub command_bar_text: String,
    pub sidebar_visible: bool,
    pub window_size: [f32; 2],
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
    fn row_state_ordering_is_error_then_loading_then_selected() {
        assert_eq!(
            row_state(true, false, true),
            crate::components::TabVisualState::Error
        );
        assert_eq!(
            row_state(true, true, false),
            crate::components::TabVisualState::Loading
        );
        assert_eq!(
            row_state(true, false, false),
            crate::components::TabVisualState::Selected
        );
        assert_eq!(
            row_state(false, false, false),
            crate::components::TabVisualState::Default
        );
        assert_eq!(
            row_state(false, true, true),
            crate::components::TabVisualState::Error
        );
    }

    #[test]
    fn pinned_count_separates_regions() {
        let snap = |index: usize, pinned: bool| bbr_browser::TabSnapshot {
            index,
            url: "https://example.com".into(),
            title: format!("tab {index}"),
            pinned,
            loading: false,
            crashed: false,
            active: index == 0,
            web_view: None,
        };
        let tabs = vec![snap(0, true), snap(1, true), snap(2, false)];
        assert_eq!(pinned_count(&tabs), 2);
        // No pinned rows → boundary at zero.
        let tabs = vec![snap(0, false)];
        assert_eq!(pinned_count(&tabs), 0);
    }
}
