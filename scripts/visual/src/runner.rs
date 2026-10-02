//! Consumer build/fixture policy. Pixel comparison, seals, process cleanup and
//! host locking belong exclusively to the immutable portable library.
use crate::{browser_output, gallery, process, Suite};
use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::{
    gate::{Gate, Options},
    process::Limits,
    store,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
    sync::atomic::AtomicBool,
    time::Duration,
};

/// This recovered consumer predates the mandatory Tart route. Fail before any
/// build, directory, lease or native execution until the VM adapter is wired.
/// There is deliberately no environment-variable bypass for this boundary.
pub fn require_vm_execution() -> Result<()> {
    anyhow::bail!("Tart visual capture is not wired yet; native host capture is forbidden")
}

pub fn options(repo: &Path, cancelled: &AtomicBool) -> Result<Options> {
    let key = match std::env::var("BBR_GUI_LOCK_KEY") {
        Ok(key) => key,
        Err(_) => {
            let common = process::output(
                Command::new("git")
                    .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
                    .current_dir(repo),
                cancelled,
            )?;
            let common = fs::canonicalize(common)?;
            format!(
                "{:x}",
                Sha256::digest(common.to_str().context("UTF-8 Git common path")?.as_bytes())
            )[..16]
                .to_owned()
        }
    };
    ensure!(
        !key.is_empty()
            && key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'),
        "unsafe compatibility lock key"
    );
    let mut options = Options::host()?;
    let directory =
        PathBuf::from(std::env::var_os("BBR_GUI_LOCK_DIR").unwrap_or_else(|| "/tmp".into()));
    options
        .compatibility
        .push(fs::canonicalize(directory)?.join(format!("bbr-gui-gate-{key}.lock")));
    if let Ok(raw) = std::env::var("BBR_GUI_LOCK_TIMEOUT_SECONDS") {
        let seconds: f64 = raw.parse()?;
        ensure!(
            seconds.is_finite() && seconds > 0.0,
            "invalid GUI gate deadline"
        );
        options.wait = Duration::try_from_secs_f64(seconds)?;
    }
    Ok(options)
}

pub fn output_directory(repo: &Path, suite: Suite, path: &Path) -> Result<()> {
    ensure!(
        path.is_absolute() && !path.components().any(|p| matches!(p, Component::ParentDir)),
        "absolute output without traversal required"
    );
    store::no_symlinks(path)?;
    let runs = match suite {
        Suite::Gallery => "reports/visual/gallery-runs",
        Suite::Browser => "reports/visual/runs",
    };
    ensure!(
        !path.exists()
            && [runs, "reports/visual/batches"]
                .iter()
                .any(|root| path.starts_with(repo.join(root)) && path != repo.join(root)),
        "fresh output below configured runs required"
    );
    Ok(())
}

/// Keep both leases until the portable executor has proven descendant cleanup.
/// Explicit options permit non-GUI contracts with private locks; the CLI always
/// supplies `options`, never a user-selectable replacement for the host gate.
pub fn execute_producer(
    options: &Options,
    command: &mut Command,
    limits: &Limits,
    cancelled: &AtomicBool,
) -> Result<()> {
    let mut gate = Gate::acquire(options, cancelled)?;
    let result = gate.execute(command, limits, cancelled, false);
    // Even timeout/cancellation can release a clean lease. Cleanup failures
    // poison it and finish refuses to erase the durable unfinished evidence.
    gate.finish()?;
    ensure!(result?.status.success(), "capture producer failed");
    Ok(())
}

pub fn produce(repo: &Path, suite: Suite, output: &Path, cancelled: &AtomicBool) -> Result<()> {
    require_vm_execution()?;
    output_directory(repo, suite, output)?;
    fs::create_dir_all(output.parent().context("output parent")?)?;
    let mut command;
    let live_inventory;
    match suite {
        Suite::Gallery => {
            let artifacts = process::build(
                repo,
                &["-p", "bbr-gallery", "--features", "visual-capture"],
                cancelled,
            )?;
            let binary = process::executable(&artifacts, "bbr-gallery", false)?;
            live_inventory = Some(process::output(
                Command::new(&binary)
                    .arg("--capture-inventory")
                    .current_dir(repo),
                cancelled,
            )?);
            command = Command::new(binary);
            command
                .arg("--capture-gallery")
                .arg(output)
                .env("SEED", "0");
        }
        Suite::Browser => {
            let artifacts = process::build(
                repo,
                &[
                    "-p",
                    "bbr",
                    "--features",
                    "visual-capture",
                    "--bin",
                    "bbr",
                    "--test",
                    "e2e_visual_shell_capture",
                ],
                cancelled,
            )?;
            let binary = process::executable(&artifacts, "bbr", false)?;
            let test = process::executable(&artifacts, "e2e_visual_shell_capture", true)?;
            fs::create_dir(output)?;
            live_inventory = None;
            command = Command::new(test);
            command
                .args(["--ignored", "--nocapture", "--test-threads=1"])
                .env("BBR_VISUAL_CAPTURE_BIN", binary)
                .env("BBR_VISUAL_CAPTURE_DIR", output);
        }
    }
    command
        .current_dir(repo)
        .env("TZ", "UTC")
        .env("LANG", "C")
        .env("LC_ALL", "C");
    execute_producer(
        &options(repo, cancelled)?,
        &mut command,
        &Limits {
            timeout: Duration::from_secs(match suite {
                Suite::Gallery => 600,
                Suite::Browser => 180,
            }),
            ..Limits::default()
        },
        cancelled,
    )?;
    match suite {
        Suite::Gallery => {
            let plan = store::read_json(&repo.join("scripts/visual/catalogs/gallery-plan.json"))?;
            gallery::validate_and_prepare(output, &plan, live_inventory.as_deref().unwrap())?;
        }
        Suite::Browser => {
            browser_output::validate_and_prepare(output)?;
        }
    }
    Ok(())
}
