use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::store::{self, Hashes};
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Deserialize)]
pub struct Provenance {
    pub source_revision: String,
    files: Hashes,
}

pub fn verify(repo: &Path) -> Result<Provenance> {
    let provenance: Provenance = store::read_json(&repo.join("scripts/visual/tool-source.json"))?;
    let root = repo.join("vendor/cargo-gpui-visual");
    let mut actual = Hashes::new();
    inventory(&root, &root, &mut actual)?;
    ensure!(
        actual == provenance.files,
        "Pinned portable export changed or contains unexpected sources"
    );
    Ok(provenance)
}

fn inventory(root: &Path, directory: &Path, hashes: &mut Hashes) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        // mbx's generated target directory/link may be absent or GC-dangling.
        // No other symlink or unlisted source is ignored.
        if directory == root && entry.file_name() == "target" {
            ensure!(
                entry.file_type()?.is_dir() || entry.file_type()?.is_symlink(),
                "unexpected regular target file"
            );
            continue;
        }
        let kind = entry.file_type()?;
        ensure!(
            !kind.is_symlink(),
            "symlink in pinned export: {}",
            path.display()
        );
        if kind.is_dir() {
            inventory(root, &path, hashes)?;
        } else {
            ensure!(kind.is_file(), "non-regular pinned source");
            hashes.insert(
                path.strip_prefix(root)?
                    .to_str()
                    .context("UTF-8 source path")?
                    .into(),
                store::digest(&path)?,
            );
        }
    }
    Ok(())
}
