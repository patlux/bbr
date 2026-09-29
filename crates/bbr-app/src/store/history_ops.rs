//! Blueprint D10: the History page's store operations — the visit clock,
//! removing single visits and clearing a time range.
//!
//! Every accepted visit stamps `last_visited` with the wall clock in
//! milliseconds, never below the next monotonic marker, so the ranking's
//! "more recent" order holds across clock steps and pre-D10 counters. A
//! removal or clear rewrites the in-memory provider and pokes the same
//! bounded writer every visit uses.

use bbr_command::corpus::HistoryEntry;

use super::AppStore;

/// Now, in milliseconds since the Unix epoch.
pub(crate) fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Whether `entry` falls in the clear range starting at `since_ms`
/// (`None`: everything, including visits of unknown time).
pub(crate) fn in_range(entry: &HistoryEntry, since_ms: Option<u64>) -> bool {
    since_ms.is_none_or(|since| entry.last_visited >= since)
}

impl AppStore {
    /// Lift the next recency marker to the wall clock before a visit.
    pub(super) fn stamp_now(&mut self) {
        self.next_recency = self.next_recency.max(now_ms());
    }

    /// Remove the visits with these ids (History › Delete on a row).
    /// Returns how many went.
    pub fn remove_history(&mut self, ids: &[u64]) -> usize {
        self.retain_history(|entry| !ids.contains(&entry.id.0))
    }

    /// Clear every visit at or after `since_ms` (`None`: all of them).
    /// Returns how many went.
    pub fn clear_history(&mut self, since_ms: Option<u64>) -> usize {
        self.retain_history(|entry| !in_range(entry, since_ms))
    }

    /// How many visits a clear from `since_ms` would remove.
    pub fn history_count(&self, since_ms: Option<u64>) -> usize {
        self.with_history(|history| {
            history
                .iter()
                .filter(|entry| in_range(entry, since_ms))
                .count()
        })
    }

    /// Blueprint D15: merge imported visits (normalized URL, title, visit
    /// count, time in ms) — an existing URL adds the count and keeps the
    /// later time; a new one is appended. Returns how many URLs were new.
    pub fn import_history(&mut self, visits: &[(String, String, u64, u64)]) -> usize {
        let mut history = self.history.lock().expect("history provider lock");
        let mut added = 0;
        let mut latest = 0;
        for (url, title, count, at) in visits {
            let normalized = bbr_web::normalize_url(url);
            let web = normalized.starts_with("https://") || normalized.starts_with("http://");
            if !web || normalized.contains(char::is_whitespace) {
                continue;
            }
            latest = latest.max(*at);
            if let Some(entry) = history.iter_mut().find(|entry| entry.url == normalized) {
                entry.visit_count = entry.visit_count.saturating_add(*count);
                entry.last_visited = entry.last_visited.max(*at);
                if entry.title.is_empty() {
                    entry.title = title.trim().to_string();
                }
            } else {
                let id = self.next_id;
                self.next_id += 1;
                history.push(bbr_command::corpus::HistoryEntry::new(
                    bbr_command::id::HistoryId(id),
                    title.trim().to_string(),
                    normalized,
                    (*count).max(1),
                    *at,
                ));
                added += 1;
            }
        }
        drop(history);
        // New visits stay more recent than anything imported.
        self.next_recency = self.next_recency.max(latest + 1);
        let _ = self.write_tx.try_send(());
        added
    }

    fn retain_history(&mut self, keep: impl Fn(&HistoryEntry) -> bool) -> usize {
        let mut history = self.history.lock().expect("history provider lock");
        let before = history.len();
        history.retain(|entry| keep(entry));
        let removed = before - history.len();
        drop(history);
        if removed > 0 {
            let _ = self.write_tx.try_send(());
        }
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_with(urls: &[&str]) -> AppStore {
        let mut store = AppStore::ephemeral();
        for url in urls {
            store.record_visit(url, "Title");
        }
        store
    }

    #[test]
    fn visits_carry_the_wall_clock_and_stay_monotonic() {
        let before = now_ms();
        let store = store_with(&["https://a.example/", "https://b.example/"]);
        store.with_history(|history| {
            assert!(history[0].last_visited >= before);
            assert!(history[1].last_visited > history[0].last_visited);
        });
    }

    #[test]
    fn imports_merge_by_url_and_keep_later_times() {
        let mut store = store_with(&["https://a.example/"]);
        let before: u64 = store.with_history(|h| h[0].last_visited);
        let added = store.import_history(&[
            ("https://a.example/".into(), "A".into(), 5, 1_000),
            ("https://b.example/".into(), "B".into(), 2, 2_000),
            ("not a url at all".into(), String::new(), 1, 3_000),
        ]);
        assert_eq!(added, 1, "a known URL merges; a malformed one is skipped");
        store.with_history(|h| {
            let a = h.iter().find(|e| e.url == "https://a.example/").unwrap();
            assert_eq!(a.visit_count, 6);
            assert_eq!(a.last_visited, before, "the later time stays");
            let b = h.iter().find(|e| e.url == "https://b.example/").unwrap();
            assert_eq!(
                (b.visit_count, b.last_visited, b.title.as_str()),
                (2, 2_000, "B")
            );
        });
    }

    #[test]
    fn removing_and_clearing_follow_ids_and_ranges() {
        let mut store = store_with(&[
            "https://a.example/",
            "https://b.example/",
            "https://c.example/",
        ]);
        let ids: Vec<u64> = store.with_history(|h| h.iter().map(|e| e.id.0).collect());
        let stamps: Vec<u64> = store.with_history(|h| h.iter().map(|e| e.last_visited).collect());
        assert_eq!(store.remove_history(&[ids[0]]), 1);
        assert_eq!(store.history_count(None), 2);
        // Clearing from the last visit's stamp takes only that one.
        assert_eq!(store.history_count(Some(stamps[2])), 1);
        assert_eq!(store.clear_history(Some(stamps[2])), 1);
        assert_eq!(
            store.with_history(|h| h.iter().map(|e| e.id.0).collect::<Vec<_>>()),
            vec![ids[1]]
        );
        assert_eq!(store.clear_history(None), 1);
        assert_eq!(store.history_count(None), 0);
    }
}
