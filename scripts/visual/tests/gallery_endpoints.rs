//! Portable requirements only: no GPUI, Tart, input or native pixel claim.
use bbr_visual::{gallery::Plan, gallery_selection::Schedule};
use cargo_gpui_visual::{config::Config, selection::Selection, store};
use serde_json::{json, Value};
use std::{fs, path::PathBuf, process::Command};

const BASE: &str = "gallery-16-state-00-standard-normal";
const ENDPOINTS: [&str; 3] = [
    "initial-unchecked",
    "checked-after-click",
    "unchecked-after-second-click",
];

fn plan() -> Value {
    let rest = json!({"base":BASE,"label":"Checkbox","state":"default",
        "group":"Core controls","ranges":[[0,2],[2,5]],"dark_ranges":[[0,1],[1,5]]});
    let mut cases = vec![rest.clone()];
    for endpoint in ENDPOINTS {
        let mut case = rest.clone();
        case["base"] = json!(format!("{BASE}-endpoint-{endpoint}"));
        case["endpoint"] = json!(endpoint);
        cases.push(case);
    }
    cases.push(
        json!({"base":"gallery-00-state-00-standard-normal","label":"Tokens",
        "state":"default","group":"Foundations","ranges":[[0,10]]}),
    );
    json!({"schema":2,"cases":cases})
}

struct Fixture {
    _temp: tempfile::TempDir,
    repo: PathBuf,
    config: Config,
    sources: store::Hashes,
    selection: Selection,
}
impl Fixture {
    fn new(plan: Value, selected: &[&str]) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().canonicalize().unwrap();
        assert!(Command::new("git")
            .args(["init", "-q"])
            .arg(&repo)
            .status()
            .unwrap()
            .success());
        store::write_json(&repo.join("plan.json"), &plan).unwrap();
        let mut pages = Vec::new();
        for case in plan["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["label"] == "Checkbox")
        {
            for page in 0..case["ranges"].as_array().unwrap().len() {
                pages.push(format!("{}-page-{page:02}", case["base"].as_str().unwrap()));
            }
        }
        store::write_json(&repo.join("stories.json"), &json!({"schema":1,"stories":[
            {"id":"gallery-16","label":"Checkbox","cases":pages},
            {"id":"gallery-00","label":"Tokens","cases":["gallery-00-state-00-standard-normal-page-00"]}
        ]})).unwrap();
        let config = Config::from_toml("schema=1\ncapture=['never']\nbaselines='refs'\nreport='review'\nruns='runs'\nstory_catalog='stories.json'\n[source_policy]\nscopes=['plan.json','stories.json']\nsuffixes=['.json']\n").unwrap();
        let selection = Selection::resolve(
            &repo,
            &config,
            &selected.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        )
        .unwrap();
        let sources = store::source_hashes(&repo, &config.source_policy, None).unwrap();
        Self {
            _temp: temp,
            repo,
            config,
            selection,
            sources,
        }
    }
    fn schedule(&self) -> anyhow::Result<Schedule> {
        Schedule::resolve(&self.repo, &self.sources, "plan.json", &self.selection)
    }
}

#[test]
fn typed_endpoints_extend_static_requirements_without_faking_selector_states() {
    let f = Fixture::new(plan(), &["gallery-00", "gallery-16"]);
    let schedule = f.schedule().unwrap();
    assert_eq!(schedule.story_ids(), &[16, 0]);
    assert_eq!(schedule.plan().schema, 2);
    assert_eq!(schedule.plan().cases.len(), 5);
    assert_eq!(schedule.selection().stories[0].cases.len(), 8);
    assert_eq!(schedule.plan().cases[0].endpoint, None);
    for (case, name) in schedule.plan().cases[1..4].iter().zip(ENDPOINTS) {
        assert_eq!(case.endpoint.unwrap().id(), name);
        assert_eq!(case.state, "default");
        assert_eq!(case.dark_ranges, Some(vec![[0, 1], [1, 5]]));
    }
    assert!(schedule
        .plan()
        .require_static_capture()
        .unwrap_err()
        .to_string()
        .contains("endpoint execution/evidence is not wired"));
    let f = Fixture::new(plan(), &["gallery-00"]);
    let static_schedule = f.schedule().unwrap();
    assert_eq!(static_schedule.story_ids(), &[0]);
    static_schedule.plan().require_static_capture().unwrap();
    assert!(!f.repo.join("refs").exists() && !f.repo.join("review").exists());
}

#[test]
fn malformed_endpoint_plans_are_rejected_independently_of_the_catalog() {
    for fault in [
        "schema1",
        "unknown-schema",
        "unknown-endpoint",
        "missing",
        "extra",
        "reorder",
        "orphan",
        "identity",
        "untyped",
        "story",
        "selector",
        "state-label",
        "label",
        "group",
        "ranges",
        "dark-ranges",
        "missing-appearance-page",
        "empty-appearance",
        "noncanonical",
        "gap",
        "duplicate",
        "unknown-field",
    ] {
        let mut p = plan();
        match fault {
            "schema1" => p["schema"] = json!(1),
            "unknown-schema" => p["schema"] = json!(3),
            "unknown-endpoint" => p["cases"][1]["endpoint"] = json!("arbitrary"),
            "missing" => {
                p["cases"].as_array_mut().unwrap().remove(2);
            }
            "extra" => {
                let c = p["cases"][3].clone();
                p["cases"].as_array_mut().unwrap().insert(4, c);
            }
            "reorder" => p["cases"].as_array_mut().unwrap().swap(1, 2),
            "orphan" => {
                p["cases"].as_array_mut().unwrap().remove(0);
            }
            "identity" => {
                p["cases"][1]["base"] = json!(format!("{BASE}-endpoint-checked-after-click"))
            }
            "untyped" => {
                p["cases"][1].as_object_mut().unwrap().remove("endpoint");
            }
            "story" | "selector" => {
                for case in p["cases"].as_array_mut().unwrap().iter_mut().take(4) {
                    let from = if fault == "story" {
                        "gallery-16"
                    } else {
                        "state-00"
                    };
                    let to = if fault == "story" {
                        "gallery-17"
                    } else {
                        "state-01"
                    };
                    case["base"] = json!(case["base"].as_str().unwrap().replace(from, to));
                }
            }
            "state-label" => p["cases"][1]["state"] = json!("checked"),
            "label" => p["cases"][1]["label"] = json!("Other"),
            "group" => p["cases"][1]["group"] = json!("Other"),
            "ranges" => p["cases"][1]["ranges"] = json!([[0, 1], [1, 5]]),
            "dark-ranges" => p["cases"][1]["dark_ranges"] = json!([[0, 2], [2, 5]]),
            "missing-appearance-page" | "empty-appearance" | "noncanonical" => {
                for case in p["cases"].as_array_mut().unwrap().iter_mut().take(4) {
                    match fault {
                        "missing-appearance-page" => case["dark_ranges"] = json!([[0, 5]]),
                        "empty-appearance" => case["dark_ranges"] = json!([]),
                        _ => {
                            case["base"] = json!(case["base"]
                                .as_str()
                                .unwrap()
                                .replace("gallery-16", "gallery-016"))
                        }
                    }
                }
            }
            "gap" => p["cases"][0]["ranges"] = json!([[0, 2], [3, 5]]),
            "duplicate" => {
                let c = p["cases"][0].clone();
                p["cases"].as_array_mut().unwrap().push(c);
            }
            "unknown-field" => p["cases"][1]["callback"] = json!("set-checked"),
            _ => unreachable!(),
        }
        let result = serde_json::from_value::<Plan>(p)
            .and_then(|p| p.validate().map_err(serde::de::Error::custom));
        assert!(result.is_err(), "{fault}");
    }
}

#[test]
fn catalog_cannot_omit_reorder_or_invent_endpoint_pages() {
    for fault in ["omit-rest", "omit-endpoint", "reorder", "invent"] {
        let mut f = Fixture::new(plan(), &["gallery-16"]);
        let path = f.repo.join("stories.json");
        let mut catalog: Value = store::read_json(&path).unwrap();
        let pages = catalog["stories"][0]["cases"].as_array_mut().unwrap();
        match fault {
            "omit-rest" => {
                pages.remove(0);
            }
            "omit-endpoint" => {
                pages.remove(3);
            }
            "reorder" => pages.swap(2, 4),
            "invent" => pages.push(json!(format!("{BASE}-endpoint-disabled-page-00"))),
            _ => unreachable!(),
        }
        store::write_json(&path, &catalog).unwrap();
        f.selection = Selection::resolve(&f.repo, &f.config, &["gallery-16".into()]).unwrap();
        f.sources
            .insert("stories.json".into(), store::digest(&path).unwrap());
        let error = f.schedule().unwrap_err();
        assert!(
            error.to_string().contains("every authored Gallery"),
            "{fault}: {error:#}"
        );
    }
}

#[test]
fn changed_endpoint_requirements_cannot_rebind_an_existing_schedule() {
    let mut f = Fixture::new(plan(), &["gallery-16"]);
    let schedule = f.schedule().unwrap();
    let mut p = plan();
    for c in p["cases"].as_array_mut().unwrap().iter_mut().take(4) {
        c["ranges"] = json!([[0, 1], [1, 5]]);
    }
    store::write_json(&f.repo.join("plan.json"), &p).unwrap();
    assert!(schedule.check_sources(&f.repo, &f.sources).is_err());
    f.sources.insert(
        "plan.json".into(),
        store::digest(&f.repo.join("plan.json")).unwrap(),
    );
    assert!(schedule.check_sources(&f.repo, &f.sources).is_err());
    f.schedule().unwrap(); // A new requirement can resolve, but still cannot execute.
}

#[test]
fn endpoint_names_and_synthetic_pngs_cannot_bypass_unwired_evidence() {
    let f = Fixture::new(plan(), &["gallery-16"]);
    let schedule = f.schedule().unwrap();
    let raw = f.repo.join("raw");
    fs::create_dir(&raw).unwrap();
    let png = f.repo.join("synthetic.png");
    image::RgbaImage::new(2400, 2000).save(&png).unwrap();
    let mut inventory = String::new();
    let mut coverage = String::new();
    for case in &schedule.plan().cases {
        for mode in ["light", "dark"] {
            let id = format!("{}--{mode}", case.base);
            inventory += &format!("{id}\t{}\t{}\n", case.label, case.state);
            let ranges = if mode == "dark" {
                case.dark_ranges.as_ref().unwrap_or(&case.ranges)
            } else {
                &case.ranges
            };
            for (page, [start, end]) in ranges.iter().enumerate() {
                let raw_id = format!("{id}-page-{page:02}");
                fs::copy(&png, raw.join(format!("{raw_id}.png"))).unwrap();
                coverage += &format!(
                    "{raw_id}\t{id}\t{}\t{}\t{start}\t{end}\t{}\n",
                    case.label,
                    case.state,
                    ranges.last().unwrap()[1]
                );
            }
        }
    }
    fs::write(raw.join("coverage.tsv"), &coverage).unwrap();
    for result in [
        bbr_visual::gallery::validate_and_prepare(&raw, schedule.plan(), &inventory),
        schedule.validate_and_prepare(&f.repo, &f.sources, &raw, &inventory),
    ] {
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("endpoint execution/evidence is not wired"));
    }
    assert_eq!(
        fs::read_to_string(raw.join("coverage.tsv")).unwrap(),
        coverage
    );
    assert_eq!(fs::read_dir(&raw).unwrap().count(), 17);
    assert!(raw.join(format!("{BASE}--light-page-00.png")).exists());
    assert!(!raw.join("inventory.json").exists() && !raw.join("selection.json").exists());
}

#[test]
fn schema1_and_schema2_static_plans_preserve_all_historical_requirements() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("catalogs/gallery-plan.json");
    let mut historical: Plan = store::read_json(&path).unwrap();
    assert_eq!(historical.schema, 1);
    assert_eq!(historical.cases.len(), 630);
    assert_eq!(
        historical
            .cases
            .iter()
            .map(|c| c.ranges.len())
            .sum::<usize>(),
        936
    );
    historical.require_static_capture().unwrap();
    historical.schema = 2;
    historical.require_static_capture().unwrap();
}

#[test]
fn selected_requests_and_stored_reviews_refuse_endpoints_before_publication() {
    use bbr_visual::{payload::Payload, review, selected_request::Request};
    use cargo_gpui_visual::store::Manifest;
    use std::sync::atomic::AtomicBool;
    for selected in [false, true] {
        let f = Fixture::new(plan(), &["gallery-16"]);
        let plan_path = "scripts/visual/catalogs/gallery-plan.json";
        fs::create_dir_all(f.repo.join("scripts/visual/catalogs")).unwrap();
        fs::copy(f.repo.join("plan.json"), f.repo.join(plan_path)).unwrap();
        fs::write(f.repo.join("visual-gallery.toml"),
            "schema=1\ncapture=['never']\nbaselines='refs'\nreport='review'\nruns='runs'\nstory_catalog='stories.json'\n[source_policy]\nscopes=['visual-gallery.toml','stories.json','scripts/visual/catalogs']\nsuffixes=['.json','.toml']\n").unwrap();
        let config = Config::load(&f.repo.join("visual-gallery.toml")).unwrap();
        let sources = store::source_hashes(&f.repo, &config.source_policy, None).unwrap();
        let payload = Payload {
            schema: 1,
            suite: "gallery".into(),
            repo: f.repo.clone(),
            sources: sources.clone(),
            files: vec![],
        };
        let error = Request::new(&payload, &"a".repeat(64), f.selection.clone()).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("endpoint execution/evidence is not wired"),
            "{error:#}"
        );
        // The same schema permits a static-only request, never implicit endpoints.
        let static_selection =
            Selection::resolve(&f.repo, &config, &["gallery-00".into()]).unwrap();
        Request::new(&payload, &"a".repeat(64), static_selection).unwrap();

        let run = f.repo.join("stored-run");
        fs::create_dir(&run).unwrap();
        let cases = f.selection.stories[0]
            .cases
            .iter()
            .flat_map(|c| [format!("{c}--light"), format!("{c}--dark")])
            .collect::<Vec<_>>();
        for case in &cases {
            image::RgbaImage::new(1, 1)
                .save(run.join(format!("{case}.png")))
                .unwrap();
        }
        let selection = selected.then_some(f.selection.clone());
        if let Some(s) = &selection {
            store::write_json(&run.join("selection.json"), s).unwrap();
        }
        let manifest = Manifest {
            schema: if selected { 2 } else { 1 },
            images: cases
                .iter()
                .map(|c| {
                    (
                        c.clone(),
                        store::digest(&run.join(format!("{c}.png"))).unwrap(),
                    )
                })
                .collect(),
            cases,
            sources,
            source_policy: config.source_policy,
            selection,
            sequences: vec![],
            environment: json!({"layer":"gpui","evidence":"synthetic-not-input"}),
        };
        store::write_json(&run.join("capture.json"), &manifest).unwrap();
        store::load_manifest(&run).unwrap();
        store::validate_images(&run, &manifest).unwrap();
        let report = f.repo.join("reports/visual/endpoint-review");
        let error = review::compare(
            &f.repo,
            &review::Request {
                gallery: Some(run.clone()),
                browser: None,
                report: report.clone(),
            },
            &AtomicBool::new(false),
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("endpoint execution/evidence is not wired"),
            "{error:#}"
        );
        assert!(!report.exists() && !f.repo.join("refs").exists());
        assert_eq!(
            store::read_json::<Manifest>(&run.join("capture.json"))
                .unwrap()
                .images,
            manifest.images
        );
    }
}
