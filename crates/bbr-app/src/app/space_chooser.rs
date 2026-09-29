//! Blueprint D13 (board PR-20, answer 6): links from other apps open in the
//! last active space; with Settings › General "Ask each time" a small
//! palette chooses the space first. ↑↓ move, Return (or a press) opens the
//! link in that space, Escape drops it.

use bbr_ui::surfaces::space_chooser::SpaceChooserModel;
use gpui::{Context, Window};

use super::BbrApp;

/// The open chooser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SpaceChooserState {
    pub url: String,
    pub selected: usize,
}

impl BbrApp {
    /// A routed link from another app: open it, or ask for the space.
    pub(crate) fn route_external_link(
        &mut self,
        url: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let snapshot = self.controller.snapshot();
        if !self.preferences.external_links_ask || snapshot.spaces.len() < 2 {
            let _ = self.open_tab(url, cx);
            return;
        }
        let selected = snapshot
            .spaces
            .iter()
            .position(|space| space.active)
            .unwrap_or(0);
        self.space_chooser = Some(SpaceChooserState {
            url: url.to_string(),
            selected,
        });
        self.engine.borrow_mut().focus_host_chrome();
        window.focus(&self.chrome_handle, cx);
        self.sync_portal_overlays(cx);
        cx.notify();
    }

    /// The chooser's model.
    pub(crate) fn space_chooser_model(&self) -> Option<SpaceChooserModel> {
        let state = self.space_chooser.as_ref()?;
        let snapshot = self.controller.snapshot();
        let first_profile = snapshot.profiles.first().map(|p| p.id);
        let host = crate::site_permissions::origin_of(&state.url)
            .map(|origin| crate::site_permissions::origin_host(&origin).to_string())
            .unwrap_or_else(|| state.url.clone());
        Some(SpaceChooserModel {
            title: format!("Open {host} in…").into(),
            rows: snapshot
                .spaces
                .iter()
                .map(|space| {
                    let profile = snapshot
                        .profiles
                        .iter()
                        .find(|p| p.id == space.profile)
                        .filter(|p| Some(p.id) != first_profile)
                        .map(|p| format!(" · Profile {}", p.title))
                        .unwrap_or_default();
                    format!("Open in {}{profile}", space.title).into()
                })
                .collect(),
            selected: state.selected,
        })
    }

    /// Open the link in the space at `index`.
    pub(crate) fn space_chooser_choose(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(state) = self.space_chooser.take() else {
            return;
        };
        let space = self.controller.snapshot().spaces.get(index).map(|s| s.id);
        if let Some(space) = space {
            let _ = self.activate_space(space, window, cx);
            if self.open_tab(&state.url, cx).is_some() {
                self.focus_web(window, cx);
            }
        }
        self.sync_portal_overlays(cx);
        cx.notify();
    }

    /// Escape: the link is dropped.
    pub(crate) fn space_chooser_cancel(&mut self, cx: &mut Context<Self>) {
        if self.space_chooser.take().is_some() {
            self.sync_portal_overlays(cx);
            cx.notify();
        }
    }

    /// The chooser's keyboard.
    pub(crate) fn on_space_chooser_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let count = self.controller.snapshot().spaces.len();
        let Some(state) = self.space_chooser.as_mut() else {
            return;
        };
        match event.keystroke.key.as_str() {
            "up" => {
                cx.stop_propagation();
                state.selected = state.selected.saturating_sub(1);
                self.sync_portal_overlays(cx);
            }
            "down" => {
                cx.stop_propagation();
                state.selected = (state.selected + 1).min(count.saturating_sub(1));
                self.sync_portal_overlays(cx);
            }
            "enter" => {
                cx.stop_propagation();
                let index = state.selected;
                self.space_chooser_choose(index, window, cx);
            }
            _ => {}
        }
    }
}
