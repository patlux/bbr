//! Presentation only: compact differences, with complete evidence behind disclosures.
use super::escape;
use crate::compare::Review;
use serde_json::Value;
use std::collections::BTreeSet;

struct Change<'a> {
    path: String,
    before: Option<&'a Value>,
    after: Option<&'a Value>,
}

fn changes<'a>(
    before: Option<&'a Value>,
    after: Option<&'a Value>,
    path: &str,
    out: &mut Vec<Change<'a>>,
) {
    if before == after {
        return;
    }
    match (before, after) {
        (Some(Value::Object(a)), Some(Value::Object(b))) => {
            for key in a.keys().chain(b.keys()).collect::<BTreeSet<_>>() {
                let token = key.replace('~', "~0").replace('/', "~1");
                changes(a.get(key), b.get(key), &format!("{path}/{token}"), out);
            }
        }
        (Some(Value::Array(a)), Some(Value::Array(b))) => {
            for i in 0..a.len().max(b.len()) {
                changes(a.get(i), b.get(i), &format!("{path}/{i}"), out);
            }
        }
        _ => out.push(Change {
            path: path.into(),
            before,
            after,
        }),
    }
}

fn value(value: Option<&Value>) -> String {
    match value {
        None => "<span class=\"value-absent\">nicht vorhanden</span>".into(),
        Some(Value::String(text)) => escape(text),
        Some(value) => escape(&value.to_string()),
    }
}

fn field(path: &str) -> String {
    let label = match path.rsplit('/').next().unwrap_or_default() {
        "os" => "Betriebssystem",
        "os_build" => "OS-Build",
        _ => {
            return format!(
                "<code>{}</code>",
                escape(if path.is_empty() { "/" } else { path })
            )
        }
    };
    format!("{label}<small>{}</small>", escape(path))
}

fn table(rows: &[Change<'_>]) -> String {
    let rows = rows
        .iter()
        .map(|c| {
            format!(
                "<tr><th scope=\"row\">{}</th><td>{}</td><td>{}</td></tr>",
                field(&c.path),
                value(c.before),
                value(c.after),
            )
        })
        .collect::<String>();
    format!("<table class=\"environment-delta\"><caption>Geänderte Werte</caption><thead><tr><th scope=\"col\">Eigenschaft</th><th scope=\"col\">Referenz</th><th scope=\"col\">Aufnahme</th></tr></thead><tbody>{rows}</tbody></table>")
}

/// Never omit classification changes. Provenance values are separated only for
/// valid partitions with the same contract and the same classified leaf set.
fn matching_partition(a: &Value, b: &Value) -> bool {
    a.get("comparison_schema") == Some(&Value::from(1))
        && b.get("comparison_schema") == Some(&Value::from(1))
        && crate::environment::validate(a).is_ok()
        && crate::environment::validate(b).is_ok()
        && a.get("contract") == b.get("contract")
        && a.get("provenance")
            .and_then(Value::as_object)
            .zip(b.get("provenance").and_then(Value::as_object))
            .is_some_and(|(a, b)| a.keys().eq(b.keys()))
}

fn difference(difference: &Value, rendering_only: bool) -> String {
    let (Some(before), Some(after)) = (difference.get("baselines"), difference.get("capture"))
    else {
        return raw("Gespeicherter Unterschied", difference);
    };
    let mut rows = Vec::new();
    changes(Some(before), Some(after), "", &mut rows);
    if rendering_only && matching_partition(before, after) {
        rows.retain(|c| !c.path.starts_with("/provenance/"));
    }
    if rows.is_empty() {
        return "<p class=\"diagnostic-note\">Keine einzelnen geänderten Werte gespeichert. Vollständige Daten prüfen.</p>".into();
    }
    let visible = rows.len().min(12);
    let mut html = table(&rows[..visible]);
    if rows.len() > visible {
        html += &format!("<details class=\"diagnostic-more\"><summary>{} weitere geänderte Werte</summary>{}</details>", rows.len() - visible, table(&rows[visible..]));
    }
    html
}

pub(super) fn raw(label: &str, data: &Value) -> String {
    let json = serde_json::to_string_pretty(data).expect("JSON values serialize");
    format!("<details class=\"raw-evidence\"><summary>{}</summary><pre tabindex=\"0\" aria-label=\"{}\">{}</pre></details>", escape(label), escape(label), escape(&json))
}

pub(super) fn environment(r: &Review) -> String {
    r.environment_mismatch.as_ref().map_or_else(String::new, |d| {
        format!("<div class=\"environment-change\"><h3>Aufnahmeumgebung weicht ab</h3><p class=\"diagnostic-note\">Nur geänderte Werte · keine automatische Übernahme.</p>{}</div>", difference(d, true))
    })
}

pub(super) fn provenance(r: &Review) -> String {
    r.provenance_difference.as_ref().map_or_else(String::new, |d| {
        format!("<details class=\"provenance-details\"><summary>Provenienz geändert · separat vom Rendering-Vergleich</summary>{}{}</details>", difference(d, false), raw("Vollständige Herkunftsdaten", d))
    })
}

pub(super) fn evidence(r: &Review) -> String {
    let mut html = raw("Vollständige Aufnahmeumgebung", &r.environment);
    if let Some(d) = &r.environment_mismatch {
        html += &raw("Vollständiger Umgebungsvergleich", d);
    }
    html
}

pub(super) fn suite(s: &crate::collection::SuiteReview) -> String {
    use crate::collection::catalog::State;
    let snapshots = s
        .catalog
        .as_ref()
        .map(|c| c.snapshots.as_slice())
        .unwrap_or_default();
    let findings = s
        .review
        .as_ref()
        .map(|r| r.results.as_slice())
        .unwrap_or_default();
    let missing = |base: &str| {
        ["light", "dark"].iter().any(|theme| {
            !findings
                .iter()
                .any(|f| f.case == format!("{base}--{theme}"))
        })
    };
    let gaps = snapshots
        .iter()
        .filter(|s| s.state == State::Withheld || (s.state == State::Required && missing(&s.base)))
        .count();
    // Only exact catalog-generated withholding notices are coverage, not errors.
    let withheld: BTreeSet<_> = snapshots
        .iter()
        .filter(|s| s.state == State::Withheld)
        .map(|s| {
            format!(
                "Withheld: {} — {}",
                s.base,
                s.reason.as_deref().unwrap_or_default()
            )
        })
        .collect();
    let errors: Vec<_> = s
        .errors
        .iter()
        .filter(|e| !withheld.contains(*e))
        .chain(s.review.iter().flat_map(|r| &r.errors))
        .collect();
    let valid = s.review.is_some() && errors.is_empty();
    let status = if valid {
        format!("{} Aufnahmen geprüft", findings.len())
    } else {
        "Prüfung unvollständig".into()
    };
    let environment = if s
        .review
        .as_ref()
        .is_some_and(|r| r.environment_mismatch.is_some())
    {
        "Umgebung geändert"
    } else if !valid {
        "Umgebung nicht geprüft"
    } else if s
        .review
        .as_ref()
        .is_some_and(|r| r.baseline_state.is_empty())
    {
        "Ohne Referenzumgebung"
    } else {
        "Umgebung unverändert"
    };
    let gap_label = match (s.catalog.is_some(), gaps) {
        (false, _) => "Abdeckung nicht geprüft".into(),
        (true, 0) => "Keine Abdeckungslücken".into(),
        (true, 1) => "1 Abdeckungslücke".into(),
        (true, n) => format!("{n} Abdeckungslücken"),
    };
    let mut html = format!("<section class=\"suite\" id=\"suite-{}\"><h2>{}</h2><p class=\"suite-summary\"><strong class=\"{}\">{status}</strong><span>{environment}</span><span>{gap_label}</span></p>", escape(&s.suite.id), escape(&s.suite.label), if valid { "checked" } else { "incomplete" });
    if let Some(r) = &s.review {
        html += &self::environment(r);
    }
    if !errors.is_empty() {
        html += &format!("<details class=\"diagnostic-errors\"><summary>{} Prüfprobleme · Details ansehen</summary><ul>{}</ul></details>", errors.len(), errors.iter().map(|e| format!("<li>{}</li>", escape(e))).collect::<String>());
    }
    let coverage: Vec<_> = snapshots
        .iter()
        .filter(|s| s.state != State::Required || missing(&s.base))
        .collect();
    if !coverage.is_empty() {
        html += "<details class=\"coverage-details\"><summary>Abdeckung · offene und nicht unterstützte Zustände</summary><ul class=\"coverage-list\">";
        for item in coverage {
            let state = match item.state {
                State::Withheld => "Noch nicht nachgewiesen",
                State::Unsupported => "Nicht unterstützt",
                State::Required => "Pflichtaufnahme fehlt",
            };
            html += &format!(
                "<li><strong>{}</strong><span>{state}</span><p>{}</p><code>{}</code></li>",
                escape(&item.label),
                escape(
                    item.reason
                        .as_deref()
                        .unwrap_or("Mindestens eine erforderliche Erscheinungsvariante fehlt.")
                ),
                escape(&item.base)
            );
        }
        html += "</ul></details>";
    }
    if let Some(r) = &s.review {
        html += &provenance(r);
    }
    html += &format!("<details class=\"technical-details\"><summary>Technische Details</summary><dl class=\"evidence-paths\"><dt>Suite</dt><dd><code>{}</code></dd><dt>Referenzen</dt><dd><code>{}</code></dd>", escape(&s.suite.id), escape(&s.suite.baselines.display().to_string()));
    if let Some(r) = &s.review {
        html += &format!(
            "<dt>Capture-Prüfsumme</dt><dd><code>{}</code></dd></dl>{}",
            escape(&r.capture_hash),
            evidence(r)
        );
    } else {
        html += "</dl>";
    }
    html + "</details></section>"
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn delta_preserves_added_removed_null_arrays_and_escaping() {
        let d = json!({"baselines":{"stable":"same","removed":null,"array":[1,2],"<key>":"<old>"},
            "capture":{"stable":"same","added":null,"array":[1,3,4],"<key>":"<new>"}});
        let html = difference(&d, false);
        assert!(!html.contains("stable") && !html.contains("/array/0"));
        for text in [
            "/array/1",
            "/array/2",
            "/added",
            "/removed",
            "nicht vorhanden",
            ">null<",
            "&lt;key&gt;",
            "&lt;old&gt;",
            "&lt;new&gt;",
        ] {
            assert!(html.contains(text), "{text}");
        }
    }

    #[test]
    fn many_changes_remain_available_but_collapsed() {
        let a: serde_json::Map<_, _> = (0..20).map(|i| (format!("key{i:02}"), json!(i))).collect();
        let b: serde_json::Map<_, _> = (0..20)
            .map(|i| (format!("key{i:02}"), json!(i + 1)))
            .collect();
        let html = difference(&json!({"baselines":a,"capture":b}), false);
        assert!(html.contains("8 weitere geänderte Werte"));
        assert!(!html.contains(" open"));
        assert_eq!(html.matches("<th scope=\"row\">").count(), 20);
    }

    #[test]
    fn partition_changes_are_not_hidden_as_provenance() {
        let before = crate::environment::partition(
            &json!({"layer":"gpui","tool":"a"}),
            json!({"id":"one"}),
            &["/tool".into()],
        )
        .unwrap();
        let mut after = before.clone();
        after["provenance"]["/tool"] = json!("b");
        assert!(matching_partition(&before, &after));
        after["contract"]["id"] = json!("two");
        assert!(!matching_partition(&before, &after));
        let html = difference(&json!({"baselines":before,"capture":after}), true);
        assert!(html.contains("/contract/id") && html.contains("/provenance/"));
    }
}
