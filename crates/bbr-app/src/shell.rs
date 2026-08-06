//! App-local shell formatting and state mapping.
//!
//! Product state lives in `bbr-browser::BrowserController`; this module owns
//! presentation-only derivation (title/URL fallbacks per DESIGN.md §11.9,
//! separator-aware one-line truncation per §4, deterministic favicon
//! monograms per §7.2, per-row visual states per §10, the §15 semantic ids of
//! the current render) and the app-local typed E2E snapshot adapter.

use bbr_browser::ControllerSnapshot;

use crate::theme::{Spacing, Theme};

/// DESIGN.md §4 one-line row budget: the title area is the sidebar width
/// minus the fixed row anatomy (2× row padding, the favicon tile, 3
/// inter-item gaps, the 8 pt state slot, and the close control). Per-char
/// width is a conservative 0.45 × the `type:title` size (SF Pro averages
/// ≈0.5 em), so [`truncate_title`] never fires for a title that would fit:
/// GPUI's `text_ellipsis` stays the pixel-exact backstop for the visual cut.
pub fn title_budget_chars(theme: &Theme) -> usize {
    let density = theme.density;
    let padding = f32::from(Spacing::sp2()) * 2.0;
    let favicon = f32::from(density.icon());
    let gaps = f32::from(Spacing::sp2()) * 3.0;
    let state_slot = 8.0;
    let close = f32::from(density.icon());
    let fixed = padding + favicon + gaps + state_slot + close;
    let area = f32::from(density.sidebar_width()) - fixed;
    let per_char = f32::from(theme.type_.title.size) * 0.45;
    (area / per_char).floor().max(1.0) as usize
}

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
    pub active_profile_title: Option<String>,
    pub active_space: bbr_testkit::protocol::TestSpaceId,
    pub tabs: Vec<bbr_browser::TabSnapshot>,
    pub active_tab_index: Option<usize>,
    pub active_web_view: Option<bbr_web::WebViewId>,
    pub engine_sessions: usize,
    pub engine_kind: &'static str,
    pub chrome_focused: bool,
    pub shell_modes: bbr_testkit::protocol::ShellModes,
    pub tab_states: Vec<bbr_testkit::protocol::TabVisualState>,
    pub row_titles: Vec<String>,
    pub visible_ids: Vec<String>,
    pub command_bar_mode: bbr_testkit::protocol::CommandBarMode,
    pub command_bar_text: String,
    pub sidebar_visible: bool,
    pub window_size: [f32; 2],
}

/// The stable `bbr-*` ids present in the current shell render (DESIGN.md
/// §15). Derived from the same structures the renderer consumes — rows via
/// the row-visual indices, the content-pane branch mirroring `Render::render`
/// (native slot / crash panel / empty pane), the command cursor mirroring the
/// command-bar component — so the E2E id assertion cannot drift from the
/// shell: if a spec row's ids are not present here, the E2E fails.
#[cfg(feature = "testkit")]
pub fn visible_ids(
    browser: &ControllerSnapshot,
    chrome_focused: bool,
    command_bar_mode: crate::command_bar::CommandBarMode,
    sidebar_visible: bool,
) -> Vec<String> {
    let mut ids: Vec<String> = vec!["bbr-root".into()];
    if sidebar_visible {
        for id in [
            "bbr-sidebar",
            "bbr-brand",
            "bbr-command-bar",
            "bbr-command-text",
            "bbr-status",
            "bbr-sidebar-toggle",
            "bbr-tab-list",
        ] {
            ids.push(id.into());
        }
    }
    if chrome_focused && command_bar_mode.is_active() {
        ids.push("bbr-command-cursor".into());
    }
    for tab in &browser.tabs {
        let index = tab.index;
        ids.push(format!("bbr-tab-row-{index}"));
        ids.push(format!("bbr-tab-title-{index}"));
        ids.push(format!("bbr-tab-favicon-{index}"));
        ids.push(format!("bbr-tab-state-{index}"));
        ids.push(format!("bbr-tab-close-{index}"));
    }
    let active_tab = browser
        .active_index
        .and_then(|index| browser.tabs.get(index));
    if browser.active_view.is_none() {
        if active_tab.is_some_and(|tab| tab.crashed) {
            ids.extend(
                ["bbr-crash-panel", "bbr-crash-reload", "bbr-crash-close"].map(String::from),
            );
        } else {
            ids.extend(["bbr-empty-new-tab", "bbr-empty-new-tab-button"].map(String::from));
        }
    }
    ids
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
    fn title_budget_tracks_density_geometry() {
        let normal = Theme::dark();
        let small = Theme::new(
            crate::theme::ThemeMode::Dark,
            crate::theme::ContrastMode::Standard,
            crate::theme::Density::Small,
        );
        let xl = Theme::new(
            crate::theme::ThemeMode::Dark,
            crate::theme::ContrastMode::Standard,
            crate::theme::Density::ExtraLarge,
        );
        // Normal density ships the 260 pt sidebar: ~30 chars at 13 pt title.
        assert!(title_budget_chars(&normal) >= 25);
        assert!(title_budget_chars(&normal) <= 40);
        // Wider sidebars get a larger budget than narrow ones.
        assert!(title_budget_chars(&xl) > title_budget_chars(&small));
    }

    #[test]
    fn profile_line_names_the_owning_profile() {
        assert_eq!(profile_line("Default"), "in Profile · Default");
        assert_eq!(profile_line("Private"), "in Profile · Private");
    }

    #[cfg(feature = "testkit")]
    #[test]
    fn visible_ids_mirror_the_render_for_each_pane_branch() {
        let snap = |index: usize, crashed: bool| bbr_browser::TabSnapshot {
            index,
            url: "https://example.com".into(),
            title: format!("tab {index}"),
            pinned: false,
            loading: false,
            crashed,
            active: index == 0,
            web_view: None,
        };
        let tabs = vec![snap(0, false), snap(1, false)];
        let browser = ControllerSnapshot {
            active_profile: bbr_web::ProfileId(1),
            active_profile_title: Some("Default".into()),
            active_profile_hue: Some(bbr_domain::ProfileHue::Cobalt),
            active_space: bbr_browser::SpaceId(1),
            tabs: tabs.clone(),
            profiles: Vec::new(),
            spaces: Vec::new(),
            active_index: Some(0),
            nav: Default::default(),
            web_view_hosted: false,
            engine_sessions: 0,
            active_view: None,
        };
        let ids = visible_ids(
            &browser,
            false,
            crate::command_bar::CommandBarMode::Inactive,
            true,
        );
        for required in [
            "bbr-root",
            "bbr-sidebar",
            "bbr-tab-list",
            "bbr-tab-row-0",
            "bbr-tab-row-1",
            "bbr-tab-title-1",
            "bbr-tab-favicon-1",
            "bbr-tab-state-1",
            "bbr-tab-close-1",
            "bbr-empty-new-tab",
        ] {
            assert!(
                ids.contains(&required.to_string()),
                "missing {required}: {ids:?}"
            );
        }
        assert!(!ids.iter().any(|id| id.starts_with("bbr-crash")));

        // Crashed active tab: the crash panel ids replace the empty pane.
        let mut crashed = tabs;
        crashed[0].crashed = true;
        let browser = ControllerSnapshot {
            active_profile: bbr_web::ProfileId(1),
            active_profile_title: Some("Default".into()),
            active_profile_hue: Some(bbr_domain::ProfileHue::Cobalt),
            active_space: bbr_browser::SpaceId(1),
            tabs: crashed,
            profiles: Vec::new(),
            spaces: Vec::new(),
            active_index: Some(0),
            nav: Default::default(),
            web_view_hosted: false,
            engine_sessions: 0,
            active_view: None,
        };
        let ids = visible_ids(
            &browser,
            false,
            crate::command_bar::CommandBarMode::Inactive,
            true,
        );
        for required in ["bbr-crash-panel", "bbr-crash-reload", "bbr-crash-close"] {
            assert!(
                ids.contains(&required.to_string()),
                "missing {required}: {ids:?}"
            );
        }
        assert!(!ids.iter().any(|id| id.starts_with("bbr-empty")));

        // Active command bar in NewTab mode: the cursor id appears.
        let ids = visible_ids(
            &browser,
            true,
            crate::command_bar::CommandBarMode::NewTab,
            true,
        );
        assert!(ids.contains(&"bbr-command-cursor".to_string()));
    }
}
