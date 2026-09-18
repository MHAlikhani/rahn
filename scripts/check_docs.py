#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Deterministic, dependency-free documentation checks for RAHN.

Validates, over the repository's Markdown files:

  1. every relative link target exists (Markdown links/images, reference
     definitions, and href/src attributes);
  2. every *.md file except the root README.md carries the CC-BY-4.0 SPDX
     line as its first line, or immediately after a YAML front-matter block.

Standard library only, no network access, deterministic; exit status 1 on any
failure. Run from anywhere:  python3 scripts/check_docs.py
"""

from __future__ import annotations

import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SPDX_DOC = "SPDX-License-Identifier: CC-BY-4.0"
README = "README.md"

LINK = re.compile(r"!?\[[^\]]*\]\(\s*<?([^)>\s]+)>?(?:\s+\"[^\"]*\")?\s*\)")
REF = re.compile(r"^\[[^\]]+\]:\s*(\S+)", re.M)
ATTR = re.compile(r'(?:href|src)="([^"]+)"')
SKIP_SCHEME = ("http://", "https://", "mailto:", "tel:", "#", "data:")


def git_markdown_files() -> list[str]:
    """Tracked and not-yet-committed Markdown files, sorted, repo-relative."""
    cmd = ["git", "ls-files", "--cached", "--others", "--exclude-standard", "--", "*.md"]
    try:
        out = subprocess.run(
            cmd, cwd=ROOT, capture_output=True, text=True, check=True
        ).stdout
    except (OSError, subprocess.CalledProcessError) as exc:  # pragma: no cover
        print(f"FAIL: cannot list Markdown files via git: {exc}")
        sys.exit(1)
    return sorted({line.strip() for line in out.splitlines() if line.strip()})


def targets(text: str):
    """Relative link targets in one document, in first-appearance order."""
    seen = set()
    for pattern in (LINK, REF, ATTR):
        for match in pattern.finditer(text):
            raw = match.group(1).strip()
            if raw.lower().startswith(SKIP_SCHEME):
                continue
            path = raw.split("#", 1)[0]
            if not path:
                continue
            if path not in seen:
                seen.add(path)
                yield raw, path


def resolve(owner: str, path: str) -> str:
    base = os.path.dirname(owner)
    return os.path.normpath(os.path.join(base, path.replace("%20", " ")))


def read(rel: str) -> str:
    with open(os.path.join(ROOT, rel), encoding="utf-8") as handle:
        return handle.read()


def check_links(docs: list[str], problems: list[str]) -> int:
    count = 0
    for doc in docs:
        for raw, path in targets(read(doc)):
            count += 1
            if not os.path.exists(os.path.join(ROOT, resolve(doc, path))):
                problems.append(f"broken link: {doc} -> {raw}")
    return count


def has_spdx(text: str) -> bool:
    lines = text.splitlines()
    if lines and lines[0].strip() == "---":  # YAML front matter (issue templates)
        for index in range(1, len(lines)):
            if lines[index].strip() == "---":
                lines = lines[index + 1 :]
                break
    for line in lines:
        if line.strip():
            return SPDX_DOC in line
    return False


def check_spdx(docs: list[str], problems: list[str]) -> None:
    for doc in docs:
        if doc == README:
            continue
        if not has_spdx(read(doc)):
            problems.append(f"missing CC-BY-4.0 SPDX line: {doc}")


def main() -> int:
    problems: list[str] = []
    docs = git_markdown_files()

    counts = check_links(docs, problems)
    check_spdx(docs, problems)

    if problems:
        print(f"check_docs: {len(problems)} problem(s):")
        for problem in problems:
            print(f"  - {problem}")
        return 1
    print(
        f"check_docs: OK ({len(docs)} markdown files, {counts} relative links, "
        "SPDX headers)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
