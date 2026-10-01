"""Request-bound cleanup evidence and independent host read-only recheck.

These records establish integrity/ordering, not authentication against malicious
same-user code. No repairs, resource reuse, key reads or numeric-PID signals
(except signal 0 for existence). Partial boot cannot manufacture absence proof.
"""
from __future__ import annotations

from dataclasses import dataclass
import json
import ipaddress
from pathlib import Path
import re

import vm_install


class EvidenceError(RuntimeError):
    pass


@dataclass(frozen=True)
class Binding:
    artifacts: Path
    vm: str
    request_sha256: str
    payload_sha256: str

    def fields(self) -> dict:
        vm_install._absolute(self.artifacts)
        if (not self.artifacts.is_dir() or self.artifacts.stat().st_mode & 0o077
                or not re.fullmatch(r"bbr-visual-[0-9]+-[0-9a-f]{12}", self.vm)):
            raise EvidenceError("private diagnostics and exact visual clone required")
        for value in (self.request_sha256, self.payload_sha256):
            vm_install._digest(value)
        return {"schema": 1, "artifacts": str(self.artifacts), "vm": self.vm,
                "request_sha256": self.request_sha256, "payload_sha256": self.payload_sha256}


def _read(path: Path) -> dict:
    handle, _ = vm_install._open(path, 16384)
    with handle:
        raw = handle.read(16385)
    if len(raw) > 16384:
        raise EvidenceError("cleanup record grew beyond bound")
    value = json.loads(raw, object_pairs_hook=vm_install._unique)
    if not isinstance(value, dict):
        raise EvidenceError("cleanup record object required")
    return value


def _publish(binding: Binding, name: str, fields: dict) -> None:
    # Reuse the control protocol's atomic no-replace publication, not a second
    # acceptance journal. Readers run after publication/transport completion.
    from vm_control import _publish as publish
    publish(binding.artifacts / name, {**binding.fields(), **fields})


def begin(binding: Binding) -> None:
    """Record the independently chosen clone before any boot mutation."""
    _publish(binding, "owned-run.json", {})


def _identity(binding: Binding) -> dict:
    expected = binding.fields()
    value = _read(binding.artifacts / "owned-run.json")
    if value != expected or type(value.get("schema")) is not int:
        raise EvidenceError("owned run identity differs")
    return expected


def _ip(ip: str) -> str:
    if not isinstance(ip, str):
        raise EvidenceError("canonical private guest IP required")
    from vm_policy import PolicyError, validate_ip
    try:
        validate_ip(ip)
    except PolicyError as error:
        raise EvidenceError("canonical private guest IP required") from error
    if str(ipaddress.IPv4Address(ip)) != ip:
        raise EvidenceError("canonical private guest IP required")
    return ip


def resources(binding: Binding, clone, ip: str) -> None:
    """Called before audit/producer, only for the lifecycle's actual Popen."""
    _identity(binding)
    if (clone.vm != binding.vm or clone.artifacts != binding.artifacts
            or clone.keep_vm or clone.tart_process is None):
        raise EvidenceError("owned resource identity differs")
    pid = clone.tart_process.pid
    if type(pid) is not int or not 1 < pid <= 2**31 - 1:
        raise EvidenceError("invalid owned Tart PID")
    _publish(binding, "owned-resources.json", {"ip": _ip(ip), "tart_pid": pid})


def _resources(binding: Binding) -> dict:
    expected = _identity(binding)
    value = _read(binding.artifacts / "owned-resources.json")
    if (set(value) != {*expected, "ip", "tart_pid"}
            or any(value.get(k) != v for k, v in expected.items())
            or type(value.get("schema")) is not int
            or type(value.get("tart_pid")) is not int or not 1 < value["tart_pid"] <= 2**31 - 1):
        raise EvidenceError("owned resource record differs")
    _ip(value["ip"])
    return value


def _completed(binding: Binding, observed: dict) -> dict:
    resource = _resources(binding)
    flags = {key: True for key in ("clone_absent", "disk_detached", "private_key_absent",
                                   "owned_process_reaped", "ssh_unreachable")}
    expected = {"schema": 1, "vm": binding.vm, "ip": resource["ip"],
                "tart_pid": resource["tart_pid"], **flags}
    if (set(observed) != {*expected, "tart_exit"}
            or any(observed.get(k) != v for k, v in expected.items())
            or type(observed.get("schema")) is not int
            or type(observed.get("tart_pid")) is not int
            or any(observed.get(k) is not True for k in flags)
            or type(observed.get("tart_exit")) is not int
            or not -255 <= observed["tart_exit"] <= 255):
        raise EvidenceError("cleanup observation differs from owned resources")
    return {**resource, **flags, "tart_exit": observed["tart_exit"]}


def complete(binding: Binding, observed: dict) -> None:
    value = _completed(binding, observed)
    _publish(binding, "cleanup-observed.json", value)


def verify(binding: Binding) -> dict:
    """Recheck on the host after child exit; records alone never prove absence."""
    resource = _resources(binding)
    recorded = _read(binding.artifacts / "cleanup-observed.json")
    observed = {key: value for key, value in recorded.items()
                if key not in {"artifacts", "request_sha256", "payload_sha256"}}
    if recorded != _completed(binding, observed):
        raise EvidenceError("cleanup record belongs to another run")
    # Child ownership/reaping is recorded; host independently checks absence
    # without trying to waitpid or signal a PID it does not own.
    from vm_cleanup import recheck
    recheck(binding.vm, binding.artifacts, resource["ip"], resource["tart_pid"])
    if (resource != _resources(binding)
            or recorded != _read(binding.artifacts / "cleanup-observed.json")):
        raise EvidenceError("cleanup evidence changed during host observation")
    return recorded


if __name__ == "__main__":
    import sys
    try:
        if len(sys.argv) != 5:
            raise EvidenceError("expected diagnostics, clone, request and payload digests")
        print(json.dumps(verify(Binding(Path(sys.argv[1]), *sys.argv[2:])), sort_keys=True))
    except (EvidenceError, vm_install.InstallError, OSError, ValueError) as error:
        print(f"cleanup unproven: {error}", file=sys.stderr)
        raise SystemExit(2)
