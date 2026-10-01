//! Git-backed source discovery; hashing/policy semantics remain in the sealed store.
use crate::{
    process::{self, Limits},
    store::{absolute, digest, no_symlinks, safe_relative, Hashes, Policy},
};
use anyhow::{bail, ensure, Context, Result};
use std::{
    fs,
    path::{Component, Path},
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

/// Defaults for each read-only Git invocation, including selected submodules.
pub fn git_limits() -> Limits {
    Limits {
        timeout: Duration::from_secs(30),
        max_output_bytes: 64 * 1024 * 1024,
        ..Limits::default()
    }
}

/// Inventory actual inputs, excluding only the supplied baseline directory.
/// With no baseline path, no directory-name exclusions are inferred.
pub fn source_hashes(repo: &Path, policy: &Policy, baselines: Option<&Path>) -> Result<Hashes> {
    source_hashes_with(
        repo,
        policy,
        baselines,
        &git_limits(),
        &AtomicBool::new(false),
    )
}

/// Same inventory with bounded Git commands and caller-owned cancellation.
/// Limits apply per Git process, not to total filesystem hashing duration.
pub fn source_hashes_with(
    repo: &Path,
    policy: &Policy,
    baselines: Option<&Path>,
    limits: &Limits,
    cancelled: &AtomicBool,
) -> Result<Hashes> {
    ensure!(
        !cancelled.load(Ordering::SeqCst),
        "Source inventory cancelled"
    );
    let repo = absolute(repo)?;
    let baselines = baselines.map(|p| absolute(&repo.join(p))).transpose()?;
    if let Some(base) = &baselines {
        no_symlinks(base)?;
        ensure!(
            !repo.starts_with(base),
            "Baseline directory must not contain the repository"
        );
    }
    ensure!(
        !policy.scopes.is_empty()
            && !policy.suffixes.is_empty()
            && policy.suffixes.iter().all(|s| !s.is_empty()),
        "Nonempty source_policy required"
    );
    for scope in &policy.scopes {
        safe_relative(scope)?;
    }
    let mut collector = Collector {
        repo: &repo,
        policy,
        baselines: baselines.as_deref(),
        limits,
        cancelled,
        result: Hashes::new(),
    };
    collector.sources(&repo, Path::new(""), &policy.scopes)?;
    for artifact in &policy.artifacts {
        safe_relative(artifact)?;
        ensure!(
            Path::new(artifact)
                .components()
                .any(|p| matches!(p, Component::Normal(_))),
            "Artifact scope must be narrow"
        );
        let path = absolute(&repo.join(artifact))?;
        ensure!(
            !baselines
                .as_ref()
                .is_some_and(|base| path.starts_with(base) || base.starts_with(&path)),
            "Baseline artifact scope refused"
        );
        collector.artifact(Path::new(artifact))?;
    }
    Ok(collector.result)
}

struct Collector<'a> {
    repo: &'a Path,
    policy: &'a Policy,
    baselines: Option<&'a Path>,
    limits: &'a Limits,
    cancelled: &'a AtomicBool,
    result: Hashes,
}
impl Collector<'_> {
    fn checkpoint(&self) -> Result<()> {
        ensure!(
            !self.cancelled.load(Ordering::SeqCst),
            "Source inventory cancelled"
        );
        Ok(())
    }
    fn git(&self, root: &Path, args: &[&str], scopes: &[String]) -> Result<process::Output> {
        process::run(
            Command::new("git")
                .current_dir(root)
                .args(args)
                .args(scopes),
            self.limits,
            self.cancelled,
            true,
        )
        .with_context(|| format!("Source inventory Git {}", args.join(" ")))
    }
    fn sources(&mut self, root: &Path, prefix: &Path, scopes: &[String]) -> Result<()> {
        let output = self.git(
            root,
            &[
                "ls-files",
                "-z",
                "--cached",
                "--others",
                "--exclude-standard",
                "--",
            ],
            scopes,
        )?;
        ensure!(output.status.success(), "git ls-files failed");
        for name in String::from_utf8(output.stdout)?
            .split('\0')
            .filter(|s| !s.is_empty())
        {
            self.checkpoint()?;
            let relative = prefix.join(name);
            let path = self.repo.join(&relative);
            if self.baselines.is_some_and(|base| path.starts_with(base))
                || relative
                    .components()
                    .any(|c| c.as_os_str().to_string_lossy().starts_with(".env"))
            {
                continue;
            }
            no_symlinks(&path)?;
            if path.is_dir() {
                // A gitlink is explicitly selected; never traverse arbitrary directories.
                let out = self.git(&path, &["rev-parse", "--show-toplevel"], &[])?;
                if out.status.success()
                    && Path::new(String::from_utf8(out.stdout)?.trim()).canonicalize()?
                        == path.canonicalize()?
                {
                    let head = self.git(&path, &["rev-parse", "HEAD"], &[])?;
                    ensure!(head.status.success(), "Uninitialized source submodule");
                    self.result.insert(
                        format!("{}@git", relative.to_string_lossy()),
                        String::from_utf8(head.stdout)?.trim().to_owned(),
                    );
                    self.sources(&path, &relative, &[".".into()])?;
                } else {
                    bail!("Uninitialized source submodule: {}", relative.display());
                }
                continue;
            }
            if !self.policy.suffixes.iter().any(|s| name.ends_with(s)) {
                continue;
            }
            no_symlinks(&path)?;
            if path.is_file() {
                self.result
                    .insert(relative.to_string_lossy().into_owned(), digest(&path)?);
            }
        }
        Ok(())
    }
    fn artifact(&mut self, relative: &Path) -> Result<()> {
        self.checkpoint()?;
        safe_relative(relative.to_str().context("Non-UTF8 artifact path")?)?;
        let path = self.repo.join(relative);
        no_symlinks(&path)?;
        if path.is_dir() {
            for entry in fs::read_dir(&path)? {
                self.artifact(&relative.join(entry?.file_name()))?;
            }
        } else {
            ensure!(
                path.is_file(),
                "Missing or non-regular artifact: {}",
                path.display()
            );
            self.result
                .insert(relative.to_string_lossy().into_owned(), digest(&path)?);
        }
        Ok(())
    }
}
