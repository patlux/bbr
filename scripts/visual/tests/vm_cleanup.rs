//! Concrete verifier wiring with synthetic resources, never Tart or a socket.
use bbr_visual::{
    vm_cleanup::{self, Expected},
    vm_supervision::{self, Policy},
};
use cargo_gpui_visual::gate::Options;
use serde_json::{json, Value};
use std::{fs, path::Path, process::Command, sync::atomic::AtomicBool, time::Duration};

fn evidence(root: &Path) -> Value {
    json!({"schema":1,"artifacts":root,"vm":"bbr-visual-123-abcdef123456",
        "request_sha256":"a".repeat(64),"payload_sha256":"b".repeat(64),
        "ip":"192.168.64.9","tart_pid":1234,"tart_exit":0,
        "clone_absent":true,"disk_detached":true,"private_key_absent":true,
        "owned_process_reaped":true,"ssh_unreachable":true})
}
#[test]
fn cleanup_evidence_requires_exact_binding_schema_and_every_absence() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let expected = Expected {
        artifacts: &root,
        vm: "bbr-visual-123-abcdef123456",
        request_sha256: &"a".repeat(64),
        payload_sha256: &"b".repeat(64),
    };
    let valid = evidence(&root);
    vm_cleanup::validate(&serde_json::to_vec(&valid).unwrap(), &expected).unwrap();
    for (key, value) in [
        ("schema", json!(true)),
        ("vm", json!("foreign")),
        ("payload_sha256", json!("c".repeat(64))),
        ("request_sha256", json!("c".repeat(64))),
        ("artifacts", json!(root.join("other"))),
        ("ip", json!("203.0.113.9")),
        ("tart_pid", json!(0)),
        ("tart_exit", json!(999)),
        ("extra", json!(true)),
        ("clone_absent", json!(false)),
        ("disk_detached", json!(false)),
        ("private_key_absent", json!(false)),
        ("owned_process_reaped", json!(false)),
        ("ssh_unreachable", json!(false)),
    ] {
        let mut bad = valid.clone();
        bad[key] = value;
        assert!(
            vm_cleanup::validate(&serde_json::to_vec(&bad).unwrap(), &expected).is_err(),
            "{key}"
        );
    }
    assert!(vm_cleanup::validate(&vec![b' '; 16385], &expected).is_err());
}

#[test]
fn real_host_verifier_refuses_missing_records_before_resource_commands() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let expected = Expected {
        artifacts: &root,
        vm: "bbr-visual-123-abcdef123456",
        request_sha256: &"a".repeat(64),
        payload_sha256: &"b".repeat(64),
    };
    assert!(vm_cleanup::verify(&repo, &expected).is_err());
    assert!(fs::read_dir(root).unwrap().next().is_none());
}

// An isolated stub implements only read-only resource observation. All record,
// lifecycle, cancellation, publication and Rust evidence validation code is real.
const CHILD: &str = r#"
import json, os, sys
from pathlib import Path
from types import SimpleNamespace
sys.path.insert(0, sys.argv[1])
import vm_control, vm_evidence
root, mode = Path(sys.argv[2]), sys.argv[3]
control = Path(os.environ['BBR_VISUAL_CONTROL'])
request = os.environ['BBR_VISUAL_REQUEST_SHA256']
binding = vm_evidence.Binding(root/'diagnostics', 'bbr-visual-123-abcdef123456', request, 'b'*64)
def operation():
    vm_evidence.begin(binding)
    clone = SimpleNamespace(vm=binding.vm, artifacts=binding.artifacts, keep_vm=False,
                            tart_process=SimpleNamespace(pid=1234))
    vm_evidence.resources(binding, clone, '192.168.64.9')
    vm_evidence.complete(binding, dict(schema=1, vm=binding.vm, ip='192.168.64.9', tart_pid=1234,
        tart_exit=0, clone_absent=True, disk_detached=True, private_key_absent=True,
        owned_process_reaped=True, ssh_unreachable=True))
    if mode == 'failed': raise RuntimeError('synthetic capture failure')
try: vm_control.run(control, request, operation, lambda: vm_evidence.verify(binding))
except BaseException: sys.exit(2)
"#;

#[test]
fn bound_verifier_is_called_under_both_leases_and_rejection_preserves_gate() {
    use std::os::unix::fs::PermissionsExt;
    for mode in ["success", "failed", "host-reject"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let scripts = root.join("repo/scripts/visual");
        fs::create_dir_all(&scripts).unwrap();
        // Copy only record dependencies from this source tree. The stub forbids
        // any network/Tart operation by providing no transport implementation.
        for name in ["vm_evidence.py", "vm_install.py", "vm_control.py"] {
            fs::copy(
                Path::new(env!("CARGO_MANIFEST_DIR")).join(name),
                scripts.join(name),
            )
            .unwrap();
        }
        fs::write(scripts.join("vm_lifecycle.py"), "from contextlib import nullcontext\nclass Cancelled(BaseException): pass\n_signals = nullcontext\n").unwrap();
        fs::write(
            scripts.join("vm_policy.py"),
            "class PolicyError(RuntimeError): pass\ndef validate_ip(ip):\n    assert ip == '192.168.64.9'\n    return ip\n",
        )
        .unwrap();
        fs::write(
            scripts.join("vm_cleanup.py"),
            format!(
                r#"
import fcntl, os
from pathlib import Path
def recheck(vm, artifacts, ip, pid):
    root = artifacts.parent
    for name in ('host.lock', 'bbr.lock'):
        with (root/name).open('r+') as lock:
            try: fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError: pass
            else: raise AssertionError('lease released too early')
    host = os.environ.get('BBR_VISUAL_CONTROL') is None
    if host:
        (root/'host-observed').write_text('both leases held')
        if {reject}: raise RuntimeError('host rejected resource absence')
"#,
                reject = if mode == "host-reject" {
                    "True"
                } else {
                    "False"
                }
            ),
        )
        .unwrap();
        let diagnostics = root.join("diagnostics");
        fs::create_dir(&diagnostics).unwrap();
        fs::set_permissions(&diagnostics, fs::Permissions::from_mode(0o700)).unwrap();
        let options = Options {
            path: root.join("host.lock"),
            compatibility: vec![root.join("bbr.lock")],
            wait: Duration::from_millis(100),
        };
        let control = root.join("control");
        let result = vm_supervision::run(
            &options,
            Command::new("python3")
                .args(["-I", "-B", "-c", CHILD])
                .arg(&scripts)
                .arg(&root)
                .arg(mode),
            &control,
            &"b".repeat(64),
            &Policy {
                operation: Duration::from_secs(5),
                grace: Duration::from_secs(3),
                ..Default::default()
            },
            &AtomicBool::new(false),
            |request| {
                vm_cleanup::verify(
                    &root.join("repo"),
                    &Expected {
                        artifacts: &diagnostics,
                        vm: "bbr-visual-123-abcdef123456",
                        request_sha256: request,
                        payload_sha256: &"b".repeat(64),
                    },
                )?;
                Ok(())
            },
        );
        assert_eq!(result.is_ok(), mode == "success", "{mode}: {result:?}");
        assert!(root.join("host-observed").exists());
        let gate: Value = serde_json::from_slice(&fs::read(&options.path).unwrap()).unwrap();
        assert_eq!(gate["active"], mode == "host-reject", "{mode}");
    }
}
