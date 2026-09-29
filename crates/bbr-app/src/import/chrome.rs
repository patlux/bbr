//! Blueprint D15: Google Chrome's bookmarks and history, per Chrome
//! profile (`~/Library/Application Support/Google/Chrome/<dir>`).

use std::path::Path;

use super::{is_web, open_copy, ImportSource, ImportedBookmark, ImportedVisit, SourceData};

/// Microseconds between 1601-01-01 (Chrome's epoch) and 1970-01-01.
const CHROME_EPOCH_OFFSET_US: i64 = 11_644_473_600_000_000;

fn chrome_root(home: &Path) -> std::path::PathBuf {
    home.join("Library/Application Support/Google/Chrome")
}

/// The Chrome profiles with bookmarks or history, in directory order.
pub(crate) fn read_all(home: &Path) -> Vec<SourceData> {
    let root = chrome_root(home);
    let names = profile_names(&root);
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut dirs: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
        .filter(|dir| dir == "Default" || dir.starts_with("Profile "))
        .filter(|dir| {
            root.join(dir).join("Bookmarks").exists() || root.join(dir).join("History").exists()
        })
        .collect();
    dirs.sort();
    dirs.into_iter()
        .map(|dir| {
            let name = names
                .iter()
                .find(|(candidate, _)| *candidate == dir)
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| dir.clone());
            read_profile(&root.join(&dir), ImportSource::Chrome { dir, name })
        })
        .collect()
}

/// Display names from `Local State` (`profile.info_cache.<dir>.name`).
fn profile_names(root: &Path) -> Vec<(String, String)> {
    let Ok(body) = std::fs::read_to_string(root.join("Local State")) else {
        return Vec::new();
    };
    let Ok(state) = serde_json::from_str::<serde_json::Value>(&body) else {
        return Vec::new();
    };
    state
        .pointer("/profile/info_cache")
        .and_then(|cache| cache.as_object())
        .map(|cache| {
            cache
                .iter()
                .filter_map(|(dir, info)| {
                    Some((dir.clone(), info.get("name")?.as_str()?.to_string()))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn read_profile(dir: &Path, source: ImportSource) -> SourceData {
    let bookmarks = std::fs::read_to_string(dir.join("Bookmarks"))
        .ok()
        .map(|body| parse_bookmarks(&body))
        .unwrap_or_default();
    let (history, error) = if dir.join("History").exists() {
        match read_history(&dir.join("History")) {
            Ok(history) => (history, None),
            Err(error) => (
                Vec::new(),
                Some(format!("History could not be read: {error}")),
            ),
        }
    } else {
        (Vec::new(), None)
    };
    SourceData {
        source,
        bookmarks,
        history,
        error,
    }
}

/// Every URL bookmark under Chrome's roots, depth first.
pub(crate) fn parse_bookmarks(body: &str) -> Vec<ImportedBookmark> {
    fn walk(node: &serde_json::Value, out: &mut Vec<ImportedBookmark>) {
        match node.get("type").and_then(|t| t.as_str()) {
            Some("url") => {
                let url = node.get("url").and_then(|u| u.as_str()).unwrap_or("");
                if is_web(url) {
                    out.push(ImportedBookmark {
                        title: node
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .to_string(),
                        url: url.to_string(),
                    });
                }
            }
            _ => {
                if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
                    for child in children {
                        walk(child, out);
                    }
                }
            }
        }
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(body) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if let Some(roots) = value.get("roots").and_then(|r| r.as_object()) {
        for key in ["bookmark_bar", "other", "synced"] {
            if let Some(root) = roots.get(key) {
                walk(root, &mut out);
            }
        }
    }
    out
}

/// Chrome's `urls` table (visible entries).
pub(crate) fn read_history(path: &Path) -> Result<Vec<ImportedVisit>, String> {
    let (connection, dir) = open_copy(path)?;
    let result = (|| {
        let mut statement = connection
            .prepare("SELECT url, title, visit_count, last_visit_time FROM urls WHERE hidden = 0")
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let mut visits = Vec::new();
        for row in rows.flatten() {
            let (url, title, count, time) = row;
            if !is_web(&url) {
                continue;
            }
            let unix_ms = (time - CHROME_EPOCH_OFFSET_US).max(0) / 1000;
            visits.push(ImportedVisit {
                url,
                title: title.unwrap_or_default(),
                visit_count: count.max(1) as u64,
                last_visited_ms: unix_ms as u64,
            });
        }
        Ok(visits)
    })();
    drop(connection);
    let _ = std::fs::remove_dir_all(dir);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bookmarks_come_from_every_root_and_skip_other_schemes() {
        let body = r#"{"roots":{
            "bookmark_bar":{"type":"folder","children":[
                {"type":"url","name":"Rust","url":"https://www.rust-lang.org/"},
                {"type":"folder","children":[{"type":"url","name":"Docs","url":"https://docs.rs/"}]},
                {"type":"url","name":"JS","url":"javascript:alert(1)"}]},
            "other":{"type":"folder","children":[{"type":"url","name":"Mail","url":"http://mail.example/"}]},
            "synced":{"type":"folder","children":[]}}}"#;
        let bookmarks = parse_bookmarks(body);
        assert_eq!(
            bookmarks.iter().map(|b| b.url.as_str()).collect::<Vec<_>>(),
            [
                "https://www.rust-lang.org/",
                "https://docs.rs/",
                "http://mail.example/"
            ]
        );
        assert_eq!(bookmarks[1].title, "Docs");
        assert!(parse_bookmarks("not json").is_empty());
    }
}
