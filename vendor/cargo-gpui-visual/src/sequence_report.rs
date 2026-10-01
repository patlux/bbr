use crate::{
    compare::Review,
    comparison_view::{self, Images},
    report::{badge, escape, split_case, status_label},
};
use std::collections::HashSet;

/// Whole journeys: the tick strip is always visible; thumbnails load on request.
pub(crate) fn html(review: &Review, accept: &dyn Fn(&str) -> String) -> String {
    let changed_images: HashSet<&str> = review
        .results
        .iter()
        .filter(|r| r.status != "same")
        .map(|r| r.case.as_str())
        .collect();
    let mut html = String::new();
    for sequence in &review.sequences {
        let actual = &sequence.actual;
        let id = &actual.id;
        let first = sequence.first_difference;
        let count = actual
            .ticks
            .len()
            .max(sequence.expected.as_ref().map_or(0, |s| s.ticks.len()));
        let status = sequence.status();
        let paths = |index: usize| {
            let tick = actual.ticks.get(index);
            let expected = sequence.expected.as_ref().and_then(|s| s.ticks.get(index));
            let e = expected
                .map(|t| format!("sequence-expected/{}.png", t.image))
                .unwrap_or_default();
            let a = tick
                .map(|t| format!("{}/actual.png", t.image))
                .unwrap_or_default();
            let d = match (tick, expected) {
                (Some(a), Some(e)) if a.image != e.image => diff_path(&a.image, &e.image),
                (Some(a), Some(_)) => format!("{}/diff.png", a.image),
                _ => String::new(),
            };
            (a, e, d)
        };
        let (a, e, d) = paths(0);
        let viewer = comparison_view::html(
            id,
            Images {
                actual: &a,
                expected: &e,
                diff: &d,
            },
            None,
        );
        let (base, theme) = split_case(id);
        let state = if sequence.expected.is_none() {
            "Freigabe ausstehend"
        } else if first.is_some() {
            "Sequenz abweichend"
        } else {
            "Sequenz identisch"
        };
        let mut ticks = String::new();
        let mut held = 0;
        let mut differing = 0;
        for index in 0..count {
            let tick = actual.ticks.get(index);
            let expected = sequence.expected.as_ref().and_then(|s| s.ticks.get(index));
            let (a, e, d) = paths(index);
            let kind = match (tick, expected) {
                _ if sequence.expected.is_none() => "new",
                (Some(t), Some(x)) if t != x || changed_images.contains(t.image.as_str()) => "diff",
                (Some(_), Some(_)) => "same",
                _ => "missing",
            };
            differing += usize::from(kind == "diff" || kind == "missing");
            let hold = tick.is_some_and(|t| !t.drawn);
            held += usize::from(hold);
            let caption = tick
                .map(|t| {
                    format!(
                        "Takt {} · {:.2} ms · {} · {}",
                        t.tick,
                        t.time_ns as f64 / 1_000_000.,
                        if t.drawn {
                            "gerendert"
                        } else {
                            "voriges Bild gehalten"
                        },
                        t.events.join(", ")
                    )
                })
                .unwrap_or_else(|| format!("Takt {index} · im Kandidaten fehlend"));
            let state = serde_json::json!({"expected":expected,"actual":tick});
            let thumbnail = if a.is_empty() { &e } else { &a };
            ticks+=&format!("<button class=\"tick {kind}{}\" title=\"{}\" data-tick=\"{index}\" data-actual=\"{}\" data-expected=\"{}\" data-diff=\"{}\" data-caption=\"{}\" data-state=\"{}\"><span class=\"tick-label\">{}{}</span><img loading=lazy data-src=\"{}\" alt=\"{}\"></button>",if hold {" held"} else {""},escape(&caption),escape(&a),escape(&e),escape(&d),escape(&caption),escape(&serde_json::to_string_pretty(&state).unwrap()),index,if tick.is_some_and(|t|t.drawn){""}else{" · hold / missing"},escape(thumbnail),escape(&caption));
        }
        let meta = format!(
            "{state} · {} Takte · {} Renderings · {held} gehalten{} · keine native FPS-Messung",
            actual.ticks.len(),
            actual.images().count(),
            if sequence.expected.is_some() {
                format!(" · {differing} abweichend")
            } else {
                String::new()
            }
        );
        let badge = match (status, first) {
            ("changed", Some(tick)) => {
                format!("<span class=\"badge changed\">ab Takt {tick} abweichend</span>")
            }
            _ => badge(status, None),
        };
        html+=&format!("<section class=\"sequence {}\" id=\"{id}\" data-hz=\"{}\"><div class=\"case-main\"><header class=\"case-head\"><div class=\"case-title\"><h2>{}</h2><span class=\"tag\">{}</span><span class=\"tag\">Sequenz · {} Hz</span>{badge}</div><p class=\"case-meta\" title=\"{}\">{}</p></header><div class=sequence-controls><button class=seq-prev>← Einzelbild</button><button class=seq-play>Abspielen</button><button class=seq-next>Einzelbild →</button><button class=seq-first data-first=\"{}\"{}>Erste Abweichung</button><input class=seq-slider type=range min=0 max=\"{}\" value=0 aria-label=\"Sequenzzeit\"></div><div class=\"seq-filmstrip\" role=\"group\" aria-label=\"Takte\">{ticks}</div><p class=\"tick-legend\"><span class=\"tick same\"></span>gleich <span class=\"tick diff\"></span>abweichend <span class=\"tick held\"></span>voriges Bild gehalten <span class=\"tick missing\"></span>fehlend <span class=\"tick new\"></span>ohne Referenz</p><output class=seq-caption></output>{viewer}</div><aside class=\"inspector\" aria-label=\"Inspektor\"><div class=\"inspector-body\"><p class=\"inview\"><span>im Blick</span><code>{id}</code></p><details open><summary>Zustand und Ereignisse: Alt / Neu</summary><pre class=seq-state></pre></details><details class=sequence-filmstrip><summary>Einzelbilder und Haltezeiten</summary><p class=\"panel-note\">Zeigt Vorschaubilder in der Taktleiste. Sie laden erst beim Öffnen.</p></details>{}</div></aside></section>",if status=="same" {"quiet"} else {"finding"},actual.hz,escape(base),escape(theme),actual.hz,status_label(status),escape(&meta),first.unwrap_or(0),if first.is_none(){" disabled"}else{""},count-1,accept(id));
    }
    html
}

pub fn diff_path(actual: &str, expected: &str) -> String {
    format!("sequence-diff/{actual}--vs--{expected}.png")
}
