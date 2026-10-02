//! Tripwire at the fresh-capture boundary, not native input evidence.
use super::*;

#[test]
fn endpoints_fail_before_transport_for_selected_and_full_captures() {
    for selected in [false, true] {
        let mut f = Fixture::new(Suite::Gallery, selected);
        let plan_path = "scripts/visual/catalogs/gallery-plan.json";
        let rest = json!({"base":"gallery-16-state-00-standard-normal","label":"Checkbox",
            "state":"default","group":"Core controls","ranges":[[0,5]]});
        let mut cases = vec![rest.clone()];
        for endpoint in [
            "initial-unchecked",
            "checked-after-click",
            "unchecked-after-second-click",
        ] {
            let mut case = rest.clone();
            case["base"] = json!(format!(
                "gallery-16-state-00-standard-normal-endpoint-{endpoint}"
            ));
            case["endpoint"] = json!(endpoint);
            cases.push(case);
        }
        let pages: Vec<_> = cases
            .iter()
            .map(|c| format!("{}-page-00", c["base"].as_str().unwrap()))
            .collect();
        store::write_json(&f.repo.join(plan_path), &json!({"schema":2,"cases":cases})).unwrap();
        store::write_json(
            &f.repo.join("stories.json"),
            &json!({"schema":1,"stories":[{
                "id":"gallery-16","label":"Checkbox","cases":pages
            }]}),
        )
        .unwrap();
        let config = Config::load(&f.repo.join(Suite::Gallery.config())).unwrap();
        f.selection =
            selected.then(|| Selection::resolve(&f.repo, &config, &["gallery-16".into()]).unwrap());
        f.staged.sources =
            payload::source_snapshot(&f.repo, &config, &AtomicBool::new(false)).unwrap();
        store::write_json(&f.share.join("payload.json"), &f.staged).unwrap();
        let run = f.run();
        let refs = f.repo.join("refs");
        let report = f.repo.join("review");
        let error = capture_with(
            f.input(),
            f.paths(&run, &refs, &report),
            &AtomicBool::new(false),
            |_| panic!("endpoint plan must not reach transport"),
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("endpoint execution/evidence is not wired"),
            "{error:#}"
        );
        assert!(!f.workspace.exists() && !run.exists() && !refs.exists() && !report.exists());
    }
}
