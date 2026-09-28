//! Sealed capture validation. Hashes guard integrity, not human authentication.
use anyhow::{bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

pub type Hashes = BTreeMap<String, String>;
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub scopes: Vec<String>,
    pub suffixes: Vec<String>,
    /// Explicit ignored, derived inputs (for example a verified extracted Cargo package).
    /// These are never discovered by Git; callers must opt in to narrow repository paths.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<String>,
}
impl Policy {
    pub fn same_inputs(&self, other: &Self) -> bool {
        self.scopes.iter().collect::<BTreeSet<_>>() == other.scopes.iter().collect()
            && self.suffixes.iter().collect::<BTreeSet<_>>() == other.suffixes.iter().collect()
            && self.artifacts.iter().collect::<BTreeSet<_>>() == other.artifacts.iter().collect()
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Manifest {
    pub schema: u32,
    pub cases: Vec<String>,
    pub environment: Value,
    pub images: Hashes,
    pub sources: Hashes,
    pub source_policy: Policy,
}
pub fn digest(path: &Path) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(fs::read(path).with_context(|| format!("Read {}", path.display()))?)
    ))
}
pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(serde_json::from_slice(
        &fs::read(path).with_context(|| format!("Read {}", path.display()))?,
    )?)
}
pub fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut data = serde_json::to_vec_pretty(value)?;
    data.push(b'\n');
    fs::write(path, data)?;
    Ok(())
}
pub fn safe_relative(name: &str) -> Result<()> {
    ensure!(
        !name.is_empty() && !Path::new(name).is_absolute(),
        "Unsafe source path: {name}"
    );
    ensure!(
        Path::new(name)
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir)),
        "Unsafe source path: {name}"
    );
    ensure!(
        !Path::new(name)
            .components()
            .any(|c| c.as_os_str().to_string_lossy().starts_with(".env")),
        "Secret source scope refused"
    );
    Ok(())
}
pub fn no_symlinks(path: &Path) -> Result<()> {
    for ancestor in path.ancestors() {
        if let Ok(meta) = fs::symlink_metadata(ancestor) {
            ensure!(
                !meta.file_type().is_symlink(),
                "Symlink path refused: {}",
                ancestor.display()
            );
        }
    }
    Ok(())
}
pub fn absolute(path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir => {
                ensure!(result.pop(), "Invalid path");
            }
            Component::CurDir => {}
            _ => result.push(part.as_os_str()),
        }
    }
    Ok(result)
}
pub fn disjoint(paths: &[&Path]) -> Result<()> {
    let roots = paths
        .iter()
        .map(|p| absolute(p))
        .collect::<Result<Vec<_>>>()?;
    for (i, a) in roots.iter().enumerate() {
        no_symlinks(a)?;
        for b in roots.iter().skip(i + 1) {
            ensure!(
                !a.starts_with(b) && !b.starts_with(a),
                "Candidates, baselines and report must be disjoint directories"
            );
        }
    }
    Ok(())
}
pub fn image_path(root: &Path, case: &str) -> Result<PathBuf> {
    let (base, appearance) = case.rsplit_once("--").context("Invalid case ID")?;
    ensure!(
        !base.is_empty()
            && base.as_bytes()[0].is_ascii_alphanumeric()
            && base
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            && matches!(appearance, "light" | "dark"),
        "Invalid case ID: {case}"
    );
    let path = root.join(format!("{case}.png"));
    no_symlinks(&path)?;
    Ok(path)
}
fn git_files(repo: &Path, scopes: &[String]) -> Result<Vec<String>> {
    let output = Command::new("git")
        .current_dir(repo)
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
        ])
        .args(scopes)
        .output()?;
    ensure!(output.status.success(), "git ls-files failed");
    Ok(String::from_utf8(output.stdout)?
        .split('\0')
        .filter(|x| !x.is_empty())
        .map(str::to_owned)
        .collect())
}
fn collect_sources(
    repo: &Path,
    root: &Path,
    prefix: &Path,
    scopes: &[String],
    suffixes: &[String],
    result: &mut Hashes,
) -> Result<()> {
    for name in git_files(root, scopes)? {
        let relative = prefix.join(&name);
        let path = repo.join(&relative);
        if relative.components().any(|c| {
            let n = c.as_os_str().to_string_lossy();
            n.starts_with(".env") || n == "baselines"
        }) {
            continue;
        }
        no_symlinks(&path)?;
        if path.is_dir() {
            // git ls-files returns submodule entries, not their files. Include the
            // exact revision AND working content without following arbitrary dirs.
            let out = Command::new("git")
                .current_dir(&path)
                .args(["rev-parse", "--show-toplevel"])
                .output()?;
            if out.status.success()
                && Path::new(String::from_utf8(out.stdout)?.trim()).canonicalize()?
                    == path.canonicalize()?
            {
                let head = Command::new("git")
                    .current_dir(&path)
                    .args(["rev-parse", "HEAD"])
                    .output()?;
                ensure!(head.status.success(), "Uninitialized source submodule");
                result.insert(
                    format!("{}@git", relative.to_string_lossy()),
                    String::from_utf8(head.stdout)?.trim().to_owned(),
                );
                collect_sources(repo, &path, &relative, &[".".into()], suffixes, result)?;
            } else {
                bail!("Uninitialized source submodule: {}", relative.display());
            }
            continue;
        }
        if !suffixes.iter().any(|s| name.ends_with(s)) {
            continue;
        }
        no_symlinks(&path)?;
        if path.is_file() {
            result.insert(relative.to_string_lossy().into_owned(), digest(&path)?);
        }
    }
    Ok(())
}
fn collect_artifact(repo: &Path, relative: &Path, result: &mut Hashes) -> Result<()> {
    safe_relative(relative.to_str().context("Non-UTF8 artifact path")?)?;
    ensure!(
        !relative.components().any(|p| p.as_os_str() == "baselines"),
        "Baseline artifact scope refused"
    );
    let path = repo.join(relative);
    no_symlinks(&path)?;
    if path.is_dir() {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            collect_artifact(repo, &relative.join(entry.file_name()), result)?;
        }
    } else {
        ensure!(
            path.is_file(),
            "Missing or non-regular artifact: {}",
            path.display()
        );
        result.insert(relative.to_string_lossy().into_owned(), digest(&path)?);
    }
    Ok(())
}
pub fn source_hashes(repo: &Path, policy: &Policy) -> Result<Hashes> {
    ensure!(
        !policy.scopes.is_empty()
            && !policy.suffixes.is_empty()
            && policy.suffixes.iter().all(|s| !s.is_empty()),
        "Nonempty source_policy required"
    );
    for scope in &policy.scopes {
        safe_relative(scope)?;
    }
    let mut result = Hashes::new();
    collect_sources(
        repo,
        repo,
        Path::new(""),
        &policy.scopes,
        &policy.suffixes,
        &mut result,
    )?;
    for artifact in &policy.artifacts {
        safe_relative(artifact)?;
        ensure!(
            Path::new(artifact)
                .components()
                .any(|p| matches!(p, Component::Normal(_))),
            "Artifact scope must be narrow"
        );
        collect_artifact(repo, Path::new(artifact), &mut result)?;
    }
    Ok(result)
}
pub fn load_manifest(run: &Path) -> Result<Manifest> {
    no_symlinks(&run.join("capture.json"))?;
    let m: Manifest = read_json(&run.join("capture.json"))?;
    ensure!(m.schema == 1, "Unsupported capture schema");
    let cases: BTreeSet<_> = m.cases.iter().collect();
    ensure!(
        !cases.is_empty() && cases.len() == m.cases.len(),
        "Capture requires unique, nonempty case IDs"
    );
    for case in &m.cases {
        image_path(run, case)?;
        let (base, theme) = case.rsplit_once("--").unwrap();
        ensure!(
            cases.contains(&format!(
                "{base}--{}",
                if theme == "light" { "dark" } else { "light" }
            )),
            "Missing appearance counterpart"
        );
    }
    ensure!(
        matches!(
            m.environment.get("layer").and_then(Value::as_str),
            Some("gpui" | "ios" | "android")
        ),
        "Explicit capture layer required"
    );
    ensure!(!m.sources.is_empty(), "Capture requires source hashes");
    ensure!(
        m.images.keys().collect::<BTreeSet<_>>() == cases,
        "Image hashes must match complete case inventory"
    );
    Ok(m)
}
pub fn check_sources(repo: &Path, m: &Manifest) -> Result<()> {
    for name in m.sources.keys() {
        safe_relative(name)?;
    }
    ensure!(
        source_hashes(repo, &m.source_policy)? == m.sources,
        "Capture source inventory changed; capture again"
    );
    Ok(())
}
pub fn baseline_state(root: &Path) -> Result<Hashes> {
    no_symlinks(root)?;
    let mut result = Hashes::new();
    if !root.exists() {
        return Ok(result);
    }
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.extension().is_some_and(|v| v == "png")
            || path.file_name().is_some_and(|v| v == "baseline.json")
        {
            no_symlinks(&path)?;
            result.insert(
                path.file_name().unwrap().to_string_lossy().into_owned(),
                digest(&path)?,
            );
        }
    }
    Ok(result)
}
pub fn validate_images(run: &Path, m: &Manifest) -> Result<()> {
    let actual: BTreeSet<_> = fs::read_dir(run)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|s| s == "png"))
        .map(|p| p.file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    ensure!(
        actual == m.cases.iter().cloned().collect(),
        "Incomplete capture: missing or unexpected candidate"
    );
    for case in &m.cases {
        let path = image_path(run, case)?;
        ensure!(
            digest(&path)? == m.images[case],
            "Candidate changed since capture: {case}"
        );
        read_png(&path)?;
    }
    Ok(())
}
pub fn read_png(path: &Path) -> Result<image::RgbaImage> {
    let reader = image::ImageReader::open(path)?.with_guessed_format()?;
    ensure!(
        reader.format() == Some(image::ImageFormat::Png),
        "Expected PNG: {}",
        path.display()
    );
    Ok(reader.decode()?.into_rgba8())
}
