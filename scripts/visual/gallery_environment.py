#!/usr/bin/env python3
"""Provenance for production Gallery pixels, not native browser/window evidence."""
import json

from environment import host_environment
from support import install_signal_handlers


def environment() -> dict:
    return {
        **host_environment(),
        "capture_scope": "gallery-registered-selector-theme-pages",
        "renderer": "GPUI-CE HeadlessAppContext / MetalHeadlessRenderer",
        "platform": "test window/scheduler, real macOS text and Metal sprite atlas",
        "native_window": False,
        "width_pt": 1200, "height_pt": 1000, "scale": 2,
        "scale_policy": "assert exact 2400x2000 readback; never resample",
        "appearance": ["light", "dark"],
        "contrast": ["standard", "increased"],
        "density": ["small", "normal", "extralarge"],
        "theme_policy": "canvas appearance; intrinsic family density and stronger accessibility overrides",
        "reduce_motion": True,
        "readiness": "measured child pages; finite production opacity duration before final layout",
        "seed": 0, "timezone": "UTC", "locale": "C",
        "coverage_limits": "interactive-state applicability and nested/shadow framing audit pending",
        "native_pixels": "excluded: WKWebView, AppKit traffic lights and portal child",
    }


if __name__ == "__main__":
    install_signal_handlers()
    print(json.dumps(environment(), sort_keys=True))
