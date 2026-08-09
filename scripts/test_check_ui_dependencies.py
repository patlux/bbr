#!/usr/bin/env python3
"""Unit tests for scripts/check_ui_dependencies.py (the bbr-ui dependency gate).

These test the pure analysis functions against synthetic `cargo metadata`
documents and synthetic source trees: the exact-direct-set check, the
first-party closure check, the every-path-through-bbr-domain rule (a direct
`bbr-ui -> bbr-web` edge must be rejected), and the forbidden-source-import
scanner. Run directly with `python3 -m unittest scripts/test_check_ui_dependencies.py`.

The proposed Orchestrator patch adds this file to the `just script-tests`
recipe alongside the existing gate-test modules.
"""

import tempfile
import unittest
from pathlib import Path

import scripts.check_ui_dependencies as gate


def metadata_for(edges):
    """Build a synthetic metadata document from `edges`: list of
    (source, target) package-name pairs (normal edges). The packages' own
    `dependencies` arrays mirror the edges so the direct-set checks work."""
    ids = {}
    packages = []

    def ensure(name):
        if name not in ids:
            ids[name] = f"pkg-{name}"
            packages.append(
                {
                    "id": f"pkg-{name}",
                    "name": name,
                    "version": "0.1.0",
                    "dependencies": [],
                }
            )
        return ids[name]

    for source, target in edges:
        ensure(source)
        ensure(target)
    adjacency = {name: [] for name in ids}
    for source, target in edges:
        adjacency[source].append(target)
    by_name = {p["name"]: p for p in packages}
    for source, targets in adjacency.items():
        by_name[source]["dependencies"] = [
            {
                "name": target,
                "source": None,
                "req": "*",
                "kind": "normal",
                "dep_kinds": [{"kind": None, "target": None}],
            }
            for target in targets
        ]
    nodes = []
    for name, pkg_id in ids.items():
        deps = []
        for target in adjacency[name]:
            deps.append(
                {
                    "name": target,
                    "pkg": ids[target],
                    "dep_kinds": [{"kind": None, "target": None}],
                }
            )
        nodes.append({"id": pkg_id, "deps": deps})
    return {
        "packages": packages,
        "resolve": {"nodes": nodes},
    }


class DirectSetTests(unittest.TestCase):
    def test_exact_direct_set_accepts_gpui_plus_domain(self):
        metadata = metadata_for([("bbr-ui", "bbr-domain"), ("bbr-ui", "gpui")])
        direct = gate.direct_dependency_names(metadata, "bbr-ui")
        self.assertEqual(direct, {"gpui", "bbr-domain"})

    def test_extra_direct_dependency_is_rejected(self):
        metadata = metadata_for(
            [("bbr-ui", "bbr-domain"), ("bbr-ui", "gpui"), ("bbr-ui", "serde")]
        )
        direct = gate.direct_dependency_names(metadata, "bbr-ui")
        self.assertNotEqual(direct, gate.ALLOWED_DIRECT)
        self.assertIn("serde", direct)

    def test_forbidden_first_party_direct_edge_is_rejected(self):
        # A direct bbr-ui -> bbr-web edge must fail the direct-set check.
        metadata = metadata_for([("bbr-ui", "bbr-web"), ("bbr-ui", "gpui")])
        direct = gate.direct_dependency_names(metadata, "bbr-ui")
        self.assertNotEqual(direct, gate.ALLOWED_DIRECT)


class ClosureTests(unittest.TestCase):
    def setUp(self):
        self.edges = [
            ("bbr-ui", "bbr-domain"),
            ("bbr-ui", "gpui"),
            ("bbr-domain", "bbr-web"),
        ]
        self.metadata = metadata_for(self.edges)
        self.packages = gate.package_by_id(self.metadata)
        self.adjacency = gate.normal_edges(self.metadata)
        self.root = next(p for p in self.metadata["packages"] if p["name"] == "bbr-ui")

    def test_approved_closure_is_domain_plus_web(self):
        closure = gate.first_party_closure(
            self.adjacency, self.packages, self.root["id"]
        )
        self.assertEqual(closure, {"bbr-domain", "bbr-web"})

    def test_a_foreign_first_party_in_the_closure_is_rejected(self):
        metadata = metadata_for(
            self.edges + [("bbr-domain", "bbr-browser")]
        )
        packages = gate.package_by_id(metadata)
        adjacency = gate.normal_edges(metadata)
        root = next(p for p in metadata["packages"] if p["name"] == "bbr-ui")
        closure = gate.first_party_closure(adjacency, packages, root["id"])
        self.assertNotEqual(closure, gate.ALLOWED_FIRST_PARTY)
        self.assertIn("bbr-browser", closure)

    def test_every_path_to_bbr_web_is_through_bbr_domain(self):
        paths = gate.paths_to(self.adjacency, self.packages, self.root["id"], "bbr-web")
        self.assertEqual(paths, [["bbr-ui", "bbr-domain", "bbr-web"]])

    def test_a_direct_bbr_ui_to_bbr_web_edge_is_rejected(self):
        metadata = metadata_for(
            [("bbr-ui", "bbr-domain"), ("bbr-ui", "bbr-web"), ("bbr-ui", "gpui")]
        )
        packages = gate.package_by_id(metadata)
        adjacency = gate.normal_edges(metadata)
        root = next(p for p in metadata["packages"] if p["name"] == "bbr-ui")
        paths = gate.paths_to(adjacency, packages, root["id"], "bbr-web")
        for path in paths:
            self.assertNotEqual(
                path,
                ["bbr-ui", "bbr-domain", "bbr-web"],
                "a direct bbr-ui -> bbr-web edge must never masquerade as approved",
            )
        self.assertTrue(any(len(p) == 2 for p in paths))


class SourceScanTests(unittest.TestCase):
    def test_clean_source_scans_clean(self):
        with tempfile.TemporaryDirectory() as tmp:
            src = Path(tmp)
            (src / "lib.rs").write_text(
                "use bbr_domain::ProfileHue;\nuse gpui::div;\n"
            )
            self.assertEqual(gate.forbidden_source_imports(src), [])

    def test_forbidden_import_is_detected(self):
        with tempfile.TemporaryDirectory() as tmp:
            src = Path(tmp)
            (src / "bad.rs").write_text("use bbr_web::ProfileId;\n")
            offending = gate.forbidden_source_imports(src)
            self.assertEqual(len(offending), 1)
            self.assertIn("bbr_web", offending[0])

    def test_forbidden_path_token_is_detected(self):
        with tempfile.TemporaryDirectory() as tmp:
            src = Path(tmp)
            (src / "bad.rs").write_text("let x = bbr_browser::ControllerSnapshot;\n")
            offending = gate.forbidden_source_imports(src)
            self.assertTrue(any("bbr_browser" in line for line in offending))

    def test_element_ids_do_not_false_positive(self):
        # Product ids such as `bbr-command-bar` are hyphenated and must not
        # trip the underscore crate-token scanner.
        with tempfile.TemporaryDirectory() as tmp:
            src = Path(tmp)
            (src / "ok.rs").write_text('.id("bbr-command-bar")\n')
            self.assertEqual(gate.forbidden_source_imports(src), [])


class ManifestKeyTests(unittest.TestCase):
    def test_manifest_keys_are_exact(self):
        with tempfile.TemporaryDirectory() as tmp:
            manifest = Path(tmp) / "Cargo.toml"
            manifest.write_text(
                "[package]\nname = \"bbr-ui\"\n\n"
                "[dependencies]\ngpui = { git = \"...\" }\nbbr-domain = { path = \"../bbr-domain\" }\n"
            )
            self.assertEqual(gate.direct_manifest_keys(manifest), {"gpui", "bbr-domain"})

    def test_extra_manifest_key_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            manifest = Path(tmp) / "Cargo.toml"
            manifest.write_text(
                "[dependencies]\ngpui = \"0.2\"\nbbr-domain = { path = \"..\" }\nserde = \"1\"\n"
            )
            keys = gate.direct_manifest_keys(manifest)
            self.assertNotEqual(keys, gate.ALLOWED_DIRECT)
            self.assertIn("serde", keys)


if __name__ == "__main__":
    unittest.main()
