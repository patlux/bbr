//! Shared still/sequence comparison controls. Missing sides never look like a blank diff.
use crate::{
    compare::Finding,
    report::{crop_box, escape, hint, marks, radios},
};

pub(crate) struct Images<'a> {
    pub expected: &'a str,
    pub actual: &'a str,
    pub diff: &'a str,
}

/// `finding` carries the per-row changed columns used for numbered boxes.
pub(crate) fn html(
    id: &str,
    images: Images<'_>,
    finding: Option<(&Finding, &[[u32; 2]])>,
) -> String {
    let comparable =
        !images.expected.is_empty() && !images.actual.is_empty() && !images.diff.is_empty();
    let crop = finding.and_then(|(f, _)| crop_box(f, None));
    let frame = crop.map_or_else(
        || "class=\"frame\"".into(),
        |b| {
            format!(
                "class=\"frame\" style=\"--x:-{}px;--y:-{}px;--w:{}px;--h:{}px\"",
                b[0], b[1], b[2], b[3]
            )
        },
    );
    let marks = finding
        .map(|(f, extents)| marks(f, extents))
        .unwrap_or_default();
    let crop_hint = crop.map_or_else(
        || "1:1-Bildfenster · zum Prüfen scrollen".into(),
        |b| format!("1:1-Ausschnitt um die Änderung ({})", hint(b)),
    );
    // Known pixel sizes reserve layout before lazy images load, so fragment
    // targets and the entry in view stay stable while scrolling.
    let item = finding.map(|(f, _)| f);
    let actual_size = item.and_then(|f| f.actual_size);
    let expected_size = item.and_then(|f| f.expected_size);
    let diff_size = actual_size.filter(|_| actual_size == expected_size);
    let dims = |size: Option<[u32; 2]>| {
        size.map_or_else(String::new, |[w, h]| {
            format!(" width=\"{w}\" height=\"{h}\"")
        })
    };
    let image = |kind: &str, path: &str, label: &str, size: Option<[u32; 2]>| {
        format!(
            "<img loading=\"lazy\" data-image=\"{kind}\"{} alt=\"{}: {}\"{}{}>",
            if path.is_empty() {
                String::new()
            } else {
                format!(" src=\"{}\"", escape(path))
            },
            label,
            escape(id),
            dims(size),
            if path.is_empty() { " hidden" } else { "" }
        )
    };
    let link = |kind: &str, path: &str, contents: String| {
        format!(
            "<a data-link=\"{kind}\"{}>{contents}</a>",
            if path.is_empty() {
                " hidden".into()
            } else {
                format!(" href=\"{}\"", escape(path))
            }
        )
    };
    let expected = image("expected", images.expected, "Alt", expected_size);
    let actual = image("actual", images.actual, "Neu", actual_size);
    let diff = image("diff", images.diff, "Diff", diff_size);
    let placeholder = |kind: &str, present: &str, fallback: &str, message: &str| {
        let size = if kind == "expected" {
            actual_size
        } else {
            expected_size
        };
        format!("<div class=\"placeholder missing-{kind}\"{}><img loading=\"lazy\" data-placeholder=\"{kind}\"{}{} alt=\"\" aria-hidden=\"true\"><span>{message}</span></div>",if present.is_empty() {""} else {" hidden"},if fallback.is_empty(){String::new()}else{format!(" src=\"{}\"",escape(fallback))},dims(size))
    };
    let expected_missing =
        placeholder("expected", images.expected, images.actual, "keine Referenz");
    let actual_missing = placeholder(
        "actual",
        images.actual,
        images.expected,
        "im Kandidaten fehlend",
    );
    format!(
        r#"<div class="viewer{}" data-comparable="{comparable}">
<div class="viewer-bar"><span class="seg comparison-modes">{}</span><input class="m-crop" type="checkbox" id="{id}-crop"><label class="toggle crop-control" for="{id}-crop">1:1</label></div><p class="hint crop-hint">{crop_hint}</p>
<p class="comparison-unavailable hint">Alt/Neu-Vergleich nicht verfügbar: Referenz oder aktuelles Bild fehlt. Vorhandene Bilder können weiterhin in 1:1 geprüft werden.</p>
<div class="unpaired images"><figure><figcaption>Alt · Referenz</figcaption>{expected_missing}{}</figure><figure><figcaption>Neu · Kandidat</figcaption>{actual_missing}{}</figure></div>
<div class="views"><div class="view v-2up images">
<figure><figcaption>Alt · Referenz</figcaption>{}</figure><figure><figcaption>Neu · Kandidat</figcaption>{}</figure></div>
<div class="view v-swipe"><p class="hint">Trennlinie ziehen oder ins Bild klicken · links Alt, rechts Neu</p><div {frame}><div class="stage">{actual}<div class="overlay">{expected}</div><div class="handle" role="slider" tabindex="0" aria-label="Trennlinie Alt / Neu" aria-valuemin="0" aria-valuemax="100" aria-valuenow="50"></div></div>{marks}</div></div>
<div class="view v-onion"><div class="blendbar"><span>Alt</span><input class="blend" type="range" min="0" max="100" value="50" aria-label="Überblendung Alt zu Neu"><span>Neu</span></div><div {frame}><div class="stage">{expected}<div class="overlay">{actual}</div></div>{marks}</div></div>
<div class="view v-diff images"><figure><figcaption>Jedes abweichende Pixel</figcaption>{}</figure></div></div></div>"#,
        if crop.is_none() { " crop-full" } else { "" },
        radios(id, false, comparable),
        link(
            "expected",
            images.expected,
            format!("<span class=\"frame\">{expected}</span>")
        ),
        link(
            "actual",
            images.actual,
            format!("<span class=\"frame\">{actual}</span>")
        ),
        link(
            "expected",
            images.expected,
            format!("<span {frame}>{expected}{marks}</span>")
        ),
        link(
            "actual",
            images.actual,
            format!("<span {frame}>{actual}{marks}</span>")
        ),
        link(
            "diff",
            images.diff,
            format!("<span {frame}>{diff}{marks}</span>")
        )
    )
}
