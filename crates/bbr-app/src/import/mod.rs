//! Blueprint D15 (board L-2, answer 12): bring bookmarks and history over
//! from Safari and Google Chrome.
//!
//! Detection reads each source's files from the user's Library (a test
//! harness points it at a fixture home instead): Chrome's `Bookmarks` JSON
//! and `History` SQLite per Chrome profile, Safari's `Bookmarks.plist` and
//! `History.db`. Foreign SQLite files are copied first and opened
//! read-only, so a running browser's lock never matters and nothing of
//! theirs is written. A source that cannot be read (Safari's files need
//! Full Disk Access) says so instead of failing silently. Bookmarks become
//! Saved Pages; history joins BBR's history.

use std::path::{Path, PathBuf};

pub(crate) mod chrome;
pub(crate) mod safari;

/// One bookmark (title, URL).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImportedBookmark {
    pub title: String,
    pub url: String,
}

/// One history entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImportedVisit {
    pub url: String,
    pub title: String,
    pub visit_count: u64,
    /// Milliseconds since the Unix epoch.
    pub last_visited_ms: u64,
}

/// Where the data comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ImportSource {
    Safari,
    /// A Chrome profile directory name ("Default", "Profile 1") and its
    /// display name.
    Chrome {
        dir: String,
        name: String,
    },
}

impl ImportSource {
    pub(crate) fn label(&self) -> String {
        match self {
            ImportSource::Safari => "Safari".into(),
            ImportSource::Chrome { name, .. } => format!("Google Chrome · {name}"),
        }
    }
}

/// What one source holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceData {
    pub source: ImportSource,
    pub bookmarks: Vec<ImportedBookmark>,
    pub history: Vec<ImportedVisit>,
    /// Why the source could not be read (it is then not importable).
    pub error: Option<String>,
}

impl SourceData {
    /// "1 204 bookmarks · 18 months of history".
    pub(crate) fn summary(&self) -> String {
        if let Some(error) = &self.error {
            return error.clone();
        }
        let bookmarks = match self.bookmarks.len() {
            1 => "1 bookmark".to_string(),
            n => format!("{} bookmarks", grouped(n)),
        };
        let span = history_span(&self.history);
        format!("{bookmarks} · {span}")
    }
}

/// Thousands grouped with a space, as the board writes them ("1 204").
pub(crate) fn grouped(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(digit);
    }
    out
}

/// "18 months of history", "2 weeks of history", "no history".
pub(crate) fn history_span(history: &[ImportedVisit]) -> String {
    let dated: Vec<u64> = history
        .iter()
        .map(|visit| visit.last_visited_ms)
        .filter(|ms| *ms > 0)
        .collect();
    let (Some(oldest), Some(newest)) = (dated.iter().min(), dated.iter().max()) else {
        return "no history".into();
    };
    let days = (newest - oldest) / 86_400_000;
    let months = days / 30;
    match (months, days) {
        (0, 0..=13) => "less than 2 weeks of history".into(),
        (0, _) => format!("{} weeks of history", days / 7),
        (1, _) => "1 month of history".into(),
        (m, _) => format!("{m} months of history"),
    }
}

/// Read every source found under `home`.
pub(crate) fn detect(home: &Path) -> Vec<SourceData> {
    let mut sources = Vec::new();
    if let Some(safari) = safari::read(home) {
        sources.push(safari);
    }
    sources.extend(chrome::read_all(home));
    sources
}

/// Copy a foreign SQLite file (and its WAL) aside and open the copy
/// read-only.
pub(crate) fn open_copy(path: &Path) -> Result<(rusqlite::Connection, PathBuf), String> {
    let dir = std::env::temp_dir().join(format!(
        "bbr-import-{}-{}",
        std::process::id(),
        path.file_name().and_then(|n| n.to_str()).unwrap_or("db")
    ));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let copy = dir.join("copy.sqlite");
    std::fs::copy(path, &copy).map_err(|e| e.to_string())?;
    let wal = PathBuf::from(format!("{}-wal", path.display()));
    if wal.exists() {
        let _ = std::fs::copy(&wal, dir.join("copy.sqlite-wal"));
    }
    let connection = rusqlite::Connection::open_with_flags(
        &copy,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| e.to_string())?;
    Ok((connection, dir))
}

/// Only web bookmarks and visits come over.
pub(crate) fn is_web(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.starts_with("https://") || lower.starts_with("http://")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn visit(ms: u64) -> ImportedVisit {
        ImportedVisit {
            url: "https://a.example/".into(),
            title: String::new(),
            visit_count: 1,
            last_visited_ms: ms,
        }
    }

    #[test]
    fn summaries_count_and_span_like_the_board() {
        assert_eq!(grouped(1204), "1 204");
        assert_eq!(grouped(312), "312");
        assert_eq!(grouped(1_000_000), "1 000 000");
        let day = 86_400_000;
        assert_eq!(history_span(&[]), "no history");
        assert_eq!(
            history_span(&[visit(day), visit(3 * day)]),
            "less than 2 weeks of history"
        );
        assert_eq!(
            history_span(&[visit(day), visit(22 * day)]),
            "3 weeks of history"
        );
        assert_eq!(
            history_span(&[visit(day), visit(560 * day)]),
            "18 months of history"
        );
        let data = SourceData {
            source: ImportSource::Safari,
            bookmarks: vec![
                ImportedBookmark {
                    title: "A".into(),
                    url: "https://a.example/".into()
                };
                1204
            ],
            history: vec![visit(day), visit(560 * day)],
            error: None,
        };
        assert_eq!(data.summary(), "1 204 bookmarks · 18 months of history");
    }
}
