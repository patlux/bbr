//! Exact schema parity with the retired probes; environment changes stay reviewable.
use crate::{provenance, Suite};
use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::process::{self, Limits};
use serde_json::{json, Value};
use std::{path::Path, process::Command, sync::atomic::AtomicBool};

pub fn probe(repo: &Path, suite: Suite, cancelled: &AtomicBool) -> Result<Value> {
    ensure!(cfg!(target_os = "macos"), "BBR capture requires macOS");
    let lock: toml::Value = toml::from_str(&std::fs::read_to_string(repo.join("Cargo.lock"))?)?;
    let gpui = lock["package"]
        .as_array()
        .context("lock packages")?
        .iter()
        .find(|p| p["name"].as_str() == Some("gpui"))
        .context("GPUI pin")?;
    let tool = provenance::verify(repo)?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let get = |argv: &[&str]| -> Result<String> {
        let remaining = deadline
            .checked_duration_since(std::time::Instant::now())
            .context("environment probe deadline exceeded")?;
        let result = process::run(
            Command::new(argv[0]).args(&argv[1..]).current_dir(repo),
            &Limits {
                timeout: remaining,
                ..Limits::default()
            },
            cancelled,
            true,
        )?;
        ensure!(
            result.status.success(),
            "environment command failed: {argv:?}"
        );
        Ok(String::from_utf8(result.stdout)?.trim().to_owned())
    };
    let mut value = json!({
        "layer":"gpui", "gpui_source": gpui["source"].as_str().context("GPUI source")?,
        "tool_revision":tool.source_revision,
        "os":get(&["/usr/bin/sw_vers","-productVersion"] )?,
        "os_build":get(&["/usr/bin/sw_vers","-buildVersion"] )?,
        "arch":get(&["/usr/bin/uname","-m"] )?,
        "host_model":get(&["/usr/sbin/sysctl","-n","hw.model"] )?,
        "rustc":get(&["rustc","--version"] )?,
        "xcode":get(&["/usr/bin/xcodebuild","-version"] )?,
        "sdk":get(&["/usr/bin/xcrun","--sdk","macosx","--show-sdk-version"] )?,
        "text_system":"CoreText / system UI fonts (OS-build bound)",
        "appearance":["light","dark"], "reduce_motion":true, "scale":2,
        "timezone":"UTC", "locale":"C",
        "native_pixels":"excluded: WKWebView, AppKit traffic lights and portal child"
    });
    let details = match suite {
        Suite::Browser => json!({
            "capture_scope":"production-chrome-only",
            "renderer":"GPUI-CE Window::render_to_image / Metal",
            "width_pt":1080,"height_pt":1407,"rail_width_pt":304,
            "settings_window_pt":[780,620], "settings_window_px":[1560,1240],
            "scale_policy":"assert exact 2160x2814 browser / 1560x1240 Settings readback; never resample",
            "contrast":"standard", "readiness":"typed settled geometry, no loading or transient moments"
        }),
        Suite::Gallery => json!({
            "capture_scope":"gallery-registered-selector-theme-pages",
            "renderer":"GPUI-CE HeadlessAppContext / MetalHeadlessRenderer",
            "platform":"test window/scheduler, real macOS text and Metal sprite atlas",
            "native_window":false,"width_pt":1200,"height_pt":1000,
            "scale_policy":"assert exact 2400x2000 readback; never resample",
            "contrast":["standard","increased"], "density":["small","normal","extralarge"],
            "theme_policy":"canvas appearance; intrinsic family density and stronger accessibility overrides",
            "readiness":"measured child pages; finite production opacity duration before final layout",
            "seed":0, "coverage_limits":"interactive-state applicability and nested/shadow framing audit pending"
        }),
    };
    value
        .as_object_mut()
        .unwrap()
        .extend(details.as_object().unwrap().clone());
    Ok(value)
}
