//! Transfer purpose is not a visual suite or permission to execute a producer.
use crate::Suite;
use anyhow::{ensure, Result};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
    Capture(Suite),
    GallerySemantic,
}

impl From<Suite> for Purpose {
    fn from(suite: Suite) -> Self {
        Self::Capture(suite)
    }
}

impl Purpose {
    pub fn parse(raw: &str) -> Result<Self> {
        if raw == "gallery-semantic" {
            Ok(Self::GallerySemantic)
        } else {
            Ok(Self::Capture(Suite::parse(raw)?))
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Capture(suite) => suite.id(),
            Self::GallerySemantic => "gallery-semantic",
        }
    }

    /// Source and host-toolchain policy only; never use this to choose execution.
    pub fn source_suite(self) -> Suite {
        match self {
            Self::Capture(suite) => suite,
            Self::GallerySemantic => Suite::Gallery,
        }
    }

    pub(super) fn required(self) -> &'static [(&'static str, bool)] {
        match self {
            Self::Capture(Suite::Gallery) => &[("bbr-gallery", false), ("bbr-visual", false)],
            Self::Capture(Suite::Browser) => &[
                ("bbr", false),
                ("e2e_visual_shell_capture", true),
                ("bbr-visual", false),
            ],
            Self::GallerySemantic => &[("bbr-gallery", true), ("bbr-visual", false)],
        }
    }

    pub(super) fn roles(self) -> &'static [&'static str] {
        match self {
            Self::Capture(Suite::Gallery) => &["gallery", "consumer"],
            Self::Capture(Suite::Browser) => &["browser", "capture-test", "consumer"],
            Self::GallerySemantic => &["gallery-test", "consumer"],
        }
    }

    pub(super) fn check_artifact(self, artifacts: &[Value], name: &str, test: bool) -> Result<()> {
        if self != Self::GallerySemantic || name != "bbr-gallery" {
            return Ok(());
        }
        let matching: Vec<_> = artifacts
            .iter()
            .filter(|a| a["target"]["name"] == name && a["profile"]["test"] == test)
            .filter(|a| a["executable"].is_string())
            .collect();
        ensure!(
            matching.len() == 1,
            "exactly one Gallery semantic executable required"
        );
        let artifact = matching[0];
        ensure!(
            test && artifact["target"]["kind"] == serde_json::json!(["bin"])
                && artifact["features"].as_array().is_some_and(|features| {
                    features.iter().any(|feature| feature == "visual-capture")
                }),
            "Gallery semantic artifact requires visual-capture binary test harness"
        );
        Ok(())
    }
}

pub(super) fn role(guest: &str) -> Result<&'static str> {
    fn hashed(path: &str, prefix: &str) -> bool {
        path.strip_prefix(prefix).is_some_and(|suffix| {
            !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_alphanumeric())
        })
    }
    match guest {
        "target/release/bbr-gallery" => Ok("gallery"),
        "scripts/visual/target/release/bbr-visual" => Ok("consumer"),
        "target/release/bbr" => Ok("browser"),
        path if hashed(path, "target/release/deps/e2e_visual_shell_capture-") => Ok("capture-test"),
        path if hashed(path, "target/release/deps/bbr_gallery-") => Ok("gallery-test"),
        _ => anyhow::bail!("unknown executable role"),
    }
}
