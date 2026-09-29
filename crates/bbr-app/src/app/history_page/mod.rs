//! Blueprint D10 (boards M-6 M-7 PR-22, DESIGN §11.16): History — a page
//! of the Library surface.
//!
//! ⌘Y opens History in the content card with the Library's anatomy and its
//! visibility authority: History is `library.open` with
//! [`LibraryPage::History`], so the active view's native slot is hidden and
//! restored exactly as for Saved Pages, Escape closes it, and ⌘T / ⌘L
//! replace it. The page lists the recorded visits (global: visits carry no
//! profile) grouped by local day, and the active space's recently closed
//! tabs. Opening a visit opens it in a new tab of the active space;
//! opening a closed tab reopens it with its identity. ⌘⇧⌫ opens Clear
//! Browsing Data.

use std::rc::Rc;

use bbr_ui::surfaces::history::{HistoryEvent, HistoryModel};
use gpui::{Context, IntoElement, KeyDownEvent, SharedString, Window};

use super::BbrApp;

pub(crate) mod clear;
pub(crate) mod model;
#[cfg(feature = "testkit")]
pub(crate) mod testkit;
pub(crate) mod time;

use clear::{ClearDataState, CLEAR_RANGES};
use model::{closed_rows, visit_rows, HistoryRows, RANGE_ALL};

/// Which page the Library surface shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum LibraryPage {
    #[default]
    SavedPages,
    History,
}

/// History's local state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HistoryPageState {
    pub search: String,
    /// Today / Week / All.
    pub range: usize,
    /// 0 All history, 1 Recently closed.
    pub view: usize,
    /// The keyboard row (a visit id, or a tab id in Recently closed).
    pub selected: Option<u64>,
    pub clear: Option<ClearDataState>,
}

impl Default for HistoryPageState {
    fn default() -> Self {
        HistoryPageState {
            search: String::new(),
            range: RANGE_ALL,
            view: 0,
            selected: None,
            clear: None,
        }
    }
}

impl BbrApp {
    /// Whether History is the open Library page.
    pub(crate) fn history_open(&self) -> bool {
        self.library.open && self.library.page == LibraryPage::History
    }

    /// ⌘Y: open History (or switch the open Library to it).
    pub fn open_history(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.history_open() {
            return;
        }
        if !self.library.open {
            self.open_saved_pages_library(window, cx);
        }
        self.library.page = LibraryPage::History;
        self.library.history = HistoryPageState::default();
        window.focus(&self.library_handle, cx);
        cx.notify();
    }

    /// ⌘⇧⌫: open History with Clear Browsing Data.
    pub fn open_clear_browsing_data(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_history(window, cx);
        if self.library.history.clear.is_none() {
            self.library.history.clear = Some(ClearDataState::default());
        }
        cx.notify();
    }

    /// The rows the current view lists.
    pub(crate) fn history_rows(&self) -> HistoryRows {
        let state = &self.library.history;
        if state.view == 1 {
            closed_rows(&self.controller.recently_closed(), &state.search)
        } else {
            let clock = time::Clock::system();
            self.store
                .with_history(|entries| visit_rows(entries, &state.search, state.range, &clock))
        }
    }

    /// The page's presentation model.
    pub(crate) fn history_model(&self, focused: bool) -> HistoryModel {
        let state = &self.library.history;
        let rows = self.history_rows();
        let empty = rows.groups.is_empty().then(|| {
            SharedString::from(if !state.search.trim().is_empty() {
                format!("Nothing matches “{}”.", state.search.trim())
            } else if state.view == 1 {
                "No recently closed tabs in this space.".to_string()
            } else {
                "Pages you open from the command bar appear here.".to_string()
            })
        });
        HistoryModel {
            theme: self.theme(),
            search: SharedString::from(state.search.clone()),
            range: state.range,
            view: state.view,
            closed_count: self.controller.recently_closed().len(),
            groups: rows.groups,
            selected: state.selected,
            empty,
            more: (rows.hidden > 0).then(|| {
                SharedString::from(format!(
                    "{} older visits — search to find them",
                    rows.hidden
                ))
            }),
            focused,
            clear: state
                .clear
                .as_ref()
                .map(|clear| self.clear_data_model(clear)),
        }
    }

    /// The page for the content card.
    pub(crate) fn history_surface(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let model = self.history_model(self.library_handle.is_focused(window));
        let entity = cx.entity();
        bbr_ui::surfaces::history::history_surface(
            &model,
            Rc::new(move |event, window, cx| {
                entity.update(cx, |app, cx| app.on_history_event(event, window, cx));
            }),
        )
    }

    /// Every History interaction (pointer, keys and the testkit).
    pub(crate) fn on_history_event(
        &mut self,
        event: HistoryEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let state = &mut self.library.history;
        match event {
            HistoryEvent::View(view) => {
                state.view = view.min(1);
                state.selected = None;
            }
            HistoryEvent::Range(range) => {
                state.range = range.min(RANGE_ALL);
                state.selected = None;
            }
            HistoryEvent::Open(id) => {
                self.history_activate(id, window, cx);
                return;
            }
            HistoryEvent::Remove(id) => self.history_remove(id),
            HistoryEvent::ClearOpen => {
                state.clear.get_or_insert_with(ClearDataState::default);
            }
            HistoryEvent::ClearRangeMenu => {
                if let Some(clear) = state.clear.as_mut() {
                    clear.range_open = !clear.range_open;
                }
            }
            HistoryEvent::ClearRange(range) => {
                if let Some(clear) = state.clear.as_mut() {
                    clear.range = range.min(CLEAR_RANGES.len() - 1);
                    clear.range_open = false;
                }
            }
            HistoryEvent::ClearToggle(index) => {
                if let Some(clear) = state.clear.as_mut() {
                    if index < clear.options.len() && clear.enabled(index) {
                        clear.options[index] = !clear.options[index];
                    }
                }
            }
            HistoryEvent::ClearCancel => state.clear = None,
            HistoryEvent::ClearConfirm => self.clear_browsing_data(cx),
        }
        cx.notify();
    }

    /// Open a row: a visit opens in a new tab of the active space; a
    /// recently closed tab reopens with its identity. History closes and
    /// the page takes the keyboard.
    fn history_activate(&mut self, id: u64, window: &mut Window, cx: &mut Context<Self>) {
        if self.library.history.view == 1 {
            self.close_saved_pages_library(window, cx);
            self.reopen_closed_tab(bbr_domain::TabId(id), window, cx);
            return;
        }
        let Some((url, title)) = self.store.history_entry(bbr_command::id::HistoryId(id)) else {
            return;
        };
        self.close_saved_pages_library(window, cx);
        if self.open_tab(&url, cx).is_some() {
            self.store.record_visit(&url, &title);
            self.finish_web_activation(window, cx);
        }
    }

    /// Delete on a row: that visit goes from History (and from the command
    /// bar's history results).
    fn history_remove(&mut self, id: u64) {
        if self.library.history.view != 0 {
            return;
        }
        let ids = self.history_rows().ids();
        let position = ids.iter().position(|row| *row == id);
        if self.store.remove_history(&[id]) > 0 {
            // The keyboard row moves to the next one (or the previous one
            // at the end).
            self.library.history.selected = position.and_then(|at| {
                ids.get(at + 1)
                    .or_else(|| at.checked_sub(1).and_then(|before| ids.get(before)))
                    .copied()
            });
        }
    }

    /// History's keyboard (the Library handle routes here while it shows
    /// History). Escape is the global binding (it closes the sheet first).
    pub(super) fn on_history_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let ks = &event.keystroke;
        if let Some(clear) = self.library.history.clear.as_ref() {
            if ks.key == "enter" && clear.any() {
                cx.stop_propagation();
                self.on_history_event(HistoryEvent::ClearConfirm, window, cx);
            }
            return;
        }
        let command = ks.modifiers.platform;
        match ks.key.as_str() {
            "up" | "down" => {
                cx.stop_propagation();
                let ids = self.history_rows().ids();
                if ids.is_empty() {
                    return;
                }
                let current = self
                    .library
                    .history
                    .selected
                    .and_then(|id| ids.iter().position(|row| *row == id));
                let next = match (current, ks.key == "down") {
                    (None, true) => 0,
                    (None, false) => ids.len() - 1,
                    (Some(at), true) => (at + 1).min(ids.len() - 1),
                    (Some(at), false) => at.saturating_sub(1),
                };
                self.library.history.selected = Some(ids[next]);
                cx.notify();
            }
            "enter" => {
                if let Some(id) = self.library.history.selected {
                    cx.stop_propagation();
                    self.history_activate(id, window, cx);
                }
            }
            "delete" => {
                if let Some(id) = self.library.history.selected {
                    cx.stop_propagation();
                    self.history_remove(id);
                    cx.notify();
                }
            }
            "backspace" if command => {
                if let Some(id) = self.library.history.selected {
                    cx.stop_propagation();
                    self.history_remove(id);
                    cx.notify();
                }
            }
            "backspace" => {
                cx.stop_propagation();
                self.library.history.search.pop();
                self.library.history.selected = None;
                cx.notify();
            }
            _ => {
                if command || ks.modifiers.control {
                    return;
                }
                if let Some(ch) = ks.key_char.as_deref().filter(|ch| !ch.is_empty()) {
                    cx.stop_propagation();
                    self.library.history.search.push_str(ch);
                    self.library.history.selected = None;
                    cx.notify();
                }
            }
        }
    }
}
