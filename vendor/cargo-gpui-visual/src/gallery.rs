//! Consumer-defined grouping; no app routes or business labels belong in the CLI.
use crate::{
    compare::Finding,
    report::{escape, split_case, status_label},
};
use anyhow::{ensure, Result};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Options {
    #[serde(default)]
    pub groups: Vec<Group>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub title: String,
    pub prefixes: Vec<String>,
}
impl Options {
    pub fn validate(&self) -> Result<()> {
        for group in &self.groups {
            ensure!(
                !group.title.trim().is_empty()
                    && !group.prefixes.is_empty()
                    && group.prefixes.iter().all(|p| !p.is_empty()),
                "Gallery groups require a title and nonempty prefixes"
            );
        }
        Ok(())
    }
    fn title<'a>(&'a self, case: &'a str) -> &'a str {
        self.groups
            .iter()
            .find(|g| g.prefixes.iter().any(|p| case.starts_with(p)))
            .map(|g| g.title.as_str())
            .unwrap_or_else(|| case.split_once('-').map_or("Weitere", |(prefix, _)| prefix))
    }
}

pub(crate) fn html(stills: &[&Finding], options: &Options) -> String {
    let mut groups: BTreeMap<&str, BTreeMap<&str, Vec<&Finding>>> = BTreeMap::new();
    for item in stills {
        groups
            .entry(options.title(&item.case))
            .or_default()
            .entry(item.case.rsplit_once("--").unwrap().0)
            .or_default()
            .push(item);
    }
    let mut html = String::new();
    for (title, entries) in groups {
        let quiet = entries.values().flatten().all(|i| i.status == "same");
        html += &format!(
            "<section class=\"gallery-group{}\"><h2>{} <span class=\"count\">{}</span></h2><div class=\"tiles\">",
            if quiet { " quiet" } else { "" },
            escape(title),
            entries.len()
        );
        for (base, items) in entries {
            let class = if items.iter().all(|i| i.status == "same") {
                "quiet"
            } else {
                "finding"
            };
            // Each pair stays in the page flow; its comparison opens in place.
            let mut shots = String::new();
            let mut figures = String::new();
            for theme in ["light", "dark"] {
                if let Some(item) = items.iter().find(|i| i.case.ends_with(theme)) {
                    let c = &item.case;
                    let (_, label) = split_case(c);
                    shots += &format!("<span class=\"shot\" id=\"s-{c}\"><img loading=\"lazy\" src=\"{c}/actual.png\" alt=\"{label}: {c}\"><span class=\"dot {}\" title=\"{}\"></span></span>", item.status, status_label(&item.status));
                    let content = if item.status == "same" {
                        crate::report::views(item, c, c, &[])
                    } else {
                        format!("<a href=\"{c}/actual.png\"><img loading=\"lazy\" src=\"{c}/actual.png\" alt=\"{c}\"></a><p><a class=\"comparison-link\" href=\"#f-{c}\">Alt/Neu-Vergleich öffnen</a></p>")
                    };
                    figures+=&format!("<figure><figcaption>{label} <span class=\"badge {}\">{}</span></figcaption>{content}</figure>",item.status,status_label(&item.status));
                }
            }
            html += &format!("<article class=\"tile {class}\"><details class=\"pair\"><summary><span class=\"shots\">{shots}</span><code class=\"tile-name\">{base}</code></summary><div class=\"images\">{figures}</div></details></article>");
        }
        html += "</div></section>";
    }
    html
}
