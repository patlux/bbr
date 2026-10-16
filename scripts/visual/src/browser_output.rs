//! Exact application-still contract, retaining all eight historical shell images.
use crate::{browser_cases, browser_still, gallery, BrowserStill};
use anyhow::{ensure, Result};
use cargo_gpui_visual::store;
use std::{collections::BTreeSet, fs, path::Path};

pub fn validate_and_prepare(output: &Path) -> Result<Vec<String>> {
    let cases = browser_cases();
    let actual: BTreeSet<_> = fs::read_dir(output)?
        .map(|e| {
            e?.file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("invalid output name"))
        })
        .collect::<Result<_>>()?;
    let mut expected: BTreeSet<_> = cases.iter().map(|case| format!("{case}.png")).collect();
    for fixture in ["ordinary", "representative"]
        .into_iter()
        .chain(crate::BROWSER_SURFACES)
    {
        for mode in ["light", "dark"] {
            let name = format!("{fixture}--{mode}.snapshot.json");
            gallery::regular(&output.join(&name))?;
            let _: serde_json::Value = store::read_json(&output.join(&name))?;
            expected.insert(name);
        }
    }
    ensure!(actual == expected, "browser inventory differs");
    for case in &cases {
        let size = match browser_still(case) {
            BrowserStill::Window => (2160, 2814),
            BrowserStill::Rail => (608, 2814),
            BrowserStill::Settings => (1560, 1240),
            BrowserStill::About => (680, 760),
            BrowserStill::Element => element_size(&output.join(format!("{case}.snapshot.json")))
                .map_err(|error| anyhow::anyhow!("{case}: {error}"))?,
        };
        gallery::png(&output.join(format!("{case}.png")), size)?;
    }
    store::write_json(&output.join("inventory.json"), &cases)?;
    Ok(cases)
}

/// The exact size of an element still: the window-pixel region the app
/// reported for its named elements, inside the 2160×2814 window and never
/// the whole window.
pub fn element_size(snapshot: &Path) -> Result<(u32, u32)> {
    let value: serde_json::Value = store::read_json(snapshot)?;
    let region = &value["visual_region"];
    let named = region["ids"].as_array().is_some_and(|ids| {
        !ids.is_empty()
            && ids
                .iter()
                .all(|id| id.as_str().is_some_and(|id| id.starts_with("bbr-")))
    });
    ensure!(named, "element still without the ids it read back");
    let [x, y, width, height]: [u32; 4] = serde_json::from_value(region["pixels"].clone())?;
    ensure!(
        width > 0
            && height > 0
            && x + width <= 2160
            && y + height <= 2814
            && (width < 2160 || height < 2814),
        "element region must be a part of the 2160x2814 window"
    );
    Ok((width, height))
}
