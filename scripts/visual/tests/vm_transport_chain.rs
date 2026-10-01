//! Full Rust supervisor -> Python transport -> return/receipt path, with only
//! runtime, lifecycle/resource and SSH boundaries replaced by inert local fixtures.
use bbr_visual::{payload, vm_environment::HostBuild, vm_transport, Suite};
use cargo_gpui_visual::{config::Config, gate::Options, process};
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
            reject = if mode == "host-reject" {
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
def _exchange(session,script,workspace):
    if {failed}: raise TransportError('injected SSH operation failure')
    share=workspace.parent/'share'
    p=json.loads((share/'payload.json').read_text())
    digest=hashlib.sha256((share/'payload.json').read_bytes()).hexdigest()
    guest=dict(schema=1,origin='tart-guest',os='guest',os_build='guest-build',arch='arm64',model='VirtualMac-fixture',
        displays=[dict(synthetic=True)],fonts={{'/System/Library/Fonts/fixture.ttf':'c'*64}},console_user='admin',
        on_console=True,screen_locked=False,window_server=True,timezone='UTC',locale='C')
    receipt=dict(schema=1,payload_sha256=('f'*64 if {swapped} else digest),sources=p['sources'],
        installed={{f['guest']:f['sha256'] for f in p['files']}},environment_before=guest,environment_after=guest)
    raw=workspace.parent/'synthetic-guest-raw'
    raw.mkdir()
    (raw/'guest-receipt.json').write_text(json.dumps(receipt))
    (raw/'fixture--light.png').write_bytes(b'fixture bytes, NOT PNG')
    vm_transfer.pack(raw,workspace/'return.tar',digest)
"#,failed=if mode=="failed" {"True"} else {"False"},swapped=if mode=="swapped-receipt" {"True"} else {"False"}));
    fs::write(scripts.join("vm_transport.py"), transport).unwrap();
}

#[test]
fn supervised_transport_checks_both_leases_and_binds_return_before_success() {
    for mode in ["success", "failed", "host-reject", "swapped-receipt"] {
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
[source_policy]
scopes=["fixture.rs","scripts/visual","visual-gallery.toml"]
suffixes=[".rs",".py",".json",".toml"]
"#;
        fs::write(repo.join("visual-gallery.toml"), config_text).unwrap();
        let config = Config::from_toml(config_text).unwrap();
        let mut artifacts = Vec::new();
        for (name, relative) in [
            ("bbr-gallery", "target/release/bbr-gallery"),
            ("bbr-visual", "scripts/visual/target/release/bbr-visual"),
        ] {
            let path = repo.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, "synthetic artifact, never execute").unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o500)).unwrap();
            artifacts
                .push(json!({"target":{"name":name},"profile":{"test":false},"executable":path}));
        }
        let cancelled = AtomicBool::new(false);
        let sources = payload::source_snapshot(&repo, &config, &cancelled).unwrap();
        let share = root.join("share");
        let staged = payload::stage(
            &repo,
            &config,
            &sources,
            &artifacts,
            Suite::Gallery,
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
        let result = vm_transport::run(
            vm_transport::Input {
                repo: &repo,
                suite: Suite::Gallery,
                share: &share,
                staged: &staged,
                host: &host,
                workspace: &workspace,
            },
            &options,
            &cancelled,
        );
        assert_eq!(result.is_ok(), mode == "success", "{mode}: {result:?}");
        assert!(root.join("host-observed").exists(), "{mode}: {result:?}");
        let gate: Value = serde_json::from_slice(&fs::read(&options.path).unwrap()).unwrap();
        assert_eq!(gate["active"], mode == "host-reject", "{mode}");
        assert!(!workspace.join("raw/capture.json").exists());
        if mode == "failed" {
            assert!(!workspace.join("raw").exists());
        }
    }
}
