"""Clone-only pre-boot hardening and strict observation validation.

No function discovers credentials, opens sockets or starts a VM. The mounted
Data volume must come from tart_vm.boot's explicit disposable-clone callback.
"""
from __future__ import annotations

import ipaddress
import os
from pathlib import Path
import plistlib

SCHEMA = 1
SSH_SETTINGS = {
    "pubkeyauthentication": "yes",
    "passwordauthentication": "no",
    "kbdinteractiveauthentication": "no",
    "authenticationmethods": "publickey",
    "permitrootlogin": "no",
    "disableforwarding": "yes",
    "allowtcpforwarding": "no",
    "allowagentforwarding": "no",
    "x11forwarding": "no",
    "permittunnel": "no",
    "gatewayports": "no",
    "authorizedkeyscommand": "none",
    "trustedusercakeys": "none",
    "hostbasedauthentication": "no",
    "allowusers": "admin",
    "authorizedkeysfile": ".ssh/authorized_keys",
}
DISABLED_JOBS = ("com.apple.screensharing", "com.apple.smbd", "com.apple.AppleFileServer",
                 "com.apple.Kerberos.kdc", "com.apple.RemoteDesktop.PrivilegeProxy")


class PolicyError(RuntimeError):
    pass


def _plain(path: Path) -> None:
    for item in (path, *path.parents):
        if item.is_symlink():
            raise PolicyError(f"symlink in clone policy path: {item}")


def harden_offline(data: Path) -> None:
    """Replace clone sshd config (no Includes/Match overrides) before first boot.

    The source image and host are never targets. Existing extra authorized keys
    are separately replaced by tart_vm's exclusive_key option.
    """
    if not data.is_absolute() or data == Path("/") or data == Path("/System/Volumes/Data"):
        raise PolicyError("explicit offline clone Data mount required")
    _plain(data)
    account = data / "Users/admin"
    ssh = data / "private/etc/ssh/sshd_config"
    disabled = data / "private/var/db/com.apple.xpc.launchd/disabled.plist"
    for path in (account, ssh, disabled):
        _plain(path)
    if not account.is_dir() or not ssh.is_file():
        raise PolicyError("not an initialized clone Data volume")
    # OpenSSH's first-value/Include semantics cannot revive a weaker setting:
    # this isolated image gets one complete minimal config, not an appended line.
    text = "# Disposable BBR visual clone only\nPort 22\n" + "".join(
        f"{name} {value}\n" for name, value in SSH_SETTINGS.items()
    ) + "Subsystem sftp internal-sftp\nUsePAM yes\n"
    ssh.write_text(text)
    os.chmod(ssh, 0o600)
    jobs = plistlib.loads(disabled.read_bytes()) if disabled.exists() else {}
    if not isinstance(jobs, dict):
        raise PolicyError("invalid clone disabled-service dictionary")
    jobs.update({name: True for name in DISABLED_JOBS})
    disabled.parent.mkdir(parents=True, exist_ok=True)
    disabled.write_bytes(plistlib.dumps(jobs))
    os.chmod(disabled, 0o600)


def validate_ip(raw: str) -> str:
    try:
        ip = ipaddress.IPv4Address(raw)
    except ipaddress.AddressValueError as exc:
        raise PolicyError("one private Tart IPv4 address required") from exc
    if not any(ip in ipaddress.IPv4Network(net) for net in (
        "10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16"
    )):
        raise PolicyError("Tart address is not RFC1918 private NAT")
    return str(ip)


def validate_sshd(text: str) -> None:
    actual: dict[str, list[str]] = {}
    for line in text.splitlines():
        parts = line.split(None, 1)
        if len(parts) != 2:
            raise PolicyError("malformed effective sshd setting")
        key, value = parts
        values = actual.setdefault(key, [])
        # OpenSSH -T emits one line per address and host-key path. Only these
        # observed list fields may repeat; security-policy fields stay singular.
        if values and (key not in {"listenaddress", "hostkey"} or value in values):
            raise PolicyError("duplicate effective sshd setting")
        values.append(value)
    for key, value in {**SSH_SETTINGS, "port": "22"}.items():
        if actual.get(key) != [value]:
            raise PolicyError(f"unsafe/missing effective sshd setting: {key}")


def validate_listeners(text: str) -> None:
    """Parse lsof -nP -iTCP -sTCP:LISTEN -Fpcn output.

    Before starting fixtures, allow only launchd/sshd TCP 22 off loopback.
    Reject unrecognized records instead of interpreting a failed audit as empty.
    """
    command = None
    found_ssh = False
    for line in text.splitlines():
        if line.startswith("p") and line[1:].isdigit():
            command = None
        elif line.startswith("c"):
            command = line[1:]
        elif line.startswith("n"):
            endpoint = line[1:]
            address, colon, port = endpoint.rpartition(":")
            if not colon or not port.isdigit() or command is None:
                raise PolicyError("malformed listener observation")
            if port == "22" and command in {"launchd", "sshd"}:
                found_ssh = True
                continue
            try:
                loopback = ipaddress.ip_address(address.strip("[]")).is_loopback
            except ValueError:
                loopback = False
            if not loopback:
                raise PolicyError(f"unexpected non-loopback TCP listener: {port}")
        elif line.startswith("f") and line[1:].isdigit():
            continue
        else:
            raise PolicyError("unrecognized listener audit record")
    if not found_ssh:
        raise PolicyError("SSH listener was not observed")
