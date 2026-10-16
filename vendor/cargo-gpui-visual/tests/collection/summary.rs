use super::*;

#[test]
fn suite_summary_separates_coverage_environment_and_technical_details() {
    let mut f = Fixture::new();
    let env = cargo_gpui_visual::environment::partition(
        &json!({"layer":"gpui","host_build":{"os":"27.0","os_build":"26A428","tool_revision":"old-tool"},"fonts":{"unchanged-font":"same-hash"}}),
        json!({"id":"fixture-v1"}),
        &["/host_build/tool_revision".into()],
    ).unwrap();
    write_json(&f.suites[0].repo.join("environment.json"), &env).unwrap();
    f.seal(0);
    f.compare().unwrap();
    let ids: Vec<_> = load_manifest(&f.suites[0].run)
        .unwrap()
        .cases
        .iter()
        .map(|c| format!("gallery:{c}"))
        .collect();
    collection::accept(&f.report, &ids, &[]).unwrap();
    let baseline = baseline_state(&f.suites[0].baselines).unwrap();
    f.catalog(0, |v| v["snapshots"].as_array_mut().unwrap().extend([
        json!({"base":"uncovered","label":"Weitere Zustände <&>","group":"Coverage","state":"withheld","reason":"Unproven <script>fixture</script>"}),
        json!({"base":"native","label":"Native Oberfläche","group":"Coverage","state":"unsupported","reason":"Outside this renderer"}),
    ]));
    let mut changed = env;
    changed["rendering"]["host_build"]["os"] = json!("27.0.1");
    changed["rendering"]["host_build"]["os_build"] = json!("26A434");
    changed["provenance"]["/host_build/tool_revision"] = json!("new-tool");
    write_json(&f.suites[0].repo.join("environment.json"), &changed).unwrap();
    f.seal(0);
    f.report = f.root.join("summary-report");
    let c = f.compare().unwrap();
    assert!(!c.passed);
    let html = f.html();
    let suite = html
        .split("id=\"suite-gallery\"")
        .nth(1)
        .unwrap()
        .split("</section>")
        .next()
        .unwrap();
    assert!(suite.contains("4 Aufnahmen geprüft"));
    assert!(suite.contains("Umgebung geändert"));
    assert!(suite.contains("1 Abdeckungslücke"));
    assert!(suite.contains("<details class=\"coverage-details\">"));
    assert!(suite.contains("<details class=\"technical-details\">"));
    assert!(!suite.contains(" open"));
    let delta = suite
        .split("class=\"environment-delta\"")
        .nth(1)
        .unwrap()
        .split("</table>")
        .next()
        .unwrap();
    assert!(delta.contains("27.0.1") && delta.contains("26A428") && delta.contains("26A434"));
    assert!(!delta.contains("unchanged-font") && !delta.contains("old-tool"));
    assert!(suite.contains("Provenienz geändert"));
    assert!(suite.contains("old-tool") && suite.contains("new-tool"));
    assert!(suite.contains("Weitere Zustände &lt;&amp;&gt;"));
    assert!(suite.contains("Unproven &lt;script&gt;fixture&lt;/script&gt;"));
    assert!(!suite.contains("<script>fixture</script>"));
    assert!(suite.contains("Nicht unterstützt"));
    assert_eq!(baseline, baseline_state(&f.suites[0].baselines).unwrap());
    if let Some(output) = std::env::var_os("GPUI_SUMMARY_FIXTURE_OUTPUT") {
        let output = PathBuf::from(output);
        assert!(!output.exists());
        // Keep the entire disposable fixture: its suite-local evidence stays separate.
        fs::rename(&f.root, &output).unwrap();
    }
}

#[test]
fn invalid_and_missing_captures_never_get_a_success_summary() {
    for invalid in [false, true] {
        let f = Fixture::new();
        if invalid {
            fs::write(f.suites[0].run.join("capture.json"), "invalid").unwrap();
        } else {
            fs::remove_file(f.suites[0].run.join("panel-page-one--light.png")).unwrap();
            f.seal(0);
        }
        let c = f.compare().unwrap();
        assert!(!c.passed);
        let html = f.html();
        let suite = html
            .split("id=\"suite-gallery\"")
            .nth(1)
            .unwrap()
            .split("</section>")
            .next()
            .unwrap();
        assert!(suite.contains("Prüfung unvollständig"));
        assert!(!suite.contains("Aufnahmen geprüft"));
        if invalid {
            assert!(suite.contains("Abdeckung nicht geprüft"));
            assert!(!suite.contains("Keine Abdeckungslücken"));
        }
        assert!(suite.contains("<details class=\"diagnostic-errors\">"));
        // A valid sibling still has its own truthful summary.
        let sibling = html
            .split("id=\"suite-browser\"")
            .nth(1)
            .unwrap()
            .split("</section>")
            .next()
            .unwrap();
        assert!(sibling.contains("4 Aufnahmen geprüft"));
        assert!(sibling.contains("Ohne Referenzumgebung"));
    }
}
