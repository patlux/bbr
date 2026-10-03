use bbr_visual::{
    payload::{Entry, Payload},
    vm_environment::{self, Guest, HostBuild, Receipt},
    Suite,
};
use serde_json::json;
use std::{collections::BTreeMap, path::PathBuf};

fn fixture() -> (HostBuild, Payload, Receipt) {
    let repo = PathBuf::from("/Users/fixture/bbr");
    let binary = repo.join("target/release/bbr-gallery");
    let mut sources = BTreeMap::from([("source.rs".into(), "a".repeat(64))]);
    sources.extend(
        bbr_visual::vm_rendering::TOOL_INPUTS
            .iter()
            .map(|p| (format!("vendor/cargo-gpui-visual/{p}"), "f".repeat(64))),
    );
    let artifacts = BTreeMap::from([(binary.to_str().unwrap().into(), "b".repeat(64))]);
    let host = HostBuild {
        os: "27.0".into(),
        os_build: "host-build".into(),
        arch: "arm64".into(),
        rustc: "rustc fixture".into(),
        xcode: "Xcode fixture".into(),
        sdk: "27.0".into(),
        gpui_source: "git+fixture#pin".into(),
        tool_revision: "fixture".into(),
        executable_hashes: artifacts.clone(),
    };
    let payload = Payload {
        schema: 1,
        suite: "gallery".into(),
        repo: repo.clone(),
        sources: sources.clone(),
        files: vec![
            Entry {
                payload: "source/source.rs".into(),
                guest: repo.join("source.rs"),
                sha256: "a".repeat(64),
                bytes: 4,
                executable: false,
            },
            Entry {
                payload: "executables/0".into(),
                guest: binary,
                sha256: "b".repeat(64),
                bytes: 4,
                executable: true,
            },
        ],
    };
    let guest = Guest {
        schema: 1,
        origin: "tart-guest".into(),
        os: "27.0".into(),
        os_build: "guest-build".into(),
        arch: "arm64".into(),
        model: "VirtualMac2,1".into(),
        displays: json!([{"origin":"guest-coregraphics-metal","screens":[{"id":1,"width_px":6400,"height_px":3600,"width_pt":3200,"height_pt":1800}],"metal":{"name":"Synthetic Metal","registry_id":100}}]),
        fonts: BTreeMap::from([("/System/Library/Fonts/fixture.ttf".into(), "c".repeat(64))]),
        console_user: "admin".into(),
        on_console: true,
        screen_locked: false,
        window_server: true,
        timezone: "UTC".into(),
        locale: "C".into(),
    };
    let mut installed = artifacts;
    installed.insert(
        repo.join("source.rs").to_str().unwrap().into(),
        "a".repeat(64),
    );
    let receipt = Receipt {
        schema: 1,
        payload_sha256: "d".repeat(64),
        sources,
        installed,
        environment_before: guest.clone(),
        environment_after: guest,
    };
    (host, payload, receipt)
}
#[test]
fn host_build_and_guest_pixels_keep_distinct_identity() {
    let (host, payload, receipt) = fixture();
    let environment =
        vm_environment::bind(Suite::Gallery, &host, &payload, &"d".repeat(64), &receipt).unwrap();
    assert_eq!(
        environment["rendering"]["host_build"]["os_build"],
        "host-build"
    );
    assert_eq!(
        environment["rendering"]["guest_rendering"]["os_build"],
        "guest-build"
    );
    assert_eq!(environment["layer"], "gpui");
    assert!(environment["rendering"]["host_build"]
        .get("executable_hashes")
        .is_none());
    assert!(environment.get("payload_sha256").is_none());
}
#[test]
fn drift_in_guest_environment_or_transfer_identity_fails_closed() {
    for mutation in [
        "fonts",
        "os",
        "display",
        "missing-source",
        "missing-binary",
        "extra-file",
        "payload",
        "suite",
        "schema",
        "host-binary",
        "host-arch",
    ] {
        let (mut host, mut payload, mut receipt) = fixture();
        match mutation {
            "fonts" => {
                receipt
                    .environment_after
                    .fonts
                    .values_mut()
                    .next()
                    .unwrap()
                    .replace_range(.., "e".repeat(64).as_str());
            }
            "os" => receipt.environment_after.os_build = "changed".into(),
            "display" => receipt.environment_after.displays = json!([{"changed":true}]),
            "missing-source" => receipt.sources.clear(),
            "missing-binary" => receipt.installed.retain(|key, _| !key.contains("target")),
            "extra-file" => {
                receipt.installed.insert("/extra".into(), "e".repeat(64));
            }
            "payload" => receipt.payload_sha256 = "e".repeat(64),
            "suite" => payload.suite = "browser".into(),
            "schema" => receipt.schema = 2,
            "host-binary" => host.executable_hashes.clear(),
            "host-arch" => host.arch = "x86_64".into(),
            _ => unreachable!(),
        }
        assert!(
            vm_environment::bind(Suite::Gallery, &host, &payload, &"d".repeat(64), &receipt)
                .is_err(),
            "{mutation}"
        );
    }
}
#[test]
fn missing_or_host_substituted_guest_observations_cannot_validate() {
    for mutation in [
        "origin",
        "model",
        "schema",
        "locked",
        "console",
        "user",
        "window-server",
        "locale",
        "timezone",
        "display",
        "fonts",
        "font-path",
        "font-hash",
    ] {
        let (host, payload, mut receipt) = fixture();
        let guest = &mut receipt.environment_before;
        match mutation {
            "origin" => guest.origin = "host".into(),
            "model" => guest.model = "Mac16,9".into(),
            "schema" => guest.schema = 2,
            "locked" => guest.screen_locked = true,
            "console" => guest.on_console = false,
            "user" => guest.console_user = "patwoz".into(),
            "window-server" => guest.window_server = false,
            "locale" => guest.locale = "de_DE".into(),
            "timezone" => guest.timezone = "local".into(),
            "display" => guest.displays = json!([]),
            "fonts" => guest.fonts.clear(),
            "font-path" => {
                guest.fonts = BTreeMap::from([("/Users/fixture/font.ttf".into(), "a".repeat(64))])
            }
            "font-hash" => guest.fonts.values_mut().next().unwrap().clear(),
            _ => unreachable!(),
        }
        receipt.environment_after = receipt.environment_before.clone();
        assert!(
            vm_environment::bind(Suite::Gallery, &host, &payload, &"d".repeat(64), &receipt)
                .is_err(),
            "{mutation}"
        );
    }
}
#[test]
fn only_validated_provenance_values_are_non_gating_between_runs() {
    use cargo_gpui_visual::environment;
    let (mut host, payload, mut receipt) = fixture();
    let original =
        vm_environment::bind(Suite::Gallery, &host, &payload, &"d".repeat(64), &receipt).unwrap();
    host.tool_revision = "next-tool-revision".into();
    receipt.environment_before.displays[0]["metal"]["registry_id"] = json!(200);
    receipt.environment_after = receipt.environment_before.clone();
    let changed =
        vm_environment::bind(Suite::Gallery, &host, &payload, &"d".repeat(64), &receipt).unwrap();
    assert_ne!(original, changed);
    assert!(environment::equivalent(&original, &changed).unwrap());
    let raw = environment::observation(&changed).unwrap();
    assert_eq!(raw["host_build"]["tool_revision"], "next-tool-revision");
    assert_eq!(
        raw["guest_rendering"]["displays"][0]["metal"]["registry_id"],
        200
    );
    receipt.environment_after.displays[0]["metal"]["registry_id"] = json!(201);
    assert!(
        vm_environment::bind(Suite::Gallery, &host, &payload, &"d".repeat(64), &receipt).is_err()
    );
}
#[test]
fn every_rendering_condition_and_tool_input_remains_exact() {
    use cargo_gpui_visual::environment;
    for field in [
        "os",
        "fonts",
        "metal",
        "pixels",
        "points",
        "sdk",
        "gpui",
        "capture-code",
    ] {
        let (mut host, mut payload, mut receipt) = fixture();
        let original =
            vm_environment::bind(Suite::Gallery, &host, &payload, &"d".repeat(64), &receipt)
                .unwrap();
        match field {
            "os" => receipt.environment_before.os_build = "new".into(),
            "fonts" => {
                *receipt
                    .environment_before
                    .fonts
                    .values_mut()
                    .next()
                    .unwrap() = "e".repeat(64)
            }
            "metal" => {
                receipt.environment_before.displays[0]["metal"]["name"] = json!("Different Metal")
            }
            "pixels" => {
                receipt.environment_before.displays[0]["screens"][0]["width_px"] = json!(3200)
            }
            "points" => {
                receipt.environment_before.displays[0]["screens"][0]["width_pt"] = json!(1600)
            }
            "sdk" => host.sdk = "new".into(),
            "gpui" => host.gpui_source = "new".into(),
            "capture-code" => {
                payload.sources.insert(
                    "vendor/cargo-gpui-visual/src/capture.rs".into(),
                    "e".repeat(64),
                );
                receipt.sources = payload.sources.clone();
            }
            _ => unreachable!(),
        }
        receipt.environment_after = receipt.environment_before.clone();
        let changed =
            vm_environment::bind(Suite::Gallery, &host, &payload, &"d".repeat(64), &receipt)
                .unwrap();
        assert!(
            !environment::equivalent(&original, &changed).unwrap(),
            "{field}"
        );
    }
}
#[test]
fn unknown_or_malformed_native_display_records_fail_closed() {
    for field in ["unknown", "registry", "geometry", "origin", "missing"] {
        let (host, payload, mut receipt) = fixture();
        let d = &mut receipt.environment_before.displays[0];
        match field {
            "unknown" => d["metal"]["unclassified"] = json!(true),
            "registry" => d["metal"]["registry_id"] = json!(0),
            "geometry" => d["screens"][0]["width_px"] = json!(0),
            "origin" => d["origin"] = json!("host"),
            "missing" => {
                d["metal"].as_object_mut().unwrap().remove("name");
            }
            _ => unreachable!(),
        }
        receipt.environment_after = receipt.environment_before.clone();
        assert!(
            vm_environment::bind(Suite::Gallery, &host, &payload, &"d".repeat(64), &receipt)
                .is_err(),
            "{field}"
        );
    }
}
#[test]
fn rendering_fingerprint_requires_all_authored_inputs_without_hashing_report_only_changes() {
    let (_, mut payload, _) = fixture();
    let inputs = bbr_visual::vm_rendering::tool_inputs(&payload.sources).unwrap();
    payload.sources.insert(
        "vendor/cargo-gpui-visual/assets/review.css".into(),
        "e".repeat(64),
    );
    assert_eq!(
        inputs,
        bbr_visual::vm_rendering::tool_inputs(&payload.sources).unwrap()
    );
    payload
        .sources
        .remove("vendor/cargo-gpui-visual/src/capture.rs");
    assert!(bbr_visual::vm_rendering::tool_inputs(&payload.sources).is_err());
}
#[test]
fn ordinary_binary_changes_do_not_impersonate_environment_drift() {
    let (mut host, mut payload, mut receipt) = fixture();
    let original =
        vm_environment::bind(Suite::Gallery, &host, &payload, &"d".repeat(64), &receipt).unwrap();
    payload.files[1].sha256 = "e".repeat(64);
    *host.executable_hashes.values_mut().next().unwrap() = "e".repeat(64);
    *receipt
        .installed
        .get_mut(payload.files[1].guest.to_str().unwrap())
        .unwrap() = "e".repeat(64);
    let changed =
        vm_environment::bind(Suite::Gallery, &host, &payload, &"d".repeat(64), &receipt).unwrap();
    assert_eq!(original, changed);
}
