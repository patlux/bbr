use super::*;
use serde_json::json;

const NOTE: &str = "report-only frame-time observations; never a product budget";

fn fixture() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().canonicalize().unwrap();
    let dir = repo.join("target/bbr-gallery-perf");
    fs::create_dir_all(&dir).unwrap();
    let raw = repo.join("raw");
    fs::create_dir(&raw).unwrap();
    for (suite, _) in REPORTS {
        store::write_json(
            &dir.join(format!("{suite}.json")),
            &json!({
                "suite":suite,"sanity_only":true,"note":NOTE,"observations_ms":{"fixture":0.25}
            }),
        )
        .unwrap();
    }
    (temp, repo, raw)
}
#[test]
fn moves_only_exact_report_only_outputs_without_changing_bytes() {
    let (_temp, repo, raw) = fixture();
    let dir = repo.join("target/bbr-gallery-perf");
    let hashes: Vec<_> = REPORTS
        .iter()
        .map(|(suite, _)| store::digest(&dir.join(format!("{suite}.json"))).unwrap())
        .collect();
    collect(&repo, &raw, || Ok(())).unwrap();
    assert!(!dir.exists());
    for ((_, name), hash) in REPORTS.iter().zip(hashes) {
        assert_eq!(store::digest(&raw.join(name)).unwrap(), hash);
    }
    assert_eq!(fs::read_dir(&raw).unwrap().count(), 2);
}
#[test]
fn freshness_and_cancellation_preserve_original_diagnostics() {
    let (_temp, repo, raw) = fixture();
    assert!(require_fresh(&repo).is_err());
    let error = collect(&repo, &raw, || anyhow::bail!("injected cancellation")).unwrap_err();
    assert!(error.to_string().contains("cancellation"));
    assert_eq!(fs::read_dir(&raw).unwrap().count(), 0);
    assert_eq!(
        fs::read_dir(repo.join("target/bbr-gallery-perf"))
            .unwrap()
            .count(),
        2
    );
    collect(&repo, &raw, || Ok(())).unwrap();
    require_fresh(&repo).unwrap();
}

#[test]
fn invalid_extra_missing_or_linked_outputs_never_move_any_report() {
    for mutation in [
        "missing",
        "extra",
        "symlink",
        "hardlink",
        "schema",
        "suite",
        "empty",
        "negative",
        "preexisting",
        "oversize",
        "unknown-field",
        "directory",
        "note",
        "null-value",
    ] {
        let (_temp, repo, raw) = fixture();
        let dir = repo.join("target/bbr-gallery-perf");
        let path = dir.join("every-story-renders.json");
        match mutation {
            "missing" => fs::remove_file(&path).unwrap(),
            "extra" => fs::write(dir.join("unexpected"), "x").unwrap(),
            "symlink" => {
                fs::remove_file(&path).unwrap();
                std::os::unix::fs::symlink(dir.join("mode-state-switching.json"), &path).unwrap();
            }
            "hardlink" => fs::hard_link(&path, repo.join("linked")).unwrap(),
            "preexisting" => fs::write(raw.join(REPORTS[1].1), "preserved").unwrap(),
            "oversize" => fs::write(&path, vec![b' '; 1024 * 1024 + 1]).unwrap(),
            "directory" => {
                fs::remove_file(&path).unwrap();
                fs::create_dir(&path).unwrap();
            }
            _ => {
                let mut value: serde_json::Value = store::read_json(&path).unwrap();
                if mutation == "schema" {
                    value["sanity_only"] = json!(false);
                }
                if mutation == "suite" {
                    value["suite"] = json!("wrong");
                }
                if mutation == "empty" {
                    value["observations_ms"] = json!({});
                }
                if mutation == "negative" {
                    value["observations_ms"] = json!({"sample":-1});
                }
                if mutation == "unknown-field" {
                    value["accepted"] = json!(true);
                }
                if mutation == "note" {
                    value["note"] = json!("budget passed");
                }
                if mutation == "null-value" {
                    value["observations_ms"] = json!({"sample":null});
                }
                store::write_json(&path, &value).unwrap();
            }
        }
        assert!(collect(&repo, &raw, || Ok(())).is_err(), "{mutation}");
        assert!(dir.join("mode-state-switching.json").exists(), "{mutation}");
        assert!(!raw.join(REPORTS[0].1).exists(), "{mutation}");
    }
}
