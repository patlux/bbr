"""Bound selected-request transport only; Rust owns catalog/plan validation.

Exact UTF-8 bytes + independent digest pass through the frozen bootstrap. No
selection inference, arbitrary path, producer execution, receipt or seal authority.
"""
import hashlib
import json
from pathlib import Path

import vm_install

LIMIT = 64 * 1024
NAME = "selected-request.json"


def decode(text: str | None, expected: str | None, payload: str, suite: str) -> bytes | None:
    if text is None and expected is None:
        return None
    if not isinstance(text, str) or expected is None or suite != "gallery":
        raise ValueError("selected request requires explicit Gallery bytes/hash")
    vm_install._digest(expected)
    raw = text.encode("utf-8")
    if not 0 < len(raw) <= LIMIT or hashlib.sha256(raw).hexdigest() != expected:
        raise ValueError("selected request bytes/hash differ or exceed bound")
    value = json.loads(raw, object_pairs_hook=vm_install._unique)
    if (not isinstance(value, dict) or set(value) != {"schema", "payload_sha256", "selection"}
            or type(value["schema"]) is not int or value["schema"] != 1
            or value["payload_sha256"] != payload or not isinstance(value["selection"], dict)):
        raise ValueError("selected request payload/schema differs")
    return raw


def read(work: Path, expected: str, payload: str, suite: str) -> str:
    handle, _ = vm_install._open(work / NAME, LIMIT)
    with handle:
        text = handle.read(LIMIT + 1).decode("utf-8")
    decode(text, expected, payload, suite)
    return text


def write(work: Path, text: str, expected: str, payload: str, suite: str) -> None:
    raw = decode(text, expected, payload, suite)
    vm_install._absolute(work / NAME)
    with (work / NAME).open("xb") as target:
        target.write(raw)
    read(work, expected, payload, suite)
