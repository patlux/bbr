//! Sealed capture validation. Hashes guard integrity, not human authentication.
pub use crate::inventory::{source_hashes, source_hashes_with};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection: Option<crate::selection::Selection>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sequences: Vec<crate::sequence::Sequence>,
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
/// Candidate/report PNG path of a validated case ID.
pub fn image_path(root: &Path, case: &str) -> Result<PathBuf> {
    reference_path(root, case, crate::codec::Format::Png)
}
/// A baseline store's reference path in its declared format.
pub fn reference_path(root: &Path, case: &str, format: crate::codec::Format) -> Result<PathBuf> {
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
    let path = root.join(format!("{case}.{}", format.extension()));
    no_symlinks(&path)?;
    Ok(path)
}
/// A reference image file name of any supported store format.
pub fn is_reference_file(name: &str) -> bool {
    name.ends_with(".png") || name.ends_with(".webp")
}
pub fn load_manifest(run: &Path) -> Result<Manifest> {
    no_symlinks(&run.join("capture.json"))?;
    let m: Manifest = read_json(&run.join("capture.json"))?;
    // Old readers only support schema 1 and must never accept a development
    // subset as complete evidence after silently ignoring its new scope field.
    ensure!(
        matches!((m.schema, m.selection.is_some()), (1, false) | (2, true)),
        "Unsupported capture schema or selection scope"
    );
    let cases: BTreeSet<_> = m.cases.iter().collect();
    ensure!(
        !cases.is_empty() && cases.len() == m.cases.len(),
        "Capture requires unique, nonempty case IDs"
    );
    for case in &m.cases {
        image_path(run, case)?;
        let (base, theme) = case.rsplit_once("--").unwrap();
        ensure!(
            case.starts_with("sequence-")
                || cases.contains(&format!(
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
    crate::sequence::validate(run, &m)?;
    if let Some(selection) = &m.selection {
        selection.validate_capture(&m)?;
    }
    Ok(m)
}
pub fn check_sources(repo: &Path, m: &Manifest, baselines: Option<&Path>) -> Result<()> {
    if let Some(selection) = &m.selection {
        selection.check_sources(repo, &m.sources)?;
    }
    for name in m.sources.keys() {
        safe_relative(name)?;
    }
    if source_hashes(repo, &m.source_policy, baselines)? != m.sources {
        return Err(crate::diagnostic::Code::SourceChanged
            .error("Capture source inventory changed; capture again")
            .into());
    }
    Ok(())
}
pub fn baseline_state(root: &Path) -> Result<Hashes> {
    no_symlinks(root)?;
    let mut result = Hashes::new();
    if !root.exists() {
        return Ok(result);
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path
            .file_name()
            .is_some_and(|v| is_reference_file(&v.to_string_lossy()) || v == "baseline.json")
        {
            paths.push(path);
        }
    }
    // Listing errors surface first; per-file errors keep directory order.
    for (name, hash) in crate::parallel::map(&paths, |path| {
        no_symlinks(path)?;
        Ok((
            path.file_name().unwrap().to_string_lossy().into_owned(),
            digest(path)?,
        ))
    })? {
        result.insert(name, hash);
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
    // Every image is still hashed and fully decoded; only independent files overlap.
    crate::parallel::map(&m.cases, |case| {
        let path = image_path(run, case)?;
        ensure!(
            digest(&path)? == m.images[case],
            "Candidate changed since capture: {case}"
        );
        read_png(&path)?;
        Ok(())
    })?;
    Ok(())
}
pub fn read_png(path: &Path) -> Result<image::RgbaImage> {
    let reader = image::ImageReader::open(path)?.with_guessed_format()?;
    ensure!(
        reader.format() == Some(image::ImageFormat::Png),
        "Expected PNG: {}",
        path.display()
    );
    let image = reader.decode()?;
    // Screenshots use at most 8 bits per channel. Do not silently quantize a
    // higher-precision input and report unequal samples as an exact match.
    ensure!(
        matches!(
            image.color(),
            image::ColorType::L8
                | image::ColorType::La8
                | image::ColorType::Rgb8
                | image::ColorType::Rgba8
        ),
        "16-bit PNG is unsupported; expected at most 8-bit channels: {}",
        path.display()
    );
    Ok(image.into_rgba8())
}
