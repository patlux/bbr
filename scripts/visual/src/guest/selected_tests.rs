//! Inject only producer execution; identity/install checks remain in dispatch.
use super::*;
use crate::{payload::Payload, Suite};
use serde_json::json;
use std::{collections::BTreeMap, os::unix::process::ExitStatusExt, process::ExitStatus};

struct Fixture {
    _temp: tempfile::TempDir,
    payload: Payload,
    work: std::path::PathBuf,
    digest: String,
    png: std::path::PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let repo = root.join("repo");
        let work = root.join("work");
        fs::create_dir(&repo).unwrap();
        fs::create_dir(&work).unwrap();
        fs::create_dir_all(repo.join("scripts/visual/catalogs")).unwrap();
        fs::write(repo.join(Suite::Gallery.config()), "schema=1\ncapture=['never']\nbaselines='refs'\nreport='review'\nruns='runs'\nstory_catalog='stories.json'\n[source_policy]\nscopes=['stories.json']\nsuffixes=['.json']\n").unwrap();
        store::write_json(&repo.join(PLAN), &json!({"schema":1,"cases":[
            {"base":"gallery-44-state-00-standard-normal","label":"Sidebar","state":"default","group":"Fixture","ranges":[[0,1],[1,2]]},
            {"base":"gallery-00-state-00-standard-normal","label":"Tokens","state":"default","group":"Fixture","ranges":[[0,1]]}
        ]})).unwrap();
        store::write_json(&repo.join("stories.json"), &json!({"schema":1,"stories":[
            {"id":"gallery-44","label":"Sidebar","cases":["gallery-44-state-00-standard-normal-page-00","gallery-44-state-00-standard-normal-page-01"]},
            {"id":"gallery-00","label":"Tokens","cases":["gallery-00-state-00-standard-normal-page-00"]}
        ]})).unwrap();
        let sources = [PLAN, "stories.json", Suite::Gallery.config()]
            .into_iter()
            .map(|p| (p.into(), store::digest(&repo.join(p)).unwrap()))
            .collect();
        let payload = Payload {
            schema: 1,
            suite: "gallery".into(),
            repo: repo.clone(),
            sources,
            files: vec![],
        };
        let config = Config::load(&repo.join(Suite::Gallery.config())).unwrap();
        let selection =
            Selection::resolve(&repo, &config, &["gallery-00".into(), "gallery-44".into()])
                .unwrap();
        store::write_json(
            &work.join(REQUEST),
            &json!({"schema":1,"payload_sha256":"a".repeat(64),"selection":selection}),
        )
        .unwrap();
        let digest = store::digest(&work.join(REQUEST)).unwrap();
        let png = root.join("synthetic.png");
        image::RgbaImage::new(2400, 2000).save(&png).unwrap();
        Self {
            _temp: temp,
            payload,
            work,
            digest,
            png,
        }
    }
    fn request(&self) -> Request {
        load(&self.payload, &self.work, &"a".repeat(64), &self.digest).unwrap()
    }
    fn inventory(&self) -> String {
        [44, 0, 99]
            .into_iter()
            .flat_map(|id| {
                ["light", "dark"].into_iter().map(move |mode| {
                    format!(
                        "gallery-{id:02}-state-00-standard-normal--{mode}\t{}\tdefault\n",
                        if id == 44 {
                            "Sidebar"
                        } else if id == 0 {
                            "Tokens"
                        } else {
                            "Unselected"
                        }
                    )
                })
            })
            .collect()
    }
    fn emit(&self, command: &Command, id: u64, extra: bool) {
        let args: Vec<_> = command.get_args().collect();
        assert_eq!(args[0], "--capture-gallery");
        assert_eq!(args[2], "--story");
        assert_eq!(args[3], id.to_string().as_str());
        let output = Path::new(args[1]);
        assert!(!output.exists());
        fs::create_dir(output).unwrap();
        let plan: gallery::Plan = store::read_json(&self.payload.repo.join(PLAN)).unwrap();
        let mut rows = String::new();
        for case in &plan.cases {
            if !extra && !case.base.starts_with(&format!("gallery-{id:02}-")) {
                continue;
            }
            for mode in ["light", "dark"] {
                for (page, [start, end]) in case.ranges.iter().enumerate() {
                    let raw = format!("{}--{mode}-page-{page:02}", case.base);
                    fs::copy(&self.png, output.join(format!("{raw}.png"))).unwrap();
                    rows += &format!(
                        "{raw}\t{}--{mode}\t{}\t{}\t{start}\t{end}\t{}\n",
                        case.base,
                        case.label,
                        case.state,
                        case.ranges.last().unwrap()[1]
                    );
                }
            }
        }
        fs::write(output.join("coverage.tsv"), rows).unwrap();
    }
}
fn ok(stdout: Vec<u8>) -> Output {
    Output {
        status: ExitStatus::from_raw(0),
        stdout,
    }
}

#[test]
fn request_is_fixed_bounded_payload_source_and_config_bound() {
    let f = Fixture::new();
    f.request();
    assert!(load(&f.payload, &f.work, &"b".repeat(64), &f.digest).is_err());
    assert!(load(&f.payload, &f.work, &"a".repeat(64), &"0".repeat(64)).is_err());
    let mut browser = f.payload.clone();
    browser.suite = "browser".into();
    assert!(load(&browser, &f.work, &"a".repeat(64), &f.digest).is_err());
    fs::hard_link(f.work.join(REQUEST), f.work.join("alias")).unwrap();
    assert!(load(&f.payload, &f.work, &"a".repeat(64), &f.digest).is_err());
    let f = Fixture::new();
    fs::write(f.payload.repo.join(Suite::Gallery.config()), "changed").unwrap();
    assert!(load(&f.payload, &f.work, &"a".repeat(64), &f.digest).is_err());
    let mut f = Fixture::new();
    fs::write(f.payload.repo.join(Suite::Gallery.config()), "schema=1\ncapture=['never']\nbaselines='refs'\nreport='review'\nruns='runs'\n[source_policy]\nscopes=[]\nsuffixes=[]\n").unwrap();
    f.payload.sources.insert(
        Suite::Gallery.config().into(),
        store::digest(&f.payload.repo.join(Suite::Gallery.config())).unwrap(),
    );
    assert!(load(&f.payload, &f.work, &"a".repeat(64), &f.digest).is_err());
}

#[test]
fn invalid_or_pre_cancelled_request_never_invokes_the_producer() {
    for fault in ["cancel", "stale-request", "raw", "stories", "inventory"] {
        let f = Fixture::new();
        let request = f.request();
        let cancelled = AtomicBool::new(fault == "cancel");
        match fault {
            "stale-request" => fs::write(f.work.join(REQUEST), "{}").unwrap(),
            "raw" => fs::create_dir(f.work.join("raw")).unwrap(),
            "stories" => fs::create_dir(f.work.join("selected-stories")).unwrap(),
            "inventory" => fs::write(f.work.join("live-inventory.tsv"), "preserved").unwrap(),
            _ => (),
        }
        let result = execute(&f.payload, &f.work, &request, &cancelled, |_, _| {
            panic!("producer invoked on {fault}")
        });
        assert!(result.is_err(), "{fault}");
        assert!(!f.work.join("raw/selection.json").exists());
    }
}

#[test]
fn request_rejects_unknown_fields_unsafe_and_oversized_inputs() {
    for fault in [
        "schema",
        "unknown",
        "empty",
        "digest-field",
        "oversized",
        "symlink",
    ] {
        let f = Fixture::new();
        let path = f.work.join(REQUEST);
        let mut request: serde_json::Value = store::read_json(&path).unwrap();
        match fault {
            "schema" => request["schema"] = json!(2),
            "unknown" => request["command"] = json!("arbitrary"),
            "digest-field" => request["digest"] = json!("b".repeat(64)),
            "empty" => request["selection"]["stories"] = json!([]),
            _ => (),
        }
        store::write_json(&path, &request).unwrap();
        if fault == "oversized" {
            fs::OpenOptions::new()
                .write(true)
                .open(&path)
                .unwrap()
                .set_len(8 * 1024 * 1024 + 1)
                .unwrap();
        }
        let digest = store::digest(&path).unwrap();
        if fault == "symlink" {
            fs::rename(&path, f.work.join("other")).unwrap();
            std::os::unix::fs::symlink(f.work.join("other"), &path).unwrap();
        }
        assert!(
            load(&f.payload, &f.work, &"a".repeat(64), &digest).is_err(),
            "{fault}"
        );
    }
}

#[test]
fn actual_guest_schedule_uses_fixed_story_commands_before_output_and_keeps_raw_bytes() {
    let f = Fixture::new();
    let request = f.request();
    let mut calls = Vec::new();
    let mut remaining = Duration::from_secs(600);
    execute(
        &f.payload,
        &f.work,
        &request,
        &AtomicBool::new(false),
        |cmd, limits| {
            assert!(
                limits.timeout <= remaining,
                "one deadline across all producer calls"
            );
            remaining = limits.timeout;
            assert_eq!(
                cmd.get_program(),
                f.payload.repo.join("target/release/bbr-gallery")
            );
            let env: BTreeMap<_, _> = cmd
                .get_envs()
                .filter_map(|(k, v)| {
                    v.map(|v| {
                        (
                            k.to_string_lossy().into_owned(),
                            v.to_string_lossy().into_owned(),
                        )
                    })
                })
                .collect();
            assert_eq!(env["HOME"], "/Users/admin");
            assert_eq!(env["LC_ALL"], "C");
            if calls.is_empty() {
                assert_eq!(cmd.get_args().collect::<Vec<_>>(), ["--capture-inventory"]);
                calls.push(99);
                return Ok(ok(f.inventory().into_bytes()));
            }
            let id = if calls.len() == 1 { 44 } else { 0 };
            f.emit(cmd, id, false);
            calls.push(id);
            Ok(ok(vec![]))
        },
    )
    .unwrap();
    assert_eq!(calls, [99, 44, 0]);
    assert_eq!(
        store::read_json::<Selection>(&f.work.join("raw/selection.json")).unwrap(),
        *request.selection()
    );
    assert_eq!(
        store::digest(&f.png).unwrap(),
        store::digest(
            &f.work
                .join("raw/gallery-44-state-00-standard-normal--light-page-00.png")
        )
        .unwrap()
    );
    assert!(!f.work.join("raw/capture.json").exists());
    assert!(!f.work.join("raw/inventory.json").exists());
}

#[test]
fn failed_inventory_ignored_selection_cancellation_or_source_drift_never_completes() {
    for fault in [
        "inventory",
        "extra-state",
        "duplicate",
        "ignored",
        "missing",
        "hardlink",
        "extra-file",
        "wrong-dimensions",
        "failed",
        "cancel",
        "source",
        "request",
        "after-capture-cancel",
        "after-capture-source",
    ] {
        let f = Fixture::new();
        let request = f.request();
        let cancelled = AtomicBool::new(false);
        let mut calls = 0;
        let result = execute(&f.payload, &f.work, &request, &cancelled, |cmd, _| {
            calls += 1;
            if calls == 1 {
                if fault == "cancel" {
                    cancelled.store(true, Ordering::SeqCst);
                }
                if fault == "source" {
                    fs::write(f.payload.repo.join(PLAN), "{}").unwrap();
                }
                if fault == "request" {
                    fs::write(f.work.join(REQUEST), "{}").unwrap();
                }
                let mut inventory = f.inventory();
                if fault == "extra-state" {
                    inventory += "gallery-44-state-01-standard-normal--light\tSidebar\tnew state\n";
                }
                if fault == "duplicate" {
                    inventory += &f.inventory();
                }
                return Ok(ok(if fault == "inventory" {
                    b"invalid\n".to_vec()
                } else {
                    inventory.into_bytes()
                }));
            }
            assert_eq!(calls, 2, "must stop before second story for {fault}");
            if fault == "failed" {
                return Ok(Output {
                    status: ExitStatus::from_raw(256),
                    stdout: vec![],
                });
            }
            f.emit(cmd, 44, fault == "ignored");
            if fault == "after-capture-cancel" {
                cancelled.store(true, Ordering::SeqCst);
            }
            if fault == "after-capture-source" {
                fs::write(f.payload.repo.join(PLAN), "{}").unwrap();
            }
            let image = f.work.join(
                "selected-stories/story-44/gallery-44-state-00-standard-normal--dark-page-01.png",
            );
            match fault {
                "missing" => fs::remove_file(&image).unwrap(),
                "hardlink" => {
                    fs::remove_file(&image).unwrap();
                    fs::hard_link(&f.png, &image).unwrap();
                }
                "extra-file" => fs::write(
                    f.work.join("selected-stories/story-44/selection.json"),
                    "{}",
                )
                .unwrap(),
                "wrong-dimensions" => image::RgbaImage::new(1, 1).save(&image).unwrap(),
                _ => (),
            }
            Ok(ok(vec![]))
        });
        assert!(result.is_err(), "{fault}");
        assert_eq!(
            calls,
            if [
                "inventory",
                "extra-state",
                "duplicate",
                "cancel",
                "source",
                "request"
            ]
            .contains(&fault)
            {
                1
            } else {
                2
            },
            "{fault}"
        );
        assert!(!f.work.join("raw/selection.json").exists(), "{fault}");
        assert!(!f.work.join("raw/capture.json").exists());
    }
}
