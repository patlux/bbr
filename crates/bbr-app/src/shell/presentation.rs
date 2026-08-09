//! Shared shell display helpers (DESIGN.md §11.8/§11.9).
//!
//! The status line and the empty-pane profile line. The title/URL
//! fallbacks, deterministic favicon monograms, and the separator-aware
//! one-line truncation (§4) moved to `bbr-ui::text` in the G1a cutover;
//! the app imports them from there and never redefines them.

use bbr_browser::ControllerSnapshot;

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

#[cfg(test)]
mod tests {
    use super::*;

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
