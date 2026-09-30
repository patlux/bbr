//! Blueprint D15 (boards L-1, L-2, PR-01, DESIGN §11.8): the first run.
//!
//! The very first launch (a fresh store, no address asked for, never
//! welcomed before) opens one empty space whose card shows the welcome
//! instead of the empty-space illustration. It never returns: the
//! preferences remember it at once. Get Started ends it and opens the ⌘T
//! palette; "Import from Safari or Chrome…" leads through three skippable
//! steps — import, default browser, first space name — after which the
//! empty space remains. Opening a tab also ends it. Import reads the
//! sources in the background and lands bookmarks in Saved Pages and
//! history in History, then says how much came over.

use bbr_ui::surfaces::welcome::{ImportRow, WelcomeEvent, WelcomeStep};
use gpui::{Context, Window};

use super::BbrApp;
use crate::import::{self, SourceData};

/// Which page shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OnboardingStep {
    Welcome,
    Import,
    DefaultBrowser,
    FirstSpace,
}

/// The first run while it shows.
pub(crate) struct OnboardingState {
    pub step: OnboardingStep,
    /// The sources found (`None` while they are read).
    pub sources: Option<Vec<SourceData>>,
    pub checked: Vec<bool>,
    pub name: String,
    pub name_focused: bool,
    read_task: Option<gpui::Task<()>>,
}

impl OnboardingState {
    fn new() -> Self {
        OnboardingState {
            step: OnboardingStep::Welcome,
            sources: None,
            checked: Vec::new(),
            name: String::new(),
            name_focused: false,
            read_task: None,
        }
    }
}

/// Whether BBR is macOS's default browser now.
pub(crate) fn is_default_browser() -> bool {
    #[cfg(target_os = "macos")]
    {
        bbr_web_macos::is_default_browser()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// "Imported 312 Saved Pages and 1 204 history entries".
pub(crate) fn imported_message(pages: usize, visits: usize) -> String {
    let pages = match pages {
        1 => "1 Saved Page".to_string(),
        n => format!("{} Saved Pages", import::grouped(n)),
    };
    let visits = match visits {
        1 => "1 history entry".to_string(),
        n => format!("{} history entries", import::grouped(n)),
    };
    format!("Imported {pages} and {visits}")
}

impl BbrApp {
    /// The first launch: show the welcome and remember it was shown.
    pub(crate) fn start_welcome(&mut self, cx: &mut Context<Self>) {
        self.onboarding = Some(OnboardingState::new());
        self.update_preferences(|p| p.welcomed = true, cx);
    }

    /// Whether the card shows the first run (the space is still empty).
    pub(crate) fn onboarding_visible(&self) -> bool {
        self.onboarding.is_some() && self.controller.read().snapshot().tabs.is_empty()
    }

    /// The page for the card.
    pub(crate) fn welcome_step(&self) -> Option<WelcomeStep> {
        let state = self.onboarding.as_ref()?;
        Some(match state.step {
            OnboardingStep::Welcome => WelcomeStep::Welcome,
            OnboardingStep::Import => WelcomeStep::Import(state.sources.as_ref().map(|sources| {
                sources
                    .iter()
                    .enumerate()
                    .map(|(index, source)| ImportRow {
                        label: source.source.label().into(),
                        detail: source.summary().into(),
                        checked: state.checked.get(index).copied().unwrap_or(false),
                        enabled: source.error.is_none(),
                    })
                    .collect()
            })),
            OnboardingStep::DefaultBrowser => WelcomeStep::DefaultBrowser {
                is_default: crate::app::onboarding::is_default_browser(),
            },
            OnboardingStep::FirstSpace => WelcomeStep::FirstSpace {
                name: state.name.clone().into(),
                focused: state.name_focused,
            },
        })
    }

    fn import_home(&self) -> std::path::PathBuf {
        self.import_home
            .clone()
            .or_else(|| std::env::var_os("HOME").map(std::path::PathBuf::from))
            .unwrap_or_default()
    }

    /// Every first-run control (pointer, keys and the testkit).
    pub(crate) fn on_welcome_event(
        &mut self,
        event: WelcomeEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(state) = self.onboarding.as_mut() else {
            return;
        };
        match event {
            WelcomeEvent::GetStarted => {
                self.onboarding = None;
                self.open_palette(window, cx);
                return;
            }
            WelcomeEvent::StartImport => {
                state.step = OnboardingStep::Import;
                state.sources = None;
                let home = self.import_home();
                let task = cx
                    .background_executor()
                    .spawn(async move { import::detect(&home) });
                let read = cx.spawn(async move |this, cx| {
                    let sources = task.await;
                    let _ = this.update(cx, |app, cx| {
                        if let Some(state) = app.onboarding.as_mut() {
                            state.checked = sources.iter().map(|s| s.error.is_none()).collect();
                            state.sources = Some(sources);
                            cx.notify();
                        }
                    });
                });
                if let Some(state) = self.onboarding.as_mut() {
                    state.read_task = Some(read);
                }
            }
            WelcomeEvent::ToggleSource(index) => {
                let enabled = state
                    .sources
                    .as_ref()
                    .and_then(|sources| sources.get(index))
                    .is_some_and(|source| source.error.is_none());
                if let (true, Some(checked)) = (enabled, state.checked.get_mut(index)) {
                    *checked = !*checked;
                }
            }
            WelcomeEvent::Skip => match state.step {
                OnboardingStep::Welcome => self.onboarding = None,
                OnboardingStep::Import => state.step = OnboardingStep::DefaultBrowser,
                OnboardingStep::DefaultBrowser => state.step = OnboardingStep::FirstSpace,
                OnboardingStep::FirstSpace => self.onboarding = None,
            },
            WelcomeEvent::Import => {
                self.apply_import(cx);
                if let Some(state) = self.onboarding.as_mut() {
                    state.step = OnboardingStep::DefaultBrowser;
                }
            }
            WelcomeEvent::MakeDefault => {
                state.step = OnboardingStep::FirstSpace;
                self.request_default_browser();
            }
            WelcomeEvent::FocusName => {
                state.name_focused = true;
                self.engine.borrow_mut().focus_host_chrome();
                window.focus(&self.chrome_handle, cx);
            }
            WelcomeEvent::Done => {
                let name = state.name.trim().to_string();
                self.onboarding = None;
                if !name.is_empty() {
                    let space = self.controller.read().snapshot().active_space;
                    let renamed = self.controller.get().rename_space(space, name);
                    match renamed {
                        Ok(_) => self.persist_structure(),
                        Err(error) => log::warn!("the first space kept its name: {error}"),
                    }
                }
            }
        }
        // The name field takes the keyboard when its step appears.
        if let Some(state) = self.onboarding.as_mut() {
            if state.step == OnboardingStep::FirstSpace && !state.name_focused {
                state.name_focused = true;
                self.engine.borrow_mut().focus_host_chrome();
                window.focus(&self.chrome_handle, cx);
            }
        }
        cx.notify();
    }

    /// Land the checked sources: bookmarks as Saved Pages, history in
    /// History; then say how much came over.
    fn apply_import(&mut self, cx: &mut Context<Self>) {
        let Some(state) = self.onboarding.as_ref() else {
            return;
        };
        let Some(sources) = state.sources.as_ref() else {
            return;
        };
        let chosen: Vec<SourceData> = sources
            .iter()
            .zip(&state.checked)
            .filter(|(source, checked)| **checked && source.error.is_none())
            .map(|(source, _)| source.clone())
            .collect();
        if chosen.is_empty() {
            return;
        }
        let mut pages = 0;
        for bookmark in chosen.iter().flat_map(|source| &source.bookmarks) {
            if let Ok(canonical) = bbr_domain::canonical_url(&bookmark.url) {
                if !self
                    .store
                    .get()
                    .save_saved_page(&canonical, &bookmark.title)
                    .existed
                {
                    pages += 1;
                }
            }
        }
        let visits: Vec<(String, String, u64, u64)> = chosen
            .iter()
            .flat_map(|source| &source.history)
            .map(|visit| {
                (
                    visit.url.clone(),
                    visit.title.clone(),
                    visit.visit_count,
                    visit.last_visited_ms,
                )
            })
            .collect();
        let history = self.store.get().import_history(&visits);
        self.show_toast(
            bbr_ui::icons::Icon::Check,
            None,
            bbr_ui::surfaces::toast::ToastTone::Ok,
            imported_message(pages, history),
            None,
            cx,
        );
    }

    /// The first-space field's keys (typed characters, Backspace, Return).
    pub(crate) fn on_onboarding_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(state) = self.onboarding.as_mut() else {
            return false;
        };
        if state.step != OnboardingStep::FirstSpace || !state.name_focused {
            return false;
        }
        let ks = &event.keystroke;
        if ks.modifiers.platform || ks.modifiers.control {
            return false;
        }
        match ks.key.as_str() {
            "enter" => {
                cx.stop_propagation();
                self.on_welcome_event(WelcomeEvent::Done, window, cx);
            }
            "backspace" => {
                cx.stop_propagation();
                state.name.pop();
                cx.notify();
            }
            _ => {
                let Some(text) = ks.key_char.as_deref().filter(|t| !t.is_empty()) else {
                    return false;
                };
                cx.stop_propagation();
                state.name.push_str(text);
                cx.notify();
            }
        }
        true
    }
}

#[cfg(feature = "testkit")]
impl BbrApp {
    pub(crate) fn onboarding_snapshot(&self) -> Option<bbr_testkit::protocol::OnboardingSnapshot> {
        if !self.onboarding_visible() {
            return None;
        }
        let state = self.onboarding.as_ref()?;
        let step = self.welcome_step()?;
        Some(bbr_testkit::protocol::OnboardingSnapshot {
            step: match state.step {
                OnboardingStep::Welcome => "welcome",
                OnboardingStep::Import => "import",
                OnboardingStep::DefaultBrowser => "default-browser",
                OnboardingStep::FirstSpace => "first-space",
            }
            .into(),
            sources: match step {
                WelcomeStep::Import(Some(rows)) => Some(
                    rows.iter()
                        .map(|row| {
                            (
                                row.label.to_string(),
                                row.detail.to_string(),
                                row.checked,
                                row.enabled,
                            )
                        })
                        .collect(),
                ),
                _ => None,
            },
            name: state.name.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_import_toast_counts_what_came_over() {
        assert_eq!(
            super::imported_message(312, 1204),
            "Imported 312 Saved Pages and 1 204 history entries"
        );
        assert_eq!(
            super::imported_message(1, 1),
            "Imported 1 Saved Page and 1 history entry"
        );
    }
}
