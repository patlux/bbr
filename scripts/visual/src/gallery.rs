//! Structural validation against an independent, source-controlled page plan.
use anyhow::{ensure, Context, Result};
use cargo_gpui_visual::store;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub schema: u32,
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub base: String,
    pub label: String,
    pub state: String,
    pub group: String,
    pub ranges: Vec<[usize; 2]>,
    /// Production Elevation has one fewer shadow row in Dark appearance.
    #[serde(default)]
    pub dark_ranges: Option<Vec<[usize; 2]>>,
}

pub fn canonical_id(raw: &str) -> Result<String> {
    let (case, index) = raw.rsplit_once("-page-").context("missing page identity")?;
    let (base, mode) = case.rsplit_once("--").context("missing appearance")?;
    ensure!(["light", "dark"].contains(&mode), "invalid appearance");
    let number: usize = index.parse()?;
    ensure!(format!("{number:02}") == index, "noncanonical page number");
    let id = format!("{base}-page-{index}--{mode}");
    store::image_path(Path::new("."), &id)?;
    Ok(id)
}

/// Validate all raw files without renaming or publishing completion metadata.
/// Selected guest captures use this same validator before combining story runs.
pub(crate) fn validate_raw(
    output: &Path,
    plan: &Plan,
    inventory: &str,
) -> Result<Vec<(String, String)>> {
    ensure!(
        plan.schema == 1 && !plan.cases.is_empty(),
        "invalid required Gallery plan"
    );
    let mut expected = BTreeMap::new();
    let mut rows = Vec::new();
    let mut ids = Vec::new();
    for case in &plan.cases {
        ensure!(
            !case.group.is_empty() && !case.ranges.is_empty(),
            "empty case metadata"
        );
        for mode in ["light", "dark"] {
            let ranges = if mode == "dark" {
                case.dark_ranges.as_ref().unwrap_or(&case.ranges)
            } else {
                &case.ranges
            };
            ensure!(
                ranges.len() == case.ranges.len(),
                "appearance page pairing differs"
            );
            let total = ranges.last().context("pages")?[1];
            let mut next = 0;
            for range in ranges {
                ensure!(
                    range[0] == next && next < range[1] && range[1] <= total,
                    "invalid independent range plan"
                );
                next = range[1];
            }
            let id = format!("{}--{mode}", case.base);
            ensure!(
                expected
                    .insert(id.clone(), (case.label.clone(), case.state.clone()))
                    .is_none(),
                "duplicate required case"
            );
            for (page, [start, end]) in ranges.iter().enumerate() {
                let raw = format!("{id}-page-{page:02}");
                ids.push((raw.clone(), canonical_id(&raw)?));
                rows.push(format!(
                    "{raw}\t{id}\t{}\t{}\t{start}\t{end}\t{total}",
                    case.label, case.state
                ));
            }
        }
    }
    let mut actual = BTreeMap::new();
    for row in inventory.lines() {
        let fields: Vec<_> = row.split('\t').collect();
        ensure!(fields.len() == 3, "malformed live registry inventory");
        ensure!(
            actual
                .insert(
                    fields[0].to_owned(),
                    (fields[1].to_owned(), fields[2].to_owned())
                )
                .is_none(),
            "duplicate live case"
        );
    }
    ensure!(
        actual == expected,
        "live registry differs from independent required plan"
    );
    regular(&output.join("coverage.tsv"))?;
    ensure!(
        fs::read_to_string(output.join("coverage.tsv"))? == rows.join("\n") + "\n",
        "missing, extra, reordered or changed Gallery pages"
    );
    let mut names: BTreeSet<_> = ids.iter().map(|(raw, _)| format!("{raw}.png")).collect();
    names.insert("coverage.tsv".into());
    let actual_names = fs::read_dir(output)?
        .map(|e| {
            e?.file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("non UTF-8 output"))
        })
        .collect::<Result<BTreeSet<_>>>()?;
    ensure!(actual_names == names, "unexpected Gallery output entries");
    // Preflight every image before renaming any. Decode without converting away
    // precision; dimensions and RGBA8 are requirements, never transformations.
    for (raw, _) in &ids {
        png(&output.join(format!("{raw}.png")), (2400, 2000))?;
    }
    Ok(ids)
}

pub fn validate_and_prepare(output: &Path, plan: &Plan, inventory: &str) -> Result<Vec<String>> {
    let ids = validate_raw(output, plan, inventory)?;
    for (raw, canonical) in &ids {
        fs::rename(
            output.join(format!("{raw}.png")),
            output.join(format!("{canonical}.png")),
        )?;
    }
    let ids: Vec<_> = ids.into_iter().map(|(_, id)| id).collect();
    store::write_json(&output.join("inventory.json"), &ids)?;
    Ok(ids)
}

pub fn regular(path: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(path)?;
    ensure!(
        meta.file_type().is_file() && meta.len() > 0,
        "missing/empty/nonregular file: {}",
        path.display()
    );
    Ok(())
}
pub fn png(path: &Path, dimensions: (u32, u32)) -> Result<()> {
    regular(path)?;
    let image = image::open(path)?;
    ensure!(
        image.color() == image::ColorType::Rgba8,
        "PNG must decode as RGBA8 without precision loss"
    );
    ensure!(
        image.as_rgba8().unwrap().dimensions() == dimensions,
        "native PNG dimensions differ; never resample"
    );
    Ok(())
}
