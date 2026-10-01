//! Bounded, human-readable output. Review JSON retains every frame and diagnostic.
use crate::compare::Review;
use std::{collections::BTreeSet, path::Path};
const LISTED: usize = 12;

pub struct Paths<'a> {
    pub repo: &'a Path,
    pub run: &'a Path,
    pub baselines: &'a Path,
    pub report: &'a Path,
}
pub(crate) fn quote(path: &str) -> String {
    format!("'{}'", path.replace('\'', "'\\''"))
}
fn short(text: &str) -> String {
    let value = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if value.chars().count() <= 160 {
        value
    } else {
        value.chars().take(159).chain(['…']).collect()
    }
}
fn uri(path: &Path) -> String {
    let mut out = String::from("file://");
    for b in path.to_string_lossy().bytes() {
        if b.is_ascii_alphanumeric() || b"/-._~".contains(&b) {
            out.push(b as char);
        } else {
            out += &format!("%{b:02X}");
        }
    }
    out
}
pub fn summary(review: &Review, paths: Paths<'_>, color: bool) -> String {
    let paint = |text: &str, code: &str| {
        if color {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_owned()
        }
    };
    let mut units: Vec<_> = review
        .results
        .iter()
        .filter(|i| !i.case.starts_with("sequence-"))
        .map(|i| {
            (
                i.case.as_str(),
                i.status.as_str(),
                i.changed_pixels
                    .map(|p| format!("{p} Pixel"))
                    .unwrap_or_else(|| {
                        if i.reason.as_deref() == Some("dimensions") {
                            "Bildgröße geändert".into()
                        } else {
                            "keine Referenz".into()
                        }
                    }),
            )
        })
        .collect();
    units.extend(review.sequences.iter().map(|s| {
        (
            s.actual.id.as_str(),
            s.status(),
            s.first_difference
                .map(|t| format!("erste Abweichung: Takt {t}"))
                .unwrap_or_else(|| "keine Referenz".into()),
        )
    }));
    let mut findings: Vec<_> = units.iter().filter(|u| u.1 != "same").collect();
    findings.sort_by_key(|u| (if u.1 == "changed" { 0 } else { 1 }, u.0));
    let mut lines = Vec::new();
    if let Some(selection) = &review.selection {
        lines.push(format!(
            "  TEILLAUF    {} · kein vollständiger Suite-Nachweis; Übernahme gesperrt",
            short(&selection.ids())
        ));
    }
    if !review.errors.is_empty() {
        lines.push(format!(
            "{} Vergleich   ungültige Eingabe · nichts übernommen",
            paint("✗", "1;31")
        ));
        lines.extend(
            review
                .errors
                .iter()
                .take(LISTED)
                .map(|e| format!("    {}", short(e))),
        );
        if review.errors.len() > LISTED {
            lines.push(format!(
                "    … {} weitere Fehler im Bericht",
                review.errors.len() - LISTED
            ));
        }
    } else if review.passed {
        lines.push(format!(
            "{} Vergleich   alle {} Prüffälle stimmen mit den Referenzen überein",
            paint("✓", "1;32"),
            units.len()
        ));
    } else {
        let changed = findings.iter().filter(|u| u.1 == "changed").count();
        let missing = findings.len() - changed;
        lines.push(format!("{} Vergleich   {changed} geändert, {missing} ohne Referenz von {} · Referenzen bleiben unverändert",paint("✗","1;31"),units.len()));
    }
    if !review.sequences.is_empty() {
        lines.push(format!(
            "  Umfang      {} Bilder · {} Sequenzen (keine native FPS-Messung)",
            review.results.len(),
            review.sequences.len()
        ));
    }
    if let Some(m) = &review.environment_mismatch {
        let keys: BTreeSet<_> = ["baselines", "capture"]
            .iter()
            .filter_map(|k| m[k].as_object())
            .flat_map(|o| o.keys())
            .filter(|k| m["baselines"][*k] != m["capture"][*k])
            .collect();
        lines.push(format!(
            "  {} Umgebung    weicht ab: {}",
            paint("!", "1;33"),
            keys.into_iter().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    if review.errors.is_empty() {
        let width = findings
            .iter()
            .take(LISTED)
            .map(|u| u.0.len())
            .max()
            .unwrap_or(0);
        for (id, _, detail) in findings.iter().take(LISTED) {
            lines.push(format!("    {id:<width$}  {detail}"));
        }
        if findings.len() > LISTED {
            lines.push(format!(
                "    … {} weitere im Bericht",
                findings.len() - LISTED
            ));
        }
    }
    lines.push(format!(
        "  Bericht     {}",
        uri(&paths.report.join("index.html"))
    ));
    lines.push(format!("  Aufnahme    {}", paths.run.display()));
    // A shell-safe exact selection example, never all cases or an environment bypass.
    if review.selection.is_none()
        && review.errors.is_empty()
        && review.environment_mismatch.is_none()
        && !findings.is_empty()
    {
        let id = findings[0].0;
        lines.push(format!("  Übernehmen  cargo gpui-visual accept --repo {} --run {} --baselines {} --report {} --case {}",quote(&paths.repo.to_string_lossy()),quote(&paths.run.to_string_lossy()),quote(&paths.baselines.to_string_lossy()),quote(&paths.report.to_string_lossy()),quote(id)));
        lines.push("              (Beispiel für genau einen Prüffall; nur nach ausdrücklicher Sichtfreigabe)".into());
    }
    lines.join("\n")
}
