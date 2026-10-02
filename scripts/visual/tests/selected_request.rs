//! Source-bound request construction/loading only; never runs a producer.
use bbr_visual::{payload::Payload, selected_request::Request};
use cargo_gpui_visual::{config::Config, selection::Selection, store};
use serde_json::json;
use std::{fs, path::Path};

fn fixture(repo: &Path) -> (Payload, Selection) {
    fs::create_dir_all(repo.join("scripts/visual/catalogs")).unwrap();
    fs::write(repo.join("visual-gallery.toml"), "schema=1\ncapture=['never']\nbaselines='baselines'\nreport='report'\nruns='runs'\nstory_catalog='stories.json'\n[source_policy]\nscopes=['stories.json']\nsuffixes=['.json']\n").unwrap();
    fs::write(repo.join("stories.json"), json!({"schema":1,"stories":[{"id":"gallery-00","label":"Fixture","cases":["gallery-00-state-00-standard-normal-page-00"]}]}).to_string()).unwrap();
    fs::write(repo.join("scripts/visual/catalogs/gallery-plan.json"), json!({"schema":1,"cases":[{"base":"gallery-00-state-00-standard-normal","label":"Fixture","state":"Default","group":"Fixture","ranges":[[0,1]]}]}).to_string()).unwrap();
    let sources = [
        "visual-gallery.toml",
        "stories.json",
        "scripts/visual/catalogs/gallery-plan.json",
    ]
    .into_iter()
    .map(|name| (name.into(), store::digest(&repo.join(name)).unwrap()))
    .collect();
    let selection = Selection::resolve(
        repo,
        &Config::load(&repo.join("visual-gallery.toml")).unwrap(),
        &["gallery-00".into()],
    )
    .unwrap();
    (
        Payload {
            schema: 1,
            suite: "gallery".into(),
            repo: repo.into(),
            sources,
            files: vec![],
        },
        selection,
    )
}

#[test]
fn host_request_preserves_exact_selection_bytes_and_rejects_rebound_identity() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().canonicalize().unwrap();
    let (payload, selection) = fixture(&repo);
    let digest = "a".repeat(64);
    let request = Request::new(&payload, &digest, selection.clone()).unwrap();
    assert_eq!(request.selection(), &selection);
    let work = repo.join("work");
    fs::create_dir(&work).unwrap();
    fs::write(work.join("selected-request.json"), request.bytes()).unwrap();
    let loaded = Request::load(&payload, &work, &digest, request.digest()).unwrap();
    assert_eq!(loaded.bytes(), request.bytes());
    assert!(Request::load(&payload, &work, &"b".repeat(64), request.digest()).is_err());
    assert!(Request::load(&payload, &work, &digest, &"b".repeat(64)).is_err());
    assert!(Request::new(&payload, "invalid", selection.clone()).is_err());
    let mut browser = payload.clone();
    browser.suite = "browser".into();
    assert!(Request::new(&browser, &digest, selection).is_err());
    fs::write(repo.join("stories.json"), "{}").unwrap();
    assert!(request.check(&payload).is_err());
    assert!(!work.join("capture.json").exists());
}

#[test]
fn invalid_plan_or_config_cannot_construct_a_host_selected_request() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().canonicalize().unwrap();
    let (mut payload, selection) = fixture(&repo);
    fs::write(repo.join("visual-gallery.toml"), "schema=1\ncapture=['never']\nbaselines='baselines'\nreport='report'\nruns='runs'\n[source_policy]\nscopes=['stories.json']\nsuffixes=['.json']\n").unwrap();
    payload.sources.insert(
        "visual-gallery.toml".into(),
        store::digest(&repo.join("visual-gallery.toml")).unwrap(),
    );
    assert!(Request::new(&payload, &"a".repeat(64), selection).is_err());
    let (mut payload, selection) = fixture(&repo);
    let plan = "scripts/visual/catalogs/gallery-plan.json";
    fs::write(repo.join(plan), "{\"schema\":1,\"cases\":[]}").unwrap();
    payload
        .sources
        .insert(plan.into(), store::digest(&repo.join(plan)).unwrap());
    assert!(Request::new(&payload, &"a".repeat(64), selection).is_err());
}
