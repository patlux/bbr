//! Baseline provenance, shared once per historical capture (schema 2).
//! Loading schema 1 is read-only. Compaction never consults current sources or accepts pixels.
//! Schema 3 adds an explicit reference format (PNG or lossless WebP) and a
//! library-derived pixel fingerprint for every case; PNG stores stay schema 2.
use crate::{codec::Format, sequence::Sequence, store::*};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    /// SHA-256 of the stored reference file (PNG or WebP bytes).
    pub image_hash: String,
    pub capture_hash: String,
    /// Schema 3: `codec::pixel_hash` of the decoded reference, computed by this
    /// library after proving the stored file decodes to the accepted pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pixel_hash: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Capture {
    pub sources: Hashes,
}
#[derive(Clone, Debug, Serialize)]
pub struct Baseline {
    pub schema: u32,
    /// Schema 3 only; schema 1/2 stores are PNG.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_format: Option<Format>,
    pub environment: Value,
    pub cases: BTreeMap<String, Case>,
    pub captures: BTreeMap<String, Capture>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sequences: Vec<Sequence>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Stored {
    schema: u32,
    image_format: Option<Format>,
    environment: Value,
    cases: BTreeMap<String, StoredCase>,
    captures: Option<BTreeMap<String, Capture>>,
    #[serde(default)]
    sequences: Vec<Sequence>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredCase {
    image_hash: String,
    capture_hash: String,
    sources: Option<Hashes>,
    pixel_hash: Option<String>,
}
fn check_hash(hash: &str) -> Result<()> {
    ensure!(
        hash.len() == 64
            && hash
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "Invalid baseline SHA-256"
    );
    Ok(())
}
impl Baseline {
    pub fn new(environment: Value) -> Self {
        Self {
            schema: 2,
            image_format: None,
            environment,
            cases: BTreeMap::new(),
            captures: BTreeMap::new(),
            sequences: vec![],
        }
    }
    /// A new store. PNG keeps the schema-2 layout; WebP requires schema 3.
    pub fn with_format(environment: Value, format: Format) -> Self {
        let mut baseline = Self::new(environment);
        if format != Format::Png {
            baseline.schema = 3;
            baseline.image_format = Some(format);
        }
        baseline
    }
    pub fn format(&self) -> Format {
        self.image_format.unwrap_or_default()
    }
    /// Schema 3 records a pixel fingerprint for every case.
    pub fn records_pixels(&self) -> bool {
        self.schema == 3
    }
    pub fn reference_path(&self, root: &Path, case: &str) -> Result<std::path::PathBuf> {
        reference_path(root, case, self.format())
    }
    pub fn load(root: &Path) -> Result<Self> {
        no_symlinks(&root.join("baseline.json"))?;
        let stored: Stored = read_json(&root.join("baseline.json"))?;
        ensure!(matches!(stored.schema, 1..=3), "Baseline schema mismatch");
        ensure!(
            (stored.schema >= 2) == stored.captures.is_some(),
            "Baseline schema/captures mismatch"
        );
        ensure!(
            (stored.schema == 3) == stored.image_format.is_some(),
            "Only schema 3 baselines declare an image format"
        );
        crate::environment::validate(&stored.environment)?;
        let mut baseline = Self::new(stored.environment);
        baseline.schema = stored.schema.max(2);
        baseline.image_format = stored.image_format;
        baseline.sequences = stored.sequences;
        baseline.captures = stored.captures.unwrap_or_default();
        for (id, case) in stored.cases {
            image_path(root, &id)?;
            check_hash(&case.image_hash)?;
            check_hash(&case.capture_hash)?;
            if stored.schema == 1 {
                let sources = case.sources.context("Legacy baseline case lacks sources")?;
                baseline.record_capture(&case.capture_hash, &sources)?;
            } else {
                ensure!(
                    case.sources.is_none(),
                    "Schema 2 baseline must not contain inline sources"
                );
            }
            ensure!(
                (stored.schema == 3) == case.pixel_hash.is_some(),
                "Schema 3 baselines require a pixel fingerprint for every case: {id}"
            );
            if let Some(hash) = &case.pixel_hash {
                check_hash(hash)?;
            }
            baseline.cases.insert(
                id,
                Case {
                    image_hash: case.image_hash,
                    capture_hash: case.capture_hash,
                    pixel_hash: case.pixel_hash,
                },
            );
        }
        let referenced: BTreeSet<_> = baseline.cases.values().map(|c| &c.capture_hash).collect();
        ensure!(
            referenced == baseline.captures.keys().collect(),
            "Baseline capture references are missing or unreferenced"
        );
        for (hash, capture) in &baseline.captures {
            check_hash(hash)?;
            ensure!(
                !capture.sources.is_empty(),
                "Baseline capture requires sources"
            );
            for (path, hash) in &capture.sources {
                safe_relative(path)?;
                if path.ends_with("@git") && hash.len() == 40 {
                    ensure!(
                        hash.bytes()
                            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
                        "Invalid baseline Git object ID"
                    );
                } else {
                    check_hash(hash)?;
                }
            }
        }
        crate::sequence::validate_inventory(
            root,
            &baseline.cases.keys().cloned().collect::<Vec<_>>(),
            &baseline.sequences,
            false,
        )?;
        Ok(baseline)
    }
    pub(crate) fn record_capture(&mut self, hash: &str, sources: &Hashes) -> Result<()> {
        if let Some(existing) = self.captures.get(hash) {
            ensure!(
                existing.sources == *sources,
                "Conflicting sources for one baseline capture"
            );
        } else {
            self.captures.insert(
                hash.into(),
                Capture {
                    sources: sources.clone(),
                },
            );
        }
        Ok(())
    }
    pub(crate) fn prune_captures(&mut self) {
        let referenced: BTreeSet<_> = self.cases.values().map(|c| &c.capture_hash).collect();
        self.captures.retain(|hash, _| referenced.contains(hash));
    }
    pub(crate) fn image_errors(&self, root: &Path, state: &Hashes) -> Result<Vec<String>> {
        let mut errors = Vec::new();
        for (case, evidence) in &self.cases {
            let path = self.reference_path(root, case)?;
            if !path.is_file() || digest(&path)? != evidence.image_hash {
                errors.push(format!("Baseline metadata/image mismatch: {case}"));
            }
        }
        let extension = self.format().extension();
        let expected: BTreeSet<_> = self
            .cases
            .keys()
            .map(|c| format!("{c}.{extension}"))
            .collect();
        // Any reference file of either format counts: a stray file of the other
        // format is a mismatch, never silently ignored.
        let actual: BTreeSet<_> = state
            .keys()
            .filter(|n| is_reference_file(n))
            .cloned()
            .collect();
        if expected != actual {
            errors.push("Baseline files and baseline metadata disagree".into());
        }
        Ok(errors)
    }
}

#[derive(Debug, Serialize)]
pub struct Compaction {
    pub old_bytes: usize,
    pub new_bytes: usize,
    pub cases: usize,
    pub captures: usize,
    pub changed: bool,
    pub written: bool,
    pub old_hash: String,
    pub new_hash: String,
}
/// Explicit, lossless metadata-only migration. The caller pins the exact input digest;
/// image bytes, historical source/capture hashes, environment and timelines are retained.
/// Any previously generated acceptance report must be regenerated after a write.
pub fn compact(root: &Path, expected_hash: &str, dry_run: bool) -> Result<Compaction> {
    use sha2::{Digest, Sha256};
    no_symlinks(root)?;
    ensure!(root.is_dir(), "Existing baseline store required");
    ensure!(
        !root.join(".accept.lock").try_exists()?,
        "Baseline acceptance is locked; inspect owner"
    );
    for entry in fs::read_dir(root)? {
        ensure!(
            entry?.path().extension().is_none_or(|ext| ext != "pending"),
            "Unfinished baseline write; inspect retained evidence"
        );
    }
    let before = baseline_state(root)?;
    let path = root.join("baseline.json");
    ensure!(
        before
            .get("baseline.json")
            .is_some_and(|hash| hash == expected_hash),
        "Baseline metadata changed; expected hash does not match"
    );
    let old = fs::read(&path)?;
    let baseline = Baseline::load(root)?;
    ensure!(
        baseline.image_errors(root, &before)?.is_empty(),
        "Invalid baseline images/inventory; compaction refused"
    );
    // Every reference is fully decoded; recorded fingerprints must still match.
    let cases: Vec<_> = baseline.cases.iter().collect();
    crate::parallel::map(&cases, |(case, evidence)| {
        let pixels = baseline
            .format()
            .read(&baseline.reference_path(root, case)?)?;
        if let Some(recorded) = &evidence.pixel_hash {
            ensure!(
                &crate::codec::pixel_hash(&pixels) == recorded,
                "Reference pixels differ from the recorded fingerprint: {case}"
            );
        }
        Ok(())
    })?;
    let mut bytes = serde_json::to_vec_pretty(&baseline)?;
    bytes.push(b'\n');
    let changed = old != bytes;
    let result = Compaction {
        old_bytes: old.len(),
        new_bytes: bytes.len(),
        cases: baseline.cases.len(),
        captures: baseline.captures.len(),
        changed,
        written: !dry_run && changed,
        old_hash: expected_hash.into(),
        new_hash: format!("{:x}", Sha256::digest(&bytes)),
    };
    ensure!(
        before == baseline_state(root)?,
        "Baselines changed during compaction preflight"
    );
    if !dry_run && changed {
        let _lock = crate::accept::BaselineLock::acquire(root)?;
        ensure!(
            before == baseline_state(root)?,
            "Baselines changed before compaction"
        );
        crate::accept::atomic_write(&path, &bytes)?;
    }
    Ok(result)
}

#[derive(Debug, Serialize)]
pub struct Conversion {
    pub from: Format,
    pub to: Format,
    pub cases: usize,
    pub old_image_bytes: usize,
    pub new_image_bytes: usize,
    pub old_metadata_bytes: usize,
    pub new_metadata_bytes: usize,
    pub changed: bool,
    pub written: bool,
    pub old_hash: String,
    pub new_hash: String,
}

/// Explicit reference-format migration (for example PNG to lossless WebP). The
/// caller pins the exact `baseline.json` digest. Every reference is decoded,
/// its recorded fingerprint (if any) checked, re-encoded and proven to decode
/// to identical RGBA8 before anything is written. Environment, captures,
/// sequences and capture hashes are retained; the result is schema 3 with a
/// pixel fingerprint per case. Old-format files are removed only after the new
/// files and metadata are durably in place. Regenerate reviews after a write.
pub fn convert(root: &Path, to: Format, expected_hash: &str, dry_run: bool) -> Result<Conversion> {
    use sha2::{Digest, Sha256};
    no_symlinks(root)?;
    ensure!(root.is_dir(), "Existing baseline store required");
    ensure!(
        !root.join(".accept.lock").try_exists()?,
        "Baseline acceptance is locked; inspect owner"
    );
    for entry in fs::read_dir(root)? {
        ensure!(
            entry?.path().extension().is_none_or(|ext| ext != "pending"),
            "Unfinished baseline write; inspect retained evidence"
        );
    }
    let before = baseline_state(root)?;
    let path = root.join("baseline.json");
    ensure!(
        before
            .get("baseline.json")
            .is_some_and(|hash| hash == expected_hash),
        "Baseline metadata changed; expected hash does not match"
    );
    let old = fs::read(&path)?;
    let baseline = Baseline::load(root)?;
    ensure!(
        baseline.image_errors(root, &before)?.is_empty(),
        "Invalid baseline images/inventory; conversion refused"
    );
    let from = baseline.format();
    let cases: Vec<_> = baseline.cases.iter().collect();
    // (case, old bytes, new bytes or None when unchanged, new file hash, pixel hash)
    let converted = crate::parallel::map(&cases, |(case, evidence)| {
        let source = baseline.reference_path(root, case)?;
        let bytes = fs::read(&source)?;
        ensure!(
            format!("{:x}", Sha256::digest(&bytes)) == evidence.image_hash,
            "Reference changed during conversion: {case}"
        );
        let pixels = from
            .decode(&bytes)
            .with_context(|| format!("Invalid reference: {case}"))?;
        let fingerprint = crate::codec::pixel_hash(&pixels);
        if let Some(recorded) = &evidence.pixel_hash {
            ensure!(
                &fingerprint == recorded,
                "Reference pixels differ from the recorded fingerprint: {case}"
            );
        }
        let encoded = (to != from).then(|| to.encode(&pixels)).transpose()?;
        let hash = encoded.as_ref().map_or_else(
            || evidence.image_hash.clone(),
            |bytes| format!("{:x}", Sha256::digest(bytes)),
        );
        Ok(((*case).clone(), bytes.len(), encoded, hash, fingerprint))
    })?;
    let mut metadata = baseline.clone();
    metadata.schema = 3;
    metadata.image_format = Some(to);
    for (case, _, _, hash, fingerprint) in &converted {
        let entry = metadata.cases.get_mut(case).unwrap();
        entry.image_hash = hash.clone();
        entry.pixel_hash = Some(fingerprint.clone());
    }
    let mut bytes = serde_json::to_vec_pretty(&metadata)?;
    bytes.push(b'\n');
    let changed = old != bytes;
    let result = Conversion {
        from,
        to,
        cases: converted.len(),
        old_image_bytes: converted.iter().map(|c| c.1).sum(),
        new_image_bytes: converted
            .iter()
            .map(|c| c.2.as_ref().map_or(c.1, Vec::len))
            .sum(),
        old_metadata_bytes: old.len(),
        new_metadata_bytes: bytes.len(),
        changed,
        written: !dry_run && changed,
        old_hash: expected_hash.into(),
        new_hash: format!("{:x}", Sha256::digest(&bytes)),
    };
    ensure!(
        before == baseline_state(root)?,
        "Baselines changed during conversion preflight"
    );
    if !dry_run && changed {
        let _lock = crate::accept::BaselineLock::acquire(root)?;
        ensure!(
            before == baseline_state(root)?,
            "Baselines changed before conversion"
        );
        for (case, _, encoded, _, _) in &converted {
            if let Some(encoded) = encoded {
                let target = reference_path(root, case, to)?;
                ensure!(!target.exists(), "Conversion target already exists: {case}");
                crate::accept::atomic_write(&target, encoded)?;
            }
        }
        crate::accept::atomic_write(&path, &bytes)?;
        if to != from {
            for (case, ..) in &converted {
                fs::remove_file(reference_path(root, case, from)?)?;
            }
        }
    }
    Ok(result)
}
