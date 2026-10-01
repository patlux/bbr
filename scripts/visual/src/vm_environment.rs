//! Separate host compiler identity from observed guest rendering identity.
//! Pure evidence validation; no host probe is ever substituted for a missing guest.
use crate::{payload::Payload, Suite};
use anyhow::{ensure, Result};
use cargo_gpui_visual::store::Hashes;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostBuild {
    pub os: String,
    pub os_build: String,
    pub arch: String,
    pub rustc: String,
    pub xcode: String,
    pub sdk: String,
    pub gpui_source: String,
    pub tool_revision: String,
    pub executable_hashes: Hashes,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Guest {
    pub schema: u32,
    pub origin: String,
    pub os: String,
    pub os_build: String,
    pub arch: String,
    pub model: String,
    /// Raw, separately observed SPDisplaysDataType records, not host GPU facts.
    pub displays: Value,
    /// Concrete readable system/local font files; user font directories must be empty.
    pub fonts: Hashes,
    pub console_user: String,
    pub on_console: bool,
    pub screen_locked: bool,
    pub window_server: bool,
    pub timezone: String,
    pub locale: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub schema: u32,
    pub payload_sha256: String,
    pub sources: Hashes,
    /// Guest destination → observed bytes, including all sources and executables.
    pub installed: Hashes,
    pub environment_before: Guest,
    pub environment_after: Guest,
}

fn text(value: &str) -> Result<()> {
    ensure!(
        !value.trim().is_empty() && value.len() < 16384 && !value.contains('\0'),
        "missing/invalid environment observation"
    );
    Ok(())
}
fn digest(value: &str) -> Result<()> {
    ensure!(
        value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid environment digest"
    );
    Ok(())
}
impl HostBuild {
    pub fn validate(&self) -> Result<()> {
        for value in [
            &self.os,
            &self.os_build,
            &self.arch,
            &self.rustc,
            &self.xcode,
            &self.sdk,
            &self.gpui_source,
            &self.tool_revision,
        ] {
            text(value)?;
        }
        ensure!(
            !self.executable_hashes.is_empty(),
            "missing host executable identity"
        );
        for hash in self.executable_hashes.values() {
            digest(hash)?;
        }
        Ok(())
    }
}
impl Guest {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.schema == 1 && self.origin == "tart-guest",
            "actual guest environment required"
        );
        for value in [&self.os, &self.os_build, &self.arch, &self.model] {
            text(value)?;
        }
        ensure!(
            self.console_user == "admin"
                && self.on_console
                && !self.screen_locked
                && self.window_server,
            "unlocked guest WindowServer session required"
        );
        ensure!(
            self.timezone == "UTC" && self.locale == "C",
            "guest capture process locale/timezone differs"
        );
        let displays = self.displays.as_array().filter(|v| !v.is_empty());
        ensure!(
            displays.is_some_and(|v| v
                .iter()
                .all(|d| d.as_object().is_some_and(|o| !o.is_empty()))),
            "missing guest display/Metal observations"
        );
        ensure!(!self.fonts.is_empty(), "missing guest font identity");
        for (path, hash) in &self.fonts {
            ensure!(
                ["/System/Library/Fonts/", "/Library/Fonts/"]
                    .iter()
                    .any(|root| path.starts_with(root))
                    && !std::path::Path::new(path)
                        .components()
                        .any(|c| matches!(c, std::path::Component::ParentDir)),
                "font outside explicit guest font roots"
            );
            digest(hash)?;
        }
        Ok(())
    }
}

/// Validate observed guest evidence against host-staged immutable bytes. The
/// result intentionally changes environment schema; adoption requires review.
pub fn bind(
    suite: Suite,
    host: &HostBuild,
    payload: &Payload,
    payload_sha256: &str,
    receipt: &Receipt,
) -> Result<Value> {
    host.validate()?;
    receipt.environment_before.validate()?;
    receipt.environment_after.validate()?;
    digest(payload_sha256)?;
    ensure!(
        payload.schema == 1 && payload.suite == suite.id() && receipt.schema == 1,
        "VM receipt suite/schema differs"
    );
    ensure!(
        receipt.payload_sha256 == payload_sha256,
        "guest used a different payload"
    );
    ensure!(
        receipt.sources == payload.sources && !payload.sources.is_empty(),
        "guest source inventory differs"
    );
    ensure!(
        receipt.environment_before == receipt.environment_after,
        "guest rendering environment changed during capture"
    );
    ensure!(
        host.arch == receipt.environment_before.arch,
        "host/guest architecture differs"
    );
    let installed: BTreeMap<_, _> = payload
        .files
        .iter()
        .map(|f| (f.guest.to_string_lossy().into_owned(), f.sha256.clone()))
        .collect();
    ensure!(
        installed.len() == payload.files.len() && receipt.installed == installed,
        "guest did not verify every source and executable"
    );
    let artifacts: BTreeMap<_, _> = payload
        .files
        .iter()
        .filter(|f| f.executable)
        .map(|f| (f.guest.to_string_lossy().into_owned(), f.sha256.clone()))
        .collect();
    ensure!(
        host.executable_hashes == artifacts,
        "host build artifacts differ from guest payload"
    );
    Ok(json!({
        "layer":"gpui", "environment_schema":2,
        // Executable/source hashes bind this run's receipt, not the rendering
        // environment. Ordinary code changes must not require environment adoption.
        "host_build": {"os":host.os,"os_build":host.os_build,"arch":host.arch,
            "rustc":host.rustc,"xcode":host.xcode,"sdk":host.sdk,
            "gpui_source":host.gpui_source,"tool_revision":host.tool_revision},
        "guest_rendering":receipt.environment_before,
        "capture_policy": {
            "suite":suite.id(), "appearance":["light","dark"], "scale":2,
            "width_pt":if suite == Suite::Gallery {1200} else {1080},
            "height_pt":if suite == Suite::Gallery {1000} else {1407},
            "renderer":if suite == Suite::Gallery {"GPUI-CE HeadlessAppContext / MetalHeadlessRenderer"}
                else {"GPUI-CE Window::render_to_image / Metal"},
            "native_pixels":"excluded: WKWebView, AppKit traffic lights and portal child",
            "precision":"exact decoded RGBA8; no masks, tolerance or resampling",
            "coverage":"independent catalog and framing audit required"
        }
    }))
}
