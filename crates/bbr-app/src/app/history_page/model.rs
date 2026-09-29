//! Blueprint D10: the History page's rows — visits filtered by the search
//! and the Today / Week / All range, newest first, grouped by local day —
//! or the active space's recently closed tabs.

use bbr_browser::ClosedTabEntry;
use bbr_command::corpus::HistoryEntry;
use bbr_ui::components::favicon::{monogram_hue, monogram_letter};
use bbr_ui::surfaces::history::{HistoryGroup, HistoryRow};
use gpui::SharedString;

use super::time::Clock;

/// The page draws at most this many rows; the search reaches the rest.
pub(crate) const HISTORY_ROW_LIMIT: usize = 300;

/// Today / Week / All.
pub(crate) const RANGE_TODAY: usize = 0;
pub(crate) const RANGE_WEEK: usize = 1;
pub(crate) const RANGE_ALL: usize = 2;

/// The listed rows plus what the page says around them.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HistoryRows {
    pub groups: Vec<HistoryGroup>,
    /// Rows past [`HISTORY_ROW_LIMIT`].
    pub hidden: usize,
}

impl HistoryRows {
    /// The listed row ids in order (the keyboard walks these).
    pub(crate) fn ids(&self) -> Vec<u64> {
        self.groups
            .iter()
            .flat_map(|group| group.rows.iter().map(|row| row.id))
            .collect()
    }
}

fn matches(search: &str, title: &str, url: &str) -> bool {
    let needle = search.trim().to_lowercase();
    needle.is_empty()
        || title.to_lowercase().contains(&needle)
        || url.to_lowercase().contains(&needle)
}

/// The URL as the row shows it: without the scheme and a bare trailing
/// slash.
fn display_url(url: &str) -> String {
    let rest = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    rest.strip_suffix('/').unwrap_or(rest).to_string()
}

fn row(id: u64, time: Option<String>, title: &str, url: &str) -> HistoryRow {
    let shown = display_url(url);
    HistoryRow {
        id,
        time: time.map(SharedString::from),
        title: SharedString::from(if title.trim().is_empty() {
            shown.clone()
        } else {
            title.trim().to_string()
        }),
        url: SharedString::from(shown),
        letter: monogram_letter(url),
        hue: monogram_hue(url),
    }
}

/// The visits for the All history view.
pub(crate) fn visit_rows(
    entries: &[HistoryEntry],
    search: &str,
    range: usize,
    clock: &Clock,
) -> HistoryRows {
    let today = clock.today();
    let mut listed: Vec<&HistoryEntry> = entries
        .iter()
        .filter(|entry| matches(search, &entry.title, &entry.url))
        .filter(|entry| {
            let dated = Clock::dated(entry.last_visited);
            match range {
                RANGE_TODAY => dated && clock.day(entry.last_visited) == today,
                RANGE_WEEK => dated && clock.day(entry.last_visited) > today - 7,
                _ => true,
            }
        })
        .collect();
    listed.sort_by_key(|entry| std::cmp::Reverse(entry.last_visited));
    let hidden = listed.len().saturating_sub(HISTORY_ROW_LIMIT);
    listed.truncate(HISTORY_ROW_LIMIT);
    let mut groups: Vec<HistoryGroup> = Vec::new();
    let mut current: Option<Option<i64>> = None;
    for entry in listed {
        let dated = Clock::dated(entry.last_visited);
        let day = dated.then(|| clock.day(entry.last_visited));
        if current != Some(day) {
            current = Some(day);
            groups.push(HistoryGroup {
                title: SharedString::from(match day {
                    Some(day) => clock.day_title(day),
                    None => "Earlier".to_string(),
                }),
                rows: Vec::new(),
            });
        }
        let time = dated.then(|| clock.time_label(entry.last_visited));
        groups
            .last_mut()
            .expect("group pushed above")
            .rows
            .push(row(entry.id.0, time, &entry.title, &entry.url));
    }
    HistoryRows { groups, hidden }
}

/// The recently closed tabs (most recent first) for their view.
pub(crate) fn closed_rows(closed: &[ClosedTabEntry], search: &str) -> HistoryRows {
    let rows: Vec<HistoryRow> = closed
        .iter()
        .filter(|entry| matches(search, &entry.title, &entry.url))
        .map(|entry| row(entry.tab.0, None, &entry.title, &entry.url))
        .collect();
    HistoryRows {
        groups: if rows.is_empty() {
            Vec::new()
        } else {
            vec![HistoryGroup {
                title: "Recently closed".into(),
                rows,
            }]
        },
        hidden: 0,
    }
}

#[cfg(test)]
mod tests {
    use bbr_command::id::HistoryId;

    use super::*;

    /// 2026-09-29 12:00:00 UTC.
    const NOON: u64 = 1_790_683_200_000;
    const HOUR: u64 = 3_600_000;

    fn clock() -> Clock {
        Clock {
            now_ms: NOON,
            offset_s: 0,
        }
    }

    fn entry(id: u64, url: &str, title: &str, at: u64) -> HistoryEntry {
        HistoryEntry::new(HistoryId(id), title.to_string(), url.to_string(), 1, at)
    }

    fn corpus() -> Vec<HistoryEntry> {
        vec![
            entry(1, "https://old.example/", "", 7),
            entry(2, "https://a.example/docs", "Alpha docs", NOON - HOUR),
            entry(3, "https://b.example/", "Beta", NOON - 30 * HOUR),
            entry(4, "https://c.example/", "Gamma", NOON - 10 * 24 * HOUR),
            entry(5, "https://d.example/", "Delta", NOON - 2 * HOUR),
        ]
    }

    #[test]
    fn visits_group_by_local_day_newest_first_with_undated_last() {
        let rows = visit_rows(&corpus(), "", RANGE_ALL, &clock());
        let titles: Vec<&str> = rows.groups.iter().map(|g| g.title.as_ref()).collect();
        assert_eq!(
            titles,
            ["Today", "Yesterday", "Saturday, 19 September", "Earlier"]
        );
        assert_eq!(rows.ids(), vec![2, 5, 3, 4, 1]);
        let first = &rows.groups[0].rows[0];
        assert_eq!(first.time.as_deref(), Some("11:00"));
        assert_eq!(first.url.as_ref(), "a.example/docs");
        assert_eq!(first.letter, 'A');
        // The untitled pre-D10 visit shows its URL and no time.
        let old = &rows.groups[3].rows[0];
        assert_eq!(old.title.as_ref(), "old.example");
        assert_eq!(old.time, None);
    }

    #[test]
    fn range_and_search_filter_the_visits() {
        let today = visit_rows(&corpus(), "", RANGE_TODAY, &clock());
        assert_eq!(today.ids(), vec![2, 5]);
        let week = visit_rows(&corpus(), "", RANGE_WEEK, &clock());
        assert_eq!(week.ids(), vec![2, 5, 3]);
        let search = visit_rows(&corpus(), "BETA", RANGE_ALL, &clock());
        assert_eq!(search.ids(), vec![3]);
        let by_url = visit_rows(&corpus(), "//d.example", RANGE_ALL, &clock());
        assert_eq!(by_url.ids(), vec![5]);
    }

    #[test]
    fn the_list_is_cut_at_the_row_limit() {
        let many: Vec<HistoryEntry> = (0..HISTORY_ROW_LIMIT as u64 + 5)
            .map(|i| entry(i + 1, &format!("https://{i}.example/"), "", NOON - i))
            .collect();
        let rows = visit_rows(&many, "", RANGE_ALL, &clock());
        assert_eq!(rows.ids().len(), HISTORY_ROW_LIMIT);
        assert_eq!(rows.hidden, 5);
    }

    #[test]
    fn closed_tabs_list_in_their_own_group() {
        let closed = vec![
            ClosedTabEntry {
                tab: bbr_domain::TabId(9),
                title: "Nine".into(),
                url: "https://nine.example/".into(),
            },
            ClosedTabEntry {
                tab: bbr_domain::TabId(4),
                title: "Four".into(),
                url: "https://four.example/".into(),
            },
        ];
        let rows = closed_rows(&closed, "");
        assert_eq!(rows.ids(), vec![9, 4]);
        assert_eq!(rows.groups[0].title.as_ref(), "Recently closed");
        assert!(closed_rows(&closed, "zzz").groups.is_empty());
    }
}
