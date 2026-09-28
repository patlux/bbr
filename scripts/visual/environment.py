#!/usr/bin/env python3
"""Read-only provenance for the exact BBR chrome capture contract."""
import json
import platform
import tomllib

from support import ROOT, install_signal_handlers, run, verify_tool


def output(*argv: str) -> str:
    return run(argv, capture=True, seconds=30).stdout.strip()


def host_environment() -> dict:
    if platform.system() != "Darwin":
        raise ValueError("BBR visual capture requires macOS")
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
    gpui = next(p for p in lock["package"] if p["name"] == "gpui")
    tool = verify_tool()
    return {
        "layer": "gpui",
        "gpui_source": gpui["source"],
        "tool_revision": tool["source_revision"],
        "os": output("/usr/bin/sw_vers", "-productVersion"),
        "os_build": output("/usr/bin/sw_vers", "-buildVersion"),
        "arch": platform.machine(),
        "host_model": output("/usr/sbin/sysctl", "-n", "hw.model"),
        "rustc": output("rustc", "--version"),
        "xcode": output("/usr/bin/xcodebuild", "-version"),
        "sdk": output("/usr/bin/xcrun", "--sdk", "macosx", "--show-sdk-version"),
        "text_system": "CoreText / system UI fonts (OS-build bound)",
    }


def environment() -> dict:
    return {
        **host_environment(),
        "capture_scope": "production-chrome-only",
        "renderer": "GPUI-CE Window::render_to_image / Metal",
        "width_pt": 1080, "height_pt": 1407, "rail_width_pt": 304,
        "scale": 2, "scale_policy": "assert exact 2160x2814 readback; never resample",
        "appearance": ["light", "dark"], "contrast": "standard", "reduce_motion": True,
        "readiness": "typed settled geometry, no loading or transient moments",
        "timezone": "UTC", "locale": "C",
        "native_pixels": "excluded: WKWebView, AppKit traffic lights and portal child",
    }


if __name__ == "__main__":
    install_signal_handlers()
    print(json.dumps(environment(), sort_keys=True))
