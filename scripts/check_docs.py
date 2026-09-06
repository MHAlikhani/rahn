#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Deterministic, dependency-free documentation checks for RAHN.

Validates, over the repository's Markdown files plus llms.txt and roadmap.json:

  1. every relative link target exists (Markdown links/images, reference
     definitions, and href/src attributes);
  2. every *.md file except the root README.md carries the CC-BY-4.0 SPDX
     line as its first line, or immediately after a YAML front-matter block;
  3. roadmap.json parses, its current_release matches the workspace version,
     and every stage/candidate title still appears in ROADMAP.md (drift);
  4. every path referenced by llms.txt exists.

Standard library only, no network access, deterministic; exit status 1 on any
failure. Run from anywhere:  python3 scripts/check_docs.py
"""

from __future__ import annotations

import json
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


def workspace_version() -> str:
    section = None
    for line in read("Cargo.toml").splitlines():
        stripped = line.strip()
        if stripped.startswith("["):
            section = stripped
        elif section == "[workspace.package]":
            match = re.match(r'version\s*=\s*"([^"]+)"', stripped)
            if match:
                return match.group(1)
    raise ValueError("workspace.package.version not found in Cargo.toml")


def normalize(text: str) -> str:
    for char in "*`_#[]()":
        text = text.replace(char, " ")
    return re.sub(r"\s+", " ", text)


def check_roadmap(problems: list[str]) -> None:
    try:
        roadmap = json.loads(read("roadmap.json"))
    except (OSError, ValueError) as exc:
        problems.append(f"roadmap.json does not parse: {exc}")
        return
    try:
        version = workspace_version()
    except (OSError, ValueError) as exc:
        problems.append(f"Cargo.toml not readable: {exc}")
        return
    if roadmap.get("current_release") != version:
        problems.append(
            "roadmap.json current_release "
            f"{roadmap.get('current_release')!r} != Cargo.toml {version!r}"
        )
    source = normalize(read("ROADMAP.md"))
    entries = [("stage", item) for item in roadmap.get("stages", [])]
    entries += [("candidate", item) for item in roadmap.get("post_v1_0", [])]
    for kind, item in entries:
        title = item.get("title", "")
        if not title:
            problems.append(f"roadmap.json {kind} entry without title: {item!r}")
        elif title not in source:
            problems.append(f"roadmap drift: {kind} title not in ROADMAP.md: {title!r}")
        if kind == "candidate" and (item.get("status") != "candidate" or item.get("scheduled")):
            problems.append(f"roadmap candidate must be unscheduled: {title!r}")
        evidence = item.get("evidence")
        if evidence and not os.path.exists(os.path.join(ROOT, evidence)):
            problems.append(f"roadmap evidence path missing: {title!r} -> {evidence}")
    limitations = roadmap.get("limitations_doc")
    if not limitations or not os.path.exists(os.path.join(ROOT, limitations)):
        problems.append("roadmap.json limitations_doc does not exist")


def main() -> int:
    problems: list[str] = []
    docs = git_markdown_files()

    for required in ("llms.txt", "roadmap.json"):
        if not os.path.exists(os.path.join(ROOT, required)):
            problems.append(f"missing required file: {required}")

    counts = check_links(docs, problems)
    if os.path.exists(os.path.join(ROOT, "llms.txt")):
        for raw, path in targets(read("llms.txt")):
            if not os.path.exists(os.path.join(ROOT, path)):
                problems.append(f"broken llms.txt path: {raw}")
    check_spdx(docs, problems)
    check_roadmap(problems)

    if problems:
        print(f"check_docs: {len(problems)} problem(s):")
        for problem in problems:
            print(f"  - {problem}")
        return 1
    print(
        f"check_docs: OK ({len(docs)} markdown files, {counts} relative links, "
        "SPDX headers, roadmap drift, llms.txt paths)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
