//! App-local typed snapshot converted to the portable testkit wire format.
//!
//! Product state lives in `bbr-browser::BrowserController`; this module owns
//! no duplicate tab or navigation model.

use bbr_browser::ControllerSnapshot;

pub fn status_line(snapshot: &ControllerSnapshot) -> String {
    if snapshot.nav.crashed {
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

#[cfg(feature = "testkit")]
#[derive(Debug, Clone, PartialEq)]
pub struct TestSnapshot {
    pub url: Option<String>,
    pub title: Option<String>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub web_view_hosted: bool,
    pub tabs: Vec<bbr_browser::TabSnapshot>,
    pub active_tab_index: Option<usize>,
    pub active_web_view: Option<bbr_web::WebViewId>,
    pub engine_sessions: usize,
    pub engine_kind: &'static str,
    pub chrome_focused: bool,
    pub command_bar_text: String,
    pub sidebar_visible: bool,
    pub window_size: [f32; 2],
}
