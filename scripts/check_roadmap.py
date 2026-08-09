#!/usr/bin/env python3
"""Validate BBR's Git-durable documentation control graph."""
from __future__ import annotations
import argparse
import hashlib
from pathlib import Path, PurePosixPath
import re
import sys
from typing import Iterable
from urllib.parse import unquote
from roadmap_validation_support import (
    GitTree,
    markdown_heading_anchors,
    markdown_section_lines,
    markdown_table_records,
    resolve_local_target,
    structural_markdown_lines,
    unquote_code,
    validate_git_reference,
)
CURRENT_DELIVERY_MARKER = "<!-- bbr:current-delivery -->"
CURRENT_DELIVERY_LINK_LABEL = "Current delivery"
ROADMAP_PATH = PurePosixPath("ROADMAP.md")
TASKS_PREFIX = PurePosixPath("docs/roadmap/tasks")
REQUIRED_TASK_SECTIONS = (
    "## Outcome",
    "## Scope",
    "## Non-goals",
    "## Constraints",
    "## Evidence",
    "## Completion gates",
    "## Iteration policy",
    "## Blocked stop condition",
)
REQUIRED_CURRENT_SECTIONS = (
    "## Control metadata",
    "## Status vocabulary",
    "## Active product wave",
    "## Post-F2a parallel wave",
    "## Serial integration order",
    "## Downstream dependency queue",
    "## Open product and evidence work",
    "## Standing decisions and constraints",
    "## Cutover and rollback protocol",
)
REQUIRED_CONTROL_METADATA_FIELDS = (
    "Cutover commit",
    "Verified product base",
    "Previous canonical pointer",
    "Rollback pointer",
    "Target Orchestrator session",
)
REQUIRED_ACTIVE_WAVE_FIELDS = (
    "ID",
    "Slice",
    "Status",
    "Worktree",
    "Owner",
    "Contract",
    "Next gate",
)
ALLOWED_DELIVERY_STATUSES = frozenset(
    {
        "planned",
        "ready",
        "running",
        "review",
        "remediation",
        "frozen",
        "blocked",
        "integrating",
        "integrated",
        "closed",
    }
)
STATUS_TABLE_SECTIONS = (
    ("## Active product wave", "Status"),
    ("## Post-F2a parallel wave", "Status before F2a close"),
    ("## Downstream dependency queue", "Status"),
    ("## Open product and evidence work", "Status"),
)
VERIFIED_BASE_PATTERN = re.compile(r"main@[0-9a-f]{40}")
ROLLBACK_POINTER_PATTERN = re.compile(
    r"docs/delivery/2026-08-08-post-mvp-delivery\.md@[0-9a-f]{40}"
)
# These exact historical packets predate the current packet template. Any edit
# changes the digest and requires the packet to adopt every required section.
LEGACY_TASK_SHA256 = {
    "docs/roadmap/tasks/browser-controller.md": "d38fcce7fd76f058b4300139f4c919aacd5e419904e77e2aac528b58cfd4fbd2",
    "docs/roadmap/tasks/dense-list-slice.md": "488551394195564f60f6949b00da527892eaae76decc77d8b48002cd7d6af1f3",
    "docs/roadmap/tasks/fake-web-engine.md": "546f1de8dbb75222eaf52cc79b171bfd69deea19233f0eeff2d3eb8c063fd5fc",
    "docs/roadmap/tasks/link-preview-peek.md": "6f112057a012267bf69d500d354e9bfa29dafd39e44ea0e9df4cd4c72b2ef938",
    "docs/roadmap/tasks/performance-e2e-foundation.md": "55bf7e7b8d73aab0903fb5708fadd83e576637b775070f1881faecd395eefb24",
    "docs/roadmap/tasks/schema-v2-migration-decision.md": "055e8c35eaf3c969ba1fe5a3d7817c50302c0c499762f9a23d2e57ba2a02ee76",
    "docs/roadmap/tasks/split-app-composition.md": "5662261b4fcdd44873e5c94cd5cada8a4c803c569fbc333e0c1a028717f896d5",
    "docs/roadmap/tasks/url-address-control-palette.md": "be2fc40f25ce27c73a7944c5d8cdc27a9527a4eeacc9f948060f3c2a8f1e68ad",
}

LINK_PATTERN = re.compile(r"!?\[([^\]]*)\]\(([^)]+)\)")
EXCLUDED_PREFIXES = ("vendor/", "target/", ".git/", ".direnv/")
class DocumentationValidator:
    def __init__(self, tree: GitTree, *, allow_pending_current: bool = False) -> None:
        self.tree = tree
        self.allow_pending_current = allow_pending_current
        self.failures: list[str] = []
        self.warnings: list[str] = []
        self.markdown_paths = tuple(
            sorted(
                path
                for path in tree.paths
                if path.suffix.lower() == ".md" and not self._excluded(path)
            )
        )
        self.text = {path: tree.read_text(path) for path in self.markdown_paths}
        self.links = {
            path: tuple(
                LINK_PATTERN.findall("\n".join(structural_markdown_lines(text)))
            )
            for path, text in self.text.items()
        }
        self.anchors = {
            path: markdown_heading_anchors(text)
            for path, text in self.text.items()
        }

    @staticmethod
    def _excluded(path: PurePosixPath) -> bool:
        value = path.as_posix()
        return any(value.startswith(prefix) for prefix in EXCLUDED_PREFIXES)

    def validate_links(self) -> None:
        for source, links in self.links.items():
            for _, raw_target in links:
                try:
                    target = resolve_local_target(source, raw_target)
                except ValueError as error:
                    self.failures.append(f"{source}: {raw_target!r}: {error}")
                    continue
                if target is not None and target not in self.tree.paths:
                    self.failures.append(
                        f"{source}: link target is not tracked in {self.tree.source}: "
                        f"{raw_target!r} -> {target}"
                    )
                    continue
                if target is None or "#" not in raw_target:
                    continue
                fragment = unquote(raw_target.split("#", 1)[1]).strip()
                if not fragment:
                    continue
                if target.suffix.lower() != ".md":
                    self.failures.append(
                        f"{source}: local fragment target is not Markdown: "
                        f"{raw_target!r} -> {target}"
                    )
                    continue
                if fragment not in self.anchors.get(target, frozenset()):
                    self.failures.append(
                        f"{source}: Markdown fragment does not exist in "
                        f"{self.tree.source}: {raw_target!r} -> {target}#{fragment}"
                    )

    def validate_current_delivery_pointer(self) -> None:
        roadmap = self.text.get(ROADMAP_PATH)
        if roadmap is None:
            self.failures.append("ROADMAP.md is not tracked")
            return
        candidates = []
        all_targets = []
        for label, raw_target in self.links.get(ROADMAP_PATH, ()):
            try:
                target = resolve_local_target(ROADMAP_PATH, raw_target)
            except ValueError as error:
                if label.strip() == CURRENT_DELIVERY_LINK_LABEL:
                    self.failures.append(
                        f"ROADMAP.md current delivery pointer {raw_target!r}: {error}"
                    )
                continue
            if target is not None:
                all_targets.append(target)
                if label.strip() == CURRENT_DELIVERY_LINK_LABEL:
                    candidates.append(target)
        if len(candidates) != 1:
            self.failures.append(
                "ROADMAP.md must contain exactly one "
                f"[{CURRENT_DELIVERY_LINK_LABEL}](...) pointer; found {len(candidates)}"
            )
            return
        target = candidates[0]
        if all_targets.count(target) != 1:
            self.failures.append(
                "ROADMAP.md must link to the current-delivery target exactly "
                f"once across all labels; found {all_targets.count(target)}"
            )
        if target not in self.tree.paths:
            return  # The link audit reports the durable-target failure.
        target_text = self.text.get(target)
        if target_text is None:
            self.failures.append(f"current delivery pointer is not Markdown: {target}")
        elif CURRENT_DELIVERY_MARKER not in target_text:
            self.failures.append(
                f"current delivery document {target} lacks "
                f"{CURRENT_DELIVERY_MARKER!r}"
            )
        else:
            structural_lines = structural_markdown_lines(target_text)
            section_counts = {s: structural_lines.count(s) for s in REQUIRED_CURRENT_SECTIONS}
            invalid_sections = [
                section for section, count in section_counts.items() if count != 1
            ]
            if invalid_sections:
                details = ", ".join(
                    f"{section} (found {section_counts[section]})"
                    for section in invalid_sections
                )
                self.failures.append(
                    f"current delivery document {target} must contain each "
                    f"operational section exactly once: {details}"
                )
            control_records = markdown_table_records(
                markdown_section_lines(target_text, "## Control metadata"))
            control_fields = [r.get("Field", "") for r in control_records]
            duplicate_control_fields = sorted(
                field for field in set(control_fields) if control_fields.count(field) > 1
            )
            if duplicate_control_fields:
                self.failures.append(
                    f"current delivery document {target} duplicates Control "
                    f"metadata fields: {', '.join(duplicate_control_fields)}"
                )
            control_values = {
                record.get("Field", ""): record.get("Value", "")
                for record in control_records
            }
            missing_control_fields = [
                field for field in REQUIRED_CONTROL_METADATA_FIELDS if not control_values.get(field)
            ]
            if missing_control_fields:
                self.failures.append(
                    f"current delivery document {target} lacks exact Control "
                    f"metadata table fields: {', '.join(missing_control_fields)}"
                )
            active_records = markdown_table_records(
                markdown_section_lines(target_text, "## Active product wave")
            )
            active_headers = set(active_records[0]) if active_records else set()
            missing_active_fields = [
                field
                for field in REQUIRED_ACTIVE_WAVE_FIELDS
                if field not in active_headers
            ]
            if missing_active_fields:
                self.failures.append(
                    f"current delivery document {target} lacks exact Active "
                    f"product wave table fields: {', '.join(missing_active_fields)}"
                )
            elif not active_records:
                self.failures.append(
                    f"current delivery document {target} Active product wave "
                    "must contain at least one operator row"
                )
            else:
                for row_number, record in enumerate(active_records, start=1):
                    empty = [
                        field
                        for field in REQUIRED_ACTIVE_WAVE_FIELDS
                        if not record.get(field, "").strip()
                    ]
                    if empty:
                        self.failures.append(
                            f"current delivery document {target} Active product "
                            f"wave row {row_number} has empty fields: {', '.join(empty)}"
                        )
            previous_pointer = unquote_code(
                control_values.get("Previous canonical pointer", "")
            )
            if previous_pointer and previous_pointer != (
                "docs/delivery/2026-08-08-post-mvp-delivery.md"
            ):
                self.failures.append(
                    f"current delivery document {target} Previous canonical "
                    "pointer must be the preserved old ledger path"
                )
            vocabulary_records = markdown_table_records(
                markdown_section_lines(target_text, "## Status vocabulary")
            )
            vocabulary_values = [
                unquote_code(record.get("Status", ""))
                for record in vocabulary_records
            ]
            vocabulary_statuses = set(vocabulary_values)
            duplicate_vocabulary = sorted(
                status for status in vocabulary_statuses if vocabulary_values.count(status) > 1
            )
            if duplicate_vocabulary:
                self.failures.append(
                    f"current delivery document {target} duplicates status "
                    f"vocabulary rows: {', '.join(duplicate_vocabulary)}"
                )
            if vocabulary_statuses != ALLOWED_DELIVERY_STATUSES:
                missing = sorted(ALLOWED_DELIVERY_STATUSES - vocabulary_statuses)
                extra = sorted(vocabulary_statuses - ALLOWED_DELIVERY_STATUSES)
                self.failures.append(
                    f"current delivery document {target} status vocabulary "
                    f"must equal the fixed set; missing={missing}, extra={extra}"
                )
            status_ids: dict[str, str] = {}
            for section, column in STATUS_TABLE_SECTIONS:
                records = markdown_table_records(markdown_section_lines(target_text, section))
                if not records or column not in records[0] or "ID" not in records[0]:
                    self.failures.append(
                        f"current delivery document {target} lacks a structured "
                        f"{section} table with 'ID' and {column!r}"
                    )
                    continue
                for record in records:
                    item = unquote_code(record.get("ID", ""))
                    status = unquote_code(record[column])
                    if not item:
                        self.failures.append(
                            f"current delivery document {target} has an empty ID "
                            f"in {section}"
                        )
                    elif item in status_ids:
                        self.failures.append(
                            f"current delivery document {target} duplicates ID "
                            f"{item!r} in {status_ids[item]} and {section}"
                        )
                    else:
                        status_ids[item] = section
                    if status not in ALLOWED_DELIVERY_STATUSES:
                        self.failures.append(
                            f"current delivery document {target} uses unknown "
                            f"status {status!r} for {item or '<unknown>'!r} in {section}"
                        )
            pending_metadata = [
                field for field, value in control_values.items() if "PENDING_" in value
            ]
            if pending_metadata:
                message = (
                    f"current delivery document {target} contains unresolved "
                    "PENDING_* cutover metadata in: " + ", ".join(pending_metadata)
                )
                if self.allow_pending_current:
                    self.warnings.append(message)
                else:
                    self.failures.append(message)
            else:
                verified_base = unquote_code(control_values.get("Verified product base", ""))
                rollback_pointer = unquote_code(control_values.get("Rollback pointer", ""))
                self.failures.extend(validate_git_reference(
                    self.tree, document=target, field="Verified product base",
                    value=verified_base, pattern=VERIFIED_BASE_PATTERN,
                    required_shape="must be main@<40-hex-SHA>",
                ))
                self.failures.extend(validate_git_reference(
                    self.tree, document=target, field="Rollback pointer",
                    value=rollback_pointer, pattern=ROLLBACK_POINTER_PATTERN,
                    required_shape="must be the old ledger path@<40-hex-SHA>",
                    required_path=PurePosixPath(
                        "docs/delivery/2026-08-08-post-mvp-delivery.md"
                    ),
                ))
        marker_occurrences = [
            path
            for path, text in self.text.items()
            for _ in range(text.count(CURRENT_DELIVERY_MARKER))
        ]
        if marker_occurrences != [target]:
            owners = ", ".join(path.as_posix() for path in marker_occurrences) or "none"
            self.failures.append(
                "exactly one current marker occurrence must exist and belong to "
                f"the ROADMAP target; occurrences: {owners}"
            )

    def validate_task_reachability(self) -> None:
        roadmap_docs = {
            path
            for path in self.markdown_paths
            if path == ROADMAP_PATH or path.as_posix().startswith("docs/roadmap/")
        }
        if ROADMAP_PATH not in roadmap_docs:
            return
        seen = {ROADMAP_PATH}
        pending = [ROADMAP_PATH]
        while pending:
            source = pending.pop()
            for _, raw_target in self.links.get(source, ()):
                try:
                    target = resolve_local_target(source, raw_target)
                except ValueError:
                    continue
                if target in roadmap_docs and target not in seen:
                    seen.add(target)
                    pending.append(target)
        tasks = {
            path
            for path in roadmap_docs
            if path.parent == TASKS_PREFIX and path.suffix == ".md"
        }
        for task in sorted(tasks - seen):
            self.failures.append(f"task packet is not reachable from ROADMAP.md: {task}")

    def validate_task_sections(self) -> None:
        tasks = sorted(
            path
            for path in self.markdown_paths
            if path.parent == TASKS_PREFIX and path.suffix == ".md"
        )
        for task in tasks:
            text = self.text[task]
            structural_lines = structural_markdown_lines(text)
            section_counts = {
                section: structural_lines.count(section)
                for section in REQUIRED_TASK_SECTIONS
            }
            invalid = [
                section for section, count in section_counts.items() if count != 1
            ]
            if not invalid:
                continue
            digest = hashlib.sha256(text.encode("utf-8")).hexdigest()
            expected = LEGACY_TASK_SHA256.get(task.as_posix())
            if expected == digest:
                self.warnings.append(
                    f"grandfathered legacy task packet {task}; any edit requires "
                    "the current template"
                )
                continue
            details = ", ".join(
                f"{section} (found {section_counts[section]})" for section in invalid
            )
            self.failures.append(
                f"task packet {task} must contain each required section exactly "
                f"once: {details}"
            )

    def run(self) -> tuple[list[str], list[str]]:
        self.validate_links()
        self.validate_current_delivery_pointer()
        self.validate_task_reachability()
        self.validate_task_sections()
        return self.failures, self.warnings

def parse_args(argv: Iterable[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parent.parent,
        help="Git repository root (default: inferred from this script)",
    )
    parser.add_argument(
        "--source",
        choices=("worktree", "index", "head"),
        default="worktree",
        help=(
            "Documentation contents to validate. Worktree is the development "
            "default but accepts only tracked paths; index and head provide "
            "staged/committed durability audits."
        ),
    )
    parser.add_argument(
        "--allow-pending-current",
        action="store_true",
        help=(
            "Preparation-only: downgrade unresolved PENDING_* cutover metadata "
            "to a warning. Normal roadmap/verify gates must never use this."
        ),
    )
    return parser.parse_args(argv)

def main(argv: Iterable[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        tree = GitTree(args.root, args.source)
        validator = DocumentationValidator(
            tree, allow_pending_current=args.allow_pending_current
        )
        failures, warnings = validator.run()
    except (OSError, RuntimeError, UnicodeDecodeError) as error:
        print(f"Roadmap validation could not run: {error}", file=sys.stderr)
        return 2

    for warning in warnings:
        print(f"Roadmap warning: {warning}", file=sys.stderr)
    if failures:
        print("Roadmap validation failed:", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print(
        f"Roadmap control graph valid ({len(validator.markdown_paths)} tracked "
        f"first-party Markdown files checked from {args.source})."
    )
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
