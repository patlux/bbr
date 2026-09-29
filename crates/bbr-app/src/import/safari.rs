//! Blueprint D15: Safari's bookmarks (`~/Library/Safari/Bookmarks.plist`)
//! and history (`History.db`). macOS protects both: without Full Disk
//! Access they cannot be read, and the source says so.

use std::path::Path;

use super::{is_web, open_copy, ImportSource, ImportedBookmark, ImportedVisit, SourceData};

/// Seconds between 2001-01-01 (Safari's epoch) and 1970-01-01.
const SAFARI_EPOCH_OFFSET_S: f64 = 978_307_200.0;

const PROTECTED: &str = "Needs Full Disk Access (System Settings › Privacy & Security)";

/// Safari's data, or `None` when there is no Safari library at all.
pub(crate) fn read(home: &Path) -> Option<SourceData> {
    let dir = home.join("Library/Safari");
    let plist = dir.join("Bookmarks.plist");
    let db = dir.join("History.db");
    if !plist.exists() && !db.exists() && !dir.exists() {
        return None;
    }
    let bookmarks = read_bookmarks(&plist);
    let history = if db.exists() {
        read_history(&db)
    } else {
        Ok(Vec::new())
    };
    let error = match (&bookmarks, &history) {
        (Err(_), _) | (_, Err(_)) => Some(PROTECTED.to_string()),
        _ => None,
    };
    Some(SourceData {
        source: ImportSource::Safari,
        bookmarks: bookmarks.unwrap_or_default(),
        history: history.unwrap_or_default(),
        error,
    })
}

#[cfg(target_os = "macos")]
fn read_bookmarks(path: &Path) -> Result<Vec<ImportedBookmark>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    Ok(bbr_web_macos::plist_bookmarks::read(path)?
        .into_iter()
        .filter(|(_, url)| is_web(url))
        .map(|(title, url)| ImportedBookmark { title, url })
        .collect())
}

#[cfg(not(target_os = "macos"))]
fn read_bookmarks(_path: &Path) -> Result<Vec<ImportedBookmark>, String> {
    Ok(Vec::new())
}

/// Safari's history: each item with its latest visit's time and title.
pub(crate) fn read_history(path: &Path) -> Result<Vec<ImportedVisit>, String> {
    let (connection, dir) = open_copy(path)?;
    let result = (|| {
        let mut statement = connection
            .prepare(
                "SELECT i.url, i.visit_count, MAX(v.visit_time), \
                 (SELECT title FROM history_visits WHERE history_item = i.id \
                  ORDER BY visit_time DESC LIMIT 1) \
                 FROM history_items i JOIN history_visits v ON v.history_item = i.id \
                 GROUP BY i.id",
            )
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        Ok(rows
            .flatten()
            .filter(|(url, _, _, _)| is_web(url))
            .map(|(url, count, time, title)| ImportedVisit {
                url,
                title: title.unwrap_or_default(),
                visit_count: count.max(1) as u64,
                last_visited_ms: ((time + SAFARI_EPOCH_OFFSET_S) * 1000.0).max(0.0) as u64,
            })
            .collect())
    })();
    drop(connection);
    let _ = std::fs::remove_dir_all(dir);
    result
}
