//! CLI-only stdout isolation. Library callers' descriptors are never changed.
//! Children inherit stderr directly: no pipe, reader thread or new process group.
use anyhow::Result;

#[cfg(unix)]
pub struct JsonOutput(std::os::fd::OwnedFd);
#[cfg(unix)]
impl JsonOutput {
    pub fn begin() -> Result<Self> {
        use std::{io::Write, os::fd::FromRawFd};
        std::io::stdout().flush()?;
        // SAFETY: duplicate a standard descriptor into a uniquely owned handle.
        let fd = unsafe { libc::fcntl(libc::STDOUT_FILENO, libc::F_DUPFD_CLOEXEC, 3) };
        if fd < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let output = Self(unsafe { std::os::fd::OwnedFd::from_raw_fd(fd) });
        // SAFETY: main is synchronous; isolate all producer output until response publication.
        if unsafe { libc::dup2(libc::STDERR_FILENO, libc::STDOUT_FILENO) } < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(output)
    }
    pub fn publish(&self, response: &cargo_gpui_visual::agent::Response) -> Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::from(self.0.try_clone()?);
        serde_json::to_writer(&mut file, response)?;
        writeln!(file)?;
        Ok(())
    }
}
#[cfg(unix)]
impl Drop for JsonOutput {
    fn drop(&mut self) {
        use std::{io::Write, os::fd::AsRawFd};
        let _ = std::io::stdout().flush();
        // SAFETY: descriptor remains owned until after restoring stdout.
        unsafe {
            libc::dup2(self.0.as_raw_fd(), libc::STDOUT_FILENO);
        }
    }
}
#[cfg(not(unix))]
pub struct JsonOutput;
#[cfg(not(unix))]
impl JsonOutput {
    pub fn begin() -> Result<Self> {
        Ok(Self)
    }
    pub fn publish(&self, response: &cargo_gpui_visual::agent::Response) -> Result<()> {
        // Non-Unix process execution already fails closed; portable operations do not spawn producers.
        use std::io::Write;
        let mut out = std::io::stdout().lock();
        serde_json::to_writer(&mut out, response)?;
        writeln!(out)?;
        Ok(())
    }
}
