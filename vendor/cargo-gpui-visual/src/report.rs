//! Static report. The existing review controls are embedded, never fetched.
mod case;
mod collection;
mod detail;
mod diagnostics;
use crate::compare::{Finding, Review};
pub use crate::gallery::{Group, Options};
use anyhow::Result;
use base64::Engine;
pub(crate) use case::{
    accept_block, accept_note, badge, finding_section, nav_detail, nav_row, navhead, split_case,
    AcceptPaths,
};
pub(crate) use collection::write_collection;
pub(crate) use detail::Detail;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn status_label(status: &str) -> &str {
    match status {
        "same" => "unverändert",
        "changed" => "geändert",
        "missing-baseline" => "neu · keine Referenz",
        _ => status,
    }
}

/// Serializes report replacement and acceptance. A killed process leaves a stale
/// lock requiring explicit recovery, rather than a potentially mixed review.
pub(crate) struct ReportLock(PathBuf);
impl ReportLock {
    pub(crate) fn acquire(root: &Path) -> Result<Self> {
        let path = root.join(".review.lock");
        crate::store::no_symlinks(&path)?;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        Ok(Self(path))
    }
}
impl Drop for ReportLock {
    fn drop(&mut self) {
        // Failure keeps the report locked, which is the safe recovery state.
        let _ = fs::remove_file(&self.0);
    }
}
pub const SCRIPT: &str = include_str!("../assets/review.js");
const CSS: &str = include_str!("../assets/review.css");
const MODES: [(&str, &str); 4] = [
    ("2up", "2-up"),
    ("swipe", "Wischen"),
    ("onion", "Überblenden"),
    ("diff", "Diff"),
];
pub fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
pub fn script_hash() -> String {
    format!(
        "'sha256-{}'",
        base64::engine::general_purpose::STANDARD.encode(Sha256::digest(SCRIPT.as_bytes()))
    )
}
pub fn crop_box(item: &Finding, rows: Option<[u32; 2]>) -> Option<[u32; 4]> {
    let [x1, y1, x2, y2] = item.bounds?;
    let [w, h] = item.actual_size?;
    let [top, bottom] = rows.unwrap_or([y1, y2]);
    let x = x1.saturating_sub(40);
    let y = top.saturating_sub(40);
    Some([
        x,
        y,
        (w - x).min(x2 - x1 + 80).min(700),
        (h - y).min(bottom - top + 80).min(520),
    ])
}
pub(crate) fn hint([x, y, w, h]: [u32; 4]) -> String {
    format!("{w}×{h} px ab {x},{y}")
}
/// Numbered boxes use exactly the same windows as the inspector and stage.
pub(crate) fn marks(item: &Finding, details: &[Detail]) -> String {
    let Some([width, height]) = item.actual_size else {
        return String::new();
    };
    if details.is_empty() {
        return String::new();
    }
    let mut html = String::from("<span class=\"marks\">");
    for (index, detail) in details.iter().enumerate() {
        let [x, y, w, h] = detail.crop;
        let n = index + 1;
        let left = f64::from(x) / f64::from(width) * 100.;
        let top = f64::from(y) / f64::from(height) * 100.;
        let span = f64::from(w) / f64::from(width) * 100.;
        let high = f64::from(h) / f64::from(height) * 100.;
        html += &format!("<i style=\"top:{top:.3}%;height:{high:.3}%;left:{left:.3}%;width:{span:.3}%\" title=\"Detailausschnitt {n} · {}\" data-x=\"{x}\" data-w=\"{w}\" data-y=\"{y}\" data-h=\"{h}\" data-rows=\"Detail {n}\" data-hint=\"{}\"></i><b style=\"top:{:.3}%;left:{left:.3}%;margin-left:-22px\" aria-hidden=\"true\">{n}</b>", hint(detail.crop), hint(detail.crop), top + high/2.);
    }
    html + "</span>"
}
pub(crate) fn radios(case: &str, global: bool, comparable: bool) -> String {
    MODES.iter().map(|(key,label)| {
        let state = if *key == "2up" { " checked" } else if !comparable { " disabled" } else { "" };
        if global {format!("<input type=\"radio\" name=\"mode-all\" id=\"all-{key}\" data-all-mode=\"{key}\"{state}><label for=\"all-{key}\">{label}</label>")}
        else {format!("<input class=\"m-{key}\" data-mode=\"{key}\" type=\"radio\" name=\"mode-{case}\" id=\"{case}-{key}\"{state}><label for=\"{case}-{key}\">{label}</label>")}
    }).collect()
}
/// `c` namespaces DOM identities; `asset` is the report-relative image folder.
pub(crate) fn views(item: &Finding, c: &str, asset: &str, details: &[Detail]) -> String {
    let expected = if item.status == "missing-baseline" {
        String::new()
    } else {
        format!("{asset}/expected.png")
    };
    let diff = if expected.is_empty() {
        String::new()
    } else {
        format!("{asset}/diff.png")
    };
    crate::comparison_view::html(
        c,
        crate::comparison_view::Images {
            expected: &expected,
            actual: &format!("{asset}/actual.png"),
            diff: &diff,
        },
        Some((item, details)),
    )
}
pub fn write(root: &Path, r: &Review) -> Result<()> {
    write_with_options(root, r, &Options::default())
}
pub fn write_with_options(root: &Path, r: &Review, options: &Options) -> Result<()> {
    write_review(root, r, options, None)
}

pub(crate) fn provenance_notice(r: &Review) -> String {
    diagnostics::provenance(r)
}

/// Counts of reviewable units (stills and whole sequences) for the top bar.
pub(crate) fn summary<'a>(statuses: impl Iterator<Item = &'a str>) -> String {
    let (mut changed, mut new, mut same) = (0, 0, 0);
    for status in statuses {
        match status {
            "same" => same += 1,
            "missing-baseline" => new += 1,
            _ => changed += 1,
        }
    }
    format!(
        "<strong>{changed}</strong> abweichend · <strong>{new}</strong> ohne Referenz · <strong>{same}</strong> unverändert"
    )
}

/// Single-suite report. `paths` enables exact per-case acceptance hints.
pub(crate) fn write_review(
    root: &Path,
    r: &Review,
    options: &Options,
    paths: Option<AcceptPaths<'_>>,
) -> Result<()> {
    options.validate()?;
    let accept = |id: &str| accept_block(r, paths, id);
    let stills: Vec<_> = r
        .results
        .iter()
        .filter(|i| !i.case.starts_with("sequence-"))
        .collect();
    let findings: Vec<_> = stills.iter().filter(|i| i.status != "same").collect();
    let unchanged: Vec<_> = stills.iter().filter(|i| i.status == "same").collect();

    let mut nav = String::from("<nav>");
    nav += &navhead("Abweichungen", findings.len());
    if findings.is_empty() {
        nav += "<p class=gap>keine</p>";
    }
    for i in &findings {
        nav += &nav_row(
            &format!("f-{}", i.case),
            &i.case,
            &i.status,
            &nav_detail(i),
            Some(&format!("{}/actual.png", i.case)),
        );
    }
    if !r.sequences.is_empty() {
        nav += &navhead("Sequenzen", r.sequences.len());
        for sequence in &r.sequences {
            let status = sequence.status();
            let detail = match (status, sequence.first_difference) {
                ("changed", Some(tick)) => format!("ab Takt {tick} abweichend"),
                _ => status_label(status).into(),
            };
            let thumb = sequence
                .actual
                .ticks
                .first()
                .map(|t| format!("{}/actual.png", t.image));
            nav += &nav_row(
                &sequence.actual.id,
                &sequence.actual.id,
                status,
                &detail,
                thumb.as_deref(),
            );
        }
    }
    nav += &navhead("Unverändert", unchanged.len());
    if unchanged.is_empty() {
        nav += "<p class=gap>keine</p>";
    }
    for i in &unchanged {
        nav += &nav_row(&format!("s-{}", i.case), &i.case, &i.status, "", None);
    }
    nav += "</nav>";

    let mut sections = format!(
        "<h2 class=\"group-title\" id=\"abweichungen\">Abweichungen <span class=\"count\">{}</span></h2>",
        findings.len()
    );
    for i in &findings {
        let (base, _) = split_case(&i.case);
        sections += &finding_section(root, i, &i.case, &i.case, &escape(base), &accept(&i.case));
    }
    if findings.is_empty() {
        sections.push_str("<p class=\"empty\">Keine visuellen Änderungen an Einzelzuständen.</p>");
    }
    if !r.sequences.is_empty() {
        sections += &format!(
            "<h2 class=\"group-title\" id=\"sequenzen\">Sequenzen <span class=\"count\">{}</span></h2>",
            r.sequences.len()
        );
        sections += &crate::sequence_report::html(r, &accept);
    }
    let gallery = crate::gallery::html(&stills, options);
    let mut errors = r
        .errors
        .iter()
        .map(|e| format!("<li>{}</li>", escape(e)))
        .collect::<String>();
    if r.environment_mismatch.is_some() {
        let advice = if r.selection.is_some() {
            "Übernahme gesperrt; vollständigen Lauf aufnehmen und prüfen."
        } else {
            "Nur explizit mit accept --adopt-environment übernehmen."
        };
        errors += &format!("<li>Aufnahmeumgebung weicht ab. {advice}</li>");
    }
    let mut environment = diagnostics::environment(r);
    environment += &format!("<details class=\"technical-details\" id=\"umgebung\"><summary>Technische Details · Aufnahmeumgebung</summary>{}</details>", diagnostics::evidence(r));
    environment += &provenance_notice(r);
    if let Some(selection) = &r.selection {
        environment.insert_str(0, &format!("<p class=selection-scope><strong>TEILLAUF · nur ausgewählte Stories: {}</strong><br>Kein vollständiger Suite-Nachweis; Baseline-Übernahme gesperrt. {}</p>", escape(&selection.ids()), if r.passed { "Auswahl stimmt exakt überein." } else { "Auswahl benötigt Prüfung." }));
    }
    let statuses = stills
        .iter()
        .map(|i| i.status.as_str())
        .chain(r.sequences.iter().map(|s| s.status()));
    write_document(
        root,
        Document {
            nav,
            sections,
            gallery,
            environment,
            errors,
            global: !stills.is_empty() || !r.sequences.is_empty(),
            comparable: stills.iter().any(|i| i.status != "missing-baseline")
                || r.sequences.iter().any(|s| s.expected.is_some()),
            passed: r.passed,
            partial: r.selection.is_some(),
            count: r.results.len(),
            layout_class: "",
            summary: summary(statuses),
            environment_drift: r.environment_mismatch.is_some(),
        },
    )
}

struct Document {
    nav: String,
    /// Complete findings area, including its heading(s).
    sections: String,
    gallery: String,
    environment: String,
    errors: String,
    /// Offer the shared all-viewer controls.
    global: bool,
    /// At least one viewer can compare two sides; otherwise only 2-up is enabled.
    comparable: bool,
    passed: bool,
    partial: bool,
    count: usize,
    layout_class: &'static str,
    /// Pre-rendered unit counts for the top bar.
    summary: String,
    /// Any reviewed environment differs from its references.
    environment_drift: bool,
}
const HELP: &str = "<details class=\"help\"><summary aria-label=\"Tastenkürzel\">?</summary><div class=\"help-panel\"><p><kbd>J</kbd> <kbd>K</kbd> nächster / voriger Eintrag</p><p><kbd>1</kbd>–<kbd>4</kbd> Ansicht für alle</p><p><kbd>Z</kbd> Ausschnitt 1:1 für alle</p><p><kbd>R</kbd> nächster Bereich im Inspektor</p><p><kbd>C</kbd> Übernahme-Befehl kopieren</p><p><kbd>/</kbd> Inhalt filtern</p></div></details>";
fn write_document(root: &Path, d: Document) -> Result<()> {
    let Document {
        nav,
        sections,
        gallery,
        environment,
        errors,
        global,
        comparable,
        passed,
        partial,
        count,
        layout_class,
        summary,
        environment_drift,
    } = d;
    let global = if global {
        format!("<span class=\"group global-mode\" hidden><span class=\"glabel\">Ansicht für alle</span><span class=\"seg modes\">{}</span><input type=\"checkbox\" id=\"crop-all\"><label class=\"toggle\" for=\"crop-all\">1:1</label><span class=\"mixed\">gemischt</span></span>",radios("",true,comparable))
    } else {
        String::new()
    };
    let heights=[("small","klein"),("medium","mittel"),("fit","Einpassen"),("tall","groß"),("full","voll")].iter().map(|(id,label)|format!("<input type=\"radio\" name=\"shot-height\" id=\"h-{id}\"{}><label for=\"h-{id}\">{label}</label>",if *id=="fit" {" checked"}else{""})).collect::<String>();
    let status = match (partial, passed) {
        (true, true) => "TEILLAUF · AUSWAHL PASS",
        (true, false) => "TEILLAUF · REVIEW REQUIRED / FAIL",
        (false, true) => "PASS",
        (false, false) => "REVIEW REQUIRED / FAIL",
    };
    let verdict = if passed { "pass" } else { "fail" };
    let drift = if environment_drift {
        "<a class=\"env-drift\" href=\"#hinweise\">Umgebung weicht ab</a>"
    } else {
        ""
    };
    let errors = if errors.is_empty() {
        String::new()
    } else {
        format!("<ul class=\"errors\">{errors}</ul>")
    };
    let document = format!(
        r#"<!doctype html><html lang="de"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src 'self' data:; style-src 'unsafe-inline'; script-src {}">
<title>GPUI Visual · {status}</title><style>{CSS}</style>
<header class="topbar"><span class="brand"><svg width="18" height="18" viewBox="0 0 18 18" fill="none" aria-hidden="true"><rect x="1.5" y="1.5" width="9" height="12" rx="1.5" stroke="currentColor" stroke-width="1.4"/><rect x="7.5" y="4.5" width="9" height="12" rx="1.5" class="brand-front" stroke-width="1.4"/></svg>GPUI Visual</span><h1 class="verdict {verdict}"><span class="dot"></span>{status}</h1><span class="summary">{summary}</span><span class="spacer"></span>{drift}<span class="step" hidden><button type="button" data-step="-1" aria-label="Voriger Eintrag">↑ <kbd>K</kbd></button><button type="button" data-step="1" aria-label="Nächster Eintrag">↓ <kbd>J</kbd></button></span>{HELP}</header>
<div class="layout{layout_class}"><aside class="toc"><div class="toc-tools"><input type="search" id="toc-filter" placeholder="Inhalt filtern" aria-label="Inhalt filtern" hidden></div>{nav}</aside><main>
<div class="controls">{global}<span class="group heights" title="Bildhöhe"><span class="glabel">Höhe</span><span class="seg">{heights}</span></span><span class="group"><input type="checkbox" id="show-regions" checked><label class="toggle" for="show-regions">Bereiche</label></span><span class="group"><input type="checkbox" id="only-findings"><label class="toggle" for="only-findings">nur Abweichungen</label></span></div>
<div class="notices" id="hinweise"><p>{count} Bilder · Änderungen werden niemals automatisch übernommen. Magenta markiert jedes abweichende RGBA-Pixel; Bilder anklicken für Originalgröße.</p>{environment}{errors}</div>{sections}
<h1 id="alle">Alle Snapshots</h1><p class="gallery-intro">Jeder aufgenommene Zustand, Hell und Dunkel nebeneinander. Ein Paar öffnet sich an Ort und Stelle.</p>{gallery}<p class="report-end">Ende des Berichts · nichts ausgeblendet</p></main></div><script>{SCRIPT}</script></html>"#,
        script_hash(),
    );
    fs::write(root.join("index.html"), document)?;
    Ok(())
}
