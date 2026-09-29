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
