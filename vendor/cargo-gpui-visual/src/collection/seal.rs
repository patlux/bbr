//! File-integrity seal for one immutable report, never proof of human review.
use crate::store::*;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const SEAL: &str = "collection-seal.json";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Seal {
    schema: u32,
    files: Hashes,
}
fn files(root: &Path, directory: &Path, result: &mut Hashes) -> Result<()> {
    no_symlinks(directory)?;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        no_symlinks(&path)?;
        let name = path
            .strip_prefix(root)?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Non-UTF8 report path"))?;
        if path.file_name().is_some_and(|n| n == ".review.lock")
            || (directory == root && matches!(name, SEAL | "acceptance.json"))
        {
            continue;
        }
        if entry.file_type()?.is_dir() {
            files(root, &path, result)?;
        } else {
            ensure!(entry.file_type()?.is_file(), "Non-regular collection asset");
            result.insert(name.to_owned(), digest(&path)?);
        }
    }
    Ok(())
}
fn inventory(root: &Path) -> Result<Hashes> {
    let mut result = Hashes::new();
    files(root, root, &mut result)?;
    ensure!(
        result.contains_key("index.html") && result.contains_key("collection.json"),
        "Incomplete collection report"
    );
    Ok(result)
}
pub(super) fn publish(root: &Path) -> Result<()> {
    ensure!(!root.join(SEAL).exists(), "Collection already sealed");
    write_json(
        &root.join(SEAL),
        &Seal {
            schema: 1,
            files: inventory(root)?,
        },
    )
}
pub(super) fn validate(root: &Path) -> Result<()> {
    no_symlinks(&root.join(SEAL))?;
    let seal: Seal = read_json(&root.join(SEAL))?;
    ensure!(
        seal.schema == 1 && seal.files == inventory(root)?,
        "Collection report changed since review"
    );
    Ok(())
}
