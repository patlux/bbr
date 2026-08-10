//! The read-only observable-state snapshot of the [`super::BrowserController`]
//! (typed [`crate::snapshot::ControllerSnapshot`] the app and testkit
//! consume). Split from `controller.rs` so the gateway module stays under
//! the ~500-line budget.

use super::BrowserController;

impl BrowserController {
    /// Read-only snapshot of the controller's observable state.
    pub fn snapshot(&self) -> crate::snapshot::ControllerSnapshot {
        let tabs: Vec<bbr_domain::TabId> = self
            .browser
            .space(self.active_space)
            .map(|s| s.tabs.clone())
            .unwrap_or_default();
        let active_tab = self
            .browser
            .space(self.active_space)
            .and_then(|s| s.active_tab);
        let tab_snapshots: Vec<crate::snapshot::TabSnapshot> = tabs
            .iter()
            .enumerate()
            .filter_map(|(index, &tab)| {
                let state = self.browser.tab(tab)?;
                let view = self.registry.view_for_tab(tab);
                let session = view.and_then(|v| self.nav.get(&v));
                Some(crate::snapshot::TabSnapshot {
                    index,
                    url: state.url.clone(),
                    title: state.title.clone(),
                    pinned: state.pinned,
                    loading: session.is_some_and(|n| n.loading),
                    crashed: self.crashed_tabs.contains(&tab),
                    active: active_tab == Some(tab),
                    web_view: view,
                    lifecycle: state.lifecycle,
                    last_used: state.last_used,
                })
            })
            .collect();
        let all_tab_snapshots: Vec<crate::snapshot::GlobalTabSnapshot> = self
            .browser
            .spaces()
            .flat_map(|space| {
                space
                    .tabs
                    .iter()
                    .copied()
                    .enumerate()
                    .filter_map(|(index, tab)| {
                        let state = self.browser.tab(tab)?;
                        Some(crate::snapshot::GlobalTabSnapshot {
                            id: tab,
                            space: space.id,
                            profile: space.profile,
                            index,
                            url: state.url.clone(),
                            title: state.title.clone(),
                            pinned: state.pinned,
                            web_view: self.registry.view_for_tab(tab),
                            lifecycle: state.lifecycle,
                            last_used: state.last_used,
                        })
                    })
            })
            .collect();
        let active_index = active_tab.and_then(|t| tabs.iter().position(|x| *x == t));
        let active_view = active_tab.and_then(|t| self.registry.view_for_tab(t));
        let nav = active_view
            .and_then(|v| self.nav.get(&v).cloned())
            .unwrap_or_default();
        let profiles = self
            .browser
            .profiles()
            .enumerate()
            .map(|(index, p)| crate::snapshot::ProfileSnapshot {
                index,
                id: p.id,
                title: p.title.clone(),
                hue: p.hue,
            })
            .collect();
        let spaces = self
            .browser
            .spaces()
            .enumerate()
            .map(|(index, s)| {
                let active_tab = s
                    .active_tab
                    .and_then(|t| s.tabs.iter().position(|x| *x == t));
                crate::snapshot::SpaceSnapshot {
                    index,
                    id: s.id,
                    profile: s.profile,
                    title: s.title.clone(),
                    active_tab,
                    active_view: s.active_tab.and_then(|t| self.registry.view_for_tab(t)),
                    active: s.id == self.active_space,
                    tab_count: s.tabs.len(),
                }
            })
            .collect();
        let active_profile = self.active_profile();
        let lifecycle_counts = crate::snapshot::LifecycleCounts::tally(&tab_snapshots);
        crate::snapshot::ControllerSnapshot {
            active_profile,
            active_profile_title: self
                .browser
                .profile(active_profile)
                .map(|p| p.title.clone()),
            active_profile_hue: self.browser.profile(active_profile).map(|p| p.hue),
            active_space: self.active_space,
            tabs: tab_snapshots,
            all_tabs: all_tab_snapshots,
            profiles,
            spaces,
            active_index,
            nav,
            web_view_hosted: active_view.is_some(),
            engine_sessions: self.engine.borrow().live_session_count(),
            active_view,
            lifecycle_counts,
        }
    }
}
