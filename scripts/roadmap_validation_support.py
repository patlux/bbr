#!/usr/bin/env python3
"""Git-backed Markdown parsing support for the BBR roadmap validator."""

from __future__ import annotations

from pathlib import Path, PurePosixPath
import re
import subprocess
from typing import Iterable


FENCE_PATTERN = re.compile(r"^[ \t]{0,3}(`{3,}|~{3,})")
HEADING_PATTERN = re.compile(r"^#{1,6}[ \t]+(.+?)(?:[ \t]+#+)?$")
HTML_TAG_PATTERN = re.compile(r"<[^>]+>")
MARKDOWN_DECORATION_PATTERN = re.compile(r"[`*_~]")


def structural_markdown_lines(text: str) -> tuple[str, ...]:
    """Return stripped Markdown lines outside fenced code blocks."""

    lines: list[str] = []
    fence: tuple[str, int] | None = None
    for raw_line in text.splitlines():
        match = FENCE_PATTERN.match(raw_line)
        if match:
            marker = match.group(1)
            candidate = (marker[0], len(marker))
            if fence is None:
                fence = candidate
            elif candidate[0] == fence[0] and candidate[1] >= fence[1]:
                fence = None
            continue
        if fence is None:
            lines.append(raw_line.strip())
    return tuple(lines)


def markdown_table_rows(lines: Iterable[str]) -> tuple[tuple[str, ...], ...]:
    """Parse pipe-delimited Markdown table rows without interpreting markup."""

    rows: list[tuple[str, ...]] = []
    for line in lines:
        if not line.startswith("|") or not line.endswith("|"):
            continue
        rows.append(tuple(cell.strip() for cell in line[1:-1].split("|")))
    return tuple(rows)


def markdown_table_records(lines: Iterable[str]) -> tuple[dict[str, str], ...]:
    """Return records from the first structurally valid Markdown table."""

    rows = markdown_table_rows(lines)
    for index in range(len(rows) - 1):
        header = rows[index]
        separator = rows[index + 1]
        if len(header) < 2 or len(separator) != len(header):
            continue
        if not all(re.fullmatch(r":?-{3,}:?", cell) for cell in separator):
            continue
        records: list[dict[str, str]] = []
        for row in rows[index + 2 :]:
            if len(row) != len(header):
                break
            records.append(dict(zip(header, row)))
        return tuple(records)
    return ()


def unquote_code(value: str) -> str:
    """Remove one pair of Markdown inline-code delimiters from a cell."""

    value = value.strip()
    if len(value) >= 2 and value.startswith("`") and value.endswith("`"):
        return value[1:-1].strip()
    return value


def github_heading_slug(heading: str) -> str:
    """Approximate GitHub's Unicode-preserving Markdown heading slug."""

    heading = HTML_TAG_PATTERN.sub("", heading)
    heading = MARKDOWN_DECORATION_PATTERN.sub("", heading).strip().lower()
    heading = re.sub(r"[^\w\- ]", "", heading, flags=re.UNICODE)
    return heading.replace(" ", "-")


def markdown_heading_anchors(text: str) -> frozenset[str]:
    """Return GitHub-style anchors for headings outside fenced code blocks."""

    anchors: set[str] = set()
    counts: dict[str, int] = {}
    for line in structural_markdown_lines(text):
        match = HEADING_PATTERN.match(line)
        if not match:
            continue
        base = github_heading_slug(match.group(1))
        if not base:
            continue
        duplicate_index = counts.get(base, 0)
        counts[base] = duplicate_index + 1
        anchor = base if duplicate_index == 0 else f"{base}-{duplicate_index}"
        anchors.add(anchor)
    return frozenset(anchors)


def markdown_section_lines(text: str, section: str) -> tuple[str, ...]:
    """Return one level-two section body from structural Markdown lines."""

    lines = structural_markdown_lines(text)
    try:
        start = lines.index(section) + 1
    except ValueError:
        return ()
    end = next(
        (index for index in range(start, len(lines)) if lines[index].startswith("## ")),
        len(lines),
    )
    return lines[start:end]


def resolve_local_target(
    source: PurePosixPath, raw_target: str
) -> PurePosixPath | None:
    """Resolve a repository-local Markdown link without escaping the root."""

    target = raw_target.split("#", 1)[0]
    if not target:
        return source if raw_target.startswith("#") else None
    if "://" in target or target.startswith(("mailto:", "data:")):
        return None
    if target.startswith("/"):
        raise ValueError("absolute filesystem link is not repository-durable")
    resolved = source.parent.joinpath(PurePosixPath(target))
    parts: list[str] = []
    for part in resolved.parts:
        if part in ("", "."):
            continue
        if part == "..":
            if not parts:
                raise ValueError("relative link escapes the repository")
            parts.pop()
        else:
            parts.append(part)
    return PurePosixPath(*parts)


class GitTree:
    """Read a candidate documentation tree from the index/worktree or HEAD."""

    def __init__(self, root: Path, source: str) -> None:
        self.root = root.resolve()
        self.source = source
        self.modes = self._list_modes()
        self.paths = frozenset(self.modes)

    def _git(self, *args: str) -> str:
        result = subprocess.run(
            ["git", "-C", str(self.root), *args],
            check=False,
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            detail = result.stderr.strip() or result.stdout.strip()
            raise RuntimeError(f"git {' '.join(args)} failed: {detail}")
        return result.stdout

    def _list_modes(self) -> dict[PurePosixPath, str]:
        if self.source == "head":
            output = self._git("ls-tree", "-r", "HEAD")
            modes: dict[PurePosixPath, str] = {}
            for line in output.splitlines():
                if not line.strip():
                    continue
                metadata, raw_path = line.split("\t", 1)
                mode, _kind, _object = metadata.split()
                modes[PurePosixPath(raw_path)] = mode
            return modes

        output = self._git("ls-files", "--stage")
        modes: dict[PurePosixPath, str] = {}
        for line in output.splitlines():
            if not line.strip():
                continue
            metadata, raw_path = line.split("\t", 1)
            mode, _object, stage = metadata.split()
            if stage != "0":
                raise RuntimeError(
                    f"unmerged index entry prevents durable audit: {raw_path}"
                )
            modes[PurePosixPath(raw_path)] = mode
        return modes

    def _git_object(self, spec: str) -> bytes:
        result = subprocess.run(
            ["git", "-C", str(self.root), "show", spec],
            check=False,
            capture_output=True,
        )
        if result.returncode != 0:
            raise RuntimeError(
                f"git show {spec} failed: "
                f"{result.stderr.decode(errors='replace').strip()}"
            )
        return result.stdout

    def commit_exists(self, commit: str) -> bool:
        result = subprocess.run(
            ["git", "-C", str(self.root), "cat-file", "-e", f"{commit}^{{commit}}"],
            check=False,
            capture_output=True,
        )
        return result.returncode == 0

    def commit_is_ancestor_of_head(self, commit: str) -> bool:
        result = subprocess.run(
            ["git", "-C", str(self.root), "merge-base", "--is-ancestor", commit, "HEAD"],
            check=False,
            capture_output=True,
        )
        return result.returncode == 0

    def path_exists_at_commit(self, commit: str, path: PurePosixPath) -> bool:
        result = subprocess.run(
            ["git", "-C", str(self.root), "cat-file", "-e", f"{commit}:{path}"],
            check=False,
            capture_output=True,
        )
        return result.returncode == 0

    def read_bytes(self, path: PurePosixPath) -> bytes:
        if path not in self.paths:
            raise FileNotFoundError(path.as_posix())
        mode = self.modes[path]
        if mode not in {"100644", "100755"}:
            raise RuntimeError(
                f"documentation path must be a regular tracked file, got mode "
                f"{mode}: {path}"
            )
        if self.source == "head":
            return self._git_object(f"HEAD:{path.as_posix()}")
        if self.source == "index":
            return self._git_object(f":{path.as_posix()}")

        working_path = self.root.joinpath(*path.parts)
        if working_path.is_symlink():
            raise RuntimeError(
                f"tracked documentation path must not be a worktree symlink: {path}"
            )
        if not working_path.is_file():
            raise RuntimeError(
                f"tracked documentation path is missing from worktree: {path}"
            )
        return working_path.read_bytes()

    def read_text(self, path: PurePosixPath) -> str:
        return self.read_bytes(path).decode("utf-8")


def validate_git_reference(
    tree: GitTree,
    *,
    document: PurePosixPath,
    field: str,
    value: str,
    pattern: re.Pattern[str],
    required_shape: str,
    required_path: PurePosixPath | None = None,
) -> tuple[str, ...]:
    """Validate a SHA-bearing metadata value against reachable Git history."""

    if not pattern.fullmatch(value):
        return (f"current delivery document {document} {field} {required_shape}",)
    commit = value.rsplit("@", 1)[1]
    if not tree.commit_exists(commit):
        return (
            f"current delivery document {document} {field} "
            f"references no commit object: {commit}",
        )
    failures: list[str] = []
    if not tree.commit_is_ancestor_of_head(commit):
        failures.append(
            f"current delivery document {document} {field} commit "
            f"is not an ancestor of HEAD: {commit}"
        )
    if required_path is not None and not tree.path_exists_at_commit(commit, required_path):
        failures.append(
            f"current delivery document {document} {field} commit "
            f"does not contain required path: {required_path}"
        )
    return tuple(failures)
