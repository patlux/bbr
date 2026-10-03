use super::*;

fn observation() -> Value {
    json!({"layer":"gpui","os":"27.0","fonts":{"font":"abc"},
        "host_build":{"tool_revision":"revision-a","sdk":"27"},
        "displays":[{"width_px":6400,"metal":{"name":"Virtual Metal","registry_id":100}}]})
}
fn partitioned() -> Value {
    json!({"layer":"gpui","comparison_schema":1,
        "contract":{"id":"fixture-rendering-v1","capture_inputs":{"capture.rs":"abc"}},
        "rendering":{"layer":"gpui","os":"27.0","fonts":{"font":"abc"},
            "host_build":{"sdk":"27"},"displays":[{"width_px":6400,"metal":{"name":"Virtual Metal"}}]},
        "provenance":{"/host_build/tool_revision":"revision-a","/displays/0/metal/registry_id":100}})
}
#[test]
fn provenance_changes_do_not_change_rendering_identity_or_acceptance() {
    let mut f = Fixture::new();
    f.env = partitioned();
    f.seal();
    f.establish();
    let baseline_environment: Value =
        read_json::<Value>(&f.base.join("baseline.json")).unwrap()["environment"].clone();
    f.env["provenance"]["/host_build/tool_revision"] = json!("revision-b");
    f.env["provenance"]["/displays/0/metal/registry_id"] = json!(200);
    f.seal();
    let review = f.compare().unwrap();
    assert!(
        review.environment_mismatch.is_none(),
        "provenance is not rendering drift"
    );
    assert!(review.passed);
    let review_json: Value = read_json(&f.report.join("review.json")).unwrap();
    assert_eq!(
        review_json["provenance_difference"]["capture"],
        f.env["provenance"]
    );
    assert!(f.doc().contains("Provenienz"));
    fails(f.accept(&[], true), "reviewed environment change");
    f.image(&f.cases[0], 20, 12, [0, 0, 0, 255]);
    f.seal();
    f.compare().unwrap();
    f.accept(&[f.cases[0].clone()], false).unwrap();
    assert_eq!(
        read_json::<Value>(&f.base.join("baseline.json")).unwrap()["environment"],
        baseline_environment
    );
}
#[test]
fn rendering_contract_and_classification_changes_still_require_adoption() {
    for mutation in [
        "os",
        "fonts",
        "sdk",
        "display",
        "metal",
        "contract",
        "classification",
        "legacy",
    ] {
        let mut f = Fixture::new();
        f.env = partitioned();
        f.seal();
        f.establish();
        match mutation {
            "os" => f.env["rendering"]["os"] = json!("28"),
            "fonts" => f.env["rendering"]["fonts"]["font"] = json!("def"),
            "sdk" => f.env["rendering"]["host_build"]["sdk"] = json!("28"),
            "display" => f.env["rendering"]["displays"][0]["width_px"] = json!(3200),
            "metal" => f.env["rendering"]["displays"][0]["metal"]["name"] = json!("different"),
            "contract" => f.env["contract"]["capture_inputs"]["capture.rs"] = json!("def"),
            "classification" => {
                f.env["rendering"]["host_build"]["tool_revision"] = json!("revision-a");
                f.env["provenance"]
                    .as_object_mut()
                    .unwrap()
                    .remove("/host_build/tool_revision");
            }
            "legacy" => f.env = observation(),
            _ => unreachable!(),
        }
        f.seal();
        let r = f.compare().unwrap();
        assert!(r.environment_mismatch.is_some(), "{mutation}");
        assert!(!r.passed);
        let before = baseline_state(&f.base).unwrap();
        fails(f.accept(&f.cases, false), "adopt-environment");
        assert_eq!(before, baseline_state(&f.base).unwrap());
    }
}
#[test]
fn explicit_partition_migration_preserves_history_and_invalidates_old_reviews() {
    use cargo_gpui_visual::{environment, environment_migration};
    let mut f = Fixture::new();
    f.env = observation();
    f.seal();
    f.establish();
    let path = f.base.join("baseline.json");
    let old: Value = read_json(&path).unwrap();
    let before = baseline_state(&f.base).unwrap();
    let target = environment::partition(
        &f.env,
        partitioned()["contract"].clone(),
        &[
            "/host_build/tool_revision".into(),
            "/displays/0/metal/registry_id".into(),
        ],
    )
    .unwrap();
    assert_eq!(target, partitioned());
    assert_eq!(environment::observation(&target).unwrap(), f.env);
    let hash = digest(&path).unwrap();
    let preview = environment_migration::migrate(&f.base, &hash, &target, true).unwrap();
    assert!(!preview.written);
    assert_eq!(before, baseline_state(&f.base).unwrap());
    f.compare().unwrap();
    let migrated = environment_migration::migrate(&f.base, &hash, &target, false).unwrap();
    assert_eq!(preview.new_hash, migrated.new_hash);
    assert_eq!(digest(&path).unwrap(), migrated.new_hash);
    let mut new: Value = read_json(&path).unwrap();
    new["environment"] = environment::observation(&new["environment"]).unwrap();
    assert_eq!(new, old);
    for (name, value) in &before {
        if name != "baseline.json" {
            assert_eq!(&digest(&f.base.join(name)).unwrap(), value);
        }
    }
    fails(f.accept(&f.cases, false), "Baselines changed since review");
    assert!(
        f.compare().unwrap().environment_mismatch.is_some(),
        "legacy capture is not silently partitioned"
    );
    fails(
        environment_migration::migrate(&f.base, &hash, &target, false),
        "expected hash",
    );
}
#[test]
fn migration_refuses_changed_observations_locks_pending_files_and_invalid_images() {
    use cargo_gpui_visual::environment_migration::migrate;
    for mutation in ["observation", "hash", "lock", "pending", "pixels"] {
        let mut f = Fixture::new();
        f.env = observation();
        f.seal();
        f.establish();
        let mut target = partitioned();
        let mut hash = digest(&f.base.join("baseline.json")).unwrap();
        match mutation {
            "observation" => {
                target["provenance"]["/host_build/tool_revision"] = json!("not-approved")
            }
            "hash" => hash = "0".repeat(64),
            "lock" => fs::write(f.base.join(".accept.lock"), "owner").unwrap(),
            "pending" => fs::write(f.base.join("baseline.pending"), "interrupted").unwrap(),
            "pixels" => fs::write(f.base.join(format!("{}.png", f.cases[0])), "invalid").unwrap(),
            _ => unreachable!(),
        }
        let before = baseline_state(&f.base).unwrap();
        assert!(
            migrate(&f.base, &hash, &target, false).is_err(),
            "{mutation}"
        );
        assert_eq!(before, baseline_state(&f.base).unwrap());
    }
}
#[test]
fn cli_partition_is_explicit_preview_then_hash_guarded_write() {
    let mut f = Fixture::new();
    f.env = observation();
    f.seal();
    f.establish();
    let path = f.repo.join("partition.json");
    write_json(&path, &partitioned()).unwrap();
    let hash = digest(&f.base.join("baseline.json")).unwrap();
    for dry in [true, false] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_cargo-gpui-visual"));
        command
            .args(["partition-environment", "--json", "--baselines"])
            .arg(&f.base)
            .args(["--expected-hash", &hash, "--environment"])
            .arg(&path);
        if dry {
            command.arg("--dry-run");
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["data"]["migration"]["written"], !dry);
        assert_eq!(digest(&f.base.join("baseline.json")).unwrap() == hash, dry);
    }
}
#[test]
fn malformed_partition_never_falls_back_to_legacy_equality() {
    for mutation in [
        "schema", "unknown", "contract", "empty", "overlap", "parent", "object", "escape", "layer",
    ] {
        let mut f = Fixture::new();
        f.env = partitioned();
        match mutation {
            "schema" => f.env["comparison_schema"] = json!(2),
            "unknown" => f.env["ignored"] = json!(true),
            "contract" => f.env["contract"] = json!({}),
            "empty" => f.env["rendering"] = json!({}),
            "overlap" => f.env["provenance"]["/os"] = json!("27.0"),
            "parent" => f.env["provenance"]["/missing/leaf"] = json!(42),
            "object" => {
                f.env["provenance"]["/host_build/tool_revision"] = json!({"hidden":"fields"})
            }
            "escape" => f.env["provenance"]["/bad~escape"] = json!(true),
            "layer" => f.env["rendering"]["layer"] = json!("ios"),
            _ => unreachable!(),
        }
        f.seal();
        assert!(
            f.compare().is_err(),
            "{mutation}: invalid partition must fail before report"
        );
        assert!(!f.report.exists());
        assert!(!f.base.exists());
    }
}
