//! Stable-inode Unix flock; shared with existing consumer flock callers.
use crate::store::no_symlinks;
use anyhow::{ensure, Context, Result};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    os::{
        fd::AsRawFd,
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub(super) struct Lock {
    file: File,
    path: PathBuf,
    pub created: bool,
}
impl Lock {
    pub fn acquire(path: &Path, deadline: Instant, cancelled: &AtomicBool) -> Result<Self> {
        ensure!(path.is_absolute(), "Gate path must be absolute");
        no_symlinks(path)?;
        let mut options = OpenOptions::new();
        options
            .read(true)
            .write(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK);
        let (file, created) = match options.clone().create_new(true).open(path) {
            Ok(file) => (file, true),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                (options.open(path)?, false)
            }
            Err(error) => return Err(error).context("Open host gate"),
        };
        let result = Self {
            file,
            path: path.to_owned(),
            created,
        };
        result.check_identity()?;
        loop {
            ensure!(
                !cancelled.load(Ordering::SeqCst),
                "Host gate acquisition cancelled"
            );
            // SAFETY: a live owned descriptor; never closes/unlinks another holder.
            if unsafe { libc::flock(result.file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
                break;
            }
            let error = io::Error::last_os_error();
            if !matches!(
                error.kind(),
                io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
            ) {
                return Err(error).context("Lock host gate");
            }
            ensure!(
                Instant::now() < deadline,
                "Host gate wait deadline exceeded: {}",
                path.display()
            );
            thread::sleep(
                Duration::from_millis(10).min(deadline.saturating_duration_since(Instant::now())),
            );
        }
        result.check_identity()?;
        Ok(result)
    }
    pub fn check_identity(&self) -> Result<()> {
        let opened = self.file.metadata()?;
        let named = fs::symlink_metadata(&self.path)?;
        ensure!(
            opened.is_file() && named.is_file() && opened.nlink() == 1 && named.nlink() == 1,
            "Gate must be a regular, unlinked file"
        );
        ensure!(
            opened.dev() == named.dev() && opened.ino() == named.ino(),
            "Gate file was replaced while held"
        );
        // SAFETY: geteuid has no pointer arguments and no side effects.
        ensure!(
            opened.uid() == unsafe { libc::geteuid() } && opened.mode() & 0o022 == 0,
            "Gate file owner or permissions are unsafe"
        );
        no_symlinks(&self.path)?;
        Ok(())
    }
    pub fn read(&mut self) -> Result<Vec<u8>> {
        self.check_identity()?;
        ensure!(
            self.file.metadata()?.len() <= 65536,
            "Host gate evidence exceeds size limit"
        );
        self.file.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        (&mut self.file).take(65537).read_to_end(&mut bytes)?;
        ensure!(
            bytes.len() <= 65536,
            "Host gate evidence exceeds size limit"
        );
        Ok(bytes)
    }
    pub fn write(&mut self, bytes: &[u8]) -> Result<()> {
        self.check_identity()?;
        ensure!(
            !bytes.is_empty() && bytes.len() <= 65536,
            "Invalid host gate evidence size"
        );
        // Never truncate to zero: a torn nonempty lease stays invalid/stale, not
        // an apparently fresh file. No rename/unlink: flock binds this inode.
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(bytes)?;
        self.file.set_len(bytes.len() as u64)?;
        self.file.sync_all()?;
        self.check_identity()?;
        Ok(())
    }
}
// Closing our File releases flock. Never clear the durable lease in Drop.
