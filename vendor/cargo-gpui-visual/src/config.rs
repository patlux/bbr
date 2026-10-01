//! Schema-1 configuration shared by the CLI and Rust consumer orchestration.
use crate::store::Policy;
use anyhow::{ensure, Result};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema: u32,
    /// Producer argv, not shell text. `{run}` expands to the absolute capture path.
    pub capture: Vec<String>,
    /// JSON environment probe run before/after raw capture. When absent, the
    /// producer owns its seal and before/after environment check.
    pub environment_command: Option<Vec<String>>,
    pub baselines: PathBuf,
    pub report: PathBuf,
    pub runs: PathBuf,
    pub source_policy: Policy,
    /// Independently authored story requirements; producer must support selection.
    pub story_catalog: Option<PathBuf>,
    /// Runtime request only, never an implicit config/default filter.
    #[serde(skip)]
    pub selection: Option<crate::selection::Selection>,
    /// Optional consumer-defined review presentation (gallery groups only).
    #[serde(default)]
    pub review: crate::report::Options,
    /// Reference format of a new store (`png` default, or lossless `webp`). An
    /// existing store keeps its declared format; a mismatch is refused until
    /// the store is converted explicitly (`convert-baselines`).
    #[serde(default)]
    pub baseline_format: Option<crate::codec::Format>,
}

impl Config {
    pub fn from_toml(source: &str) -> Result<Self> {
        let config: Self = toml::from_str(source)?;
        config.validate_schema()?;
        Ok(config)
    }

    /// Read only the selected file; no discovery, process execution or writes.
    pub fn load(path: &Path) -> Result<Self> {
        Self::from_toml(&fs::read_to_string(path)?)
    }

    /// Resolve exact story IDs before building or capturing. Empty selections,
    /// duplicates and unknown IDs fail; the default configuration stays full.
    pub fn select(&self, repo: &Path, stories: &[String]) -> Result<Self> {
        let mut config = self.clone();
        config.selection = Some(crate::selection::Selection::resolve(repo, self, stories)?);
        Ok(config)
    }

    pub(crate) fn validate_schema(&self) -> Result<()> {
        ensure!(self.schema == 1, "Unsupported config schema");
        self.review.validate()
    }
}
