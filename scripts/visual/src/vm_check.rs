//! The repository's official capture entry (`just visual-check`): host payload
//! build → supervised disposable Tart transport → upstream seal → Collection
//! review. This is the only executable capture route; the legacy host
//! producer stays refused (`runner::require_vm_execution`), and nothing here
//! accepts, imports or reseals images.
//!
//! Patrick's standing authorization (AGENTS.md, 2026-10-16) covers the Tart
//! profile this route uses: the existing golden image, a private NAT clone
//! with a per-run key, no host GUI, and the clone removed afterwards.
use crate::{payload, prepare, runner, vm_capture, vm_transport, Suite};
use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::{
    capture,
    collection::{self, Suite as CollectionSuite},
    config::Config,
    store,
};
use std::{
    collections::BTreeSet,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

/// Explicit suite ids in order, never an implicit "all". The read-only
/// preview keeps its `--dry-run` flag form; the live route takes bare ids.
pub fn parse_suites(args: &[String]) -> Result<Vec<Suite>> {
    ensure!(
        !args.is_empty(),
        "check requires explicit suites (browser and/or gallery); the read-only preview is `check --dry-run`"
    );
    let mut seen = BTreeSet::new();
    args.iter()
        .map(|raw| {
            ensure!(
                !raw.starts_with("--"),
                "check takes bare suite ids; `--dry-run` previews are read-only"
            );
            let suite = Suite::parse(raw)?;
            ensure!(seen.insert(suite.id()), "duplicate suite: {raw}");
            Ok(suite)
        })
        .collect()
}

/// Private host scratch outside the repository: the transport requires the
/// payload share and workspace to be disjoint from the checkout. Canonical
/// (`/private/tmp`, not the `/tmp` symlink) because payload staging binds
/// exact absolute paths.
pub fn scratch_root() -> Result<PathBuf> {
    let root = match std::env::var_os("BBR_VISUAL_SCRATCH") {
        Some(raw) => PathBuf::from(raw),
        None => PathBuf::from("/private/tmp/bbr-visual"),
    };
    ensure!(root.is_absolute(), "BBR_VISUAL_SCRATCH must be absolute");
    if !root.exists() {
        fs::create_dir_all(&root)?;
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
    }
    let root = fs::canonicalize(&root)?;
    ensure!(
        fs::symlink_metadata(&root)?.is_dir(),
        "visual scratch root must be a directory"
    );
    Ok(root)
}

#[derive(Debug)]
pub struct Outcome {
    pub batch: PathBuf,
    pub report: PathBuf,
    pub passed: bool,
}

/// One sealed run per suite under `reports/visual/batches/<stamp>/<suite>`,
/// then one Collection review. A failed suite leaves `<suite>-error.txt` and
/// its transport diagnostics; the review still presents valid siblings.
pub fn check(
    repo: &Path,
    suites: &[Suite],
    stamp: &str,
    cancelled: &AtomicBool,
) -> Result<Outcome> {
    ensure!(!suites.is_empty(), "no suites selected");
    ensure!(
        !stamp.is_empty()
            && stamp
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-'),
        "unsafe batch stamp"
    );
    let batch = repo.join("reports/visual/batches").join(stamp);
    store::no_symlinks(&batch)?;
    ensure!(!batch.exists(), "fresh batch directory required");
    fs::create_dir_all(&batch)?;
    let scratch = scratch_root()?;
    let mut reviewed = Vec::new();
    for &suite in suites {
        let config = Config::load(&repo.join(suite.config()))?;
        let run = batch.join(suite.id());
        let report = batch.join(format!("{}-diagnostic", suite.id()));
        let share = scratch.join(format!("{stamp}-{}-payload", suite.id()));
        let workspace = scratch.join(format!("{stamp}-{}-transport", suite.id()));
        println!(
            "{}: payload {} · transport {}",
            suite.id(),
            share.display(),
            workspace.display()
        );
        if let Err(error) = capture_suite(
            repo,
            suite,
            &config,
            &share,
            &workspace,
            capture::Paths {
                repo,
                run: &run,
                baselines: &repo.join(&config.baselines),
                report: &report,
            },
            cancelled,
        ) {
            eprintln!("{} capture failed: {error:#}", suite.id());
            fs::write(
                batch.join(format!("{}-error.txt", suite.id())),
                format!("{error:#}\n"),
            )?;
        }
        ensure!(
            !cancelled.load(Ordering::SeqCst),
            "visual check cancelled; partial evidence retained without a review"
        );
        reviewed.push(CollectionSuite {
            id: suite.id().into(),
            label: match suite {
                Suite::Gallery => "Gallery",
                Suite::Browser => "Browser chrome",
            }
            .into(),
            repo: repo.into(),
            run,
            baselines: repo.join(config.baselines),
            catalog: format!("scripts/visual/catalogs/{}.json", suite.id()),
        });
    }
    let report = batch.join("review");
    let result = collection::compare(&reviewed, &report)?;
    Ok(Outcome {
        batch,
        report,
        passed: result.passed,
    })
}

fn capture_suite(
    repo: &Path,
    suite: Suite,
    config: &Config,
    share: &Path,
    workspace: &Path,
    paths: capture::Paths<'_>,
    cancelled: &AtomicBool,
) -> Result<()> {
    ensure!(
        !share.exists() && !workspace.exists(),
        "fresh payload share and transport workspace required"
    );
    // Host-only release build and immutable payload staging; nothing runs.
    let host = prepare::prepare(repo, payload::Purpose::Capture(suite), share, cancelled)?;
    let staged: payload::Payload = store::read_json(&share.join("payload.json"))?;
    payload::verify(share, &staged, cancelled)?;
    ensure!(
        staged.repo == repo && staged.suite == suite.id(),
        "staged payload identity differs"
    );
    ensure!(
        paths.baselines == repo.join(&config.baselines),
        "baseline identity differs"
    );
    // Supervised transport into one disposable guest; upstream seals the run.
    vm_capture::run(
        vm_transport::Input {
            repo,
            suite,
            share,
            staged: &staged,
            host: &host,
            workspace,
            selection: None,
        },
        paths,
        &runner::options(repo, cancelled)?,
        cancelled,
    )
    .with_context(|| format!("{} Tart capture", suite.id()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(raw: &[&str]) -> Vec<String> {
        raw.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn suites_are_explicit_ordered_and_unique() {
        let suites = parse_suites(&args(&["browser", "gallery"])).unwrap();
        assert_eq!(suites, [Suite::Browser, Suite::Gallery]);
        assert!(parse_suites(&args(&[])).is_err());
        assert!(parse_suites(&args(&["browser", "browser"])).is_err());
        assert!(parse_suites(&args(&["--suite", "browser"])).is_err());
        assert!(parse_suites(&args(&["--dry-run"])).is_err());
        assert!(parse_suites(&args(&["gallery-semantic"])).is_err());
    }

    #[test]
    fn check_refuses_unsafe_stamps_and_existing_batches_before_any_build() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().canonicalize().unwrap();
        let cancelled = AtomicBool::new(false);
        for stamp in ["", "../x", "a b", "x/y"] {
            let error = check(&repo, &[Suite::Browser], stamp, &cancelled).unwrap_err();
            assert!(
                error.to_string().contains("unsafe batch stamp"),
                "{stamp:?}"
            );
        }
        let existing = repo.join("reports/visual/batches/taken");
        fs::create_dir_all(&existing).unwrap();
        let error = check(&repo, &[Suite::Browser], "taken", &cancelled).unwrap_err();
        assert!(error.to_string().contains("fresh batch directory"));
        assert!(!repo.join("reports/visual/batches/taken/browser").exists());
    }
}
