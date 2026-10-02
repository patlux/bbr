//! Independent still requirements, not a producer inventory or input receipt.
use anyhow::{ensure, Context, Result};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Endpoint {
    InitialUnchecked,
    CheckedAfterClick,
    UncheckedAfterSecondClick,
}
impl Endpoint {
    pub const ORDER: [Self; 3] = [
        Self::InitialUnchecked,
        Self::CheckedAfterClick,
        Self::UncheckedAfterSecondClick,
    ];
    pub fn id(self) -> &'static str {
        match self {
            Self::InitialUnchecked => "initial-unchecked",
            Self::CheckedAfterClick => "checked-after-click",
            Self::UncheckedAfterSecondClick => "unchecked-after-second-click",
        }
    }
}

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
    /// Schema-2 still identity only. Never a Gallery State selector or sequence.
    #[serde(default)]
    pub endpoint: Option<Endpoint>,
}

fn number(raw: &str) -> Result<u64> {
    let value = raw.parse::<u64>()?;
    ensure!(raw == format!("{value:02}"), "noncanonical Gallery number");
    Ok(value)
}
impl Case {
    fn static_base(&self) -> Result<&str> {
        match self.endpoint {
            None => Ok(&self.base),
            Some(endpoint) => self
                .base
                .strip_suffix(&format!("-endpoint-{}", endpoint.id()))
                .context("endpoint identity differs from typed endpoint"),
        }
    }
    pub(crate) fn story_number(&self) -> Result<u64> {
        let parts: Vec<_> = self.static_base()?.split('-').collect();
        ensure!(
            parts.len() == 6
                && parts[0] == "gallery"
                && parts[2] == "state"
                && ["standard", "increased"].contains(&parts[4])
                && ["small", "normal", "extralarge"].contains(&parts[5]),
            "invalid Gallery case identity"
        );
        number(parts[3])?;
        number(parts[1])
    }
}

impl Plan {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            [1, 2].contains(&self.schema) && !self.cases.is_empty(),
            "empty/invalid independent plan"
        );
        let mut bases = BTreeSet::new();
        let mut stories = BTreeMap::new();
        let mut previous = None;
        for case in &self.cases {
            ensure!(
                self.schema == 2 || case.endpoint.is_none(),
                "schema 1 forbids endpoints"
            );
            let story = case.story_number()?;
            ensure!(
                bases.insert(&case.base)
                    && [&case.label, &case.state, &case.group]
                        .iter()
                        .all(|value| !value.trim().is_empty()
                            && !value.chars().any(char::is_control)),
                "duplicate case/invalid Gallery metadata"
            );
            ensure!(
                previous == Some(story) || !stories.contains_key(&story),
                "noncontiguous Gallery story plan"
            );
            previous = Some(story);
            let metadata = (&case.label, &case.group);
            ensure!(
                *stories.entry(story).or_insert(metadata) == metadata,
                "story label/group differs within plan"
            );
            for ranges in [
                &case.ranges,
                case.dark_ranges.as_ref().unwrap_or(&case.ranges),
            ] {
                ensure!(
                    !ranges.is_empty() && ranges.len() == case.ranges.len(),
                    "incomplete appearance page pairing"
                );
                let mut next = 0;
                for [start, end] in ranges {
                    ensure!(
                        *start == next && start < end,
                        "nonconsecutive independent child ranges"
                    );
                    next = *end;
                }
            }
        }
        let mut index = 0;
        while let Some(rest) = self.cases.get(index) {
            ensure!(rest.endpoint.is_none(), "orphan/reordered endpoint case");
            index += 1;
            if !self.cases.get(index).is_some_and(|c| c.endpoint.is_some()) {
                continue;
            }
            ensure!(
                rest.base.starts_with("gallery-16-state-00-")
                    && rest.label == "Checkbox"
                    && rest.state == "default",
                "endpoints require Checkbox story 16 / State 00 / default"
            );
            for endpoint in Endpoint::ORDER {
                let case = self.cases.get(index).context("missing Checkbox endpoint")?;
                ensure!(
                    case.endpoint == Some(endpoint)
                        && case.base == format!("{}-endpoint-{}", rest.base, endpoint.id())
                        && case.label == rest.label
                        && case.state == rest.state
                        && case.group == rest.group
                        && case.ranges == rest.ranges
                        && case.dark_ranges == rest.dark_ranges,
                    "incomplete/reordered Checkbox endpoints or changed metadata/ranges"
                );
                index += 1;
            }
        }
        Ok(())
    }

    /// Planning is supported; native input and sealed semantic evidence are not.
    /// Keep this at every execution/publication boundary until both are wired.
    pub fn require_static_capture(&self) -> Result<()> {
        self.validate()?;
        ensure!(
            self.cases.iter().all(|case| case.endpoint.is_none()),
            "Gallery endpoint execution/evidence is not wired"
        );
        Ok(())
    }
}
