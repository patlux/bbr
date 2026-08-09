#!/usr/bin/env python3
"""Focused contract tests for scripts/check_roadmap.py."""

from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_roadmap

CURRENT_BODY = """<!-- bbr:current-delivery -->
# Current delivery

## Control metadata

| Field | Value |
|---|---|
| Cutover commit | abc |
| Verified product base | main@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa |
| Previous canonical pointer | docs/delivery/2026-08-08-post-mvp-delivery.md |
| Rollback pointer | docs/delivery/2026-08-08-post-mvp-delivery.md@bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb |
| Target Orchestrator session | jkl |

## Status vocabulary

| Status | Meaning |
|---|---|
| planned | meaning |
| ready | meaning |
| running | meaning |
| review | meaning |
| remediation | meaning |
| frozen | meaning |
| blocked | meaning |
| integrating | meaning |
| integrated | meaning |
| closed | meaning |

## Active product wave

| ID | Slice | Status | Worktree | Owner | Contract | Next gate |
|---|---|---|---|---|---|---|
| F2a | Slice | running | /tmp/lane | writer | packet | review |

## Post-F2a parallel wave

| ID | Status before F2a close | Start condition |
|---|---|---|
| A | planned | F2a integrated |

## Serial integration order

1. Integrate.

## Downstream dependency queue

| ID | Status | Depends on |
|---|---|---|
| B2 | planned | B1 |

## Open product and evidence work

| ID | Status | Condition |
|---|---|---|
| MANUAL | blocked | user input |

## Standing decisions and constraints

No weakened gates.

## Cutover and rollback protocol

Atomic handoff.
"""

TASK_BODY = """# Task packet — test slice

## Outcome

Outcome.

## Scope

Scope.

## Non-goals

Non-goals.

## Constraints

Constraints.

## Evidence

Evidence.

## Completion gates

```bash
true
```

## Iteration policy

Iterate.

## Blocked stop condition

Stop with evidence.
"""

class RoadmapControlGraphTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        self.git("init", "-q")
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "user.name", "BBR Test")

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def git(self, *args: str) -> str:
        return subprocess.run(
            ["git", "-C", str(self.root), *args],
            check=True,
            capture_output=True,
            text=True,
        ).stdout

    def write(self, relative: str, content: str) -> None:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)

    def seed_valid_graph(self) -> None:
        self.write(
            "ROADMAP.md",
            "# Roadmap\n\n"
            "[Current delivery](docs/delivery/current.md)\n\n"
            "[Active phase](docs/roadmap/phases/current.md)\n",
        )
        self.write(
            "docs/roadmap/phases/current.md",
            "# Current phase\n\n[Task](../tasks/test-slice.md)\n",
        )
        self.write("docs/roadmap/tasks/test-slice.md", TASK_BODY)
        self.write("docs/delivery/2026-08-08-post-mvp-delivery.md", "# Old ledger\n")
        self.write("docs/delivery/current.md", "# Seed placeholder\n")
        self.git("add", ".")
        self.git("commit", "-qm", "seed durable ledger")
        self.base_sha = self.git("rev-parse", "HEAD").strip()
        self.main_branch = self.git("branch", "--show-current").strip()
        self.valid_current_body = CURRENT_BODY.replace(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", self.base_sha
        ).replace("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", self.base_sha)
        self.write("docs/delivery/current.md", self.valid_current_body)
        self.git("add", "docs/delivery/current.md")
        self.git("commit", "-qm", "add current snapshot")

    def validate(
        self, source: str = "worktree", *, allow_pending_current: bool = False
    ) -> tuple[list[str], list[str]]:
        tree = check_roadmap.GitTree(self.root, source)
        return check_roadmap.DocumentationValidator(
            tree, allow_pending_current=allow_pending_current
        ).run()

    def test_valid_committed_graph_passes_from_all_sources(self) -> None:
        self.seed_valid_graph()
        self.assertEqual(self.validate("worktree")[0], [])
        self.assertEqual(self.validate("index")[0], [])
        self.assertEqual(self.validate("head")[0], [])

    def test_link_to_untracked_file_fails_even_when_file_exists(self) -> None:
        self.seed_valid_graph()
        roadmap = self.root / "ROADMAP.md"
        roadmap.write_text(
            roadmap.read_text()
            + "\n[Untracked packet](docs/roadmap/tasks/untracked.md)\n"
        )
        self.write("docs/roadmap/tasks/untracked.md", TASK_BODY)
        failures, _ = self.validate()
        self.assertTrue(
            any("not tracked" in failure and "untracked.md" in failure for failure in failures),
            failures,
        )

    def test_local_markdown_fragment_must_exist(self) -> None:
        self.seed_valid_graph()
        phase = self.root / "docs/roadmap/phases/current.md"
        phase.write_text(
            phase.read_text() + "\n[Missing anchor](../tasks/test-slice.md#absent)\n"
        )
        failures, _ = self.validate()
        self.assertTrue(any("fragment does not exist" in item for item in failures), failures)

    def test_same_document_fragment_is_validated(self) -> None:
        self.seed_valid_graph()
        phase = self.root / "docs/roadmap/phases/current.md"
        phase.write_text(phase.read_text() + "\n[Missing](#absent)\n")
        failures, _ = self.validate()
        self.assertTrue(any("fragment does not exist" in item for item in failures), failures)

    def test_links_inside_fenced_code_are_not_audited(self) -> None:
        self.seed_valid_graph()
        roadmap = self.root / "ROADMAP.md"
        roadmap.write_text(
            roadmap.read_text()
            + "\n```markdown\n[Example](docs/roadmap/tasks/missing.md)\n```\n"
        )
        failures, _ = self.validate()
        self.assertEqual(failures, [])

    def test_local_image_target_must_be_tracked(self) -> None:
        self.seed_valid_graph()
        roadmap = self.root / "ROADMAP.md"
        roadmap.write_text(roadmap.read_text() + "\n![Diagram](docs/missing.png)\n")
        failures, _ = self.validate()
        self.assertTrue(any("missing.png" in item and "not tracked" in item for item in failures), failures)

    def test_unreachable_tracked_task_fails(self) -> None:
        self.seed_valid_graph()
        self.write("docs/roadmap/tasks/orphan.md", TASK_BODY)
        self.git("add", "docs/roadmap/tasks/orphan.md")
        failures, _ = self.validate()
        self.assertIn(
            "task packet is not reachable from ROADMAP.md: "
            "docs/roadmap/tasks/orphan.md",
            failures,
        )

    def test_new_task_without_required_sections_fails(self) -> None:
        self.seed_valid_graph()
        self.write("docs/roadmap/tasks/incomplete.md", "# Incomplete\n")
        phase = self.root / "docs/roadmap/phases/current.md"
        phase.write_text(phase.read_text() + "\n[Incomplete](../tasks/incomplete.md)\n")
        self.git("add", "docs/roadmap/tasks/incomplete.md")
        failures, _ = self.validate()
        self.assertTrue(
            any(
                "incomplete.md must contain each required section exactly once"
                in failure
                for failure in failures
            ),
            failures,
        )

    def test_task_sections_in_fenced_code_do_not_satisfy_template(self) -> None:
        self.seed_valid_graph()
        fake = "# Incomplete\n\n```markdown\n" + "\n".join(
            check_roadmap.REQUIRED_TASK_SECTIONS
        ) + "\n```\n"
        self.write("docs/roadmap/tasks/incomplete.md", fake)
        phase = self.root / "docs/roadmap/phases/current.md"
        phase.write_text(phase.read_text() + "\n[Incomplete](../tasks/incomplete.md)\n")
        self.git("add", "docs/roadmap/tasks/incomplete.md")
        failures, _ = self.validate()
        self.assertTrue(
            any("incomplete.md must contain each required section" in item for item in failures),
            failures,
        )

    def test_exactly_one_current_pointer_and_marker_are_required(self) -> None:
        self.seed_valid_graph()
        roadmap = self.root / "ROADMAP.md"
        roadmap.write_text(
            roadmap.read_text()
            + "\n[Current delivery](docs/delivery/other.md)\n"
        )
        self.write("docs/delivery/other.md", self.valid_current_body.replace("Current", "Other", 1))
        self.git("add", "docs/delivery/other.md")
        failures, _ = self.validate()
        self.assertTrue(any("exactly one [Current delivery]" in item for item in failures))

    def test_current_target_must_not_have_a_second_alias_link(self) -> None:
        self.seed_valid_graph()
        roadmap = self.root / "ROADMAP.md"
        roadmap.write_text(
            roadmap.read_text()
            + "\n[Runtime snapshot](docs/delivery/current.md)\n"
        )
        failures, _ = self.validate()
        self.assertTrue(any("target exactly once" in item for item in failures), failures)

    def test_duplicate_marker_in_current_document_fails(self) -> None:
        self.seed_valid_graph()
        current = self.root / "docs/delivery/current.md"
        current.write_text(self.valid_current_body + "\n<!-- bbr:current-delivery -->\n")
        failures, _ = self.validate()
        self.assertTrue(any("exactly one current marker occurrence" in item for item in failures))

    def test_index_and_head_ignore_unstaged_pointer_breakage(self) -> None:
        self.seed_valid_graph()
        self.write("docs/delivery/current.md", "# Marker removed in worktree\n")
        self.assertTrue(any("lacks" in item for item in self.validate("worktree")[0]))
        self.assertEqual(self.validate("index")[0], [])
        self.assertEqual(self.validate("head")[0], [])

    def test_pending_cutover_metadata_fails_normal_gate(self) -> None:
        self.seed_valid_graph()
        self.write(
            "docs/delivery/current.md",
            self.valid_current_body.replace(
                f"| Verified product base | main@{self.base_sha} |",
                "| Verified product base | PENDING_CUTOVER |",
            ),
        )
        failures, _ = self.validate("worktree")
        self.assertTrue(any("PENDING_*" in item for item in failures), failures)
        allowed_failures, warnings = self.validate(
            "worktree", allow_pending_current=True
        )
        self.assertEqual(allowed_failures, [])
        self.assertTrue(any("PENDING_*" in item for item in warnings), warnings)

    def test_current_delivery_requires_operator_readback_fields(self) -> None:
        self.seed_valid_graph()
        self.write(
            "docs/delivery/current.md",
            "<!-- bbr:current-delivery -->\n# Current delivery\n",
        )
        failures, _ = self.validate("worktree")
        self.assertTrue(any("operational section exactly once" in item for item in failures))
        self.assertTrue(any("Control metadata table fields" in item for item in failures))
        self.assertTrue(any("Active product wave table fields" in item for item in failures))

    def test_current_fields_in_prose_or_code_do_not_satisfy_contract(self) -> None:
        self.seed_valid_graph()
        body = "<!-- bbr:current-delivery -->\n# Current delivery\n\n"
        body += "\n\n".join(check_roadmap.REQUIRED_CURRENT_SECTIONS)
        all_fields = (
            check_roadmap.REQUIRED_CONTROL_METADATA_FIELDS
            + check_roadmap.REQUIRED_ACTIVE_WAVE_FIELDS
        )
        body += "\n\n```text\n" + "; ".join(all_fields) + "\n```\n"
        self.write("docs/delivery/current.md", body)
        failures, _ = self.validate("worktree")
        self.assertFalse(any("operational section" in item for item in failures), failures)
        self.assertTrue(any("Control metadata table fields" in item for item in failures), failures)
        self.assertTrue(any("Active product wave table fields" in item for item in failures), failures)

    def test_pending_prose_does_not_block_resolved_metadata(self) -> None:
        self.seed_valid_graph()
        current = self.root / "docs/delivery/current.md"
        current.write_text(self.valid_current_body + "\nPENDING_* is preparation terminology.\n")
        failures, _ = self.validate("worktree")
        self.assertEqual(failures, [])

    def test_unknown_current_status_fails(self) -> None:
        self.seed_valid_graph()
        current = self.root / "docs/delivery/current.md"
        current.write_text(self.valid_current_body.replace("| F2a | Slice | running |", "| F2a | Slice | active |"))
        failures, _ = self.validate("worktree")
        self.assertTrue(any("unknown status 'active'" in item for item in failures), failures)

    def test_duplicate_current_ids_fail_across_status_tables(self) -> None:
        self.seed_valid_graph()
        current = self.root / "docs/delivery/current.md"
        current.write_text(self.valid_current_body.replace("| B2 | planned | B1 |", "| F2a | planned | B1 |"))
        failures, _ = self.validate("worktree")
        self.assertTrue(any("duplicates ID 'F2a'" in item for item in failures), failures)

    def test_active_wave_rows_require_all_operator_fields(self) -> None:
        self.seed_valid_graph()
        current = self.root / "docs/delivery/current.md"
        current.write_text(self.valid_current_body.replace("| F2a | Slice | running | /tmp/lane | writer | packet | review |", "| F2a | Slice | running |  | writer | packet | review |"))
        failures, _ = self.validate("worktree")
        self.assertTrue(any("has empty fields: Worktree" in item for item in failures), failures)

    def test_previous_pointer_must_name_preserved_ledger(self) -> None:
        self.seed_valid_graph()
        current = self.root / "docs/delivery/current.md"
        current.write_text(
            self.valid_current_body.replace(
                "| Previous canonical pointer | docs/delivery/2026-08-08-post-mvp-delivery.md |",
                "| Previous canonical pointer | docs/delivery/other.md |",
            )
        )
        failures, _ = self.validate("worktree")
        self.assertTrue(any("Previous canonical pointer must be" in item for item in failures), failures)

    def test_status_vocabulary_must_equal_fixed_set(self) -> None:
        self.seed_valid_graph()
        current = self.root / "docs/delivery/current.md"
        current.write_text(self.valid_current_body.replace("| closed | meaning |\n", ""))
        failures, _ = self.validate("worktree")
        self.assertTrue(any("status vocabulary must equal" in item for item in failures), failures)

    def test_duplicate_status_vocabulary_row_fails(self) -> None:
        self.seed_valid_graph()
        current = self.root / "docs/delivery/current.md"
        current.write_text(
            self.valid_current_body.replace(
                "| closed | meaning |\n",
                "| closed | meaning |\n| closed | duplicate |\n",
            )
        )
        failures, _ = self.validate("worktree")
        self.assertTrue(any("duplicates status vocabulary" in item for item in failures), failures)

    def test_duplicate_control_metadata_field_fails(self) -> None:
        self.seed_valid_graph()
        current = self.root / "docs/delivery/current.md"
        current.write_text(
            self.valid_current_body.replace(
                "| Cutover commit | abc |\n",
                "| Cutover commit | abc |\n| Cutover commit | duplicate |\n",
            )
        )
        failures, _ = self.validate("worktree")
        self.assertTrue(any("duplicates Control metadata" in item for item in failures), failures)

    def test_resolved_cutover_metadata_requires_exact_sha_shapes(self) -> None:
        self.seed_valid_graph()
        current = self.root / "docs/delivery/current.md"
        current.write_text(
            self.valid_current_body.replace(
                f"main@{self.base_sha}",
                "main@abc",
            ).replace(
                f"docs/delivery/2026-08-08-post-mvp-delivery.md@{self.base_sha}",
                "previous main",
            )
        )
        failures, _ = self.validate("worktree")
        self.assertTrue(any("Verified product base must be" in item for item in failures), failures)
        self.assertTrue(any("Rollback pointer must be" in item for item in failures), failures)

    def test_resolved_cutover_metadata_rejects_unknown_commits(self) -> None:
        self.seed_valid_graph()
        unknown = "f" * 40
        current = self.root / "docs/delivery/current.md"
        current.write_text(
            self.valid_current_body.replace(f"main@{self.base_sha}", f"main@{unknown}")
        )
        failures, _ = self.validate("worktree")
        self.assertTrue(any("references no commit object" in item for item in failures), failures)

    def test_rollback_commit_must_contain_preserved_ledger(self) -> None:
        self.seed_valid_graph()
        head = self.git("rev-parse", "HEAD").strip()
        self.git("rm", "docs/delivery/2026-08-08-post-mvp-delivery.md")
        self.git("commit", "-qm", "remove old ledger")
        without_ledger = self.git("rev-parse", "HEAD").strip()
        current = self.root / "docs/delivery/current.md"
        current.write_text(
            self.valid_current_body.replace(
                f"docs/delivery/2026-08-08-post-mvp-delivery.md@{self.base_sha}",
                f"docs/delivery/2026-08-08-post-mvp-delivery.md@{without_ledger}",
            )
        )
        failures, _ = self.validate("worktree")
        self.assertTrue(any("does not contain required path" in item for item in failures), failures)
        self.git("reset", "--hard", head)

    def test_deleted_tracked_markdown_fails_worktree_audit(self) -> None:
        self.seed_valid_graph()
        (self.root / "docs/delivery/current.md").unlink()
        with self.assertRaisesRegex(RuntimeError, "missing from worktree"):
            check_roadmap.GitTree(self.root, "worktree").read_text(
                check_roadmap.PurePosixPath("docs/delivery/current.md")
            )

    def test_tracked_markdown_symlink_is_rejected_from_all_sources(self) -> None:
        self.seed_valid_graph()
        current = self.root / "docs/delivery/current.md"
        current.unlink()
        current.symlink_to("../../external-current.md")
        self.write("external-current.md", self.valid_current_body)
        self.git("add", "docs/delivery/current.md")
        self.git("commit", "-qm", "track symlink")
        for source in ("worktree", "index", "head"):
            with self.subTest(source=source):
                with self.assertRaisesRegex(RuntimeError, "regular tracked file"):
                    check_roadmap.GitTree(self.root, source).read_text(
                        check_roadmap.PurePosixPath("docs/delivery/current.md")
                    )

    def test_unstaged_worktree_symlink_replacement_is_rejected(self) -> None:
        self.seed_valid_graph()
        current = self.root / "docs/delivery/current.md"
        current.unlink()
        current.symlink_to("../../external-current.md")
        self.write("external-current.md", self.valid_current_body)
        with self.assertRaisesRegex(RuntimeError, "worktree symlink"):
            check_roadmap.GitTree(self.root, "worktree").read_text(
                check_roadmap.PurePosixPath("docs/delivery/current.md")
            )
        self.assertEqual(self.validate("index")[0], [])

if __name__ == "__main__":
    unittest.main()
