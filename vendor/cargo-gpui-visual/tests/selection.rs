//! Portable selection protocol using disposable PNGs, never consumer/native data.
use cargo_gpui_visual::{
    accept,
    capture::{self, Paths},
    collection, compare,
    config::Config,
    selection::{Catalog, Selection, Story, Timeline, ENVIRONMENT},
    sequence::{Sequence, Tick},
    store,
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::AtomicBool,
};

#[path = "selection/adversarial.rs"]
mod adversarial;

struct Fixture {
    _temp: tempfile::TempDir,
    repo: PathBuf,
    config: Config,
}
fn catalog() -> Catalog {
    Catalog {
        schema: 1,
        stories: vec![
            Story {
                id: "tokens".into(),
                label: "Tokens".into(),
                cases: vec!["tokens-page-01".into(), "tokens-page-02".into()],
                sequences: vec![],
            },
            Story {
                id: "opening".into(),
                label: "Opening".into(),
                cases: vec![],
                sequences: vec![Timeline {
                    base: "sequence-opening-60hz".into(),
                    hz: 60,
                    ticks: 3,
                }],
            },
            Story {
                id: "other".into(),
                label: "Other".into(),
                cases: vec!["other".into()],
                sequences: vec![],
            },
        ],
    }
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().canonicalize().unwrap();
        assert!(Command::new("git")
            .args(["init", "-q"])
            .arg(&repo)
            .status()
            .unwrap()
            .success());
        store::write_json(&repo.join("stories.json"), &catalog()).unwrap();
        fs::write(repo.join("source.rs"), "// production fixture\n").unwrap();
        let mut config = Config::from_toml("schema=1\ncapture=['unused']\nenvironment_command=['sh','-c','printf \\\"unused\\\"']\nbaselines='refs'\nreport='review'\nruns='runs'\nstory_catalog='stories.json'\n[source_policy]\nscopes=['source.rs','stories.json','visual.toml']\nsuffixes=['.rs','.json','.toml']\n").unwrap();
        config.capture = vec![
            std::env::current_exe().unwrap().to_string_lossy().into(),
            "--exact".into(),
            "raw_producer_fixture".into(),
            "--nocapture".into(),
        ];
        config.environment_command = Some(vec![
            "sh".into(),
            "-c".into(),
            "test -z \"${GPUI_VISUAL_SELECTION+x}\" || exit 9; printf '%s' '{\"layer\":\"gpui\"}'"
                .into(),
        ]);
        let value = json!({"schema":1,"capture":config.capture,"environment_command":config.environment_command,"baselines":"refs","report":"review","runs":"runs","story_catalog":"stories.json","source_policy":{"scopes":["source.rs","stories.json","visual.toml"],"suffixes":[".rs",".json",".toml"]}});
        fs::write(repo.join("visual.toml"), toml::to_string(&value).unwrap()).unwrap();
        Self {
            _temp: temp,
            repo,
            config,
        }
    }
    fn capture(
        &self,
        name: &str,
        stories: &[&str],
        failure: &str,
    ) -> anyhow::Result<store::Manifest> {
        let config = if stories.is_empty() {
            self.config.clone()
        } else {
            self.config.select(
                &self.repo,
                &stories.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            )?
        };
        let run = self.repo.join(name);
        capture::capture_local(
            Paths {
                repo: &self.repo,
                run: &run,
                baselines: &self.repo.join("refs"),
                report: &self.repo.join("review"),
            },
            &config,
            &AtomicBool::new(false),
            |output| emit(output, config.selection.as_ref(), failure),
            || Ok(json!({"layer":"gpui"})),
        )
    }
    fn cli(&self, args: &[&str]) -> std::process::Output {
        cargo_gpui_visual::process::run(
            Command::new(env!("CARGO_BIN_EXE_cargo-gpui-visual"))
                .arg("--repo")
                .arg(&self.repo)
                .args(args),
            &cargo_gpui_visual::process::Limits {
                timeout: std::time::Duration::from_secs(15),
                ..Default::default()
            },
            &AtomicBool::new(false),
            true,
        )
        .map(|o| std::process::Output {
            status: o.status,
            stdout: o.stdout,
            stderr: vec![],
        })
        .unwrap()
    }
    fn establish(&self) {
        let m = self.capture("full", &[], "none").unwrap();
        compare::compare(
            &self.repo,
            &self.repo.join("full"),
            &self.repo.join("refs"),
            &self.repo.join("review"),
        )
        .unwrap();
        let mut ids: Vec<_> = m
            .cases
            .into_iter()
            .filter(|c| !c.starts_with("sequence-"))
            .collect();
        ids.extend(m.sequences.into_iter().map(|s| s.id));
        accept::accept(
            &self.repo,
            &self.repo.join("full"),
            &self.repo.join("refs"),
            &self.repo.join("review"),
            &ids,
            false,
        )
        .unwrap();
    }
}
fn emit(output: &Path, selection: Option<&Selection>, failure: &str) -> anyhow::Result<()> {
    fs::create_dir(output)?;
    let all = catalog();
    let stories = if failure == "ignored" {
        &all.stories
    } else {
        selection.map_or(&all.stories, |s| &s.stories)
    };
    let mut cases = vec![];
    let mut sequences = vec![];
    let mut invoked = vec![];
    for story in stories {
        // Consumer-owned observation at the render-schedule boundary, not inferred
        // from emitted PNGs. Portable validation cannot prove hidden render work.
        invoked.push(story.id.clone());
        for mode in ["light", "dark"] {
            for base in &story.cases {
                cases.push(format!("{base}--{mode}"));
            }
            for required in &story.sequences {
                let mut ticks = vec![];
                for tick in 0..required.ticks {
                    let image = format!("{}-frame-{:04}--{mode}", required.base, tick.min(1));
                    if tick < 2 {
                        cases.push(image.clone());
                    }
                    ticks.push(Tick {
                        tick: tick as u64,
                        time_ns: tick as u64 * 1_000_000_000 / required.hz,
                        drawn: tick < 2,
                        dirty_before_callbacks: tick == 1,
                        callbacks_pending: false,
                        image,
                        events: vec![],
                        state: json!({"ready":tick>0}),
                    });
                }
                if failure == "timeline" {
                    ticks.pop();
                }
                sequences.push(Sequence {
                    id: format!("{}--{mode}", required.base),
                    hz: required.hz,
                    ticks,
                });
            }
        }
    }
    store::write_json(&output.join("invoked-stories.json"), &invoked)?;
    if failure == "missing" {
        cases.pop();
    }
    if failure == "page" {
        cases.retain(|c| !c.contains("page-02"));
    }
    if failure == "appearance" {
        cases.retain(|c| !c.ends_with("--dark"));
    }
    if failure == "extra" {
        cases.push("extra--light".into());
        cases.push("extra--dark".into());
    }
    for case in &cases {
        image::RgbaImage::from_pixel(2, 3, image::Rgba([1, 2, 3, 255]))
            .save(store::image_path(output, case)?)?;
    }
    store::write_json(&output.join("inventory.json"), &cases)?;
    store::write_json(&output.join("sequences.json"), &sequences)?;
    if let Some(selection) = selection {
        if failure != "receipt" {
            let mut receipt = selection.clone();
            if failure == "mismatched" {
                receipt.stories[0].label = "Wrong story".into();
            }
            store::write_json(&output.join("selection.json"), &receipt)?;
        }
    }
    Ok(())
}
#[test]
fn raw_producer_fixture() {
    let Some(output) = std::env::var_os("GPUI_VISUAL_OUTPUT") else {
        return;
    };
    let selection: Option<Selection> = std::env::var(ENVIRONMENT)
        .ok()
        .map(|s| serde_json::from_str(&s).unwrap());
    emit(Path::new(&output), selection.as_ref(), "none").unwrap();
}
#[test]
fn list_is_read_only_and_exact_multiple_selection_reaches_the_real_cli_producer() {
    let f = Fixture::new();
    let list = f.cli(&["list"]);
    assert!(list.status.success());
    let read: Catalog = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(read, catalog());
    assert!(!f.repo.join("runs").exists());
    assert!(!f.repo.join("refs").exists());
    for args in [
        vec!["check", "--story", "typo", "--report", "bad"],
        vec![
            "check", "--story", "tokens", "--story", "tokens", "--report", "bad",
        ],
        vec!["check", "--story", "tokens"],
        vec!["compare", "--story", "tokens"],
    ] {
        assert_eq!(f.cli(&args).status.code(), Some(2));
        assert!(!f.repo.join("runs").exists());
    }
    let result = f.cli(&[
        "check",
        "--story",
        "opening",
        "--story",
        "tokens",
        "--run",
        "selected",
        "--report",
        "selected-review",
    ]);
    assert_eq!(result.status.code(), Some(1));
    let m = store::load_manifest(&f.repo.join("selected")).unwrap();
    assert_eq!(m.schema, 2);
    let selected = m.selection.unwrap();
    assert_eq!(
        selected
            .stories
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        ["tokens", "opening"]
    );
    assert_eq!(m.cases.len(), 8);
    assert_eq!(m.sequences.len(), 2);
    assert!(!m.cases.iter().any(|c| c.starts_with("other")));
    assert!(!f.repo.join("refs").exists());
    let stdout = String::from_utf8(result.stdout).unwrap();
    assert!(stdout.contains("TEILLAUF"));
    assert!(!stdout.contains("--case"));
}
#[test]
fn producer_selection_invokes_only_requested_stories_and_preserves_full_scope_pixels() {
    let f = Fixture::new();
    let full = f.capture("all-invocations", &[], "none").unwrap();
    let all: Vec<String> =
        store::read_json(&f.repo.join("all-invocations/invoked-stories.json")).unwrap();
    assert_eq!(all, ["tokens", "opening", "other"]);
    for (name, ids, images, sequences) in [
        ("static", vec!["other"], 2, 0),
        ("pages", vec!["tokens"], 4, 0),
        ("timeline", vec!["opening"], 4, 2),
    ] {
        let result = f.cli(&[
            "check",
            "--story",
            ids[0],
            "--run",
            name,
            "--report",
            &format!("{name}-review"),
            "--json",
        ]);
        assert_eq!(result.status.code(), Some(1));
        let response: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(response["data"]["scope"]["kind"], "selected");
        let invoked: Vec<String> =
            store::read_json(&f.repo.join(name).join("invoked-stories.json")).unwrap();
        assert_eq!(invoked, ids);
        let selected = store::load_manifest(&f.repo.join(name)).unwrap();
        assert_eq!(selected.cases.len(), images);
        assert_eq!(selected.sequences.len(), sequences);
        for case in &selected.cases {
            assert_eq!(selected.images[case], full.images[case]);
        }
        for timeline in &selected.sequences {
            assert_eq!(
                Some(timeline),
                full.sequences.iter().find(|s| s.id == timeline.id)
            );
        }
    }
    assert!(!f.repo.join("refs").exists());
}

#[test]
fn selected_report_never_offers_an_acceptance_command() {
    let f = Fixture::new();
    f.capture("selected", &["tokens"], "none").unwrap();
    let review = compare::compare(
        &f.repo,
        &f.repo.join("selected"),
        &f.repo.join("refs"),
        &f.repo.join("selected-review"),
    )
    .unwrap();
    assert!(review
        .results
        .iter()
        .any(|r| r.status == "missing-baseline"));
    let html = fs::read_to_string(f.repo.join("selected-review/index.html")).unwrap();
    assert!(html.contains("Teillauf: Übernahme gesperrt"));
    assert!(!html.contains("class=\"command\""));
}

#[test]
fn selected_comparison_ignores_unselected_references_without_granting_suite_pass_or_acceptance() {
    let f = Fixture::new();
    f.establish();
    let before = store::baseline_state(&f.repo.join("refs")).unwrap();
    f.capture("selected", &["tokens"], "none").unwrap();
    let review = compare::compare(
        &f.repo,
        &f.repo.join("selected"),
        &f.repo.join("refs"),
        &f.repo.join("selected-review"),
    )
    .unwrap();
    assert!(review.passed);
    assert!(review.errors.is_empty());
    assert!(review.selection.is_some());
    assert_eq!(review.results.len(), 4);
    assert!(review.sequences.is_empty());
    let html = fs::read_to_string(f.repo.join("selected-review/index.html")).unwrap();
    assert!(html.contains("TEILLAUF · AUSWAHL PASS"));
    assert!(html.contains("Kein vollständiger Suite-Nachweis"));
    assert!(!html.contains("sequence-expected/"));
    for adopt in [false, true] {
        assert!(accept::accept(
            &f.repo,
            &f.repo.join("selected"),
            &f.repo.join("refs"),
            &f.repo.join("selected-review"),
            &["tokens-page-01--light".into()],
            adopt
        )
        .unwrap_err()
        .to_string()
        .contains("development-only"));
    }
    assert_eq!(before, store::baseline_state(&f.repo.join("refs")).unwrap());
    let suite = collection::Suite {
        id: "test".into(),
        label: "Test".into(),
        repo: f.repo.clone(),
        run: f.repo.join("selected"),
        baselines: f.repo.join("refs"),
        catalog: "stories.json".into(),
    };
    let result = collection::compare(&[suite], &f.repo.join("collection")).unwrap();
    assert!(!result.passed);
    assert!(result.suites[0].errors[0].contains("not full-suite"));
}
#[test]
fn selected_sequences_remain_whole_and_full_capture_removal_still_fails() {
    let f = Fixture::new();
    f.establish();
    f.capture("selected", &["opening"], "none").unwrap();
    let r = compare::compare(
        &f.repo,
        &f.repo.join("selected"),
        &f.repo.join("refs"),
        &f.repo.join("selected-review"),
    )
    .unwrap();
    assert!(r.passed);
    assert_eq!(r.sequences.len(), 2);
    assert_eq!(r.results.len(), 4);
    let path = f.repo.join("selected/capture.json");
    let mut manifest: store::Manifest = store::read_json(&path).unwrap();
    manifest.selection = None;
    store::write_json(&path, &manifest).unwrap();
    assert!(store::load_manifest(&f.repo.join("selected")).is_err());
    // A genuinely full-schema capture still diagnoses omitted baseline cases.
    manifest.schema = 1;
    store::write_json(&path, &manifest).unwrap();
    let r = compare::compare(
        &f.repo,
        &f.repo.join("selected"),
        &f.repo.join("refs"),
        &f.repo.join("full-review"),
    )
    .unwrap();
    assert!(!r.passed);
    assert!(r.errors.iter().any(|s| s.contains("absent")));
}
#[test]
fn producer_ignoring_selection_missing_page_receipt_or_tick_cannot_publish_a_seal() {
    for (failure, stories) in [
        ("ignored", vec!["tokens"]),
        ("missing", vec!["tokens"]),
        ("receipt", vec!["tokens"]),
        ("timeline", vec!["opening"]),
        ("page", vec!["tokens"]),
        ("appearance", vec!["tokens"]),
        ("extra", vec!["tokens"]),
        ("mismatched", vec!["tokens"]),
    ] {
        let f = Fixture::new();
        assert!(
            f.capture("invalid", &stories, failure).is_err(),
            "{failure}"
        );
        assert!(!f.repo.join("invalid/capture.json").exists());
        assert!(!f.repo.join("refs").exists());
    }
}
#[test]
fn unsealed_or_changed_catalog_and_forged_selection_fail_closed() {
    let mut f = Fixture::new();
    f.config
        .source_policy
        .scopes
        .retain(|s| s != "stories.json");
    assert!(f.capture("invalid", &["tokens"], "none").is_err());
    assert!(!f.repo.join("invalid").exists());
    f.config.source_policy.scopes.push("stories.json".into());
    f.capture("selected", &["tokens"], "none").unwrap();
    let path = f.repo.join("selected/capture.json");
    let mut manifest: store::Manifest = store::read_json(&path).unwrap();
    manifest.selection.as_mut().unwrap().stories[0].label = "Forged label".into();
    store::write_json(&path, &manifest).unwrap();
    assert!(compare::compare(
        &f.repo,
        &f.repo.join("selected"),
        &f.repo.join("refs"),
        &f.repo.join("review")
    )
    .is_err());
    fs::write(f.repo.join("stories.json"), "{}").unwrap();
    assert!(compare::compare(
        &f.repo,
        &f.repo.join("selected"),
        &f.repo.join("refs"),
        &f.repo.join("review")
    )
    .is_err());
}
#[test]
fn catalog_rejects_duplicates_empty_stories_frames_and_invalid_timelines() {
    for mode in 0..5 {
        let mut c = catalog();
        match mode {
            0 => c.stories.push(c.stories[0].clone()),
            1 => c.stories[0].cases.clear(),
            2 => c.stories[0].cases = vec!["sequence-opening-frame-0001".into()],
            3 => c.stories[1].sequences[0].ticks = 0,
            _ => {
                let duplicate = c.stories[0].cases[0].clone();
                c.stories[2].cases.push(duplicate);
            }
        }
        assert!(c.validate().is_err());
    }
}
