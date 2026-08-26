#!/usr/bin/env python3
"""Check relative Markdown links in this repository. No network.

Usage (from repo root):
    python3 scripts/check-md-links.py
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SKIP_DIRS = {".git", "node_modules", "target", "dist", ".svelte-kit"}
LINK_RE = re.compile(r"\[[^\]]*\]\(([^)]+)\)")


def iter_markdown() -> list[Path]:
    files: list[Path] = []
    for path in ROOT.rglob("*.md"):
        if any(part in SKIP_DIRS for part in path.parts):
            continue
        files.append(path)
    return files


def href_target(raw: str) -> str | None:
    href = raw.strip()
    if href.startswith("<") and href.endswith(">"):
        href = href[1:-1].strip()
    if not href or href.startswith(("#", "http://", "https://", "mailto:")):
        return None
    # Drop optional title: (path "title")
    if href[0] in "\"'":
        return None
    href = href.split()[0].strip("\"'")
    path_part = href.split("#", 1)[0]
    return path_part or None


def main() -> int:
    broken: list[str] = []
    checked = 0
    for md in iter_markdown():
        text = md.read_text(encoding="utf-8")
        for match in LINK_RE.finditer(text):
            rel = href_target(match.group(1))
            if rel is None:
                continue
            checked += 1
            target = (md.parent / rel).resolve()
            if not target.exists():
                broken.append(f"{md.relative_to(ROOT)} → {rel}")
    if broken:
        print(f"Broken relative Markdown links ({len(broken)}):", file=sys.stderr)
        for item in broken:
            print(f"  {item}", file=sys.stderr)
        return 1
    print(f"Markdown links OK ({checked} relative targets checked).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
