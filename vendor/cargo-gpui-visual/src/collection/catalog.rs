//! Consumer-owned coverage and presentation, sealed as an ordinary source input.
use crate::store::image_path;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub schema: u32,
    /// Authored order, never inferred from emitted files or sorted case IDs.
    pub snapshots: Vec<Snapshot>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub base: String,
    pub label: String,
    pub group: String,
    pub sequence: Option<String>,
    pub moment: Option<String>,
    pub page: Option<Page>,
    pub state: State,
    pub reason: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Page {
    /// Stable identity for the complete page set, including its variant.
    pub set: String,
    pub number: u32,
    pub total: u32,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum State {
    Required,
    Withheld,
    Unsupported,
}
impl State {
    pub fn label(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Withheld => "withheld",
            Self::Unsupported => "unsupported",
        }
    }
}
impl Catalog {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.schema == 1, "Unsupported catalog schema");
        ensure!(!self.snapshots.is_empty(), "Catalog must not be empty");
        let mut bases = BTreeSet::new();
        let mut pages = BTreeMap::<_, Vec<&Page>>::new();
        for s in &self.snapshots {
            ensure!(bases.insert(&s.base), "Duplicate catalog base: {}", s.base);
            image_path(Path::new("."), &format!("{}--light", s.base))?;
            ensure!(
                !s.label.trim().is_empty() && !s.group.trim().is_empty(),
                "Catalog label/group required"
            );
            ensure!(
                s.sequence.is_some() == s.moment.is_some(),
                "Sequence and moment must be paired"
            );
            for value in [&s.sequence, &s.moment, &s.reason].into_iter().flatten() {
                ensure!(!value.trim().is_empty(), "Empty catalog metadata");
            }
            ensure!(
                s.state == State::Required || s.reason.is_some(),
                "Withheld/unsupported reason required"
            );
            if let Some(p) = &s.page {
                ensure!(
                    !p.set.trim().is_empty() && p.number > 0 && p.number <= p.total,
                    "Invalid catalog page range/set"
                );
                pages
                    .entry((&s.group, &s.sequence, &s.moment, &p.set))
                    .or_default()
                    .push(p);
            }
        }
        for members in pages.values() {
            let total = members[0].total;
            ensure!(
                members.len() == total as usize
                    && members
                        .iter()
                        .enumerate()
                        .all(|(i, p)| p.total == total && p.number as usize == i + 1),
                "Catalog page set must include every page exactly once in order"
            );
        }
        Ok(())
    }

    pub(crate) fn coverage(&self, emitted: &[String]) -> Vec<String> {
        let mut errors = Vec::new();
        let mut planned = BTreeSet::new();
        for s in &self.snapshots {
            if s.state == State::Withheld {
                errors.push(format!(
                    "Withheld: {} — {}",
                    s.base,
                    s.reason.as_deref().unwrap_or_default()
                ));
            }
            for theme in ["light", "dark"] {
                let case = format!("{}--{theme}", s.base);
                if s.state == State::Required && !emitted.contains(&case) {
                    errors.push(format!("Missing required capture: {case}"));
                } else if s.state != State::Required && emitted.contains(&case) {
                    errors.push(format!(
                        "Non-required capture: {case} ({})",
                        s.state.label()
                    ));
                }
                planned.insert(case);
            }
        }
        for case in emitted {
            if !planned.contains(case) {
                errors.push(format!("Unexpected capture: {case}"));
            }
        }
        errors
    }
}
