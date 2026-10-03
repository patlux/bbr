//! Fresh synthetic transport seam; no VM, SSH, GPUI or historical image import.
use super::*;
use crate::{
    payload::{Entry, Payload},
    vm_environment::{Guest, HostBuild, Receipt},
};
use cargo_gpui_visual::{compare, selection::Selection};
use serde_json::json;
use std::{path::PathBuf, process::Command};

#[path = "vm_capture_endpoint_tests.rs"]
mod endpoints;

struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    repo: PathBuf,
    share: PathBuf,
    workspace: PathBuf,
    staged: Payload,
    host: HostBuild,
    selection: Option<Selection>,
    suite: Suite,
}
impl Fixture {
    fn new(suite: Suite, selected: bool) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let repo = root.join("repo");
        fs::create_dir(&repo).unwrap();
        assert!(Command::new("git")
            .args(["init", "-q"])
            .arg(&repo)
            .status()
            .unwrap()
            .success());
        fs::create_dir_all(repo.join("scripts/visual/catalogs")).unwrap();
        fs::write(repo.join("fixture.rs"), "// synthetic renderer source\n").unwrap();
        fs::write(repo.join(suite.config()), "schema=1\ncapture=['tripwire-never-execute']\nenvironment_command=['tripwire-no-host-probe']\nbaselines='refs'\nreport='review'\nruns='runs'\nstory_catalog='stories.json'\n[source_policy]\nscopes=['fixture.rs','visual.toml','visual-gallery.toml','scripts/visual/catalogs','stories.json','vendor/cargo-gpui-visual']\nsuffixes=['.rs','.toml','.json','.lock']\n").unwrap();
        fs::write(repo.join("scripts/visual/catalogs/gallery-plan.json"), json!({"schema":1,"cases":[{"base":"gallery-00-state-00-standard-normal","label":"Fixture","state":"default","group":"Foundations","ranges":[[0,1],[1,3]],"dark_ranges":[[0,2],[2,3]]}]}).to_string()).unwrap();
        fs::write(repo.join("stories.json"), json!({"schema":1,"stories":[{"id":"gallery-00","label":"Fixture","cases":["gallery-00-state-00-standard-normal-page-00","gallery-00-state-00-standard-normal-page-01"]}]}).to_string()).unwrap();
        for path in crate::vm_rendering::TOOL_INPUTS {
            let path = repo.join("vendor/cargo-gpui-visual").join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "synthetic capture input").unwrap();
        }
        let config = Config::load(&repo.join(suite.config())).unwrap();
        let selection =
            selected.then(|| Selection::resolve(&repo, &config, &["gallery-00".into()]).unwrap());
        let sources = payload::source_snapshot(&repo, &config, &AtomicBool::new(false)).unwrap();
        let staged = Payload {
            schema: 1,
            suite: suite.id().into(),
            repo: repo.clone(),
            sources,
            files: vec![Entry {
                payload: "executables/0".into(),
                guest: repo.join("synthetic-binary"),
                bytes: 7,
                sha256: "b".repeat(64),
                executable: true,
            }],
        };
        let host = HostBuild {
            os: "host-fixture".into(),
            os_build: "build".into(),
            arch: "arm64".into(),
            rustc: "fixture".into(),
            xcode: "fixture".into(),
            sdk: "fixture".into(),
            gpui_source: "pinned-fixture".into(),
            tool_revision: "fixture".into(),
            executable_hashes: [(
                repo.join("synthetic-binary").to_string_lossy().into_owned(),
                "b".repeat(64),
            )]
            .into(),
        };
        let share = root.join("share");
        fs::create_dir(&share).unwrap();
        fs::write(
            share.join("payload.json"),
            serde_json::to_vec(&staged).unwrap(),
        )
        .unwrap();
        Self {
            workspace: root.join("transport"),
            _temp: temp,
            root,
            repo,
            share,
            staged,
            host,
            selection,
            suite,
        }
    }
    fn input(&self) -> vm_transport::Input<'_> {
        vm_transport::Input {
            repo: &self.repo,
            suite: self.suite,
            share: &self.share,
            staged: &self.staged,
            host: &self.host,
            workspace: &self.workspace,
            selection: self.selection.as_ref(),
        }
    }
    fn run(&self) -> PathBuf {
        self.repo.join("reports/visual/batches/fixture/run")
    }
    fn paths<'a>(&'a self, run: &'a Path, refs: &'a Path, report: &'a Path) -> capture::Paths<'a> {
        capture::Paths {
            repo: &self.repo,
            run,
            baselines: refs,
            report,
        }
    }
    fn emit(&self, change: &str) -> vm_transport::Captured {
        fs::create_dir(&self.workspace).unwrap();
        let raw = self.workspace.join("raw");
        fs::create_dir(&raw).unwrap();
        if self.suite == Suite::Gallery {
            let mut inventory = String::new();
            let mut coverage = String::new();
            let png = self.root.join("synthetic.png");
            image::RgbaImage::new(2400, 2000).save(&png).unwrap();
            for mode in ["light", "dark"] {
                let id = format!("gallery-00-state-00-standard-normal--{mode}");
                inventory += &format!("{id}\tFixture\tdefault\n");
                let ranges = if mode == "light" {
                    [[0, 1], [1, 3]]
                } else {
                    [[0, 2], [2, 3]]
                };
                for (page, [start, end]) in ranges.iter().enumerate() {
                    let name = format!("{id}-page-{page:02}");
                    fs::copy(&png, raw.join(format!("{name}.png"))).unwrap();
                    coverage += &format!("{name}\t{id}\tFixture\tdefault\t{start}\t{end}\t3\n");
                }
            }
            fs::write(raw.join("coverage.tsv"), coverage).unwrap();
            fs::write(raw.join("live-inventory.tsv"), inventory).unwrap();
        } else {
            for (region, w) in [("full", 2160), ("rail", 608)] {
                let png = self.root.join(format!("{region}.png"));
                image::RgbaImage::new(w, 2814).save(&png).unwrap();
                for fixture in ["ordinary", "representative"] {
                    for mode in ["light", "dark"] {
                        fs::copy(&png, raw.join(format!("{fixture}-{region}--{mode}.png")))
                            .unwrap();
                        fs::write(
                            raw.join(format!("{fixture}--{mode}.snapshot.json")),
                            "{\"synthetic\":true}",
                        )
                        .unwrap();
                    }
                }
            }
            let settings = self.root.join("settings.png");
            image::RgbaImage::new(1560, 1240).save(&settings).unwrap();
            for surface in crate::BROWSER_SURFACES {
                let png = if surface.starts_with("settings-") {
                    settings.clone()
                } else {
                    self.root.join("full.png")
                };
                for mode in ["light", "dark"] {
                    fs::copy(&png, raw.join(format!("{surface}--{mode}.png"))).unwrap();
                    fs::write(
                        raw.join(format!("{surface}--{mode}.snapshot.json")),
                        "{\"synthetic\":true}",
                    )
                    .unwrap();
                }
            }
        }
        if let Some(s) = &self.selection {
            fs::write(raw.join("selection.json"), serde_json::to_vec(s).unwrap()).unwrap();
        }
        let guest = Guest {
            schema: 1,
            origin: "tart-guest".into(),
            os: "synthetic guest".into(),
            os_build: "guest-build".into(),
            arch: "arm64".into(),
            model: "VirtualMac-fixture".into(),
            displays: json!([{"origin":"guest-coregraphics-metal","screens":[{"id":1,"width_px":6400,"height_px":3600,"width_pt":3200,"height_pt":1800}],"metal":{"name":"Synthetic Metal","registry_id":100}}]),
            fonts: [("/System/Library/Fonts/fixture.ttf".into(), "c".repeat(64))].into(),
            console_user: "admin".into(),
            on_console: true,
            screen_locked: false,
            window_server: true,
            timezone: "UTC".into(),
            locale: "C".into(),
        };
        let digest = store::digest(&self.share.join("payload.json")).unwrap();
        let receipt = Receipt {
            schema: 1,
            payload_sha256: digest.clone(),
            sources: self.staged.sources.clone(),
            installed: self
                .staged
                .files
                .iter()
                .map(|e| (e.guest.to_string_lossy().into_owned(), e.sha256.clone()))
                .collect(),
            environment_before: guest.clone(),
            environment_after: guest,
        };
        fs::write(
            raw.join("guest-receipt.json"),
            serde_json::to_vec(&receipt).unwrap(),
        )
        .unwrap();
        let image = fs::read_dir(&raw)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.extension().is_some_and(|s| s == "png"))
            .unwrap();
        match change {
            "missing" => fs::remove_file(&image).unwrap(),
            "extra" => {
                fs::copy(&image, raw.join("unexpected--light.png")).unwrap();
            }
            "metadata" => fs::write(raw.join("unknown.json"), "{}").unwrap(),
            "sealed" => fs::write(raw.join("capture.json"), "{}").unwrap(),
            "dimensions" => image::RgbaImage::new(1, 1).save(&image).unwrap(),
            "settings-dimensions" => image::RgbaImage::new(2160, 2814)
                .save(raw.join("settings-general--light.png"))
                .unwrap(),
            "settings-missing" => fs::remove_file(raw.join("settings-privacy--dark.png")).unwrap(),
            "find-missing" => fs::remove_file(raw.join("find-open--dark.png")).unwrap(),
            "find-dimensions" => image::RgbaImage::new(1560, 1240)
                .save(raw.join("find-matches--light.png"))
                .unwrap(),
            "find-metadata-missing" => {
                fs::remove_file(raw.join("find-no-matches--dark.snapshot.json")).unwrap()
            }
            "palette-missing" => {
                fs::remove_file(raw.join("command-palette-open--dark.png")).unwrap()
            }
            "palette-dimensions" => image::RgbaImage::new(1560, 1240)
                .save(raw.join("command-palette-actions-match--light.png"))
                .unwrap(),
            "palette-metadata-missing" => {
                fs::remove_file(raw.join("command-palette-actions-empty--dark.snapshot.json"))
                    .unwrap()
            }
            "closed-missing" => {
                fs::remove_file(raw.join("history-recently-closed--dark.png")).unwrap()
            }
            "scroll-dimensions" => image::RgbaImage::new(2160, 2814)
                .save(raw.join("settings-shortcuts-scroll-3--light.png"))
                .unwrap(),
            "library-metadata-missing" => {
                fs::remove_file(raw.join("library-selection-multiple--light.snapshot.json"))
                    .unwrap()
            }
            "scroll-metadata-invalid" => fs::write(
                raw.join("settings-shortcuts-scroll-2--dark.snapshot.json"),
                "not json",
            )
            .unwrap(),
            "rgb" => image::RgbImage::new(2400, 2000).save(&image).unwrap(),
            "duplicate" => {
                let p = raw.join("live-inventory.tsv");
                let text = fs::read_to_string(&p).unwrap();
                fs::write(p, format!("{text}{text}")).unwrap();
            }
            "range" => {
                let p = raw.join("coverage.tsv");
                fs::write(
                    &p,
                    fs::read_to_string(&p)
                        .unwrap()
                        .replace("\t0\t1\t3", "\t0\t2\t3"),
                )
                .unwrap();
            }
            "snapshot" => fs::write(raw.join("ordinary--light.snapshot.json"), "not json").unwrap(),
            "inventory" => fs::write(raw.join("inventory.json"), "[]").unwrap(),
            "sequences" => fs::write(raw.join("sequences.json"), "[]").unwrap(),
            _ => (),
        }
        let mut files = serde_json::Map::new();
        for entry in fs::read_dir(&raw).unwrap() {
            let path = entry.unwrap().path();
            files.insert(path.file_name().unwrap().to_str().unwrap().into(),json!({"bytes":path.metadata().unwrap().len(),"sha256":store::digest(&path).unwrap()}));
        }
        let marker = json!({"schema":1,"payload_sha256":digest,"files":files});
        fs::write(raw.join("return.json"), marker.to_string()).unwrap();
        let mut evidence = marker;
        evidence["raw"] = json!(raw);
        let request = self
            .selection
            .clone()
            .map(|s| selected_request::Request::new(&self.staged, &digest, s).unwrap());
        let mut captured = vm_transport::validate_return(
            &serde_json::to_vec(&evidence).unwrap(),
            &self.workspace,
            vm_transport::ExpectedReturn {
                suite: self.suite,
                staged: &self.staged,
                digest: &digest,
                host: &self.host,
                selected: request.as_ref(),
            },
            &AtomicBool::new(false),
        )
        .unwrap();
        match change {
            "bytes" => fs::write(&image, "changed after transport validation").unwrap(),
            "source" => fs::write(self.repo.join("fixture.rs"), "// changed").unwrap(),
            "receipt" => fs::write(raw.join("guest-receipt.json"), "{}").unwrap(),
            "hardlink" => fs::hard_link(&image, self.root.join("alias.png")).unwrap(),
            "environment" => {
                captured.environment["rendering"]["guest_rendering"]["os_build"] = json!("mutated")
            }
            "scope" => captured.selection = None,
            "payload" => fs::write(self.share.join("payload.json"), "{}").unwrap(),
            "symlink" => {
                let alias = self.root.join("moved.png");
                fs::rename(&image, &alias).unwrap();
                std::os::unix::fs::symlink(alias, &image).unwrap();
            }
            _ => (),
        }
        captured
    }
}

#[test]
fn fresh_selected_full_gallery_and_browser_reuse_upstream_seals_without_pixel_rewriting() {
    for (suite, selected, count) in [
        (Suite::Gallery, true, 4),
        (Suite::Gallery, false, 4),
        (Suite::Browser, false, 78),
    ] {
        let f = Fixture::new(suite, selected);
        let run = f.run();
        let refs = f.repo.join("refs");
        let report = f.repo.join("review");
        let manifest = capture_with(
            f.input(),
            f.paths(&run, &refs, &report),
            &AtomicBool::new(false),
            |_| Ok(f.emit("none")),
        )
        .unwrap();
        assert_eq!(manifest.cases.len(), count);
        assert_eq!(manifest.schema, if selected { 2 } else { 1 });
        assert_eq!(manifest.selection, f.selection);
        assert_eq!(manifest.sources, f.staged.sources);
        assert_eq!(
            manifest.environment["rendering"]["guest_rendering"]["os_build"],
            "guest-build"
        );
        for (name, hash) in &manifest.images {
            let original = if suite == Suite::Gallery {
                let (base, mode) = name.rsplit_once("--").unwrap();
                let (base, page) = base.rsplit_once("-page-").unwrap();
                format!("{base}--{mode}-page-{page}.png")
            } else {
                format!("{name}.png")
            };
            assert_eq!(
                *hash,
                store::digest(&f.workspace.join("raw").join(original)).unwrap()
            );
        }
        assert_eq!(
            store::digest(&run.join("guest-receipt.json")).unwrap(),
            store::digest(&f.workspace.join("raw/guest-receipt.json")).unwrap()
        );
        let review = compare::compare(&f.repo, &run, &refs, &report).unwrap();
        assert!(!review.passed);
        if selected {
            assert!(fs::read_to_string(report.join("index.html"))
                .unwrap()
                .contains("TEILLAUF"));
            let error = cargo_gpui_visual::accept::accept(
                &f.repo,
                &run,
                &refs,
                &report,
                &manifest.cases,
                false,
            )
            .unwrap_err();
            assert!(error.to_string().contains("development-only"));
        }
        assert!(store::baseline_state(&refs).unwrap().is_empty());
        assert!(capture_with(
            f.input(),
            f.paths(&run, &refs, &report),
            &AtomicBool::new(false),
            |_| panic!("must not recapture existing run")
        )
        .is_err());
    }
}

#[test]
fn invalid_or_changed_return_never_gets_a_capture_seal() {
    for (suite, change) in [
        (Suite::Gallery, "missing"),
        (Suite::Gallery, "extra"),
        (Suite::Gallery, "metadata"),
        (Suite::Gallery, "sealed"),
        (Suite::Gallery, "dimensions"),
        (Suite::Gallery, "rgb"),
        (Suite::Gallery, "duplicate"),
        (Suite::Gallery, "range"),
        (Suite::Gallery, "bytes"),
        (Suite::Gallery, "source"),
        (Suite::Gallery, "receipt"),
        (Suite::Gallery, "hardlink"),
        (Suite::Gallery, "symlink"),
        (Suite::Gallery, "environment"),
        (Suite::Gallery, "scope"),
        (Suite::Gallery, "payload"),
        (Suite::Gallery, "inventory"),
        (Suite::Gallery, "sequences"),
        (Suite::Browser, "missing"),
        (Suite::Browser, "snapshot"),
        (Suite::Browser, "metadata"),
        (Suite::Browser, "extra"),
        (Suite::Browser, "dimensions"),
        (Suite::Browser, "rgb"),
        (Suite::Browser, "settings-dimensions"),
        (Suite::Browser, "settings-missing"),
        (Suite::Browser, "closed-missing"),
        (Suite::Browser, "find-missing"),
        (Suite::Browser, "find-dimensions"),
        (Suite::Browser, "find-metadata-missing"),
        (Suite::Browser, "palette-missing"),
        (Suite::Browser, "palette-dimensions"),
        (Suite::Browser, "palette-metadata-missing"),
        (Suite::Browser, "scroll-dimensions"),
        (Suite::Browser, "library-metadata-missing"),
        (Suite::Browser, "scroll-metadata-invalid"),
    ] {
        let f = Fixture::new(suite, suite == Suite::Gallery);
        let run = f.run();
        let refs = f.repo.join("refs");
        let report = f.repo.join("review");
        let result = capture_with(
            f.input(),
            f.paths(&run, &refs, &report),
            &AtomicBool::new(false),
            |_| Ok(f.emit(change)),
        );
        let expected = match change {
            "range" => "pages",
            "missing" | "extra" | "metadata" => "unexpected Gallery output",
            "sealed" | "inventory" | "sequences" => "must not supply",
            "dimensions"
            | "settings-dimensions"
            | "scroll-dimensions"
            | "palette-dimensions"
            | "find-dimensions" => "dimensions",
            "settings-missing" | "closed-missing" | "palette-missing" | "find-missing" => {
                "browser inventory differs"
            }
            // Metadata regular-file validation precedes the aggregate inventory.
            "library-metadata-missing" | "palette-metadata-missing" | "find-metadata-missing" => {
                "No such file or directory"
            }
            "rgb" => "RGBA8",
            "duplicate" => "duplicate live case",
            "bytes" | "receipt" | "hardlink" => "invalid returned file",
            "source" => "source/payload changed",
            "symlink" => "Symlink path refused",
            "environment" | "scope" => "captured identity changed",
            "payload" => "source/payload changed",
            "snapshot" | "scroll-metadata-invalid" => "expected ident",
            _ => unreachable!(),
        };
        let expected =
            if suite == Suite::Browser && ["missing", "extra", "metadata"].contains(&change) {
                "browser inventory differs"
            } else {
                expected
            };
        let error = result.unwrap_err().to_string();
        assert!(error.contains(expected), "{change}: {error}");
        assert!(!run.join("capture.json").exists(), "{change}");
        assert!(!refs.exists());
        assert!(!report.exists());
    }
}

#[test]
fn copied_and_prepared_bytes_remain_bound_until_upstream_sealing() {
    let f = Fixture::new(Suite::Gallery, true);
    let raw = f.emit("none");
    let png = raw
        .files()
        .unwrap()
        .into_iter()
        .find(|(name, _)| name.ends_with(".png"))
        .unwrap();
    let target = f.root.join("copied.png");
    let cancelled = AtomicBool::new(true);
    assert!(copy(
        &raw.raw.join(&png.0),
        &target,
        &png.1,
        &cancelled,
        Instant::now() + Duration::from_secs(1)
    )
    .is_err());
    assert!(!target.exists());
    cancelled.store(false, Ordering::SeqCst);
    assert!(copy(
        &raw.raw.join(&png.0),
        &target,
        &png.1,
        &cancelled,
        Instant::now()
    )
    .is_err());
    assert!(!target.exists());
    let wrong = vm_transport::ReturnedFile {
        bytes: png.1.bytes,
        sha256: "0".repeat(64),
    };
    assert!(copy(
        &raw.raw.join(&png.0),
        &target,
        &wrong,
        &cancelled,
        Instant::now() + Duration::from_secs(1)
    )
    .is_err());
    let output = f.root.join("prepared");
    fs::create_dir(&output).unwrap();
    let mut ids = Vec::new();
    for (name, _) in raw.files().unwrap() {
        let canonical = if let Some(id) = name.strip_suffix(".png") {
            let id = gallery::canonical_id(id).unwrap();
            ids.push(id.clone());
            format!("{id}.png")
        } else {
            name.clone()
        };
        fs::copy(raw.raw.join(name), output.join(canonical)).unwrap();
    }
    store::write_json(&output.join("inventory.json"), &ids).unwrap();
    verify_prepared(&raw, &output, Suite::Gallery, &cancelled).unwrap();
    for name in ["guest-receipt.json", "inventory.json", "selection.json"] {
        let path = output.join(name);
        let bytes = fs::read(&path).unwrap();
        fs::write(&path, "{}").unwrap();
        assert!(verify_prepared(&raw, &output, Suite::Gallery, &cancelled).is_err());
        fs::write(&path, bytes).unwrap();
    }
    let image = output.join(format!("{}.png", ids[0]));
    fs::write(&image, "changed").unwrap();
    assert!(verify_prepared(&raw, &output, Suite::Gallery, &cancelled).is_err());
    assert!(!output.join("capture.json").exists());
}

#[test]
fn cancellation_and_stale_inputs_never_run_or_seal_and_transport_failure_is_not_retried() {
    for mode in ["cancelled", "source", "transport", "during"] {
        let f = Fixture::new(Suite::Gallery, true);
        let run = f.run();
        let refs = f.repo.join("refs");
        let report = f.repo.join("review");
        let cancelled = AtomicBool::new(mode == "cancelled");
        if mode == "source" {
            fs::write(f.repo.join("fixture.rs"), "// stale").unwrap();
        }
        let mut calls = 0;
        let result = capture_with(f.input(), f.paths(&run, &refs, &report), &cancelled, |_| {
            calls += 1;
            if mode == "transport" {
                anyhow::bail!("fixture transport failed");
            }
            let captured = f.emit("none");
            cancelled.store(true, Ordering::SeqCst);
            Ok(captured)
        });
        assert!(result.is_err());
        assert_eq!(calls, usize::from(["transport", "during"].contains(&mode)));
        assert!(!run.join("capture.json").exists());
    }
}
