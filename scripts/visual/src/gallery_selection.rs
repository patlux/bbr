//! Pure bridge from upstream story selection to the independent Gallery plan.
//! No renderer, environment/global mutation, catalog generation or native launch.
use crate::gallery::{self, Plan};
use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::{
    selection::{Catalog, Selection},
    store,
};
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Component, Path},
};

/// Immutable, validated schedule. A caller must consume this BEFORE invoking
/// producers; output validation cannot detect hidden unselected rendering work.
#[derive(Debug)]
pub struct Schedule {
    selection: Selection,
    plan_path: String,
    plan_hash: String,
    ids: Vec<u64>,
    plan: Plan,
}

/// Full-suite execution/review preflight; callers with a selection use Schedule.
/// The plan must be an original sealed source, not a producer-supplied inventory.
pub(crate) fn static_plan(repo: &Path, sources: &store::Hashes) -> Result<Plan> {
    let path = "scripts/visual/catalogs/gallery-plan.json";
    let hash = sources
        .get(path)
        .context("unbound independent Gallery plan")?;
    let plan: Plan = bound_json(repo, path, hash)?;
    plan.require_static_capture()?;
    Ok(plan)
}

struct StoryPlan {
    label: String,
    pages: Vec<String>,
}

/// Read the same bounded regular bytes whose independently expected hash binds
/// the requirement. No arbitrary file, symlink, hardlink or implicit latest input.
fn bound_json<T: DeserializeOwned>(repo: &Path, relative: &str, expected: &str) -> Result<T> {
    Ok(serde_json::from_slice(&bound_bytes(
        repo, relative, expected,
    )?)?)
}

pub(crate) fn bound_bytes(repo: &Path, relative: &str, expected: &str) -> Result<Vec<u8>> {
    store::safe_relative(relative)?;
    ensure!(
        Path::new(relative)
            .components()
            .all(|p| matches!(p, Component::Normal(_))),
        "canonical source-relative plan/catalog required"
    );
    ensure!(
        expected.len() == 64 && expected.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid source hash"
    );
    let path = repo.join(relative);
    store::no_symlinks(&path)?;
    let mut file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(&path)?
        .take(8 * 1024 * 1024 + 1);
    let meta = file.get_ref().metadata()?;
    ensure!(
        meta.is_file() && meta.nlink() == 1 && meta.len() <= 8 * 1024 * 1024,
        "bounded single-link requirement file required"
    );
    let mut raw = Vec::new();
    file.read_to_end(&mut raw)?;
    ensure!(
        raw.len() as u64 == meta.len() && format!("{:x}", Sha256::digest(&raw)) == expected,
        "source requirement changed"
    );
    Ok(raw)
}

fn number(raw: &str) -> Result<u64> {
    let value = raw.parse::<u64>()?;
    ensure!(raw == format!("{value:02}"), "noncanonical Gallery number");
    Ok(value)
}
fn requirements(plan: &Plan) -> Result<BTreeMap<String, StoryPlan>> {
    plan.validate()?;
    let mut stories = BTreeMap::<String, StoryPlan>::new();
    for case in &plan.cases {
        let id = format!("gallery-{:02}", case.story_number()?);
        let story = stories.entry(id).or_insert_with(|| StoryPlan {
            label: case.label.clone(),
            pages: Vec::new(),
        });
        for page in 0..case.ranges.len() {
            let canonical = gallery::canonical_id(&format!("{}--light-page-{page:02}", case.base))?;
            story
                .pages
                .push(canonical.strip_suffix("--light").unwrap().to_owned());
        }
    }
    Ok(stories)
}

impl Schedule {
    pub fn resolve(
        repo: &Path,
        sources: &store::Hashes,
        plan_path: &str,
        selection: &Selection,
    ) -> Result<Self> {
        selection.validate()?;
        let catalog: Catalog = bound_json(repo, &selection.catalog, &selection.catalog_hash)?;
        catalog.validate()?;
        selection.check_sources(repo, sources)?;
        // Upstream check_sources also rejects a reordered or forged resolution;
        // keep the in-memory bounded catalog comparison at this bridge boundary.
        let authored: Vec<_> = catalog
            .stories
            .into_iter()
            .filter(|s| selection.stories.iter().any(|r| r.id == s.id))
            .collect();
        ensure!(
            authored == selection.stories,
            "selection differs from authored order/requirements"
        );
        let plan_hash = sources
            .get(plan_path)
            .context("independent plan must be a sealed source")?;
        let plan: Plan = bound_json(repo, plan_path, plan_hash)?;
        let required = requirements(&plan)?;
        let mut ids = Vec::new();
        let mut cases = Vec::new();
        for story in &selection.stories {
            let planned = required
                .get(&story.id)
                .context("selected Gallery story is not independently planned")?;
            ensure!(story.sequences.is_empty() && story.label == planned.label && story.cases == planned.pages,
                "selection must own every authored Gallery state/page in order; no sequence/partial/cross-story requirement");
            let id = number(
                story
                    .id
                    .strip_prefix("gallery-")
                    .context("Gallery story prefix")?,
            )?;
            ids.push(id);
            for case in &plan.cases {
                if case.story_number()? == id {
                    cases.push(case.clone());
                }
            }
        }
        Ok(Self {
            selection: selection.clone(),
            plan_path: plan_path.into(),
            plan_hash: plan_hash.clone(),
            ids,
            plan: Plan {
                schema: plan.schema,
                cases,
            },
        })
    }

    pub fn story_ids(&self) -> &[u64] {
        &self.ids
    }
    pub fn plan(&self) -> &Plan {
        &self.plan
    }
    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    /// Recheck before native scheduling and again before publishing a receipt.
    /// A newly calculated source map must not silently adopt a changed plan.
    pub fn check_sources(&self, repo: &Path, sources: &store::Hashes) -> Result<()> {
        ensure!(
            sources.get(&self.plan_path) == Some(&self.plan_hash),
            "independent plan identity changed"
        );
        let current = Self::resolve(repo, sources, &self.plan_path, &self.selection)?;
        ensure!(
            current.plan == self.plan && current.ids == self.ids,
            "resolved schedule changed"
        );
        Ok(())
    }

    pub fn validate_and_prepare(
        &self,
        repo: &Path,
        sources: &store::Hashes,
        output: &Path,
        inventory: &str,
    ) -> Result<Vec<String>> {
        self.check_sources(repo, sources)?;
        self.plan.require_static_capture()?;
        store::no_symlinks(output)?;
        ensure!(output.is_dir(), "selected raw directory required");
        for entry in fs::read_dir(output)? {
            let entry = entry?;
            let meta = fs::symlink_metadata(entry.path())?;
            ensure!(
                meta.is_file() && meta.nlink() == 1,
                "regular single-link selected output required"
            );
        }
        // Existing validator preflights the complete inventory, child ranges,
        // exact output entries and native RGBA8 dimensions before any rename.
        let ids = gallery::validate_and_prepare(output, &self.plan, inventory)?;
        self.check_sources(repo, sources)?;
        use std::io::Write;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output.join("selection.json"))?
            .write_all(&serde_json::to_vec(&self.selection)?)?;
        // Upstream capture_local, not this bridge, owns schema-2 seals and
        // selected review/acceptance rules. No capture.json is produced here.
        Ok(ids)
    }
}
