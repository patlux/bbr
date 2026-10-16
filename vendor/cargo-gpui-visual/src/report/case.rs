//! One case per section: header, stage and a sticky inspector beside it.
//! The inspector only reads stored findings; acceptance stays an explicit CLI step.
use super::{detail, escape, hint, status_label, views, Detail};
use crate::compare::{Finding, Review};
use std::path::Path;

/// Exact paths of the comparison that produced this report. Direct report writes
/// have none and therefore never print an acceptance command.
#[derive(Clone, Copy)]
pub(crate) struct AcceptPaths<'a> {
    pub(crate) repo: &'a Path,
    pub(crate) run: &'a Path,
    pub(crate) baselines: &'a Path,
    pub(crate) report: &'a Path,
}

/// `settings--light` → (`settings`, `hell`). Unknown suffixes stay verbatim.
pub(crate) fn split_case(case: &str) -> (&str, &str) {
    match case.rsplit_once("--") {
        Some((base, "light")) => (base, "hell"),
        Some((base, "dark")) => (base, "dunkel"),
        Some((base, theme)) => (base, theme),
        None => (case, ""),
    }
}

/// German digit grouping: 8612 → `8.612`.
pub(crate) fn grouped(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(c);
    }
    out
}

/// Share with two decimals and a decimal comma; never rounds a change to zero.
fn percent(part: u64, total: u64) -> String {
    if total == 0 {
        return "0".into();
    }
    let hundredths = (u128::from(part) * 10_000 + u128::from(total) / 2) / u128::from(total);
    if hundredths == 0 && part > 0 {
        return "<0,01".into();
    }
    format!("{},{:02}", hundredths / 100, hundredths % 100)
}

pub(crate) fn badge(status: &str, reason: Option<&str>) -> String {
    let label = match (status, reason) {
        ("changed", Some("dimensions")) => "geändert · Bildgröße",
        _ => status_label(status),
    };
    format!(
        "<span class=\"badge {}\">{}</span>",
        escape(status),
        escape(label)
    )
}

fn size([w, h]: [u32; 2]) -> String {
    format!("{w}×{h}")
}

/// One-line measurement summary below the case name.
pub(crate) fn meta(item: &Finding) -> String {
    if item.reason.as_deref() == Some("dimensions") {
        return match (item.expected_size, item.actual_size) {
            (Some(e), Some(a)) => format!("Bildgröße {} → {}", size(e), size(a)),
            _ => "Bildgröße geändert".into(),
        };
    }
    match (item.changed_pixels, item.bounds, item.actual_size) {
        (Some(px), Some([x1, y1, x2, y2]), Some(dim)) if px > 0 => {
            let areas = item.rows.len();
            format!(
                "{} px ({} %) · {areas} {} · Bounds {x1},{y1} → {x2},{y2} · {}",
                grouped(px),
                percent(px, u64::from(dim[0]) * u64::from(dim[1])),
                if areas == 1 {
                    "Zeilenband"
                } else {
                    "Zeilenbänder"
                },
                size(dim)
            )
        }
        (Some(_), _, Some(dim)) => format!("pixelgleich · {}", size(dim)),
        (_, _, Some(dim)) if item.status == "missing-baseline" => format!(
            "erste Aufnahme · {} · Vergleich erst nach bewusster Übernahme",
            size(dim)
        ),
        _ if item.status == "missing-baseline" => {
            "erste Aufnahme · Vergleich erst nach bewusster Übernahme".into()
        }
        _ => "keine Messwerte".into(),
    }
}

/// Emphasize dimension drift with the report's existing change color.
/// Keep other measurements muted and escape text before adding markup.
fn meta_html(item: &Finding) -> String {
    let text = escape(&meta(item));
    if item.reason.as_deref() == Some("dimensions") {
        format!("<span class=\"dimension-change\">{text}</span>")
    } else {
        text
    }
}

/// Short status line for the table of contents.
pub(crate) fn nav_detail(item: &Finding) -> String {
    match item.status.as_str() {
        "changed" if item.reason.as_deref() == Some("dimensions") => "Bildgröße geändert".into(),
        "changed" => {
            let areas = item.rows.len();
            format!(
                "geändert · {areas} {}",
                if areas == 1 {
                    "Zeilenband"
                } else {
                    "Zeilenbänder"
                }
            )
        }
        status => status_label(status).into(),
    }
}

pub(crate) fn navhead(label: &str, count: usize) -> String {
    format!("<p class=\"navhead\">{label} <span class=count>{count}</span></p>")
}

/// Table-of-contents row. A thumbnail row for findings, a compact row otherwise.
pub(crate) fn nav_row(
    href: &str,
    case: &str,
    status: &str,
    detail: &str,
    thumb: Option<&str>,
) -> String {
    let (base, theme) = split_case(case);
    let quiet = if status == "same" { "quiet" } else { "finding" };
    let (href, base, theme, status) = (escape(href), escape(base), escape(theme), escape(status));
    match thumb {
        Some(src) => format!(
            "<a class=\"toc-row {quiet}\" href=\"#{href}\"><span class=\"thumb\"><img loading=\"lazy\" src=\"{}\" alt=\"\"></span><span class=\"toc-text\"><span class=\"toc-name\"><code>{base}</code> <small>{theme}</small></span><span class=\"toc-status\"><span class=\"dot {status}\"></span>{}</span></span></a>",
            escape(src),
            escape(detail)
        ),
        None => format!(
            "<a class=\"toc-row compact {quiet}\" href=\"#{href}\"><span class=\"dot {status}\"></span><code>{base}</code><small>{theme}</small></a>"
        ),
    }
}

fn note(text: &str) -> String {
    format!("<p class=\"panel-note\">{text}</p>")
}

/// Spatial 1:1 windows with actual changes; the first one is opened.
fn regions(item: &Finding, asset: &str, details: &[Detail]) -> String {
    if item.status == "missing-baseline" {
        return note("Neu · keine Referenz. Die erste Aufnahme wird erst nach einer bewussten Übernahme zur Referenz.");
    }
    if item.reason.as_deref() == Some("dimensions") {
        return note(&format!(
            "{}. Ein Pixelvergleich ist nicht möglich; das Diff markiert die ganze Fläche.",
            meta_html(item)
        ));
    }
    let Some([w, h]) = item.actual_size else {
        return String::new();
    };
    if item.rows.is_empty() {
        return String::new();
    }
    if details.is_empty() {
        return note("Detailausschnitte nicht verfügbar: gespeichertes Diff fehlt oder ist inkonsistent. Vollständige Bilder prüfen.");
    }
    let asset = escape(asset);
    let mut html = format!(
        "<div class=\"regions\"><p class=\"panel-head\"><strong>Detailausschnitte</strong> <span class=\"count\">{}</span><span class=\"keyhint\">nächster <kbd>R</kbd></span></p>{}",
        details.len(), note("Räumliche Ausschnitte mit geänderten Pixeln · keine zusammenhängenden UI-Bereiche. Lange Änderungen sind aufgeteilt; nichts wird ausgelassen.")
    );
    for (index, detail) in details.iter().enumerate() {
        let n = index + 1;
        let [x, y, lw, lh] = detail.crop;
        let [x1, top, x2, bottom] = detail.bounds;
        let coordinates = format!("x {x1}–{x2} · y {top}–{bottom}");
        let loupe = |label: &str, file: &str| {
            format!("<figure tabindex=\"0\" aria-label=\"{label}, Detail {n} in Originalgröße\"><figcaption>{label}</figcaption><div class=\"loupe\" style=\"width:{lw}px;height:{lh}px\"><img loading=\"lazy\" src=\"{asset}/{file}\" alt=\"{label}, Ausschnitt {}\" width=\"{w}\" height=\"{h}\" style=\"left:-{x}px;top:-{y}px\"></div></figure>", hint(detail.crop))
        };
        let stage = format!("<button type=\"button\" class=\"to-stage\" hidden data-x=\"{x}\" data-w=\"{lw}\" data-y=\"{y}\" data-h=\"{lh}\" data-rows=\"Detail {n}\" data-hint=\"{}\">Auf der Bühne 1:1 zeigen</button>", hint(detail.crop));
        html += &format!(
            "<details class=\"region\"{}><summary><span class=\"rmark\">{n}</span><code>{coordinates}</code><span class=\"rsize\">{} px geändert</span></summary>{}<div class=\"loupes\">{}{}{}</div>{stage}</details>",
            if index == 0 { " open" } else { "" },
            grouped(detail.changed),
            note(&format!("1:1 · {}", hint(detail.crop))),
            loupe("Alt", "expected.png"),
            loupe("Neu", "actual.png"),
            loupe("Diff · jedes abweichende Pixel", "diff.png"),
        );
    }
    html + "</div>"
}

pub(crate) fn accept_note(text: &str) -> String {
    format!(
        "<div class=\"accept\"><p class=\"panel-head\"><strong>Übernahme nur per CLI</strong></p>{}</div>",
        note(text)
    )
}

/// The same exact single-case hint the terminal offers, under the same conditions.
pub(crate) fn accept_block(review: &Review, paths: Option<AcceptPaths<'_>>, id: &str) -> String {
    if review.selection.is_some() {
        return accept_note(
            "Teillauf: Übernahme gesperrt. Vollständigen Lauf aufnehmen und prüfen.",
        );
    }
    if !review.errors.is_empty() {
        return accept_note(
            "Ungültige Eingabe: nichts übernehmbar, bis ein neuer Vergleich gültig ist.",
        );
    }
    if review.environment_mismatch.is_some() {
        return accept_note("Die Aufnahmeumgebung weicht ab. Zuerst den Hinweis oben prüfen.");
    }
    let quote = crate::terminal::quote;
    let Some(p) = paths else {
        return accept_note(&format!(
            "Der vollständige Befehl für <code>--case {}</code> steht in der Ausgabe von check oder compare.",
            escape(&quote(id))
        ));
    };
    let command = format!(
        "cargo gpui-visual accept \\\n  --repo {} \\\n  --run {} \\\n  --baselines {} \\\n  --report {} \\\n  --case {}",
        quote(&p.repo.to_string_lossy()),
        quote(&p.run.to_string_lossy()),
        quote(&p.baselines.to_string_lossy()),
        quote(&p.report.to_string_lossy()),
        quote(id)
    );
    format!(
        "<div class=\"accept\"><p class=\"panel-head\"><strong>Übernahme nur per CLI</strong></p>{}<pre class=\"command\"><code>{}</code></pre><div class=\"accept-actions\"><button type=\"button\" class=\"copy\" hidden>Befehl kopieren <kbd>C</kbd></button><label class=\"seen\" title=\"Nur eine lokale Merkhilfe in diesem Browser, keine Übernahme\"><input type=\"checkbox\" data-seen=\"{}\"> gesehen</label></div>{}</div>",
        note("Der Bericht übernimmt nichts. Erst ansehen, dann bewusst im Terminal bestätigen."),
        escape(&command),
        escape(id),
        note("Nur nach ausdrücklicher Sichtfreigabe ausführen.")
    )
}

/// A changed or new still: `key` namespaces DOM identities, `asset` locates images
/// below the report `root`.
pub(crate) fn finding_section(
    root: &Path,
    item: &Finding,
    key: &str,
    asset: &str,
    title: &str,
    accept: &str,
) -> String {
    let (_, theme) = split_case(&item.case);
    // New cases have no stored size; read it from the PNG header for layout only.
    let mut sized = item.clone();
    if sized.actual_size.is_none() {
        let path = root.join(asset).join("actual.png");
        sized.actual_size = crate::store::no_symlinks(&path)
            .ok()
            .and_then(|()| image::image_dimensions(&path).ok())
            .map(|(w, h)| [w, h]);
    }
    let item = &sized;
    let details = detail::load(root, asset, item).unwrap_or_default();
    let tag = if theme.is_empty() {
        String::new()
    } else {
        format!("<span class=\"tag\">{}</span>", escape(theme))
    };
    format!(
        "<section class=\"case finding\" id=\"f-{key}\"><div class=\"case-main\"><header class=\"case-head\"><div class=\"case-title\"><h2>{title}</h2>{tag}{}</div><p class=\"case-meta\">{}</p></header>{}</div><aside class=\"inspector\" aria-label=\"Inspektor\"><div class=\"inspector-body\"><p class=\"inview\"><span>im Blick</span><code>{key}</code></p>{}{accept}</div></aside></section>",
        badge(&item.status, item.reason.as_deref()),
        meta_html(item),
        views(item, key, asset, &details),
        regions(item, asset, &details),
        key = escape(key),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn german_numbers_never_hide_a_change() {
        assert_eq!(grouped(8612), "8.612");
        assert_eq!(grouped(1_234_567), "1.234.567");
        assert_eq!(grouped(12), "12");
        assert_eq!(percent(8612, 804 * 1748), "0,61");
        assert_eq!(percent(1, 1_000_000), "<0,01");
        assert_eq!(percent(0, 10), "0,00");
    }

    #[test]
    fn themes_are_labelled_without_guessing() {
        assert_eq!(split_case("settings--light"), ("settings", "hell"));
        assert_eq!(split_case("a--b--dark"), ("a--b", "dunkel"));
        assert_eq!(split_case("plain"), ("plain", ""));
    }
}
