//! Favicons kept across a relaunch (task packet `favicon-disk-cache`).
//!
//! The controller's bounded per-profile favicon store lives in memory; this
//! module keeps it in `favicons/` in the profile directory, next to the
//! store, so a tab that owns no session after a relaunch — a deferred tab,
//! a closed pinned row, a Favorite — still shows its site's icon instead of
//! the monogram. Layout, one directory per browser profile (profiles never
//! share a record, like their web sessions):
//!
//! ```text
//! favicons/<profile id>/index         "1", then one hex host per line (insertion order)
//! favicons/<profile id>/<hex host>    the image bytes
//! ```
//!
//! Writes are diffs of what was last written: a new or changed record
//! writes one file, an evicted one deletes one, and the index is rewritten
//! when the set or the order changed. Every file is written to a temporary
//! name and renamed. Unreadable or oversized records are skipped (the
//! monogram renders, as for any failed acquisition); a directory of a
//! profile that no longer exists is removed at launch. Losing the cache is
//! harmless: the next visit acquires the icon again.

use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use bbr_web::ProfileId;

const VERSION: &str = "1";
const INDEX: &str = "index";
/// The acquisition bound (DESIGN §7.2): no kept record is larger.
pub const MAX_RECORD_BYTES: usize = 1024 * 1024;
/// At most this many records are read per profile (the store's bound).
pub const MAX_RECORDS: usize = 64;

/// One profile's records in insertion order.
pub type Records = Vec<(String, Vec<u8>)>;

/// The cache directory in a profile directory.
pub fn dir_in(profile_dir: &Path) -> PathBuf {
    profile_dir.join("favicons")
}

/// What the cache last wrote: per profile, host and content fingerprint in
/// insertion order.
#[derive(Debug, Default)]
pub struct FaviconDisk {
    dir: Option<PathBuf>,
    written: BTreeMap<u64, Vec<(String, u64)>>,
}

impl FaviconDisk {
    /// Read the kept records of every profile in `known` from `dir` and
    /// remove the directories of every other profile. The returned records
    /// count as written.
    pub fn open(dir: PathBuf, known: &[ProfileId]) -> (Self, Vec<(ProfileId, Records)>) {
        let mut loaded = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            let mut profiles: Vec<(u64, PathBuf)> = entries
                .flatten()
                .filter_map(|entry| {
                    let id = entry.file_name().to_str()?.parse::<u64>().ok()?;
                    Some((id, entry.path()))
                })
                .collect();
            profiles.sort();
            for (id, path) in profiles {
                if known.contains(&ProfileId(id)) {
                    let records = read_profile(&path);
                    if !records.is_empty() {
                        loaded.push((ProfileId(id), records));
                    }
                } else if let Err(error) = std::fs::remove_dir_all(&path) {
                    log::warn!("favicon cache of a deleted profile not removed: {error}");
                }
            }
        }
        let written = loaded
            .iter()
            .map(|(profile, records)| (profile.0, fingerprints(records)))
            .collect();
        let disk = Self {
            dir: Some(dir),
            written,
        };
        (disk, loaded)
    }

    /// Bring the directory in line with `current` (every profile's records
    /// in insertion order; a profile missing from it has none). Writes only
    /// what changed since the last write; failures are logged and retried
    /// with the next change.
    pub fn sync(&mut self, current: &[(ProfileId, Records)]) {
        let Some(dir) = self.dir.clone() else {
            return;
        };
        let gone: Vec<u64> = self
            .written
            .keys()
            .filter(|id| !current.iter().any(|(profile, _)| profile.0 == **id))
            .copied()
            .collect();
        for id in gone {
            match std::fs::remove_dir_all(dir.join(id.to_string())) {
                Ok(()) => {
                    self.written.remove(&id);
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    self.written.remove(&id);
                }
                Err(error) => log::warn!("favicon cache of profile {id} not removed: {error}"),
            }
        }
        for (profile, records) in current {
            let wanted = fingerprints(records);
            let before = self.written.get(&profile.0).cloned().unwrap_or_default();
            if wanted == before {
                continue;
            }
            match write_profile(&dir.join(profile.0.to_string()), records, &before) {
                Ok(()) => {
                    self.written.insert(profile.0, wanted);
                }
                Err(error) => log::warn!(
                    "favicon cache of profile {} not written: {error}",
                    profile.0
                ),
            }
        }
    }
}

fn fingerprints(records: &Records) -> Vec<(String, u64)> {
    records
        .iter()
        .map(|(host, bytes)| {
            let mut hasher = DefaultHasher::new();
            bytes.hash(&mut hasher);
            (host.clone(), hasher.finish())
        })
        .collect()
}

/// A host as a file name: lowercase hex of its bytes.
fn file_name(host: &str) -> String {
    host.bytes().map(|byte| format!("{byte:02x}")).collect()
}

fn host_of(name: &str) -> Option<String> {
    if name.is_empty() || !name.len().is_multiple_of(2) {
        return None;
    }
    let bytes = (0..name.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(name.get(at..at + 2)?, 16).ok())
        .collect::<Option<Vec<u8>>>()?;
    String::from_utf8(bytes).ok()
}

fn read_profile(path: &Path) -> Records {
    let Ok(index) = std::fs::read_to_string(path.join(INDEX)) else {
        return Vec::new();
    };
    let mut lines = index.lines();
    if lines.next() != Some(VERSION) {
        return Vec::new();
    }
    lines
        .filter_map(|name| {
            let host = host_of(name.trim())?;
            let bytes = std::fs::read(path.join(name.trim())).ok()?;
            (!bytes.is_empty() && bytes.len() <= MAX_RECORD_BYTES).then_some((host, bytes))
        })
        .take(MAX_RECORDS)
        .collect()
}

fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let temporary = path.with_extension("tmp");
    std::fs::write(&temporary, bytes)?;
    std::fs::rename(&temporary, path)
}

fn write_profile(path: &Path, records: &Records, before: &[(String, u64)]) -> std::io::Result<()> {
    if records.is_empty() {
        return match std::fs::remove_dir_all(path) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        };
    }
    std::fs::create_dir_all(path)?;
    let wanted = fingerprints(records);
    for ((host, bytes), entry) in records.iter().zip(&wanted) {
        if !before.contains(entry) {
            write_atomic(&path.join(file_name(host)), bytes)?;
        }
    }
    let mut index = String::from(VERSION);
    for (host, _) in records {
        index.push('\n');
        index.push_str(&file_name(host));
    }
    write_atomic(&path.join(INDEX), index.as_bytes())?;
    for (host, _) in before {
        if !records.iter().any(|(kept, _)| kept == host) {
            match std::fs::remove_file(path.join(file_name(host))) {
                Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(error),
                _ => {}
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "favicon_cache_tests.rs"]
mod tests;
