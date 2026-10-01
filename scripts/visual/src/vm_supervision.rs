//! Cooperative Tart transport cancellation around the unchanged upstream process
//! authority. Hard termination can reap its group, but cannot prove VM cleanup:
//! absent/mismatched acknowledgement leaves the durable gate active.
use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::{
    gate::{Gate, Options},
    process::{Limits, Output},
    store,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    os::unix::{fs::PermissionsExt, process::ExitStatusExt},
    path::Path,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub struct Policy {
    pub operation: Duration,
    pub grace: Duration,
    pub process_cleanup: Duration,
    pub output_bytes: u64,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            operation: Duration::from_secs(1200),
            grace: Duration::from_secs(180),
            process_cleanup: Duration::from_secs(10),
            output_bytes: 8 * 1024 * 1024,
        }
    }
}
#[derive(Serialize)]
struct Request<'a> {
    schema: u32,
    control: &'a Path,
    payload_sha256: &'a str,
    operation_ms: u64,
    grace_ms: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Terminal {
    schema: u32,
    request_sha256: String,
    outcome: String,
}

fn create(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut file = tempfile::NamedTempFile::new_in(path.parent().context("control parent")?)?;
    file.as_file()
        .set_permissions(fs::Permissions::from_mode(0o400))?;
    file.write_all(&serde_json::to_vec(value)?)?;
    file.as_file().sync_all()?;
    file.persist_noclobber(path)?;
    Ok(())
}
fn millis(value: Duration) -> Result<u64> {
    let value = u64::try_from(value.as_millis())?;
    ensure!(
        value > 0 && value <= 86_400_000,
        "VM supervision budget must be positive and at most one day"
    );
    Ok(value)
}
fn terminal(control: &Path, expected: &str) -> Result<Terminal> {
    let path = control.join("terminal.json");
    store::no_symlinks(&path)?;
    let meta = fs::symlink_metadata(&path).context("missing terminal cleanup acknowledgement")?;
    ensure!(
        meta.is_file() && meta.len() <= 16384,
        "invalid terminal acknowledgement file"
    );
    let value: Terminal = store::read_json(&path)?;
    ensure!(
        value.schema == 1 && value.request_sha256 == expected,
        "terminal acknowledgement belongs to another request"
    );
    ensure!(
        ["completed", "failed", "cancelled"].contains(&value.outcome.as_str()),
        "invalid terminal outcome"
    );
    Ok(value)
}

/// The caller must additionally validate its concrete owned-resource cleanup
/// evidence (clone/key/disk/process absence), not infer cleanup from exit 0.
/// Neither an acknowledgement nor this transport API is a capture seal.
/// Use a fresh external control directory; no automatic gate recovery/retry.
pub fn run(
    options: &Options,
    command: &mut Command,
    control: &Path,
    payload_sha256: &str,
    policy: &Policy,
    cancelled: &AtomicBool,
    verify_cleanup: impl FnOnce() -> Result<()>,
) -> Result<Output> {
    ensure!(
        !cancelled.load(Ordering::SeqCst),
        "VM transport cancelled before preparation"
    );
    ensure!(
        control.is_absolute()
            && store::absolute(control)? == control
            && !control.exists()
            && control.parent().is_some_and(Path::is_dir),
        "fresh canonical VM control directory required"
    );
    ensure!(
        payload_sha256.len() == 64
            && payload_sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "invalid payload digest"
    );
    let request = Request {
        schema: 1,
        control,
        payload_sha256,
        operation_ms: millis(policy.operation)?,
        grace_ms: millis(policy.grace)?,
    };
    ensure!(
        !policy.process_cleanup.is_zero() && policy.output_bytes > 0,
        "positive VM process limits required"
    );
    let timeout = policy
        .operation
        .checked_add(policy.grace)
        .context("VM deadline overflow")?;
    ensure!(
        Instant::now().checked_add(timeout).is_some(),
        "VM monotonic deadline overflow"
    );
    // Acquire first; a failed lock must not create a seemingly active request.
    let mut gate = Gate::acquire(options, cancelled)?;
    fs::create_dir(control)?;
    fs::set_permissions(control, fs::Permissions::from_mode(0o700))?;
    create(&control.join("request.json"), &request)?;
    let request_sha256 = store::digest(&control.join("request.json"))?;
    command
        .env("BBR_VISUAL_CONTROL", control)
        .env("BBR_VISUAL_REQUEST_SHA256", &request_sha256);
    let stopped = AtomicBool::new(false);
    let hard_cancel = AtomicBool::new(false);
    let start = Instant::now();
    let (execution, request_result) = thread::scope(|scope| {
        let monitor = scope.spawn(|| -> Result<Option<&'static str>> {
            let mut sent = None;
            while !stopped.load(Ordering::SeqCst) {
                if sent.is_none() {
                    let reason = if cancelled.load(Ordering::SeqCst) {
                        Some("cancelled")
                    } else if start.elapsed() >= policy.operation {
                        Some("deadline")
                    } else {
                        None
                    };
                    if let Some(reason) = reason {
                        let result = create(
                            &control.join("cancel.json"),
                            &serde_json::json!({
                                "schema":1,"request_sha256":request_sha256,"reason":reason
                            }),
                        );
                        if result.is_err() {
                            hard_cancel.store(true, Ordering::SeqCst);
                            result?;
                        }
                        sent = Some((reason, Instant::now()));
                    }
                }
                if sent.is_some_and(|(_, at)| at.elapsed() >= policy.grace) {
                    hard_cancel.store(true, Ordering::SeqCst);
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Ok(sent.map(|(reason, _)| reason))
        });
        struct StopOnDrop<'a>(&'a AtomicBool);
        impl Drop for StopOnDrop<'_> {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }
        let stop_on_panic = StopOnDrop(&stopped);
        // Do NOT pass user cancellation to upstream: its SIGKILL would bypass
        // the Tart owner's finally. The monitor first requests cooperation.
        let execution = gate.execute(
            command,
            &Limits {
                timeout,
                cleanup_timeout: policy.process_cleanup,
                max_output_bytes: policy.output_bytes,
            },
            &hard_cancel,
            true,
        );
        drop(stop_on_panic);
        (execution, monitor.join())
    });
    let reason =
        request_result.map_err(|_| anyhow::anyhow!("VM cancellation monitor panicked"))??;
    // Any upstream error/hard kill is unproven external cleanup even when its
    // ordinary process group was reaped. Dropping gate preserves active=true.
    let output = execution.context("VM transport supervision interrupted; cleanup unproven")?;
    ensure!(
        !hard_cancel.load(Ordering::SeqCst),
        "VM transport exceeded cleanup grace"
    );
    ensure!(
        output.status.signal().is_none(),
        "VM transport terminated by signal; cleanup unproven"
    );
    let terminal = terminal(control, &request_sha256)?;
    verify_cleanup().context("VM owned-resource cleanup evidence failed")?;
    gate.finish()?;
    ensure!(
        reason.is_none(),
        "VM transport {} after cooperative cleanup",
        reason.unwrap_or_default()
    );
    ensure!(
        !cancelled.load(Ordering::SeqCst),
        "VM transport cancelled after cleanup"
    );
    ensure!(
        terminal.outcome == "completed" && output.status.success(),
        "VM transport did not complete: {} / {}",
        terminal.outcome,
        output.status
    );
    Ok(output)
}
