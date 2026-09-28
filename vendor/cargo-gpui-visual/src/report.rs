//! Static report. The existing review controls are embedded, never fetched.
use crate::compare::{Finding, Review};
use anyhow::Result;
use base64::Engine;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};
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
fn hint([x, y, w, h]: [u32; 4]) -> String {
    format!("{w}×{h} px ab {x},{y}")
}
fn marks(item: &Finding) -> String {
    let Some([_, height]) = item.actual_size else {
        return String::new();
    };
    if item.rows.is_empty() {
        return String::new();
    }
    let mut s = String::from("<span class=\"marks\">");
    for &[top, bottom] in &item.rows {
        if let Some(b) = crop_box(item, Some([top, bottom])) {
            s+=&format!("<i style=\"top:{:.3}%;height:{:.3}%\" title=\"Abweichung y {top}–{bottom} · klicken für 1:1-Ausschnitt\" data-y=\"{}\" data-h=\"{}\" data-rows=\"y {top}–{bottom}\" data-hint=\"{}\"></i>",f64::from(top)/f64::from(height)*100.,f64::from(bottom-top)/f64::from(height)*100.,b[1],b[3],hint(b));
        }
    }
    s + "</span>"
}
fn radios(case: &str, global: bool) -> String {
    MODES.iter().map(|(key,label)| {
        if global {format!("<input type=\"radio\" name=\"mode-all\" id=\"all-{key}\" data-all-mode=\"{key}\"{}><label for=\"all-{key}\">{label}</label>",if *key=="2up" {" checked"}else{""})}
        else {format!("<input class=\"m-{key}\" data-mode=\"{key}\" type=\"radio\" name=\"mode-{case}\" id=\"{case}-{key}\"{}><label for=\"{case}-{key}\">{label}</label>",if *key=="2up" {" checked"}else{""})}
    }).collect()
}
fn views(item: &Finding) -> String {
    let c = &item.case;
    if item.status == "missing-baseline" {
        return format!("<div class=\"images\"><figure><figcaption>Alt</figcaption><div class=\"placeholder\"><img src=\"{c}/actual.png\" alt=\"\" aria-hidden=\"true\"><span>keine Referenz</span></div></figure><figure><figcaption>Neu</figcaption><a href=\"{c}/actual.png\"><img src=\"{c}/actual.png\" alt=\"Neu: {c}\"></a></figure></div>");
    }
    let mut switches = format!("<span class=\"seg\">{}</span>", radios(c, false));
    let frame = if let Some(b) = crop_box(item, None) {
        switches+=&format!("<input class=\"m-crop\" type=\"checkbox\" id=\"{c}-crop\"><label class=\"toggle\" for=\"{c}-crop\">Ausschnitt 1:1</label><p class=\"hint crop-hint\">1:1-Ausschnitt um die Änderung ({})</p>",hint(b));
        format!(
            "class=\"frame\" style=\"--x:-{}px;--y:-{}px;--w:{}px;--h:{}px\"",
            b[0], b[1], b[2], b[3]
        )
    } else {
        "class=\"frame\"".into()
    };
    let marks = marks(item);
    format!(
        r#"<div class="viewer">{switches}<div class="views">
<div class="view v-2up images">
<figure><figcaption>Alt</figcaption><a href="{c}/expected.png"><span {frame}><img src="{c}/expected.png" alt="Alt: {c}">{marks}</span></a></figure>
<figure><figcaption>Neu</figcaption><a href="{c}/actual.png"><span {frame}><img src="{c}/actual.png" alt="Neu: {c}">{marks}</span></a></figure></div>
<div class="view v-swipe"><p class="hint">Trennlinie ziehen · links Alt, rechts Neu</p><div {frame}><div class="stage"><img src="{c}/actual.png" alt="Neu: {c}"><div class="overlay"><img src="{c}/expected.png" alt="Alt: {c}"></div><div class="handle" role="slider" tabindex="0" aria-label="Trennlinie Alt / Neu" aria-valuemin="0" aria-valuemax="100" aria-valuenow="50"></div></div>{marks}</div></div>
<div class="view v-onion"><div class="blendbar"><span>Alt</span><input class="blend" type="range" min="0" max="100" value="50" aria-label="Überblendung Alt zu Neu"><span>Neu</span></div><div {frame}><div class="stage"><img src="{c}/expected.png" alt="Alt: {c}"><div class="overlay"><img src="{c}/actual.png" alt="Neu: {c}"></div></div>{marks}</div></div>
<div class="view v-diff images"><figure><figcaption>Jedes abweichende Pixel</figcaption><a href="{c}/diff.png"><span {frame}><img src="{c}/diff.png" alt="Diff: {c}">{marks}</span></a></figure></div>
</div></div>"#
    )
}
pub fn write(root: &Path, r: &Review) -> Result<()> {
    let mut nav = String::from("<nav>");
    for (same, label, anchor) in [(false, "Abweichungen", "f"), (true, "Unverändert", "s")] {
        nav += &format!("<p class=\"navhead\">{label}</p>");
        for i in r.results.iter().filter(|i| (i.status == "same") == same) {
            nav += &format!(
                "<a href=\"#{anchor}-{}\"><span class=\"dot {}\"></span>{}</a>",
                i.case, i.status, i.case
            );
        }
    }
    nav += "</nav>";
    let mut sections = String::new();
    for i in r.results.iter().filter(|i| i.status != "same") {
        sections += &format!(
            "<section id=\"f-{}\"><h2>{}</h2><p>{} · Pixel: {} · Bounds: {}</p>{}</section>",
            i.case,
            i.case,
            i.status,
            i.changed_pixels
                .map_or_else(|| "–".into(), |v| v.to_string()),
            i.bounds.map_or_else(
                || "–".into(),
                |v| format!("{},{}–{},{}", v[0], v[1], v[2], v[3])
            ),
            views(i)
        );
    }
    let mut groups: BTreeMap<&str, Vec<&Finding>> = BTreeMap::new();
    for i in &r.results {
        groups
            .entry(i.case.rsplit_once("--").unwrap().0)
            .or_default()
            .push(i);
    }
    let mut gallery = String::new();
    for (base, items) in groups {
        let class = if items.iter().any(|i| i.status != "same") {
            "finding"
        } else {
            "quiet"
        };
        gallery += &format!("<article class=\"{class}\"><h3>{base}</h3><div class=\"images\">");
        for theme in ["light", "dark"] {
            if let Some(i) = items.iter().find(|i| i.case.ends_with(theme)) {
                let c = &i.case;
                let status = &i.status;
                gallery+=&format!("<figure id=\"s-{c}\"><figcaption>{theme} <span class=\"badge {status}\">{status}</span></figcaption><a href=\"{c}/actual.png\"><img loading=\"lazy\" src=\"{c}/actual.png\" alt=\"{c}\"></a></figure>");
            }
        }
        gallery += "</div></article>";
    }
    let comparable = r.results.iter().any(|i| i.status == "changed");
    let global = if comparable {
        format!("<span class=\"group global-mode\" hidden><span class=\"glabel\">Ansicht für alle</span><span class=\"seg modes\">{}</span><input type=\"checkbox\" id=\"crop-all\"><label class=\"toggle\" for=\"crop-all\">Ausschnitt 1:1</label><span class=\"mixed\">gemischt</span></span>",radios("",true))
    } else {
        String::new()
    };
    let heights=[("small","klein"),("medium","mittel"),("fit","Viewport"),("tall","groß"),("full","voll")].iter().map(|(id,label)|format!("<input type=\"radio\" name=\"shot-height\" id=\"h-{id}\"{}><label for=\"h-{id}\">{label}</label>",if *id=="fit" {" checked"}else{""})).collect::<String>();
    let mut errors = r
        .errors
        .iter()
        .map(|e| format!("<li>{}</li>", escape(e)))
        .collect::<String>();
    if let Some(mismatch) = &r.environment_mismatch {
        errors+=&format!("<li>Aufnahmeumgebung weicht ab: {}. Nur explizit mit accept --adopt-environment übernehmen.</li>",escape(&mismatch.to_string()));
    }
    let status = if r.passed {
        "PASS"
    } else {
        "REVIEW REQUIRED / FAIL"
    };
    let document = format!(
        r#"<!doctype html><html lang="de"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src 'self' data:; style-src 'unsafe-inline'; script-src {}">
<title>GPUI Mobile Kit · Visual Review</title><style>{CSS}</style><div class="layout">{nav}<main>
<h1>GPUI Visual · {status}</h1><p>{} Bilder · Änderungen werden niemals automatisch übernommen.</p>
<p>Magenta markiert jedes abweichende RGBA-Pixel. Bilder anklicken für Originalgröße.</p>
<div class="controls"><span class="group"><span class="glabel">Höhe</span><span class="seg">{heights}</span></span>{global}<span class="group"><input type="checkbox" id="only-findings"><label class="toggle" for="only-findings">nur Abweichungen</label></span></div>
<details><summary>Aufnahmeumgebung</summary><code>{}</code></details><ul>{errors}</ul><h2>Änderungen</h2>{sections}
<h1 id="alle">Alle Snapshots</h1><p>Jeder aufgenommene Zustand, Hell und Dunkel nebeneinander.</p>{gallery}</main></div><script>{SCRIPT}</script></html>"#,
        script_hash(),
        r.results.len(),
        escape(&r.environment.to_string())
    );
    fs::write(root.join("index.html"), document)?;
    Ok(())
}
