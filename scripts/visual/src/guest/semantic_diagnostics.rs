//! Preserve the two existing sweep reports outside the immutable installed tree.
use crate::semantic::{validate_diagnostic, DIAGNOSTICS as REPORTS};
use anyhow::{ensure, Result};
use cargo_gpui_visual::store;
use std::{
    fs,
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};

pub(super) fn require_fresh(repo: &Path) -> Result<()> {
    let directory = repo.join("target/bbr-gallery-perf");
    store::no_symlinks(&directory)?;
    ensure!(
        !directory.exists(),
        "fresh semantic diagnostic directory required"
    );
    Ok(())
}

fn read_report(path: &Path) -> Result<Vec<u8>> {
    store::no_symlinks(path)?;
    let mut file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?
        .take(1024 * 1024 + 1);
    let meta = file.get_ref().metadata()?;
    ensure!(
        meta.is_file() && meta.nlink() == 1 && (1..=1024 * 1024).contains(&meta.len()),
        "bounded single-link semantic diagnostic required"
    );
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 == meta.len(),
        "semantic diagnostic changed"
    );
    Ok(bytes)
}

pub(super) fn collect(repo: &Path, raw: &Path, checkpoint: impl Fn() -> Result<()>) -> Result<()> {
    checkpoint()?;
    let directory = repo.join("target/bbr-gallery-perf");
    store::no_symlinks(&directory)?;
    store::no_symlinks(raw)?;
    ensure!(
        directory.is_dir() && raw.is_dir(),
        "semantic diagnostic directories missing"
    );
    let mut names = Vec::new();
    for entry in fs::read_dir(&directory)? {
        checkpoint()?;
        ensure!(
            names.len() < REPORTS.len(),
            "unexpected semantic diagnostic"
        );
        names.push(
            entry?
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("invalid diagnostic name"))?,
        );
    }
    names.sort();
    ensure!(
        names == REPORTS.map(|(suite, _)| format!("{suite}.json")),
        "semantic diagnostic inventory differs"
    );
    let mut verified = Vec::new();
    for (suite, target) in REPORTS {
        let source = directory.join(format!("{suite}.json"));
        checkpoint()?;
        let bytes = read_report(&source)?;
        validate_diagnostic(suite, &bytes)?;
        let destination = raw.join(target);
        store::no_symlinks(&destination)?;
        ensure!(
            !destination.exists(),
            "fresh semantic diagnostic destination required"
        );
        verified.push((source, destination, bytes));
    }
    for (source, destination, bytes) in verified {
        use std::io::Write;
        checkpoint()?;
        // create_new cannot overwrite raced output. Original stays until unchanged
        // copied bytes are verified; any partial error leaves no success result.
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)?
            .write_all(&bytes)?;
        ensure!(
            read_report(&source)? == bytes && read_report(&destination)? == bytes,
            "semantic diagnostic changed during copy"
        );
        checkpoint()?;
        fs::remove_file(source)?;
    }
    checkpoint()?;
    fs::remove_dir(directory)?; // only the exact now-empty generated directory
    Ok(())
}

#[cfg(test)]
#[path = "semantic_diagnostics_tests.rs"]
mod tests;
