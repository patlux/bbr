use super::*;
use cargo_gpui_visual::sequence::{Sequence, Tick};

pub(super) fn sequences() -> Fixture {
    let mut f = Fixture::new();
    f.cases.clear();
    for entry in fs::read_dir(&f.run).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "png") {
            fs::remove_file(path).unwrap();
        }
    }
    let mut sequences = Vec::new();
    for theme in ["light", "dark"] {
        let id = format!("sequence-story-60hz--{theme}");
        let ticks = (0..3)
            .map(|tick| {
                let image = format!("sequence-story-60hz-frame-{:04}--{theme}", tick.min(1));
                if tick < 2 {
                    f.image(&image, 2, 3, [tick as u8, 30, 40, 255]);
                    f.cases.push(image.clone());
                }
                Tick {
                    tick,
                    time_ns: tick * 1_000_000_000 / 60,
                    drawn: tick < 2,
                    dirty_before_callbacks: tick == 1,
                    callbacks_pending: false,
                    image,
                    events: vec![],
                    state: json!({"ready": tick > 0}),
                }
            })
            .collect();
        sequences.push(Sequence { id, hz: 60, ticks });
    }
    f.seal();
    f.manifest(|m| m["sequences"] = json!(sequences));
    f
}
pub(super) fn ids(f: &Fixture) -> Vec<String> {
    load_manifest(&f.run)
        .unwrap()
        .sequences
        .into_iter()
        .map(|s| s.id)
        .collect()
}
pub(super) fn establish(f: &Fixture) {
    f.compare().unwrap();
    f.accept(&ids(f), false).unwrap();
}

#[test]
fn whole_sequences_only_and_unchanged_refused() {
    let f = sequences();
    f.compare().unwrap();
    fails(f.accept(&f.cases[..1], false), "complete sequence");
    assert!(!f.base.exists());
    f.accept(&ids(&f)[..1], false).unwrap();
    assert!(!f.compare().unwrap().passed);
    f.accept(&ids(&f)[1..], false).unwrap();
    assert!(f.compare().unwrap().passed);
    fails(f.accept(&ids(&f), false), "Not a reviewed");
}

#[test]
fn accept_all_takes_complete_journeys_never_frames() {
    let f = sequences();
    f.compare().unwrap();
    let preview = preview_all(&f.run, &f.report).unwrap();
    assert!(preview.changed.is_empty() && preview.missing.is_empty());
    assert_eq!(preview.sequences, ids(&f));
    accept_all(&f.repo, &f.run, &f.base, &f.report, &preview, false).unwrap();
    assert!(f.compare().unwrap().passed);
    assert!(preview_all(&f.run, &f.report).unwrap().is_empty());
}

#[test]
fn timeline_mutations_fail_with_identical_pixels() {
    for field in [
        "events",
        "state",
        "callbacks_pending",
        "dirty_before_callbacks",
    ] {
        let f = sequences();
        establish(&f);
        f.manifest(|m| {
            m["sequences"][0]["ticks"][1][field] = match field {
                "events" => json!(["response"]),
                "state" => json!({"ready":false}),
                "callbacks_pending" => json!(true),
                _ => json!(false),
            }
        });
        let r = f.compare().unwrap();
        assert!(!r.passed);
        assert!(r.results.iter().all(|r| r.status == "same"));
        assert_eq!(r.sequences[0].first_difference, Some(1));
        f.accept(&ids(&f)[..1], false).unwrap();
        assert!(f.compare().unwrap().passed);
    }
}

#[test]
fn malformed_ticks_frames_and_pairs_fail_closed() {
    for mode in 0..9 {
        let f = sequences();
        f.manifest(|m| match mode {
            0 => m["sequences"][0]["ticks"]
                .as_array_mut()
                .unwrap()
                .swap(0, 1),
            1 => {
                m["sequences"][0]["ticks"].as_array_mut().unwrap().remove(1);
            }
            2 => m["sequences"][0]["ticks"][0]["drawn"] = json!(false),
            3 => {
                m["sequences"][0]["ticks"][2]["image"] =
                    m["sequences"][0]["ticks"][0]["image"].clone()
            }
            4 => m["sequences"][0]["ticks"][1]["time_ns"] = json!(99),
            5 => {
                m["sequences"].as_array_mut().unwrap().pop();
            }
            6 => m["sequences"][0]["hz"] = json!(0),
            7 => m["sequences"][0]["id"] = json!("../bad--light"),
            _ => m["sequences"][0]["ticks"][2]["drawn"] = json!(true),
        });
        assert!(f.compare().is_err(), "mode {mode}");
        assert!(!f.base.exists());
    }
}

#[test]
fn removed_render_is_reviewable_and_deleted_only_for_selected_sequence() {
    let f = sequences();
    establish(&f);
    let before = baseline_state(&f.base).unwrap();
    let removed = "sequence-story-60hz-frame-0001--light";
    fs::remove_file(f.run.join(format!("{removed}.png"))).unwrap();
    f.manifest(|m| {
        m["cases"].as_array_mut().unwrap().retain(|c| c != removed);
        m["images"].as_object_mut().unwrap().remove(removed);
        let image = m["sequences"][0]["ticks"][0]["image"].clone();
        for i in [1, 2] {
            m["sequences"][0]["ticks"][i]["drawn"] = json!(false);
            m["sequences"][0]["ticks"][i]["image"] = image.clone();
        }
    });
    let r = f.compare().unwrap();
    assert!(r.errors.is_empty());
    assert!(!r.passed);
    assert!(f
        .report
        .join(format!("sequence-expected/{removed}.png"))
        .exists());
    f.accept(&ids(&f)[..1], false).unwrap();
    assert!(!f.base.join(format!("{removed}.png")).exists());
    let after = baseline_state(&f.base).unwrap();
    for (key, hash) in before.iter().filter(|(k, _)| k.ends_with("--dark.png")) {
        assert_eq!(after.get(key), Some(hash));
    }
    assert!(f.compare().unwrap().passed);
}

#[test]
fn removed_whole_journey_fails_closed() {
    let f = sequences();
    establish(&f);
    f.manifest(|m| {
        m["sequences"] = json!([]);
    });
    assert!(f.compare().is_err());
}

#[test]
fn added_frame_and_shortened_timeline_are_reviewable() {
    let f = sequences();
    establish(&f);
    let image = "sequence-story-60hz-frame-0002--light";
    f.image(image, 2, 3, [40; 4]);
    f.manifest(|m| {
        m["cases"].as_array_mut().unwrap().push(json!(image));
        m["images"][image] = json!(digest(&f.run.join(format!("{image}.png"))).unwrap());
        m["sequences"][0]["ticks"][2]["drawn"] = json!(true);
        m["sequences"][0]["ticks"][2]["image"] = json!(image);
        m["sequences"][1]["ticks"].as_array_mut().unwrap().pop();
    });
    let r = f.compare().unwrap();
    assert!(r.errors.is_empty());
    assert_eq!(r.sequences[0].first_difference, Some(2));
    assert_eq!(r.sequences[1].first_difference, Some(2));
    assert!(f.doc().contains("im Kandidaten fehlend"));
    assert!(f
        .report
        .join(format!(
            "sequence-diff/{image}--vs--sequence-story-60hz-frame-0001--light.png"
        ))
        .exists());
    f.accept(&ids(&f), false).unwrap();
    assert!(f.compare().unwrap().passed);
}

#[test]
fn pixel_change_precedes_metadata_difference() {
    let f = sequences();
    establish(&f);
    let c = &f.cases[0];
    f.image(c, 2, 3, [99; 4]);
    f.manifest(|m| {
        m["images"][c] = json!(digest(&f.run.join(format!("{c}.png"))).unwrap());
        m["sequences"][0]["ticks"][2]["events"] = json!(["late"]);
    });
    let r = f.compare().unwrap();
    assert_eq!(r.sequences[0].first_difference, Some(0));
}

#[test]
fn timeline_tampering_after_review_invalidates_acceptance() {
    let f = sequences();
    f.compare().unwrap();
    f.manifest(|m| m["sequences"][0]["ticks"][2]["events"] = json!(["late"]));
    fails(f.accept(&ids(&f), false), "Capture changed");
    assert!(!f.base.exists());
}

#[test]
fn adoption_cannot_hide_unselected_timeline_drift() {
    let f = sequences();
    establish(&f);
    f.manifest(|m| {
        m["environment"]["os"] = json!("new");
        m["sequences"][0]["ticks"][2]["events"] = json!(["late"]);
    });
    f.compare().unwrap();
    fails(f.accept(&[], true), "Unreviewed sequence");
    f.accept(&ids(&f)[..1], true).unwrap();
    assert!(f.compare().unwrap().passed);
}

#[test]
fn sequence_report_has_playback_and_escaped_state() {
    let f = sequences();
    f.manifest(|m| {
        m["sequences"][0]["ticks"][2]["state"] = json!("</script><script>alert(1)</script>")
    });
    f.compare().unwrap();
    let doc = f.doc();
    for text in [
        "seq-play",
        "seq-prev",
        "seq-next",
        "seq-slider",
        "seq-first",
        "voriges Bild gehalten",
        "Freigabe ausstehend",
        "&lt;/script&gt;",
    ] {
        assert!(doc.contains(text), "{text}");
    }
    assert_eq!(doc.matches("<script").count(), 1);
}
