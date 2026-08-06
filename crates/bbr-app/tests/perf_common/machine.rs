//! Machine/build identity for benchmark artifacts and canonical-machine
#![allow(dead_code)]
//! classification. Collected through std process probes and `cfg!`; no new
//! dependencies.

use serde::{Deserialize, Serialize};
use std::process::Command;

/// Machine + build identity stamped onto every report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineInfo {
    pub build_revision: String,
    pub build_profile: String,
    pub binary: String,
    pub engine_kind: String,
    pub fixture_mode: String,
    pub host_model: String,
    pub arch: String,
    pub os_version: String,
    /// Display refresh in Hz when the OS API is reliably obtainable. The
    /// benchmark harness keeps the window offscreen/headless, so this is
    /// recorded as `null` with an explicit reason unless a display blocker
    /// reports otherwise.
    pub display_refresh_hz: Option<f64>,
    pub display_refresh_reason: String,
    /// True when `host_model` matches the checked-in canonical machine.
    pub canonical: bool,
}

impl MachineInfo {
    pub fn collect(
        binary: &str,
        engine_kind: &str,
        fixture_mode: &str,
        canonical_host_model: &str,
    ) -> Self {
        let host_model = sh("sysctl", &["-n", "hw.model"]).unwrap_or_else(|| "unknown".into());
        let arch = sh("sysctl", &["-n", "hw.machine"])
            .or_else(|| sh("uname", &["-m"]))
            .unwrap_or_else(|| "unknown".into());
        let os_version = sh("sw_vers", &["-productVersion"])
            .or_else(|| sh("sw_vers", &["-productName"]))
            .unwrap_or_else(|| "unknown".into());
        let build_revision = std::env::var("BBR_BUILD_REVISION")
            .ok()
            .or_else(|| sh("git", &["rev-parse", "HEAD"]))
            .unwrap_or_else(|| "unknown".into());
        let build_profile = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
        .to_string();
        let canonical = !canonical_host_model.is_empty() && canonical_host_model == host_model;
        Self {
            build_revision,
            build_profile,
            binary: binary.to_string(),
            engine_kind: engine_kind.to_string(),
            fixture_mode: fixture_mode.to_string(),
            host_model,
            arch,
            os_version,
            display_refresh_hz: None,
            display_refresh_reason:
                "not sampled: the benchmark window runs offscreen and the OS refresh API is not \
                 linked in the harness; record on a display-attached canonical runner"
                    .to_string(),
            canonical,
        }
    }
}

fn sh(bin: &str, args: &[&str]) -> Option<String> {
    Command::new(bin)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_returns_non_empty_identity() {
        let info = MachineInfo::collect("/bin/bbr", "native-wkwebview", "cold-fresh", "");
        assert!(!info.host_model.is_empty());
        assert!(!info.arch.is_empty());
        assert!(matches!(info.build_profile.as_str(), "debug" | "release"));
        assert!(info.display_refresh_hz.is_none());
        assert!(!info.display_refresh_reason.is_empty());
    }

    #[test]
    fn canonical_flag_true_only_on_exact_match() {
        let info = MachineInfo::collect("/bin/bbr", "native-wkwebview", "cold-fresh", "Mac16,5");
        // On a machine whose hw.model is Mac16,5 this flips; the important
        // invariant is the boolean is derived from the exact comparison.
        assert_eq!(info.canonical, info.host_model == "Mac16,5");
    }
}
