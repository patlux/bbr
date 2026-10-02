//! Existing eight-image browser output contract, shared by retained and VM routes.
use crate::{browser_cases, gallery};
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
    for fixture in ["ordinary", "representative"] {
        for mode in ["light", "dark"] {
            let name = format!("{fixture}--{mode}.snapshot.json");
            gallery::regular(&output.join(&name))?;
            let _: serde_json::Value = store::read_json(&output.join(&name))?;
            expected.insert(name);
        }
    }
    ensure!(actual == expected, "browser inventory differs");
    for case in &cases {
        gallery::png(
            &output.join(format!("{case}.png")),
            if case.contains("-rail--") {
                (608, 2814)
            } else {
                (2160, 2814)
            },
        )?;
    }
    store::write_json(&output.join("inventory.json"), &cases)?;
    Ok(cases)
}
