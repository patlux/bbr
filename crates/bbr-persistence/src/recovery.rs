//! Blueprint D17 (L-11): the startup recovery rules that never lose data
//! silently.
//!
//! - A store stamped by a **newer** BBR is refused on the write path too and
//!   left byte-for-byte unchanged (the caller runs with a fresh in-memory
//!   session until BBR is updated).
//! - A **corrupt** current-schema store is copied aside before it is
//!   discarded, byte for byte (with its write-ahead log, if any), so the
//!   unreadable session is kept for recovery. Startup has written nothing
//!   yet, so the copy is the store as the last session left it.
//!
//! Older stamps keep the closed schema policy: they are discarded without a
//! copy (the copy would promise a recovery BBR cannot do).

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use crate::{schema, PersistenceError};

/// Refuse a store stamped by a newer BBR without touching it.
pub(crate) fn refuse_newer(connection: &Connection) -> Result<(), PersistenceError> {
    let found = schema::user_version(connection)?;
    if found > schema::SCHEMA_VERSION {
        return Err(PersistenceError::SchemaMismatch {
            found,
            supported: schema::SCHEMA_VERSION,
        });
    }
    Ok(())
}

/// The kept copy's path next to `store`: `bbr.sqlite` →
/// `bbr.unreadable-<unix seconds>.sqlite` (a counter suffix avoids a clash).
fn aside_path(store: &Path) -> PathBuf {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let stem = store
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("bbr");
    let mut candidate = store.with_file_name(format!("{stem}.unreadable-{seconds}.sqlite"));
    let mut counter = 1;
    while candidate.exists() {
        candidate = store.with_file_name(format!("{stem}.unreadable-{seconds}-{counter}.sqlite"));
        counter += 1;
    }
    candidate
}

/// Copy the unreadable store aside before it is discarded. Returns the
/// kept copy's path, or `None` when no copy could be written (the discard
/// still proceeds: startup never fails on recovery bookkeeping).
pub(crate) fn keep_aside(store: &Path) -> Option<PathBuf> {
    let target = aside_path(store);
    std::fs::copy(store, &target).ok()?;
    let wal = PathBuf::from(format!("{}-wal", store.display()));
    if wal.exists() {
        let _ = std::fs::copy(&wal, format!("{}-wal", target.display()));
    }
    Some(target)
}
