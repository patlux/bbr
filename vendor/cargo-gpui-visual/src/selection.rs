//! Explicit development scope, resolved from consumer-authored source requirements.
//! A story owns complete appearance pairs, pages and timelines, never frame filters.
use crate::{
    config::Config,
    store::{self, Manifest},
};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};

pub const ENVIRONMENT: &str = "GPUI_VISUAL_SELECTION";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub schema: u32,
    pub stories: Vec<Story>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Story {
    pub id: String,
    pub label: String,
    /// Appearance-free still/page IDs. Enumerate every required page explicitly.
    #[serde(default)]
    pub cases: Vec<String>,
    #[serde(default)]
    pub sequences: Vec<Timeline>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Timeline {
    pub base: String,
    pub hz: u64,
    /// Full virtual timeline length per appearance, including held ticks.
    pub ticks: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub catalog: String,
    pub catalog_hash: String,
    /// Resolved in authored catalog order, regardless of CLI argument order.
    pub stories: Vec<Story>,
}
fn identity(base: &str) -> Result<()> {
    store::image_path(Path::new("."), &format!("{base}--light"))?;
    Ok(())
}
impl Catalog {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.schema == 1 && !self.stories.is_empty(),
            "Invalid story catalog"
        );
        let mut ids = BTreeSet::new();
        let mut units = BTreeSet::new();
        for story in &self.stories {
            identity(&story.id)?;
            ensure!(ids.insert(&story.id), "Duplicate story ID: {}", story.id);
            ensure!(
                !story.label.trim().is_empty()
                    && (!story.cases.is_empty() || !story.sequences.is_empty()),
                "Story needs a label and requirements"
            );
            for base in &story.cases {
                identity(base)?;
                ensure!(
                    !base.starts_with("sequence-") && units.insert(base),
                    "Duplicate/sequence still requirement: {base}"
                );
            }
            for sequence in &story.sequences {
                identity(&sequence.base)?;
                ensure!(
                    sequence.base.starts_with("sequence-") && units.insert(&sequence.base),
                    "Duplicate/invalid sequence requirement"
                );
                ensure!(
                    [60, 120].contains(&sequence.hz)
                        && sequence.ticks > 0
                        && sequence.ticks <= 120 * 60,
                    "Invalid required timeline"
                );
            }
        }
        Ok(())
    }
    pub fn load(repo: &Path, config: &Config) -> Result<(String, Self)> {
        let path = config.story_catalog.as_ref().ok_or_else(|| {
            crate::diagnostic::Code::SelectionUnsupported
                .error("Story selection requires a configured story_catalog and producer support")
        })?;
        let relative = path.to_str().context("UTF-8 story catalog path required")?;
        store::safe_relative(relative)?;
        ensure!(
            path.components()
                .all(|p| matches!(p, std::path::Component::Normal(_))),
            "Canonical repository-relative story catalog required"
        );
        store::no_symlinks(&repo.join(path))?;
        let catalog: Self = store::read_json(&repo.join(path))?;
        catalog.validate()?;
        Ok((relative.into(), catalog))
    }
}
impl Selection {
    pub fn resolve(repo: &Path, config: &Config, requested: &[String]) -> Result<Self> {
        let (path, catalog) = Catalog::load(repo, config)?;
        let ids: BTreeSet<_> = requested.iter().collect();
        ensure!(
            !ids.is_empty() && ids.len() == requested.len(),
            "Explicit unique story IDs required"
        );
        for id in &ids {
            if !catalog.stories.iter().any(|s| &s.id == *id) {
                return Err(crate::diagnostic::Code::UnknownStory
                    .error(format!("Unknown story: {id}; use list"))
                    .into());
            }
        }
        Ok(Self {
            catalog_hash: store::digest(&repo.join(&path))?,
            catalog: path,
            stories: catalog
                .stories
                .into_iter()
                .filter(|s| ids.contains(&s.id))
                .collect(),
        })
    }
    pub fn validate(&self) -> Result<()> {
        store::safe_relative(&self.catalog)?;
        ensure!(
            Path::new(&self.catalog)
                .components()
                .all(|p| matches!(p, std::path::Component::Normal(_))),
            "Canonical story catalog required"
        );
        ensure!(
            self.catalog_hash.len() == 64
                && self.catalog_hash.bytes().all(|b| b.is_ascii_hexdigit()),
            "Invalid story catalog hash"
        );
        Catalog {
            schema: 1,
            stories: self.stories.clone(),
        }
        .validate()
    }
    pub fn check_sources(&self, repo: &Path, sources: &store::Hashes) -> Result<()> {
        self.validate()?;
        ensure!(
            sources.get(&self.catalog) == Some(&self.catalog_hash),
            "Story catalog must be included in sealed source policy"
        );
        let path = repo.join(&self.catalog);
        store::no_symlinks(&path)?;
        ensure!(
            store::digest(&path)? == self.catalog_hash,
            "Story catalog changed"
        );
        let catalog: Catalog = store::read_json(&path)?;
        catalog.validate()?;
        let selected: Vec<_> = catalog
            .stories
            .into_iter()
            .filter(|s| self.stories.iter().any(|r| r.id == s.id))
            .collect();
        ensure!(
            selected == self.stories,
            "Selection differs from authored story requirements"
        );
        Ok(())
    }
    pub fn validate_capture(&self, m: &Manifest) -> Result<()> {
        self.validate()?;
        ensure!(
            m.sources.get(&self.catalog) == Some(&self.catalog_hash),
            "Unsealed story catalog"
        );
        let mut cases = BTreeSet::new();
        let mut sequences = BTreeSet::new();
        for story in &self.stories {
            for mode in ["light", "dark"] {
                for base in &story.cases {
                    cases.insert(format!("{base}--{mode}"));
                }
                for required in &story.sequences {
                    let id = format!("{}--{mode}", required.base);
                    let timeline = m
                        .sequences
                        .iter()
                        .find(|s| s.id == id)
                        .with_context(|| format!("Selected story missing sequence: {id}"))?;
                    ensure!(
                        timeline.hz == required.hz && timeline.ticks.len() == required.ticks,
                        "Selected story has incomplete/changed timeline: {id}"
                    );
                    cases.extend(timeline.images().cloned());
                    sequences.insert(id);
                }
            }
        }
        ensure!(
            sequences == m.sequences.iter().map(|s| s.id.clone()).collect(),
            "Unexpected selected-story sequence"
        );
        ensure!(
            cases == m.cases.iter().cloned().collect(),
            "Selected stories have missing or unexpected cases/pages"
        );
        Ok(())
    }
    pub(crate) fn includes_baseline(
        &self,
        case: &str,
        sequences: &[crate::sequence::Sequence],
    ) -> bool {
        self.stories.iter().any(|story| {
            story.cases.iter().any(|base| {
                ["light", "dark"]
                    .iter()
                    .any(|mode| case == format!("{base}--{mode}"))
            }) || story.sequences.iter().any(|required| {
                sequences.iter().any(|s| {
                    s.id.rsplit_once("--")
                        .is_some_and(|(base, _)| base == required.base)
                        && s.images().any(|image| image == case)
                })
            })
        })
    }
    pub(crate) fn includes_sequence(&self, id: &str) -> bool {
        id.rsplit_once("--").is_some_and(|(base, _)| {
            self.stories
                .iter()
                .any(|s| s.sequences.iter().any(|r| r.base == base))
        })
    }
    pub(crate) fn ids(&self) -> String {
        self.stories
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}
