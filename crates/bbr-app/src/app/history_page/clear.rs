//! Blueprint D10 (board M-7): Clear Browsing Data — the sheet's state and
//! the one clear it runs.
//!
//! Visits and the active profile's website data (cookies and site data,
//! cached files) clear by time range: the engine removes the data modified
//! since the range start. Download history and Saved Pages carry no wall
//! time, so they clear only with "All time" (the rows are dimmed
//! otherwise); Saved Pages are shared by every profile, and the row says so.

use bbr_ui::surfaces::history::{ClearDataModel, ClearOption};
use bbr_ui::surfaces::toast::ToastTone;
use gpui::{Context, SharedString};

use crate::app::BbrApp;
use crate::store::history_ops::now_ms;

/// `(label, span in ms)`; `None` is All time.
pub(crate) const CLEAR_RANGES: [(&str, Option<u64>); 5] = [
    ("Last hour", Some(3_600_000)),
    ("Last 24 hours", Some(86_400_000)),
    ("Last 7 days", Some(7 * 86_400_000)),
    ("Last 4 weeks", Some(28 * 86_400_000)),
    ("All time", None),
];

pub(crate) const OPTION_HISTORY: usize = 0;
pub(crate) const OPTION_COOKIES: usize = 1;
pub(crate) const OPTION_CACHES: usize = 2;
pub(crate) const OPTION_DOWNLOADS: usize = 3;
pub(crate) const OPTION_SAVED_PAGES: usize = 4;

/// The open sheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClearDataState {
    pub range: usize,
    pub options: [bool; 5],
    pub range_open: bool,
}

impl Default for ClearDataState {
    fn default() -> Self {
        ClearDataState {
            range: 1,
            options: [true, true, true, false, false],
            range_open: false,
        }
    }
}

impl ClearDataState {
    fn all_time(&self) -> bool {
        CLEAR_RANGES[self.range].1.is_none()
    }

    /// Whether option `index` can be chosen in the current range.
    pub(crate) fn enabled(&self, index: usize) -> bool {
        !matches!(index, OPTION_DOWNLOADS | OPTION_SAVED_PAGES) || self.all_time()
    }

    /// Whether option `index` takes part in the clear.
    pub(crate) fn effective(&self, index: usize) -> bool {
        self.options[index] && self.enabled(index)
    }

    pub(crate) fn any(&self) -> bool {
        (0..self.options.len()).any(|index| self.effective(index))
    }

    /// The range start in ms since the epoch (`None`: everything).
    pub(crate) fn since_ms(&self, now_ms: u64) -> Option<u64> {
        CLEAR_RANGES[self.range]
            .1
            .map(|span| now_ms.saturating_sub(span))
    }
}

impl BbrApp {
    /// The sheet's presentation model.
    pub(crate) fn clear_data_model(&self, state: &ClearDataState) -> ClearDataModel {
        let snapshot = self.controller.snapshot();
        let active = self.controller.active_profile();
        let profile = snapshot.profiles.iter().find(|p| p.id == active);
        let title = profile.map(|p| p.title.clone()).unwrap_or_default();
        let theme = self.theme();
        let visits = self.store.history_count(state.since_ms(now_ms()));
        let labels = [
            match visits {
                1 => "Browsing history · 1 page".to_string(),
                n => format!("Browsing history · {n} pages"),
            },
            "Cookies and site data · signs you out of most sites".to_string(),
            "Cached images and files".to_string(),
            if state.all_time() {
                "Download history".to_string()
            } else {
                "Download history · all time only".to_string()
            },
            if state.all_time() {
                "Saved Pages (all profiles)".to_string()
            } else {
                "Saved Pages (all profiles) · all time only".to_string()
            },
        ];
        ClearDataModel {
            profile: SharedString::from(title.clone()),
            profile_letter: title
                .chars()
                .next()
                .map(|c| c.to_ascii_uppercase())
                .unwrap_or('P'),
            profile_hue: profile
                .map(|p| bbr_ui::palette::ProfileHueRamp::tile(p.hue, theme.mode).tile)
                .unwrap_or(theme.palette.accent),
            range: state.range,
            ranges: CLEAR_RANGES
                .iter()
                .map(|(label, _)| SharedString::from(*label))
                .collect(),
            options: labels
                .into_iter()
                .enumerate()
                .map(|(index, label)| ClearOption {
                    label: label.into(),
                    checked: state.effective(index),
                    enabled: state.enabled(index),
                })
                .collect(),
            range_open: state.range_open,
            confirm_enabled: state.any(),
        }
    }

    /// Clear Data: remove what the sheet names, close it and say so.
    pub(crate) fn clear_browsing_data(&mut self, cx: &mut Context<Self>) {
        let Some(state) = self.library.history.clear.take() else {
            return;
        };
        if !state.any() {
            return;
        }
        let since = state.since_ms(now_ms());
        if state.effective(OPTION_HISTORY) {
            self.store.clear_history(since);
            self.library.history.selected = None;
        }
        let kinds = bbr_web::WebsiteDataKinds {
            cookies_and_site_data: state.effective(OPTION_COOKIES),
            caches: state.effective(OPTION_CACHES),
        };
        if kinds.cookies_and_site_data || kinds.caches {
            let profile = self.controller.active_profile();
            self.engine.borrow_mut().clear_profile_data(
                profile,
                kinds,
                since.map(|ms| ms as f64 / 1000.0),
            );
        }
        if state.effective(OPTION_DOWNLOADS) {
            crate::app::downloads_history::downloads_clear(self, cx);
        }
        if state.effective(OPTION_SAVED_PAGES) {
            let ids: Vec<u64> = self
                .store
                .with_saved_pages(|pages, _| pages.iter().map(|page| page.id).collect());
            self.store.remove_saved_pages(&ids);
        }
        // Board PR-12: "Cleared 24 hours of data from Default".
        let profile = self.controller.active_profile();
        let name = self
            .controller
            .snapshot()
            .profiles
            .iter()
            .find(|p| p.id == profile)
            .map(|p| p.title.clone())
            .unwrap_or_default();
        self.show_toast(
            bbr_ui::icons::Icon::Check,
            None,
            ToastTone::Ok,
            cleared_message(state.range, &name),
            None,
            cx,
        );
        cx.notify();
    }
}

/// The toast after a clear.
pub(crate) fn cleared_message(range: usize, profile: &str) -> String {
    let span = match range {
        0 => "the last hour of data",
        1 => "24 hours of data",
        2 => "7 days of data",
        3 => "4 weeks of data",
        _ => "all data",
    };
    format!("Cleared {span} from {profile}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_toast_names_the_range_and_the_profile() {
        assert_eq!(
            cleared_message(1, "Default"),
            "Cleared 24 hours of data from Default"
        );
        assert_eq!(cleared_message(4, "Work"), "Cleared all data from Work");
    }

    #[test]
    fn downloads_and_saved_pages_clear_only_with_all_time() {
        let mut state = ClearDataState {
            options: [false, false, false, true, true],
            ..ClearDataState::default()
        };
        assert!(!state.any());
        assert!(!state.enabled(OPTION_DOWNLOADS));
        state.range = CLEAR_RANGES.len() - 1;
        assert!(state.effective(OPTION_DOWNLOADS));
        assert!(state.effective(OPTION_SAVED_PAGES));
        assert_eq!(state.since_ms(10_000), None);
    }

    #[test]
    fn the_default_sheet_clears_the_last_day_of_visits_and_site_data() {
        let state = ClearDataState::default();
        assert_eq!(CLEAR_RANGES[state.range].0, "Last 24 hours");
        assert_eq!(state.since_ms(100_000_000), Some(100_000_000 - 86_400_000));
        assert!(state.effective(OPTION_HISTORY));
        assert!(state.effective(OPTION_COOKIES));
        assert!(state.effective(OPTION_CACHES));
        assert!(!state.effective(OPTION_DOWNLOADS));
    }
}
