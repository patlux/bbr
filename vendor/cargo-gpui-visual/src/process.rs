//! Bounded, synchronous producer processes. No renderer or test-runner semantics.
//!
//! Unix children use a private process group, killed before their leader is reaped
//! (avoids PID reuse). Descendants must not daemonize, call setsid/setpgid or escape
//! the group: those are not supported producers. SIGKILL/host crashes cannot run
//! cleanup; capture seals must not be published after interrupted supervision.
use anyhow::{ensure, Result};
use std::{
    process::{Command, ExitStatus},
    sync::atomic::AtomicBool,
    time::Duration,
};

#[derive(Clone, Debug)]
pub struct Limits {
    pub timeout: Duration,
    pub cleanup_timeout: Duration,
    /// Limit for captured stdout, checked while running and before reading it.
    pub max_output_bytes: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(1800),
            cleanup_timeout: Duration::from_secs(10),
            max_output_bytes: 1024 * 1024,
        }
    }
}

#[derive(Debug)]
pub struct Output {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
}

/// Execute within the deadline, cleaning the complete owned process group before
/// returning even when the leader already exited. Nonzero exit is returned intact.
/// Callers decide whether it is a producer error. No shells are added implicitly.
/// stdin is disabled; stderr is inherited. stdout is inherited or held in a private
/// anonymous tempfile (not a pipe that an orphan could hold open indefinitely).
pub fn run(
    command: &mut Command,
    limits: &Limits,
    cancelled: &AtomicBool,
    capture_stdout: bool,
) -> Result<Output> {
    run_observed(command, limits, cancelled, capture_stdout, |_| Ok(()))
}

/// A failed cleanup means no caller may treat its host gate as clean.
#[derive(Debug)]
pub struct CleanupFailure(pub(crate) String);
impl std::fmt::Display for CleanupFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for CleanupFailure {}

pub(crate) fn run_observed(
    command: &mut Command,
    limits: &Limits,
    cancelled: &AtomicBool,
    capture_stdout: bool,
    spawned: impl FnOnce(u32) -> Result<()>,
) -> Result<Output> {
    ensure!(
        !limits.timeout.is_zero()
            && !limits.cleanup_timeout.is_zero()
            && limits.max_output_bytes > 0,
        "Process limits must be positive"
    );
    ensure!(
        std::time::Instant::now()
            .checked_add(limits.timeout)
            .is_some()
            && std::time::Instant::now()
                .checked_add(limits.cleanup_timeout)
                .is_some(),
        "Process limits exceed the monotonic clock range"
    );
    #[cfg(unix)]
    {
        unix::run(command, limits, cancelled, capture_stdout, spawned)
    }
    #[cfg(not(unix))]
    {
        let _ = (command, cancelled, capture_stdout, spawned);
        anyhow::bail!("Bounded producer execution is not implemented for this platform")
    }
}

#[cfg(unix)]
mod unix {
    use super::*;
    use anyhow::Context;
    use std::{
        fs::File,
        io::{self, Read, Seek, SeekFrom},
        os::unix::process::CommandExt,
        process::{Child, Stdio},
        sync::atomic::Ordering,
        thread,
        time::Instant,
    };

    const TICK: Duration = Duration::from_millis(10);

    struct OwnedGroup {
        child: Child,
        armed: bool,
    }
    impl OwnedGroup {
        fn signal(&self, signal: i32) -> io::Result<()> {
            // Safety: process_group(0) made the child its own PGID; the unreaped
            // leader reserves that ID until cleanup. Never target our own group.
            let result = unsafe { libc::kill(-(self.child.id() as i32), signal) };
            if result == 0 {
                return Ok(());
            }
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ESRCH) {
                Ok(())
            } else {
                Err(error)
            }
        }
        fn exited_without_reaping(&mut self) -> io::Result<bool> {
            // Safety: zeroed siginfo_t is a valid output buffer. WNOWAIT retains
            // the direct-child leader, so later group cleanup cannot signal a
            // reused PID after successful exit. No process-global SIGCHLD handler.
            let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
            let result = unsafe {
                libc::waitid(
                    libc::P_PID,
                    self.child.id(),
                    &mut info,
                    libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
                )
            };
            if result == 0 {
                Ok(unsafe { info.si_pid() } != 0)
            } else {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::Interrupted {
                    Ok(false)
                } else {
                    if error.raw_os_error() == Some(libc::ECHILD) {
                        // An external reaper/SIGCHLD policy violated ownership.
                        // The PID may be reused; never signal it in cleanup/Drop.
                        self.armed = false;
                    }
                    Err(error)
                }
            }
        }
        fn cleanup(&mut self, timeout: Duration) -> Result<ExitStatus> {
            ensure!(
                self.armed,
                "Producer child ownership was lost; cleanup not proven"
            );
            let deadline = Instant::now() + timeout;
            // Check ownership before signaling, including error paths.
            let _ = self.exited_without_reaping()?;
            if let Err(error) = self.signal(libc::SIGKILL) {
                // Darwin returns EPERM for a group containing only an unreaped
                // zombie (including signal 0). Do not silently accept EPERM:
                // require the leader already exited, reap it below, and still
                // require the entire group to disappear before returning.
                ensure!(
                    error.raw_os_error() == Some(libc::EPERM) && self.exited_without_reaping()?,
                    "Kill producer group: {error}"
                );
            }
            while !self.exited_without_reaping()? {
                ensure!(
                    Instant::now() < deadline,
                    "Producer cleanup deadline exceeded before leader exit"
                );
                thread::sleep(TICK);
            }
            let status = self.child.wait().context("Reap producer leader")?;
            // Once reaped, never send another signal to this numeric PGID.
            self.armed = false;
            wait_until_absent(deadline, || {
                // Safety: signal 0 only tests existence/permission; does not
                // signal a possibly reused process group.
                let result = unsafe { libc::kill(-(self.child.id() as i32), 0) };
                if result == 0 {
                    return Ok(true);
                }
                let error = io::Error::last_os_error();
                if error.raw_os_error() == Some(libc::ESRCH) {
                    Ok(false)
                } else {
                    Err(error)
                }
            })?;
            Ok(status)
        }
    }

    fn wait_until_absent(
        deadline: Instant,
        mut exists: impl FnMut() -> io::Result<bool>,
    ) -> Result<()> {
        loop {
            match exists() {
                Ok(false) => return Ok(()),
                Ok(true) => {}
                // After the direct leader is reaped, Darwin can still report
                // EPERM for killed descendant zombies awaiting the OS reaper.
                // This is NOT absence: keep observing within the original
                // deadline, without signaling the now-unreserved numeric PGID.
                #[cfg(target_os = "macos")]
                Err(error) if error.raw_os_error() == Some(libc::EPERM) => {}
                Err(error) => return Err(error).context("Observe cleaned producer group"),
            }
            ensure!(
                Instant::now() < deadline,
                "Producer cleanup deadline exceeded: group still exists"
            );
            thread::sleep(TICK.min(deadline.saturating_duration_since(Instant::now())));
        }
    }

    impl Drop for OwnedGroup {
        fn drop(&mut self) {
            if self.armed {
                // Panic/error fallback cannot claim successful cleanup. Avoid
                // blocking indefinitely here; normal paths call bounded cleanup.
                let _ = self.signal(libc::SIGKILL);
                let _ = self.child.try_wait();
            }
        }
    }

    pub(super) fn run(
        command: &mut Command,
        limits: &Limits,
        cancelled: &AtomicBool,
        capture_stdout: bool,
        spawned: impl FnOnce(u32) -> Result<()>,
    ) -> Result<Output> {
        ensure!(
            !cancelled.load(Ordering::SeqCst),
            "Producer cancelled before spawn"
        );
        command
            .process_group(0)
            .stdin(Stdio::null())
            .stderr(Stdio::inherit());
        let mut output: Option<File> = if capture_stdout {
            Some(tempfile::tempfile()?)
        } else {
            None
        };
        command.stdout(match &output {
            Some(file) => Stdio::from(file.try_clone()?),
            None => Stdio::inherit(),
        });
        let mut group = OwnedGroup {
            child: command.spawn().context("Spawn producer")?,
            armed: true,
        };
        let deadline = Instant::now() + limits.timeout;
        let outcome = (|| -> Result<()> {
            spawned(group.child.id())?;
            loop {
                ensure!(!cancelled.load(Ordering::SeqCst), "Producer cancelled");
                if let Some(file) = &output {
                    ensure!(
                        file.metadata()?.len() <= limits.max_output_bytes,
                        "Producer output limit exceeded"
                    );
                }
                if group.exited_without_reaping()? {
                    return Ok(());
                }
                if Instant::now() >= deadline {
                    return Err(crate::diagnostic::Code::ProcessTimeout
                        .error("Producer deadline exceeded")
                        .into());
                }
                thread::sleep(TICK.min(deadline.saturating_duration_since(Instant::now())));
            }
        })();
        let cleanup = group.cleanup(limits.cleanup_timeout);
        let status = match cleanup {
            Ok(status) => status,
            Err(error) => {
                return Err(CleanupFailure(format!(
                    "Producer cleanup failed: {error:#}; execution: {:?}",
                    outcome.err()
                ))
                .into())
            }
        };
        outcome?;
        let mut stdout = Vec::new();
        if let Some(file) = &mut output {
            ensure!(
                file.metadata()?.len() <= limits.max_output_bytes,
                "Producer output limit exceeded"
            );
            file.seek(SeekFrom::Start(0))?;
            file.take(limits.max_output_bytes.saturating_add(1))
                .read_to_end(&mut stdout)?;
            ensure!(
                stdout.len() as u64 <= limits.max_output_bytes,
                "Producer output limit exceeded"
            );
        }
        Ok(Output { status, stdout })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[cfg(target_os = "macos")]
        #[test]
        fn transient_permission_error_still_requires_observed_absence() {
            let mut observations = 0;
            wait_until_absent(Instant::now() + Duration::from_secs(1), || {
                observations += 1;
                match observations {
                    1 => Err(io::Error::from_raw_os_error(libc::EPERM)),
                    2 => Ok(true),
                    _ => Ok(false),
                }
            })
            .unwrap();
            assert_eq!(observations, 3);
        }

        #[cfg(target_os = "macos")]
        #[test]
        fn persistent_permission_error_never_proves_cleanup() {
            let deadline = Instant::now() + Duration::from_millis(20);
            let error =
                wait_until_absent(deadline, || Err(io::Error::from_raw_os_error(libc::EPERM)))
                    .unwrap_err();
            assert!(error.to_string().contains("deadline"), "{error:#}");
            assert!(Instant::now() >= deadline);
        }

        #[test]
        fn unexpected_observation_error_is_not_retried() {
            let mut observations = 0;
            let error = wait_until_absent(Instant::now() + Duration::from_secs(1), || {
                observations += 1;
                Err(io::Error::from_raw_os_error(libc::EINVAL))
            })
            .unwrap_err();
            assert!(format!("{error:#}").contains("Observe cleaned producer group"));
            assert_eq!(observations, 1);
        }
    }
}
