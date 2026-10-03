use super::*;

#[test]
fn provenance_only_drift_keeps_withheld_coverage_red_and_seals_complete_evidence() {
    use cargo_gpui_visual::environment;
    let mut f = Fixture::new();
    let suite = &f.suites[0];
    let raw = json!({"layer":"gpui","os":"same","tool_revision":"a"});
    let env = environment::partition(&raw, json!({"id":"fixture-v1"}), &["/tool_revision".into()])
        .unwrap();
    write_json(&suite.repo.join("environment.json"), &env).unwrap();
    f.catalog(0, |v| v["snapshots"].as_array_mut().unwrap().push(json!({
        "base":"uncovered","label":"Still uncovered","group":"Remaining","state":"withheld","reason":"not implemented"})));
    f.seal(0);
    f.compare().unwrap();
    let ids: Vec<_> = load_manifest(&suite.run)
        .unwrap()
        .cases
        .iter()
        .map(|c| format!("gallery:{c}"))
        .collect();
    collection::accept_acknowledging_withheld(&f.report, &ids, &[], &["gallery:uncovered".into()])
        .unwrap();
    let before = baseline_state(&suite.baselines).unwrap();
    let mut changed = env;
    changed["provenance"]["/tool_revision"] = json!("b");
    write_json(&suite.repo.join("environment.json"), &changed).unwrap();
    f.seal(0);
    f.report = f.root.join("repeat");
    let result = f.compare().unwrap();
    let gallery = &result.suites[0];
    let review = gallery.review.as_ref().unwrap();
    assert!(review.passed && review.environment_mismatch.is_none());
    assert!(review.provenance_difference.is_some());
    assert!(review.results.iter().all(|r| r.status == "same"));
    assert!(!gallery.passed && !result.passed);
    assert_eq!(gallery.errors.len(), 1);
    assert!(gallery.errors[0].contains("Withheld"));
    assert!(f.html().contains("Provenienz geändert"));
    assert_eq!(before, baseline_state(&f.suites[0].baselines).unwrap());
}
