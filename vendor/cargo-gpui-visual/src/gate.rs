//! Cooperative host GPU/GUI exclusion with durable unfinished-run evidence.
//!
//! A lease is cleaned only by explicit `finish`, never by Drop. OS lock release
//! after a crash does not erase its active marker. All participating consumers
//! must use the same host path; compatibility locks bridge existing flock gates.
mod lock;
use crate::{
    process::{self, CleanupFailure, Limits, Output},
    store::absolute,
};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub struct Options {
    pub path: PathBuf,
    /// Existing consumer flock paths, taken after the shared host gate. Do not
    /// wrap this inside the same legacy lock (non-reentrant, would deadlock).
    pub compatibility: Vec<PathBuf>,
    pub wait: Duration,
}
impl Options {
    /// All same-user consumers on this host share one path, independent of repo,
    /// worktree and TMPDIR. Other users/sessions need a separately agreed gate.
    pub fn host() -> Result<Self> {
        let root = std::fs::canonicalize("/tmp")?;
        // SAFETY: geteuid is side-effect-free.
        let uid = unsafe { libc::geteuid() };
        Ok(Self {
            path: root.join(format!("gpui-visual-host-{uid}.lock")),
            compatibility: vec![],
            wait: Duration::from_secs(1800),
        })
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema: u32,
    active: bool,
    owner: u32,
    pending: bool,
    groups: Vec<u32>,
    compatibility: Vec<PathBuf>,
    recovery: Option<Recovery>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Recovery {
    previous_hash: String,
    reason: String,
}

#[derive(Debug)]
pub struct Gate {
    primary: lock::Lock,
    compatibility: Vec<lock::Lock>,
    record: Record,
    poisoned: bool,
}
impl Gate {
    pub fn acquire(options: &Options, cancelled: &AtomicBool) -> Result<Self> {
        let (mut primary, compatibility, paths) = Self::locks(options, cancelled)?;
        let bytes = primary.read()?;
        let recovery = if bytes.is_empty() && primary.created {
            None
        } else {
            let previous: Record = serde_json::from_slice(&bytes)
                .context("Invalid host gate evidence; explicit investigation required")?;
            ensure!(previous.schema == 1, "Unsupported host gate schema");
            if previous.active || previous.pending || !previous.groups.is_empty() {
                return Err(crate::diagnostic::Code::HostGateUnfinished.error("Host gate has an unfinished lease; inspect owner/groups and recover explicitly").into());
            }
            previous.recovery
        };
        let record = Record {
            schema: 1,
            active: true,
            owner: std::process::id(),
            pending: false,
            groups: vec![],
            compatibility: paths,
            recovery,
        };
        primary.write(&serde_json::to_vec(&record)?)?;
        Ok(Self {
            primary,
            compatibility,
            record,
            poisoned: false,
        })
    }
    fn locks(
        options: &Options,
        cancelled: &AtomicBool,
    ) -> Result<(lock::Lock, Vec<lock::Lock>, Vec<PathBuf>)> {
        ensure!(
            !cancelled.load(Ordering::SeqCst),
            "Host gate acquisition cancelled"
        );
        ensure!(!options.wait.is_zero(), "Host gate wait must be positive");
        let deadline = Instant::now()
            .checked_add(options.wait)
            .context("Host gate wait exceeds clock range")?;
        let primary_path = absolute(&options.path)?;
        ensure!(options.path.is_absolute(), "Gate path must be absolute");
        let mut paths = options
            .compatibility
            .iter()
            .map(|path| {
                ensure!(
                    path.is_absolute(),
                    "Compatibility gate path must be absolute"
                );
                absolute(path)
            })
            .collect::<Result<Vec<_>>>()?;
        paths.sort();
        ensure!(
            !paths.contains(&primary_path) && !paths.windows(2).any(|pair| pair[0] == pair[1]),
            "Duplicate host/compatibility gate path"
        );
        let mut primary = lock::Lock::acquire(&primary_path, deadline, cancelled)?;
        // Another contender may have acquired this just-created inode before
        // its creator. Never overwrite evidence that appeared while waiting.
        if primary.created && primary.read()?.is_empty() {
            let idle = Record {
                schema: 1,
                active: false,
                owner: std::process::id(),
                pending: false,
                groups: vec![],
                compatibility: vec![],
                recovery: None,
            };
            primary.write(&serde_json::to_vec(&idle)?)?;
        }
        let mut compatibility = Vec::new();
        for path in &paths {
            compatibility.push(lock::Lock::acquire(path, deadline, cancelled)?);
        }
        Ok((primary, compatibility, paths))
    }
    fn persist(&mut self) -> Result<()> {
        let result = (|| {
            for lock in &self.compatibility {
                lock.check_identity()?;
            }
            self.primary.write(&serde_json::to_vec(&self.record)?)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    /// Execute while the lease and compatibility locks stay held. Persist the
    /// spawn intent first and its PGID immediately after spawn. Successful group
    /// cleanup clears the pending marker even for deadline/cancellation errors.
    /// Unproven cleanup or evidence-write failure permanently poisons this lease.
    pub fn execute(
        &mut self,
        command: &mut Command,
        limits: &Limits,
        cancelled: &AtomicBool,
        capture_stdout: bool,
    ) -> Result<Output> {
        ensure!(
            !self.poisoned && !self.record.pending,
            "Host gate lease is poisoned or already executing"
        );
        self.record.pending = true;
        self.persist()?;
        let result = process::run_observed(command, limits, cancelled, capture_stdout, |pid| {
            self.record.groups.push(pid);
            self.persist()
        });
        if result
            .as_ref()
            .err()
            .is_some_and(|e| e.downcast_ref::<CleanupFailure>().is_some())
        {
            self.poisoned = true;
        }
        if !self.poisoned {
            self.record.pending = false;
            self.record.groups.clear();
            self.persist()?;
        }
        result
    }
    /// Mark only a proven-clean lease idle before releasing its flock handles.
    /// A panic, crash or unfinished Drop leaves the durable active record intact.
    pub fn finish(mut self) -> Result<()> {
        ensure!(
            !self.poisoned && !self.record.pending && self.record.groups.is_empty(),
            "Host gate cleanup not proven; lease remains unfinished"
        );
        self.record.active = false;
        self.persist()
    }
    /// Explicit recovery after external process/host inspection, not automatic
    /// PID-age guessing. The caller must attest no escaped/unrecorded producer is
    /// alive. Exact locked bytes, dead owner and absent recorded groups required.
    /// Never kills any process, deletes a lock, or changes capture/baseline files.
    pub fn recover(
        options: &Options,
        expected_hash: &str,
        reason: &str,
        cancelled: &AtomicBool,
    ) -> Result<()> {
        ensure!(
            !reason.trim().is_empty() && reason.len() <= 4096,
            "Recovery requires a bounded inspection reason"
        );
        let (mut primary, _compatibility, paths) = Self::locks(options, cancelled)?;
        let bytes = primary.read()?;
        ensure!(
            format!("{:x}", Sha256::digest(&bytes)) == expected_hash,
            "Host gate evidence changed; inspect again"
        );
        let mut record: Record = serde_json::from_slice(&bytes)
            .context("Malformed evidence requires manual investigation; cannot prove dead owner")?;
        ensure!(
            record.schema == 1 && record.active,
            "No supported unfinished lease to recover"
        );
        ensure!(
            record.compatibility == paths,
            "Recovery must hold the recorded compatibility gates"
        );
        ensure!(
            absent(record.owner, false)?,
            "Host gate owner is still alive or cannot be proven absent"
        );
        for &group in &record.groups {
            ensure!(
                absent(group, true)?,
                "Recorded producer group still exists or cannot be proven absent"
            );
        }
        record.active = false;
        record.pending = false;
        record.groups.clear();
        record.recovery = Some(Recovery {
            previous_hash: expected_hash.to_owned(),
            reason: reason.to_owned(),
        });
        primary.write(&serde_json::to_vec(&record)?)
    }
}
fn absent(id: u32, group: bool) -> Result<bool> {
    ensure!(
        id > 1 && id <= i32::MAX as u32,
        "Invalid recorded process identity"
    );
    let pid = if group { -(id as i32) } else { id as i32 };
    // SAFETY: signal 0 checks existence only. EPERM is NOT absence.
    if unsafe { libc::kill(pid, 0) } == 0 {
        return Ok(false);
    }
    let error = std::io::Error::last_os_error();
    Ok(error.raw_os_error() == Some(libc::ESRCH))
}
