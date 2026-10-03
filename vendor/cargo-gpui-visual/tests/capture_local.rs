//! Local producers share seals with external producers without nested supervisors.
use cargo_gpui_visual::{
    capture::{capture_local, Paths},
    config::Config,
    store,
};
use serde_json::json;
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
};

#[test]
fn local_producer_is_sealed_and_drift_cancel_or_invalid_pixels_never_publish() {
    for failure in [
        "none",
        "source",
        "environment",
        "provenance",
        "cancel",
        "pixels",
        "producer",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().canonicalize().unwrap();
        assert!(Command::new("git")
            .args(["init", "-q"])
            .arg(&repo)
            .status()
            .unwrap()
            .success());
        fs::write(repo.join("source.rs"), "before").unwrap();
        let config=Config::from_toml("schema=1\ncapture=['unused']\nenvironment_command=['unused']\nbaselines='refs'\nreport='review'\nruns='runs'\n[source_policy]\nscopes=['source.rs']\nsuffixes=['.rs']\n").unwrap();
        let run = repo.join("run");
        let cancelled = AtomicBool::new(false);
        let mut probes = 0;
        let result = capture_local(
            Paths {
                repo: &repo,
                run: &run,
                baselines: &config.baselines,
                report: &config.report,
            },
            &config,
            &cancelled,
            |output| {
                assert_eq!(output, run);
                if failure == "producer" {
                    anyhow::bail!("producer failure");
                }
                fs::create_dir(output)?;
                for mode in ["light", "dark"] {
                    image::RgbaImage::new(2, 2)
                        .save(output.join(format!("fixture--{mode}.png")))?;
                }
                store::write_json(
                    &output.join("inventory.json"),
                    &["fixture--light", "fixture--dark"],
                )?;
                match failure {
                    "source" => fs::write(repo.join("source.rs"), "after")?,
                    "cancel" => cancelled.store(true, Ordering::SeqCst),
                    "pixels" => fs::write(output.join("fixture--dark.png"), "invalid")?,
                    _ => {}
                }
                Ok(())
            },
            || {
                probes += 1;
                if failure == "provenance" {
                    return cargo_gpui_visual::environment::partition(
                        &json!({"layer":"gpui","os":"same","instance":probes}),
                        json!({"id":"fixture"}),
                        &["/instance".into()],
                    );
                }
                Ok(json!({"layer":"gpui","probe":if failure=="environment" {probes} else {0}}))
            },
        );
        assert_eq!(result.is_ok(), failure == "none", "{failure}: {result:?}");
        assert_eq!(run.join("capture.json").exists(), failure == "none");
        assert!(!repo.join("refs").exists());
        assert!(!repo.join("review").exists());
        if let Ok(manifest) = result {
            store::check_sources(&repo, &manifest, None).unwrap();
            store::validate_images(&run, &manifest).unwrap();
        }
    }
}
