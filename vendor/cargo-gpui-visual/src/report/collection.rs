//! Collection presentation reuses the original document and every image control.
use super::{accept_note, escape, finding_section, summary, write_document, Document};
use crate::{
    collection::{
        catalog::{Snapshot, State},
        Collection, SuiteReview,
    },
    compare::Finding,
};
use anyhow::Result;
use std::path::Path;

pub(crate) fn write_collection(root: &Path, collection: &Collection) -> Result<()> {
    let mut d = Document {
        nav: "<nav>".into(),
        sections: "<h2 class=\"group-title\" id=\"abweichungen\">Änderungen</h2>".into(),
        gallery: String::new(),
        environment: String::new(),
        errors: String::new(),
        global: false,
        comparable: false,
        passed: collection.passed,
        partial: false,
        count: 0,
        layout_class: " collection",
        summary: summary(
            collection
                .suites
                .iter()
                .filter_map(|s| s.review.as_ref())
                .flat_map(|r| r.results.iter().map(|f| f.status.as_str())),
        ),
        environment_drift: collection
            .suites
            .iter()
            .filter_map(|s| s.review.as_ref())
            .any(|r| r.environment_mismatch.is_some()),
    };
    for s in &collection.suites {
        let id = &s.suite.id;
        let label = escape(&s.suite.label);
        d.nav += &format!("<p class=\"navhead\"><a href=\"#suite-{id}\">{label}</a></p>");
        d.sections += &format!(
            "<section class=\"suite\" id=\"suite-{id}\"><h2>{label}</h2><p>{} · Suite: <code>{id}</code></p>",
            if s.passed {
                "PASS"
            } else {
                "REVIEW REQUIRED / FAIL"
            }
        );
        for error in &s.errors {
            d.sections += &format!("<p>{}</p>", escape(error));
        }
        if let Some(r) = &s.review {
            d.count += r.results.len();
            d.global |= r.results.iter().any(|i| i.status == "changed");
            d.sections += &format!("<details><summary>Aufnahmeumgebung · {label}</summary><code>{}</code></details><p>Capture: <code>{}</code></p><p>Referenzen: <code>{}</code></p>",escape(&r.environment.to_string()),escape(&r.capture_hash),escape(&s.suite.baselines.display().to_string()));
            d.sections += &super::provenance_notice(r);
            for error in &r.errors {
                d.sections += &format!("<p>{}</p>", escape(error));
            }
            if let Some(mismatch) = &r.environment_mismatch {
                d.sections += &format!(
                    "<p>Aufnahmeumgebung weicht ab: {}. Keine automatische Übernahme.</p>",
                    escape(&mismatch.to_string())
                );
            }
        }
        d.sections += "</section>";
        d.gallery += &format!("<h2 id=\"gallery-{id}\">{label}</h2>");
        let mut previous_group = None;
        let mut previous_sequence = None;
        if let Some(catalog) = &s.catalog {
            for snapshot in &catalog.snapshots {
                if previous_group != Some(&snapshot.group) {
                    let group = escape(&snapshot.group);
                    d.nav += &format!("<p class=\"navhead\">{group}</p>");
                    d.gallery += &format!("<h3>{group}</h3>");
                    previous_group = Some(&snapshot.group);
                    previous_sequence = None;
                }
                if previous_sequence != snapshot.sequence.as_ref() {
                    if let Some(sequence) = &snapshot.sequence {
                        d.gallery += &format!("<h4>{}</h4>", escape(sequence));
                    }
                    previous_sequence = snapshot.sequence.as_ref();
                }
                snapshot_html(root, &mut d, s, snapshot);
            }
        }
        if let Some(r) = &s.review {
            for finding in &r.results {
                if !s.catalog.as_ref().is_some_and(|c| {
                    c.snapshots.iter().any(|snapshot| {
                        finding.case == format!("{}--light", snapshot.base)
                            || finding.case == format!("{}--dark", snapshot.base)
                    })
                }) {
                    let key = format!("{id}:{}", finding.case);
                    d.nav += &format!("<a class=\"toc-row compact finding\" href=\"#f-{key}\"><span class=\"dot {}\"></span><span class=\"toc-label\">Unexpected: {key}</span></a>", escape(&finding.status));
                    d.sections += &finding_section(
                        root,
                        finding,
                        &key,
                        &format!("suites/{id}/{}", finding.case),
                        &format!("Unexpected capture · {}", escape(&key)),
                        &collection_accept(),
                    );
                }
            }
        }
    }
    d.nav += "</nav>";
    d.comparable = d.global;
    write_document(root, d)
}
fn snapshot_html(root: &Path, d: &mut Document, suite: &SuiteReview, snapshot: &Snapshot) {
    let id = &suite.suite.id;
    let findings = suite
        .review
        .as_ref()
        .map(|r| r.results.as_slice())
        .unwrap_or_default();
    let paired: Vec<_> = ["light", "dark"]
        .into_iter()
        .map(|theme| {
            let case = format!("{}--{theme}", snapshot.base);
            let finding = findings.iter().find(|f| f.case == case);
            (theme, case, finding)
        })
        .collect();
    let quiet = snapshot.state == State::Required
        && paired
            .iter()
            .all(|(_, _, f)| f.is_some_and(|f| f.status == "same"));
    let title = caption(snapshot);
    d.gallery += &format!(
        "<article class=\"{}\"><h3>{title}</h3>",
        if quiet { "quiet" } else { "finding" }
    );
    if let Some(reason) = &snapshot.reason {
        d.gallery += &format!("<p>{}: {}</p>", snapshot.state.label(), escape(reason));
    }
    d.gallery += "<div class=\"images\">";
    for (theme, case, finding) in paired {
        let key = format!("{id}:{case}");
        let status = finding.map(|f| f.status.as_str()).unwrap_or_else(|| {
            if snapshot.state == State::Required {
                "missing-capture"
            } else {
                snapshot.state.label()
            }
        });
        let anchor = if finding.is_some_and(|f| f.status != "same") {
            "f"
        } else {
            "s"
        };
        d.nav += &format!("<a class=\"toc-row compact {}\" href=\"#{anchor}-{key}\"><span class=\"dot {status}\"></span><span class=\"toc-label\">{} · {theme} · {status}</span></a>",if status == "same" { "quiet" } else { "finding" },escape(&snapshot.label));
        d.gallery += &format!("<figure id=\"s-{key}\"><figcaption>{theme} <span class=\"badge {status}\">{status}</span></figcaption>");
        if let Some(f) = finding {
            let asset = format!("suites/{id}/{case}");
            d.gallery += &format!("<a href=\"{asset}/actual.png\"><img loading=\"lazy\" src=\"{asset}/actual.png\" alt=\"{key}\"></a>");
            if f.status != "same" {
                finding_html(root, d, f, &key, &asset, &title);
            }
        } else {
            d.gallery += &format!("<p>{status} · keine Aufnahme</p>");
        }
        d.gallery += "</figure>";
    }
    d.gallery += "</div></article>";
}
fn caption(s: &Snapshot) -> String {
    let mut title = escape(&s.label);
    if let Some(moment) = &s.moment {
        title += &format!(" · {}", escape(moment));
    }
    if let Some(page) = &s.page {
        title += &format!(" · Seite {} / {}", page.number, page.total);
    }
    title
}
fn finding_html(root: &Path, d: &mut Document, f: &Finding, key: &str, asset: &str, title: &str) {
    d.sections += &finding_section(root, f, key, asset, title, &collection_accept());
}
fn collection_accept() -> String {
    accept_note("Eine Sammlung wird nur mit <code>cargo gpui-visual accept-collection</code> und ausdrücklich gewählten Fällen übernommen.")
}
