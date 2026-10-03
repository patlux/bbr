//! BBR's source-authored rendering classification, never a portable ignore list.
use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::{environment, store::Hashes};
use serde::Deserialize;
use serde_json::{json, Value};

/// Rendering/execution semantics and their complete locked dependency graph.
/// Report, acceptance and validation-only changes are intentionally not these
/// inputs. New capture/codec modules require a reviewed contract/list update.
pub const TOOL_INPUTS: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "src/capture.rs",
    "src/codec.rs",
    "src/config.rs",
    "src/sequence.rs",
    "src/selection.rs",
    "src/process.rs",
    "src/gate.rs",
];
pub fn tool_inputs(sources: &Hashes) -> Result<Hashes> {
    TOOL_INPUTS
        .iter()
        .map(|path| {
            let hash = sources
                .get(&format!("vendor/cargo-gpui-visual/{path}"))
                .with_context(|| format!("missing source-bound rendering tool input: {path}"))?;
            ensure!(
                hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()),
                "invalid rendering input digest"
            );
            Ok(((*path).into(), hash.clone()))
        })
        .collect()
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Display {
    origin: String,
    screens: Vec<Screen>,
    metal: Metal,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Screen {
    id: u32,
    width_px: u32,
    height_px: u32,
    width_pt: u32,
    height_pt: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Metal {
    name: String,
    registry_id: u64,
}

pub fn validate_displays(value: &Value) -> Result<()> {
    let displays: Vec<Display> = serde_json::from_value(value.clone())?;
    ensure!(
        displays.len() == 1,
        "one direct guest display probe required"
    );
    let display = &displays[0];
    ensure!(
        display.origin == "guest-coregraphics-metal",
        "direct guest display origin required"
    );
    ensure!(
        !display.screens.is_empty() && display.screens.len() < 16,
        "bounded active displays required"
    );
    let mut ids = std::collections::BTreeSet::new();
    for screen in &display.screens {
        ensure!(
            screen.id > 0 && ids.insert(screen.id),
            "unique nonzero display ID required"
        );
        ensure!(
            [
                screen.width_px,
                screen.height_px,
                screen.width_pt,
                screen.height_pt
            ]
            .iter()
            .all(|v| (1..=32768).contains(v)),
            "invalid guest display geometry"
        );
    }
    ensure!(
        !display.metal.name.trim().is_empty()
            && display.metal.name.len() <= 256
            && display.metal.registry_id > 0,
        "actual guest Metal device required"
    );
    Ok(())
}
/// Explicit lossless conversion of the old BBR schema, for both new captures and
/// an authorized exact-hash metadata migration. Unknown fields remain rendering
/// keys; only these two validated scalar leaves can become provenance.
pub fn partition(observation: &Value, inputs: &Hashes) -> Result<Value> {
    ensure!(
        observation["layer"] == "gpui" && observation["environment_schema"] == 2,
        "BBR legacy environment schema required"
    );
    let guest: crate::vm_environment::Guest =
        serde_json::from_value(observation["guest_rendering"].clone())?;
    guest.validate()?;
    ensure!(
        observation["host_build"]["tool_revision"]
            .as_str()
            .is_some_and(|v| !v.is_empty()),
        "observed tool revision required"
    );
    ensure!(
        inputs
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>()
            == TOOL_INPUTS.iter().copied().collect(),
        "complete rendering tool input set required"
    );
    for hash in inputs.values() {
        ensure!(
            hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid rendering input digest"
        );
    }
    environment::partition(
        observation,
        json!({"id":"bbr-tart-rendering-v1","tool_inputs":inputs}),
        &[
            "/host_build/tool_revision".into(),
            "/guest_rendering/displays/0/metal/registry_id".into(),
        ],
    )
}
