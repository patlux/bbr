//! Full Rust supervisor -> Python transport -> return/receipt path, with only
//! runtime, lifecycle/resource and SSH boundaries replaced by inert local fixtures.
use bbr_visual::{payload, vm_environment::HostBuild, vm_semantic, vm_transport, Suite};
use cargo_gpui_visual::{config::Config, gate::Options, process, selection::Selection};
use serde_json::{json, Value};
use std::{
    fs, os::unix::fs::PermissionsExt, path::Path, process::Command, sync::atomic::AtomicBool,
    time::Duration,
};

fn source(name: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(name)).unwrap()
}

fn fixture_scripts(repo: &Path, mode: &str) {
    let scripts = repo.join("scripts/visual");
    fs::create_dir_all(&scripts).unwrap();
    for name in [
        "vm_control.py",
        "vm_evidence.py",
        "vm_install.py",
        "vm_transfer.py",
        "vm_selection.py",
    ] {
        fs::write(scripts.join(name), source(name)).unwrap();
    }
    // Pure deterministic substitute for hardware/Nix availability. Rust still
    // validates receipt schema, original lock, closure size and executable set.
    fs::write(scripts.join("vm_runtime.py"), r#"
import json
import vm_install
def verify(share,digest,repo,suite):
    p=vm_install.inspect(share,digest,repo,suite)
    lock=json.loads((share/'source/scripts/visual/guest-runtime.json').read_text())
    return dict(schema=1,origin='host-runtime-prerequisite',payload_sha256=digest,
        runtime_lock_sha256=p['sources']['scripts/visual/guest-runtime.json'],python=lock['python'],
        dependencies={f['guest']:[] for f in p['files'] if f['executable']},closure_paths=1,closure_nar_bytes=17)
"#).unwrap();
    fs::write(
        scripts.join("vm_runtime_transfer.py"),
        "# Hardware/store boundary replaced in _provision below.\n",
    )
    .unwrap();
    fs::write(scripts.join("guest-runtime.json"),json!({"python":{"path":"/fixture/python","version":"fixture","sha256":"f".repeat(64)},"paths":{"fixture":{"narSize":17}}}).to_string()).unwrap();
    fs::write(
        scripts.join("vm_bootstrap.py"),
        "def plan(*args): return 'fixture, never executed'\n",
    )
    .unwrap();
    fs::write(scripts.join("vm_policy.py"), "class PolicyError(RuntimeError): pass\ndef validate_ip(ip):\n    assert ip == '192.168.64.2'\n    return ip\n").unwrap();
    // Preserve real cancellation signal handler, replace only resource operations.
    let lifecycle = source("vm_lifecycle.py");
    let start = lifecycle.find("class Cancelled").unwrap();
    let signals_start = lifecycle.find("@contextmanager\ndef _signals").unwrap();
    let signals_end = lifecycle[signals_start..]
        .find("\n\n@contextmanager\ndef session")
        .unwrap()
        + signals_start;
    fs::write(scripts.join("vm_lifecycle.py"),format!("from contextlib import contextmanager\nfrom types import SimpleNamespace\nimport signal\n{}\n{}\n{}", &lifecycle[start..lifecycle.find("@dataclass").unwrap()], &lifecycle[signals_start..signals_end], r#"
@contextmanager
def session(share,artifacts,*,binding):
    import vm_evidence
    vm_evidence.begin(binding)
    clone=SimpleNamespace(vm=binding.vm,artifacts=artifacts,keep_vm=False,tart_process=SimpleNamespace(pid=1234))
    vm_evidence.resources(binding,clone,'192.168.64.2')
    try: yield SimpleNamespace(clone=clone)
    finally:
        vm_evidence.complete(binding,dict(schema=1,vm=binding.vm,ip='192.168.64.2',tart_pid=1234,tart_exit=0,
            clone_absent=True,disk_detached=True,private_key_absent=True,owned_process_reaped=True,ssh_unreachable=True))
"#)).unwrap();
    fs::write(
        scripts.join("vm_cleanup.py"),
        format!(
            r#"
import fcntl, os
from pathlib import Path
def recheck(vm,artifacts,ip,pid):
    root=artifacts.parent.parent
    for name in ('host.lock','bbr.lock'):
        with (root/name).open('r+') as lock:
            try: fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
            except BlockingIOError: pass
            else: raise AssertionError('lease released before independent cleanup check')
    if os.environ.get('BBR_VISUAL_CONTROL') is None:
        (root/'host-observed').write_text('observed under both locks')
        if {reject}: raise RuntimeError('injected host resource rejection')
"#,
            reject = if [
                "host-reject",
                "selected-host-reject",
                "semantic-host-reject"
            ]
            .contains(&mode)
            {
                "True"
            } else {
                "False"
            }
        ),
    )
    .unwrap();
    let mut transport = source("vm_transport.py");
    transport.push_str(&format!(r#"
# Isolated test boundary only. The production module has no injection flags.
def _provision(*args): pass
def _exchange(session,script,workspace):
    if {failed}: raise TransportError('injected SSH operation failure')
    if {cancelled}: raise vm_lifecycle.Cancelled('injected selected cancellation')
    share=workspace.parent/'share'
    p=json.loads((share/'payload.json').read_text())
    digest=hashlib.sha256((share/'payload.json').read_bytes()).hexdigest()
    guest=dict(schema=1,origin='tart-guest',os='guest',os_build='guest-build',arch='arm64',model='VirtualMac-fixture',
        displays=[dict(origin='guest-coregraphics-metal',screens=[dict(id=1,width_px=6400,height_px=3600,width_pt=3200,height_pt=1800)],metal=dict(name='Synthetic Metal',registry_id=100))],fonts={{'/System/Library/Fonts/fixture.ttf':'c'*64}},console_user='admin',
        on_console=True,screen_locked=False,window_server=True,timezone='UTC',locale='C')
    receipt=dict(schema=1,payload_sha256=('f'*64 if {swapped} else digest),sources=p['sources'],
        installed={{f['guest']:f['sha256'] for f in p['files']}},environment_before=guest,environment_after=guest)
    raw=workspace.parent/'synthetic-guest-raw'
    raw.mkdir()
    (raw/'guest-receipt.json').write_text(json.dumps(receipt))
    if p['suite'] == 'gallery-semantic':
        plan='scripts/visual/catalogs/gallery-semantic.json'
        binary=next(f for f in p['files'] if '/deps/bbr_gallery-' in f['guest'])
        result=dict(schema=1,origin='gallery-semantic-libtest',payload_sha256=digest,
            plan_sha256=p['sources'][plan],executable=binary['guest'],executable_sha256=binary['sha256'],
            listing='e2e::fixture::one: test\ne2e::fixture::two: test\n2 tests, 0 benchmarks\n',
            execution='running 2 tests\ntest e2e::fixture::one ... ok\ntest e2e::fixture::two ... ok\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.01s\n')
        if {semantic_invalid}: result['execution']='failed synthetic fixture'
        (raw/'semantic-results.json').write_text(json.dumps(result))
        for suite in ('every-story-renders','mode-state-switching'):
            (raw/('semantic-'+suite+'.json')).write_text(json.dumps(dict(suite=suite,sanity_only=True,
                note='report-only frame-time observations; never a product budget',observations_ms={{'fixture':0.25}})))
    else:
        (raw/'fixture--light.png').write_bytes(b'fixture bytes, NOT PNG')
    request_path=workspace/'selected-request.json'
    if request_path.exists():
        selected=json.loads(request_path.read_text())['selection']
        if {selection_swap}: selected['stories'][0]['label']='swapped'
        if not {selection_missing}:
            (raw/'selection.json').write_text(json.dumps(selected,separators=(',',':'),ensure_ascii=False))
        if {request_swap}:
            request_path.chmod(0o600)
            request_path.write_text('{{}}')
    if {unexpected}: (raw/'selection.json').write_text('{{}}')
    vm_transfer.pack(raw,workspace/'return.tar',digest)
    if {semantic_source_drift}:
        (Path(p['repo'])/'fixture.rs').write_text('changed host source')
"#,failed=if ["failed", "selected-failed", "semantic-failed"].contains(&mode) {"True"} else {"False"},swapped=if ["swapped-receipt", "semantic-swapped-receipt"].contains(&mode) {"True"} else {"False"},
cancelled=if ["selected-cancelled", "semantic-cancelled"].contains(&mode) {"True"} else {"False"},
semantic_invalid=if mode=="semantic-invalid" {"True"} else {"False"},
semantic_source_drift=if mode=="semantic-source-drift" {"True"} else {"False"},
selection_swap=if mode=="selected-swapped" {"True"} else {"False"},
selection_missing=if mode=="selected-missing" {"True"} else {"False"},
request_swap=if mode=="selected-request-swap" {"True"} else {"False"},
unexpected=if mode=="unexpected-selection" {"True"} else {"False"}));
    fs::write(scripts.join("vm_transport.py"), transport).unwrap();
}

#[test]
fn supervised_transport_checks_both_leases_and_binds_return_before_success() {
    for mode in [
        "success",
        "failed",
        "host-reject",
        "swapped-receipt",
        "selected-success",
        "selected-missing",
        "selected-swapped",
        "selected-request-swap",
        "selected-failed",
        "selected-cancelled",
        "selected-host-reject",
        "unexpected-selection",
        "endpoint-full",
        "selected-endpoint",
        "semantic-success",
        "semantic-failed",
        "semantic-cancelled",
        "semantic-host-reject",
        "semantic-swapped-receipt",
        "semantic-invalid",
        "semantic-source-drift",
    ] {
        let semantic = mode.starts_with("semantic-");
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let repo = root.join("repo");
        fs::create_dir(&repo).unwrap();
        process::run(
            Command::new("git").args(["init", "-q"]).arg(&repo),
            &process::Limits::default(),
            &AtomicBool::new(false),
            true,
        )
        .unwrap();
        fixture_scripts(&repo, mode);
        fs::write(repo.join("fixture.rs"), "// inert fixture\n").unwrap();
        let config_text = r#"schema=1
capture=["never-executed"]
baselines="baselines"
report="report"
runs="runs"
story_catalog="scripts/visual/catalogs/stories.json"
[source_policy]
scopes=["fixture.rs","scripts/visual","visual-gallery.toml","vendor/cargo-gpui-visual"]
suffixes=[".rs",".py",".json",".toml",".lock"]
"#;
        fs::write(repo.join("visual-gallery.toml"), config_text).unwrap();
        for path in bbr_visual::vm_rendering::TOOL_INPUTS {
            let path = repo.join("vendor/cargo-gpui-visual").join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "synthetic capture input").unwrap();
        }
        let config = Config::from_toml(config_text).unwrap();
        let catalogs = repo.join("scripts/visual/catalogs");
        fs::create_dir(&catalogs).unwrap();
        fs::write(catalogs.join("stories.json"), json!({"schema":1,"stories":[{"id":"gallery-00","label":"Fixture","cases":["gallery-00-state-00-standard-normal-page-00"]}]}).to_string()).unwrap();
        fs::write(catalogs.join("gallery-plan.json"), json!({"schema":1,"cases":[{"base":"gallery-00-state-00-standard-normal","label":"Fixture","state":"Default","group":"Fixture","ranges":[[0,1]]}]}).to_string()).unwrap();
        fs::write(
            catalogs.join("gallery-semantic.json"),
            json!({"schema":1,"tests":["e2e::fixture::one","e2e::fixture::two"]}).to_string(),
        )
        .unwrap();
        let endpoints = mode.contains("endpoint");
        if endpoints {
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
            cargo_gpui_visual::store::write_json(
                &catalogs.join("gallery-plan.json"),
                &json!({"schema":2,"cases":cases}),
            )
            .unwrap();
            cargo_gpui_visual::store::write_json(
                &catalogs.join("stories.json"),
                &json!({"schema":1,"stories":[{
                    "id":"gallery-16","label":"Checkbox","cases":pages
                }]}),
            )
            .unwrap();
        }
        let selection = mode.starts_with("selected-").then(|| {
            Selection::resolve(
                &repo,
                &config,
                &[if endpoints {
                    "gallery-16"
                } else {
                    "gallery-00"
                }
                .into()],
            )
            .unwrap()
        });
        let mut artifacts = Vec::new();
        for (name, relative) in [
            (
                "bbr-gallery",
                if semantic {
                    "target/release/deps/bbr_gallery-fixture"
                } else {
                    "target/release/bbr-gallery"
                },
            ),
            ("bbr-visual", "scripts/visual/target/release/bbr-visual"),
        ] {
            let path = repo.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, "synthetic artifact, never execute").unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o500)).unwrap();
            artifacts.push(json!({"target":{"name":name,"kind":["bin"]},"profile":{"test":semantic && name=="bbr-gallery"},"features":["visual-capture"],"executable":path}));
        }
        let cancelled = AtomicBool::new(false);
        let sources = payload::source_snapshot(&repo, &config, &cancelled).unwrap();
        let share = root.join("share");
        let staged = payload::stage(
            &repo,
            &config,
            &sources,
            &artifacts,
            if semantic {
                payload::Purpose::GallerySemantic
            } else {
                Suite::Gallery.into()
            },
            &share,
            &cancelled,
        )
        .unwrap();
        let host = HostBuild {
            os: "host".into(),
            os_build: "host-build".into(),
            arch: "arm64".into(),
            rustc: "fixture".into(),
            xcode: "fixture".into(),
            sdk: "fixture".into(),
            gpui_source: "fixture".into(),
            tool_revision: "fixture".into(),
            executable_hashes: staged
                .files
                .iter()
                .filter(|e| e.executable)
                .map(|e| (e.guest.to_string_lossy().into_owned(), e.sha256.clone()))
                .collect(),
        };
        let workspace = root.join("transport");
        let options = Options {
            path: root.join("host.lock"),
            compatibility: vec![root.join("bbr.lock")],
            wait: Duration::from_millis(100),
        };
        let result = if semantic {
            vm_semantic::run(
                vm_semantic::Input {
                    repo: &repo,
                    share: &share,
                    staged: &staged,
                    host: &host,
                    workspace: &workspace,
                },
                &options,
                &cancelled,
            )
            .map(|verified| {
                verified
                    .recheck(
                        &staged,
                        &cargo_gpui_visual::store::digest(&share.join("payload.json")).unwrap(),
                        &host,
                        &cancelled,
                    )
                    .unwrap();
                None
            })
        } else {
            vm_transport::run(
                vm_transport::Input {
                    repo: &repo,
                    suite: Suite::Gallery,
                    share: &share,
                    staged: &staged,
                    host: &host,
                    workspace: &workspace,
                    selection: selection.as_ref(),
                },
                &options,
                &cancelled,
            )
            .map(|captured| captured.selection)
        };
        assert_eq!(
            result.is_ok(),
            ["success", "selected-success", "semantic-success"].contains(&mode),
            "{mode}: {result:?}"
        );
        if let Ok(returned_selection) = result.as_ref() {
            assert_eq!(*returned_selection, selection);
        }
        if [
            "selected-missing",
            "selected-swapped",
            "unexpected-selection",
        ]
        .contains(&mode)
        {
            assert!(
                result
                    .as_ref()
                    .unwrap_err()
                    .to_string()
                    .contains("returned selection"),
                "{mode}: {result:?}"
            );
        }
        if let Some(expected) = match mode {
            "semantic-invalid" => Some("semantic result count differs"),
            "semantic-swapped-receipt" => Some("guest used a different payload"),
            "semantic-source-drift" => Some("source/payload changed during transport"),
            _ => None,
        } {
            assert!(
                result.as_ref().unwrap_err().to_string().contains(expected),
                "{mode}: {result:?}"
            );
        }
        if endpoints {
            let error = result.unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("endpoint execution/evidence is not wired"),
                "{mode}: {error:#}"
            );
            assert!(
                !workspace.exists()
                    && !options.path.exists()
                    && !root.join("host-observed").exists()
            );
            continue;
        }
        assert!(root.join("host-observed").exists(), "{mode}: {result:?}");
        let gate: Value = serde_json::from_slice(&fs::read(&options.path).unwrap()).unwrap();
        assert_eq!(
            gate["active"],
            [
                "host-reject",
                "selected-host-reject",
                "semantic-host-reject"
            ]
            .contains(&mode),
            "{mode}"
        );
        assert!(!workspace.join("raw/capture.json").exists());
        if [
            "failed",
            "selected-failed",
            "selected-cancelled",
            "selected-request-swap",
            "semantic-failed",
            "semantic-cancelled",
        ]
        .contains(&mode)
        {
            assert!(!workspace.join("raw").exists());
        }
    }
}
