//! Immutable host expectation shared by the fixed selected guest action.
//! Catalog/plan authority stays in the upstream Selection + independent Schedule.
use crate::{
    gallery_selection::{bound_bytes, Schedule},
    payload::Payload,
    Suite,
};
use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::{config::Config, selection::Selection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

pub const PATH: &str = "selected-request.json";
const PLAN: &str = "scripts/visual/catalogs/gallery-plan.json";
pub const LIMIT: usize = 64 * 1024;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema: u32,
    payload_sha256: String,
    selection: Selection,
}

#[derive(Debug)]
pub struct Request {
    wire: Wire,
    bytes: Vec<u8>,
    digest: String,
    schedule: Schedule,
}

impl Request {
    pub fn new(payload: &Payload, digest: &str, selection: Selection) -> Result<Self> {
        Self::parse(
            payload,
            digest,
            serde_json::to_vec(&Wire {
                schema: 1,
                payload_sha256: digest.into(),
                selection,
            })?,
        )
    }
    fn parse(payload: &Payload, digest: &str, bytes: Vec<u8>) -> Result<Self> {
        ensure!(
            bytes.len() <= LIMIT,
            "selected request exceeds bootstrap bound"
        );
        let wire: Wire = serde_json::from_slice(&bytes)?;
        ensure!(
            payload.suite == "gallery",
            "selected guest request requires Gallery"
        );
        ensure!(
            digest.len() == 64
                && digest
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                && wire.schema == 1
                && wire.payload_sha256 == digest,
            "selected request payload/schema differs"
        );
        Self::configured(payload, &wire.selection)?;
        let schedule = Schedule::resolve(&payload.repo, &payload.sources, PLAN, &wire.selection)?;
        schedule.plan().require_static_capture()?;
        Ok(Self {
            digest: format!("{:x}", Sha256::digest(&bytes)),
            wire,
            bytes,
            schedule,
        })
    }
    fn configured(payload: &Payload, selection: &Selection) -> Result<()> {
        let path = Suite::Gallery.config();
        let hash = payload
            .sources
            .get(path)
            .context("unsealed Gallery config")?;
        let bytes = bound_bytes(&payload.repo, path, hash)?;
        let config = Config::from_toml(std::str::from_utf8(&bytes)?)?;
        ensure!(
            config.story_catalog.as_deref() == Some(Path::new(&selection.catalog)),
            "selected request is not the configured story catalog"
        );
        Ok(())
    }
    pub fn load(payload: &Payload, work: &Path, digest: &str, hash: &str) -> Result<Self> {
        Self::parse(payload, digest, bound_bytes(work, PATH, hash)?)
    }
    pub fn check(&self, payload: &Payload) -> Result<()> {
        ensure!(
            payload.suite == "gallery",
            "selected request requires Gallery"
        );
        Self::configured(payload, self.selection())?;
        self.schedule.check_sources(&payload.repo, &payload.sources)
    }
    pub fn selection(&self) -> &Selection {
        &self.wire.selection
    }
    pub fn payload_digest(&self) -> &str {
        &self.wire.payload_sha256
    }
    pub fn digest(&self) -> &str {
        &self.digest
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn schedule(&self) -> &Schedule {
        &self.schedule
    }
}
