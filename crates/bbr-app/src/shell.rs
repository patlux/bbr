//! App-local shell formatting and state mapping.
//!
//! Product state lives in `bbr-browser::BrowserController`; this module owns
//! presentation-only derivation (title/URL fallbacks per DESIGN.md §11.9,
//! separator-aware one-line truncation per §4, deterministic favicon
//! monograms per §7.2, per-row visual states per §10, the §15 semantic ids of
//! the current render) and the app-local typed E2E snapshot adapter.

use bbr_browser::{ControllerSnapshot, SpaceSnapshot};

use crate::components::{ProfileChipSpec, SpaceRowVisuals, SpaceVisualState};
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

/// DESIGN.md §12 profile chip monogram: the first 1–2 title characters,
/// uppercased. Identity is readable per row without a header.
pub fn profile_monogram(profile_title: &str) -> String {
    let mut chars = profile_title.chars();
    let first = chars.next().unwrap_or('?');
    let second = chars.next();
    let mut monogram = String::new();
    monogram.push(first.to_ascii_uppercase());
    if let Some(ch) = second {
        monogram.push(ch.to_ascii_uppercase());
    }
    monogram
}

/// One sidebar space group: the spaces of one profile, in creation order,
/// preceded by a hairline when it is not the first group (DESIGN.md §11.3:
/// profile groups stack under a hairline, no repeated profile header in v1).
pub struct SpaceGroup {
    pub rows: Vec<SpaceRowVisuals>,
    pub hairline_before: bool,
}

/// Render-order space rows grouped by profile, derived from the controller
/// snapshot — the same source the sidebar renders (DESIGN.md §11.3).
pub fn space_groups(browser: &ControllerSnapshot, budget: usize) -> Vec<SpaceGroup> {
    let mut groups: Vec<SpaceGroup> = Vec::new();
    for profile in &browser.profiles {
        let spaces: Vec<&SpaceSnapshot> = browser
            .spaces
            .iter()
            .filter(|s| s.profile == profile.id)
            .collect();
        if spaces.is_empty() {
            continue;
        }
        let rows = spaces
            .iter()
            .map(|s| SpaceRowVisuals {
                index: s.index,
                profile_index: profile.index,
                title: crate::shell::truncate_title(&s.title, budget).into(),
                count: s.tab_count,
                state: if s.active {
                    SpaceVisualState::Selected
                } else {
                    SpaceVisualState::Default
                },
                chip: ProfileChipSpec {
                    letter: profile_monogram(&profile.title),
                    hue: profile.hue,
                },
            })
            .collect();
        groups.push(SpaceGroup {
            rows,
            hairline_before: !groups.is_empty(),
        });
    }
    groups
}

/// The §11.3 space-row title budget: sidebar width minus the fixed row
/// anatomy (2× padding, the 20 pt chip, 2 gaps, the count column). Per-char
/// width is the same conservative 0.45 × `type:title` estimate as the tab
/// row budget.
pub fn space_title_budget_chars(theme: &Theme) -> usize {
    let density = theme.density;
    let padding = f32::from(Spacing::sp2()) * 2.0;
    let chip = 20.0;
    let gaps = f32::from(Spacing::sp2()) * 2.0;
    let count = 24.0;
    let area = f32::from(density.sidebar_width()) - padding - chip - gaps - count;
    let per_char = f32::from(theme.type_.title.size) * 0.45;
    (area / per_char).floor().max(1.0) as usize
}

/// §11.3 space visual states in render order (grouped), mirroring the rows
/// the sidebar renders — `selected` for the active space, else `default`.
#[cfg(any(feature = "testkit", test))]
pub fn space_states(browser: &ControllerSnapshot) -> Vec<SpaceVisualState> {
    space_groups(browser, usize::MAX)
        .into_iter()
        .flat_map(|group| group.rows.into_iter().map(|row| row.state))
        .collect()
}

/// §11.3 space row titles in render order, after §4 truncation — the same
/// `space_groups` source the sidebar renders.
#[cfg(any(feature = "testkit", test))]
pub fn space_titles(browser: &ControllerSnapshot, budget: usize) -> Vec<String> {
    space_groups(browser, budget)
        .into_iter()
        .flat_map(|group| group.rows.into_iter().map(|row| row.title.to_string()))
        .collect()
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
    /// The active profile's identity hue name (DESIGN.md §12).
    pub active_profile_hue: Option<String>,
    pub active_space: bbr_testkit::protocol::TestSpaceId,
    pub tabs: Vec<bbr_browser::TabSnapshot>,
    /// Complete profile/space collections in creation order.
    pub profiles: Vec<bbr_browser::ProfileSnapshot>,
    pub spaces: Vec<bbr_browser::SpaceSnapshot>,
    pub active_tab_index: Option<usize>,
    pub active_web_view: Option<bbr_web::WebViewId>,
    pub engine_sessions: usize,
    pub engine_kind: &'static str,
    pub chrome_focused: bool,
    pub shell_modes: bbr_testkit::protocol::ShellModes,
    pub tab_states: Vec<bbr_testkit::protocol::TabVisualState>,
    pub row_titles: Vec<String>,
    /// §11.3 space states in render order (selected for the active space).
    pub space_states: Vec<bbr_testkit::protocol::SpaceVisualState>,
    /// §11.3 space row titles in render order, after §4 truncation.
    pub space_titles: Vec<String>,
    pub visible_ids: Vec<String>,
    pub command_bar_mode: bbr_testkit::protocol::CommandBarMode,
    pub command_bar_text: String,
    pub sidebar_visible: bool,
    /// The effective sidebar rail width: the DESIGN §5 adjustable width
    /// clamped to the window (0 when collapsed — the rail is gone).
    pub sidebar_width: f32,
    /// The DESIGN §5 adjustable-range contract, exposed so E2E asserts the
    /// same constants the render enforces.
    pub sidebar_min: f32,
    pub sidebar_default: f32,
    pub sidebar_max: f32,
    /// The resize divider drag state (idle vs dragging).
    pub divider_dragging: bool,
    /// The stable divider label (future Accessibility; §15 shared identity).
    pub divider_label: String,
    /// The last native slot rect handed to the engine (WKWebView frame).
    pub content_rect: [f32; 4],
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
            "bbr-space-list",
            "bbr-space-new",
            "bbr-tab-list",
            "bbr-sidebar-divider",
        ] {
            ids.push(id.into());
        }
        for space in &browser.spaces {
            ids.push(format!("bbr-space-row-{}", space.index));
            ids.push(format!("bbr-space-title-{}", space.index));
            ids.push(format!("bbr-space-count-{}", space.index));
        }
        for profile in &browser.profiles {
            ids.push(crate::components::profile_chip_id(profile.index).to_string());
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

    #[test]
    fn profile_monogram_uses_first_two_title_chars() {
        assert_eq!(profile_monogram("Default"), "DE");
        assert_eq!(profile_monogram("Private"), "PR");
        assert_eq!(profile_monogram("w"), "W");
        assert_eq!(profile_monogram(""), "?");
    }

    /// Build a snapshot with two profiles and three spaces (two in the first
    /// profile, one in the second) for the grouping derivation tests.
    fn space_snapshot() -> ControllerSnapshot {
        let space =
            |index: usize, id: u64, profile: u64, title: &str, active: bool, tabs: usize| {
                bbr_browser::SpaceSnapshot {
                    index,
                    id: bbr_browser::SpaceId(id),
                    profile: bbr_web::ProfileId(profile),
                    title: title.into(),
                    active_tab: Some(0),
                    active_view: None,
                    active,
                    tab_count: tabs,
                }
            };
        ControllerSnapshot {
            active_profile: bbr_web::ProfileId(1),
            active_profile_title: Some("Default".into()),
            active_profile_hue: Some(bbr_browser::ProfileHue::Cobalt),
            active_space: bbr_browser::SpaceId(2),
            tabs: Vec::new(),
            profiles: vec![
                bbr_browser::ProfileSnapshot {
                    index: 0,
                    id: bbr_web::ProfileId(1),
                    title: "Default".into(),
                    hue: bbr_browser::ProfileHue::Cobalt,
                },
                bbr_browser::ProfileSnapshot {
                    index: 1,
                    id: bbr_web::ProfileId(3),
                    title: "Private".into(),
                    hue: bbr_browser::ProfileHue::Violet,
                },
            ],
            spaces: vec![
                space(0, 2, 1, "Default", true, 3),
                space(1, 4, 3, "Private", false, 1),
                space(2, 5, 1, "Reading", false, 0),
            ],
            active_index: Some(0),
            nav: Default::default(),
            web_view_hosted: false,
            engine_sessions: 0,
            active_view: None,
        }
    }

    #[test]
    fn space_groups_stack_profiles_with_hairlines() {
        let browser = space_snapshot();
        let groups = space_groups(&browser, 30);
        assert_eq!(groups.len(), 2);
        // Default profile first (creation order), then Private; a hairline
        // separates the groups; every row carries its profile chip.
        assert!(!groups[0].hairline_before);
        assert!(groups[1].hairline_before);
        assert_eq!(
            groups[0].rows.iter().map(|r| r.index).collect::<Vec<_>>(),
            vec![0, 2],
            "both default-profile spaces render in creation order"
        );
        assert_eq!(
            groups[1].rows.iter().map(|r| r.index).collect::<Vec<_>>(),
            vec![1]
        );
        // Chip identity: hue + monogram per row, never a repeated header.
        assert_eq!(groups[0].rows[0].chip.hue, bbr_browser::ProfileHue::Cobalt);
        assert_eq!(groups[0].rows[0].chip.letter, "DE");
        assert_eq!(groups[1].rows[0].chip.hue, bbr_browser::ProfileHue::Violet);
        assert_eq!(groups[1].rows[0].chip.letter, "PR");
        // The §11.3 count column mirrors each space's tab count.
        assert_eq!(groups[0].rows[0].count, 3);
        assert_eq!(groups[1].rows[0].count, 1);
    }

    #[test]
    fn space_states_mark_only_the_active_space() {
        let browser = space_snapshot();
        assert_eq!(
            space_states(&browser),
            vec![
                SpaceVisualState::Selected,
                SpaceVisualState::Default,
                SpaceVisualState::Default,
            ]
        );
        assert_eq!(
            space_titles(&browser, 30),
            vec![
                "Default".to_string(),
                "Reading".to_string(),
                "Private".to_string()
            ]
        );
    }

    #[test]
    fn space_titles_follow_the_truncation_contract() {
        let browser = space_snapshot();
        // A long name with an early separator cuts so the head survives (§4);
        // names without a separator keep the full title (GPUI's text_ellipsis
        // is the pixel-exact backstop).
        let mut long = space_snapshot();
        long.spaces[0].title =
            "Reading List – a long suffix that surely exceeds the row budget".into();
        let titles = space_titles(&long, 20);
        assert_eq!(titles[0], "Reading List…");
        assert_eq!(
            space_titles(&browser, 6)[0],
            "Default",
            "no-separator names stay whole"
        );
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
            active_profile_hue: Some(bbr_browser::ProfileHue::Cobalt),
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
        let mut crashed = tabs;
        crashed[0].crashed = true;
        let browser = ControllerSnapshot {
            active_profile: bbr_web::ProfileId(1),
            active_profile_title: Some("Default".into()),
            active_profile_hue: Some(bbr_browser::ProfileHue::Cobalt),
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

    #[cfg(feature = "testkit")]
    #[test]
    fn space_row_chips_key_to_the_owning_profile_index() {
        // DESIGN.md §12/§15: the chip id follows the profile's creation-order
        // index, never the space index. In `space_snapshot` profile 0 owns
        // spaces 0 and 2 while profile 1 owns space 1, so space indices and
        // chip ids diverge. Regression: the render previously keyed the chip
        // to the space index, attaching `bbr-profile-chip-2` to a row the
        // contract identifies as `bbr-profile-chip-0`.
        let browser = space_snapshot();
        let ids = visible_ids(
            &browser,
            false,
            crate::command_bar::CommandBarMode::Inactive,
            true,
        );
        let chips = space_groups(&browser, usize::MAX)
            .into_iter()
            .flat_map(|group| group.rows)
            .map(|row| crate::components::profile_chip_id(row.profile_index).to_string())
            .collect::<Vec<_>>();
        // Render order: Default's spaces (indices 0, 2), then Private's space
        // (index 1). Both Default rows attach the same profile-keyed chip.
        assert_eq!(
            chips,
            vec![
                "bbr-profile-chip-0".to_string(),
                "bbr-profile-chip-0".to_string(),
                "bbr-profile-chip-1".to_string(),
            ]
        );
        for chip in &chips {
            assert!(
                ids.contains(chip),
                "rendered chip {chip} missing from the semantic snapshot"
            );
        }
        // The divergent space index 2 must never leak into a chip id.
        assert!(!ids.iter().any(|id| id == "bbr-profile-chip-2"));
    }

    #[cfg(feature = "testkit")]
    #[test]
    fn visible_ids_include_space_rows_and_profile_chips() {
        let browser = space_snapshot();
        let ids = visible_ids(
            &browser,
            false,
            crate::command_bar::CommandBarMode::Inactive,
            true,
        );
        for required in [
            "bbr-space-list",
            "bbr-space-new",
            "bbr-space-row-0",
            "bbr-space-row-1",
            "bbr-space-row-2",
            "bbr-space-title-0",
            "bbr-space-count-1",
            "bbr-profile-chip-0",
            "bbr-profile-chip-1",
        ] {
            assert!(
                ids.contains(&required.to_string()),
                "missing {required}: {ids:?}"
            );
        }
        // Sidebar hidden: space rows and chips disappear with the rail.
        let hidden = visible_ids(
            &browser,
            false,
            crate::command_bar::CommandBarMode::Inactive,
            false,
        );
        assert!(!hidden.iter().any(|id| id.starts_with("bbr-space")));
    }
}
