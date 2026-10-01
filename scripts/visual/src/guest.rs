//! Guest-only fixed producer execution through the immutable upstream executor.
//! No arbitrary commands, host fallback, baseline writes or capture seals.
use crate::{payload, process, Suite};
use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::{process::Limits, store};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::AtomicBool,
    time::Duration,
};

#[cfg(test)]
#[path = "guest_tests.rs"]
mod tests;

fn require_guest(cancelled: &AtomicBool) -> Result<()> {
    ensure!(
        cfg!(target_os = "macos")
            && std::env::var_os("HOME").as_deref() == Some(std::ffi::OsStr::new("/Users/admin")),
        "guest execution requires disposable macOS admin home; host capture forbidden"
    );
    ensure!(
        process::output(Command::new("/usr/bin/id").arg("-un"), cancelled)? == "admin",
        "guest execution requires actual admin identity"
    );
    let model = process::output(
        Command::new("/usr/sbin/sysctl").args(["-n", "hw.model"]),
        cancelled,
    )?;
    ensure!(
        model.starts_with("VirtualMac"),
        "guest execution requires Apple VirtualMac hardware"
    );
    Ok(())
}
fn action(suite: Suite, action: &str) -> Result<()> {
    ensure!(
        action == "capture" || (suite == Suite::Gallery && action == "inventory"),
        "unsupported guest suite action"
    );
    Ok(())
}

/// Internal transport entrypoint; do not expose a host bypass. The shared
/// payload and installed consumer must both match independently expected bytes.
pub fn dispatch(args: &[String], cancelled: &AtomicBool) -> Result<bool> {
    if args.first().map(String::as_str) != Some("guest-execute") {
        return Ok(false);
    }
    require_guest(cancelled)?;
    ensure!(
        args.len() == 7,
        "guest-execute requires suite, action, repo, share, digest and work"
    );
    let suite = Suite::parse(&args[1])?;
    action(suite, &args[2])?;
    let repo = PathBuf::from(&args[3]);
    let share = PathBuf::from(&args[4]);
    let work = PathBuf::from(&args[6]);
    for path in [&repo, &share, &work] {
        ensure!(
            path.is_absolute() && store::absolute(path)? == *path && path.is_dir(),
            "canonical guest directory required"
        );
        store::no_symlinks(path)?;
    }
    store::disjoint(&[&repo, &share, &work])?;
    ensure!(
        repo == Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()?,
        "compiled guest repository differs"
    );
    let manifest = share.join("payload.json");
    let payload = read_manifest(&manifest, &args[5], cancelled)?;
    ensure!(
        payload.repo == repo && payload.suite == suite.id(),
        "guest payload identity differs"
    );
    payload::verify(&share, &payload, cancelled)?;
    ensure!(
        std::env::current_exe()?.canonicalize()?
            == repo.join("scripts/visual/target/release/bbr-visual"),
        "installed guest consumer path differs"
    );
    verify_installed(&payload, cancelled)?;
    let mut command = command(&payload, suite, &args[2], &work)?;
    command
        .current_dir(&repo)
        .env_clear()
        .env("HOME", "/Users/admin")
        .env("USER", "admin")
        .env("LOGNAME", "admin")
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .env("TMPDIR", "/private/tmp")
        .env("TZ", "UTC")
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .env("SEED", "0");
    if suite == Suite::Browser {
        command
            .env("BBR_VISUAL_CAPTURE_BIN", repo.join("target/release/bbr"))
            .env("BBR_VISUAL_CAPTURE_DIR", work.join("raw"));
    }
    let result = cargo_gpui_visual::process::run(
        &mut command,
        &Limits {
            timeout: Duration::from_secs(if args[2] == "inventory" {
                30
            } else if suite == Suite::Gallery {
                600
            } else {
                180
            }),
            max_output_bytes: 8 * 1024 * 1024,
            ..Limits::default()
        },
        cancelled,
        true,
    )?;
    if args[2] == "capture" || !result.status.success() {
        use std::io::Write;
        std::io::stderr().write_all(&result.stdout)?;
    }
    ensure!(
        result.status.success(),
        "guest producer failed: {}",
        result.status
    );
    verify_installed(&payload, cancelled)?;
    payload::verify(&share, &payload, cancelled)?;
    read_manifest(&manifest, &args[5], cancelled)?;
    if args[2] == "inventory" {
        let text = String::from_utf8(result.stdout)?;
        ensure!(!text.trim().is_empty(), "empty guest inventory");
        use std::io::Write;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(work.join("live-inventory.tsv"))?
            .write_all(text.as_bytes())?;
    }
    Ok(true)
}

fn read_manifest(path: &Path, expected: &str, cancelled: &AtomicBool) -> Result<payload::Payload> {
    use sha2::{Digest, Sha256};
    use std::{
        io::Read,
        os::unix::fs::{MetadataExt, OpenOptionsExt},
        sync::atomic::Ordering,
    };
    const LIMIT: u64 = 8 * 1024 * 1024;
    ensure!(
        !cancelled.load(Ordering::SeqCst),
        "guest preflight cancelled"
    );
    store::no_symlinks(path)?;
    let mut file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?
        .take(LIMIT + 1);
    let meta = file.get_ref().metadata()?;
    ensure!(
        meta.is_file() && meta.nlink() == 1 && meta.len() <= LIMIT,
        "bounded regular guest manifest required"
    );
    let mut raw = Vec::new();
    file.read_to_end(&mut raw)?;
    ensure!(
        raw.len() as u64 <= LIMIT && raw.len() as u64 == meta.len(),
        "guest manifest size changed"
    );
    ensure!(
        format!("{:x}", Sha256::digest(&raw)) == expected,
        "guest payload hash differs"
    );
    Ok(serde_json::from_slice(&raw)?)
}

fn verify_installed(payload: &payload::Payload, cancelled: &AtomicBool) -> Result<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    for file in &payload.files {
        store::no_symlinks(&file.guest)?;
        let meta = fs::symlink_metadata(&file.guest)?;
        ensure!(
            meta.is_file()
                && meta.nlink() == 1
                && meta.len() == file.bytes
                && meta.permissions().mode() & 0o7777
                    == if file.executable { 0o500 } else { 0o400 },
            "installed guest size/mode differs"
        );
        ensure!(
            payload::hash(&file.guest, cancelled)? == file.sha256,
            "installed guest bytes differ"
        );
    }
    Ok(())
}
fn command(
    payload: &payload::Payload,
    suite: Suite,
    action_name: &str,
    work: &Path,
) -> Result<Command> {
    action(suite, action_name)?;
    let repo = &payload.repo;
    let raw = work.join("raw");
    store::no_symlinks(&raw)?;
    store::no_symlinks(&work.join("live-inventory.tsv"))?;
    if action_name == "inventory" {
        ensure!(
            !work.join("live-inventory.tsv").exists(),
            "fresh guest inventory required"
        );
    }
    if action_name == "capture" {
        ensure!(!raw.exists(), "fresh guest producer output required");
    }
    match suite {
        Suite::Gallery => {
            let mut command = Command::new(repo.join("target/release/bbr-gallery"));
            if action_name == "inventory" {
                command.arg("--capture-inventory");
            } else {
                command.arg("--capture-gallery").arg(raw);
            }
            Ok(command)
        }
        Suite::Browser => {
            let test = payload
                .files
                .iter()
                .find(|f| {
                    f.executable
                        && f.guest.file_name().is_some_and(|n| {
                            n.to_string_lossy().starts_with("e2e_visual_shell_capture-")
                        })
                })
                .context("missing installed browser capture test")?;
            let mut command = Command::new(&test.guest);
            command.args(["--ignored", "--nocapture", "--test-threads=1"]);
            // Existing browser producer creates its own fresh output directory.
            Ok(command)
        }
    }
}
