//! Shared shell display helpers (DESIGN.md §11.8/§11.9).
//!
//! The status-line text derivation and the empty-pane profile line. The
//! status row anatomy moved to `bbr-ui::components::status` in the G2
//! cutover; the app keeps only the controller-snapshot derivation
//! ([`status_model`]) and never redefines the row. The title/URL
//! fallbacks, deterministic favicon monograms, and the separator-aware
//! one-line truncation (§4) moved to `bbr-ui::text` in the G1a cutover;
//! the app imports them from there and never redefines them.

use bbr_browser::ControllerSnapshot;
use bbr_ui::components::StatusModel;

/// DESIGN.md §11.8 empty-pane profile line ("in Profile · Work").
pub fn profile_line(profile_title: &str) -> String {
    format!("in Profile · {profile_title}")
}

/// The §11.4 status-line text derived from the controller snapshot (the
/// app-owned half of the G2 status cutover; the row anatomy lives in
/// `bbr-ui::components::status`).
pub fn status_model(snapshot: &ControllerSnapshot) -> StatusModel {
    // A crash destroyed the session (and its nav state); the error surface
    // derives from the tab's persistent crash flag (§10.7), not `nav`. The
    // line carries no `⚠` glyph: §14 forbids emoji in chrome, and the error
    // state's non-color companions (§3.3) are the crash panel's drawn §7
    // lifering glyph, the error surface tint, and this copy.
    let text: String = if snapshot
        .active_index
        .and_then(|index| snapshot.tabs.get(index))
        .is_some_and(|tab| tab.crashed)
    {
        "Web content process terminated".into()
    } else if snapshot
        .active_index
        .and_then(|index| snapshot.tabs.get(index))
        .is_some_and(|tab| tab.unresponsive)
    {
        // §10.11: the renderer stopped answering liveness probes; the
        // session and tab stay alive under the §11.8a recovery surface.
        "Page is not responding".into()
    } else if snapshot.nav.loading {
        "Loading…".into()
    } else {
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
    };
    StatusModel { text: text.into() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot_with_tab(crashed: bool, title: &str, url: &str) -> ControllerSnapshot {
        ControllerSnapshot {
            active_profile: bbr_web::ProfileId(1),
            active_profile_title: None,
            active_profile_hue: None,
            active_space: bbr_browser::SpaceId(1),
            tabs: vec![bbr_browser::TabSnapshot {
                index: 0,
                url: url.into(),
                title: title.into(),
                pinned: false,
                loading: false,
                crashed,
                unresponsive: false,
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
            preview: None,
        }
    }

    #[test]
    fn profile_line_names_the_owning_profile() {
        assert_eq!(profile_line("Default"), "in Profile · Default");
        assert_eq!(profile_line("Private"), "in Profile · Private");
    }

    #[test]
    fn status_model_never_embeds_emoji_glyphs() {
        // DESIGN.md §14: no emoji in chrome. The crash line names the state
        // in plain text; the error glyph is the drawn §7 lifering in the
        // crash panel and the row error surface, never a Unicode symbol.
        let mut snapshot = snapshot_with_tab(true, "Example", "https://example.com");
        let model = status_model(&snapshot);
        assert_eq!(model.text, "Web content process terminated");
        assert!(
            model.text.is_ascii(),
            "status line carries no non-ASCII/emoji characters: {:?}",
            model.text
        );
        snapshot.tabs[0].crashed = false;
        snapshot.tabs[0].active = true;
        assert_eq!(status_model(&snapshot).text, "Example");
    }

    #[test]
    fn status_model_derives_the_section_11_8_branches() {
        // Loading wins over the nav title/URL fallbacks.
        let mut snapshot = snapshot_with_tab(false, "Example", "https://example.com");
        snapshot.nav.loading = true;
        assert_eq!(status_model(&snapshot).text, "Loading…");
        snapshot.nav.loading = false;

        // A nav title beats the URL; an empty title falls back to the URL.
        snapshot.nav.url = Some("https://example.com/path".into());
        snapshot.nav.title = Some("Example".into());
        assert_eq!(status_model(&snapshot).text, "Example");
        snapshot.nav.title = Some(String::new());
        assert_eq!(status_model(&snapshot).text, "https://example.com/path");
        snapshot.nav.title = None;
        assert_eq!(status_model(&snapshot).text, "https://example.com/path");

        // Without nav state the active tab's title/URL carry the line.
        snapshot.nav.url = None;
        snapshot.nav.title = None;
        assert_eq!(status_model(&snapshot).text, "Example");
        snapshot.tabs[0].title = String::new();
        assert_eq!(status_model(&snapshot).text, "https://example.com");

        // No active tab: the empty line.
        snapshot.active_index = None;
        assert_eq!(status_model(&snapshot).text, "No page");
    }
}
