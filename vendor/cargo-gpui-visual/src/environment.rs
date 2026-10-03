//! Explicit rendering/provenance partition. Legacy JSON keeps exact equality.
//! The partition is reversible: provenance contains only removed object leaves,
//! addressed by canonical JSON pointers. Classification is itself a comparison key.
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Partition {
    layer: String,
    comparison_schema: u32,
    contract: Value,
    rendering: Value,
    provenance: BTreeMap<String, Value>,
}

fn parts(pointer: &str) -> Result<(String, String)> {
    ensure!(
        pointer.starts_with('/'),
        "Provenance requires a JSON pointer"
    );
    let (parent, leaf) = pointer.rsplit_once('/').unwrap();
    ensure!(!leaf.is_empty(), "Empty provenance leaf");
    // Require canonical RFC 6901 spelling, never lossy unescaping.
    for token in pointer[1..].split('/') {
        let decoded = token.replace("~1", "/").replace("~0", "~");
        ensure!(
            decoded.replace('~', "~0").replace('/', "~1") == token,
            "Invalid provenance pointer escape"
        );
    }
    ensure!(
        pointer != "/layer",
        "Capture layer cannot become provenance"
    );
    Ok((parent.into(), leaf.replace("~1", "/").replace("~0", "~")))
}
fn parent<'a>(
    value: &'a mut Value,
    pointer: &str,
) -> Result<(&'a mut serde_json::Map<String, Value>, String)> {
    let (path, leaf) = parts(pointer)?;
    let object = value
        .pointer_mut(&path)
        .and_then(Value::as_object_mut)
        .context("Provenance parent must exist and be an object")?;
    Ok((object, leaf))
}
impl Partition {
    fn reconstruct(&self) -> Result<Value> {
        ensure!(
            self.comparison_schema == 1,
            "Unsupported environment comparison schema"
        );
        ensure!(
            matches!(self.layer.as_str(), "gpui" | "ios" | "android"),
            "Explicit capture layer required"
        );
        ensure!(
            self.contract.as_object().is_some_and(|o| !o.is_empty()),
            "Nonempty rendering contract required"
        );
        ensure!(
            self.rendering.as_object().is_some_and(|o| !o.is_empty())
                && self.rendering["layer"] == self.layer,
            "Rendering layer differs"
        );
        ensure!(
            !self.provenance.is_empty(),
            "Explicit provenance leaves required"
        );
        let mut observation = self.rendering.clone();
        for (pointer, value) in &self.provenance {
            ensure!(
                value.is_string() || value.is_number() || value.is_boolean(),
                "Provenance must be a scalar observation, not a hidden subtree"
            );
            let (object, leaf) = parent(&mut observation, pointer)?;
            ensure!(!object.contains_key(&leaf), "Rendering/provenance overlap");
            object.insert(leaf, value.clone());
        }
        Ok(observation)
    }
}
fn parse(value: &Value) -> Result<Option<Partition>> {
    if value.get("comparison_schema").is_none() {
        return Ok(None);
    }
    let partition: Partition = serde_json::from_value(value.clone())?;
    partition.reconstruct()?;
    Ok(Some(partition))
}
/// Reject malformed opt-in envelopes. Unmarked legacy environments are unchanged.
pub fn validate(value: &Value) -> Result<()> {
    parse(value).map(|_| ())
}
/// Losslessly split exact observed scalar leaves. No automatic field classification.
pub fn partition(observation: &Value, contract: Value, pointers: &[String]) -> Result<Value> {
    ensure!(
        observation.get("comparison_schema").is_none(),
        "Environment already partitioned"
    );
    let mut rendering = observation.clone();
    let mut provenance = BTreeMap::new();
    for pointer in pointers {
        let (object, leaf) = parent(&mut rendering, pointer)?;
        let value = object
            .remove(&leaf)
            .context("Missing or duplicate provenance leaf")?;
        provenance.insert(pointer.clone(), value);
    }
    let split = Partition {
        layer: observation["layer"]
            .as_str()
            .context("Explicit capture layer required")?
            .into(),
        comparison_schema: 1,
        contract,
        rendering,
        provenance,
    };
    ensure!(
        split.reconstruct()? == *observation,
        "Environment partition is not reversible"
    );
    Ok(serde_json::to_value(split)?)
}
/// Exact original observation; useful for validating an explicit metadata migration.
pub fn observation(value: &Value) -> Result<Value> {
    parse(value)?
        .context("Partitioned environment required")?
        .reconstruct()
}
/// Only two valid, equally classified contracts compare their rendering portion.
/// A legacy/partitioned pair remains a mismatch until explicit metadata migration.
pub fn equivalent(a: &Value, b: &Value) -> Result<bool> {
    match (parse(a)?, parse(b)?) {
        (Some(a), Some(b)) => Ok(a.layer == b.layer
            && a.contract == b.contract
            && a.rendering == b.rendering
            && a.provenance.keys().eq(b.provenance.keys())),
        (None, None) => Ok(a == b),
        _ => Ok(false),
    }
}
/// Full provenance values remain sealed and visible even when rendering matches.
pub fn provenance_difference(a: &Value, b: &Value) -> Result<Option<Value>> {
    match (parse(a)?, parse(b)?) {
        (Some(a), Some(b)) if a.provenance != b.provenance => Ok(Some(
            json!({"baselines":a.provenance,"capture":b.provenance}),
        )),
        _ => Ok(None),
    }
}
