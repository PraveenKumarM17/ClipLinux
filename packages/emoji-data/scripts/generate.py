#!/usr/bin/env python3
"""Generate compact ClipLinux emoji catalog from Unicode emoji-test + CLDR.

Usage (from repo root):
    python3 packages/emoji-data/scripts/generate.py

Does not require network when vendor files are present.
"""
from __future__ import annotations

import json
import re
import unicodedata
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VENDOR = ROOT / "vendor"
OUT = ROOT / "emoji.compact.json"

UNICODE_VERSION = "17.0"
GROUPS = [
    "Smileys & Emotion",
    "People & Body",
    "Animals & Nature",
    "Food & Drink",
    "Travel & Places",
    "Activities",
    "Objects",
    "Symbols",
    "Flags",
]
TONE_SUFFIX = {
    "light skin tone": 0,
    "medium-light skin tone": 1,
    "medium skin tone": 2,
    "medium-dark skin tone": 3,
    "dark skin tone": 4,
}
LINE_RE = re.compile(
    r"^(?P<codes>[0-9A-F ]+);\s+(?P<status>[^#]+)#\s+(?P<glyph>\S+)\s+E(?P<ver>[0-9.]+)\s+(?P<name>.+)$"
)


def slugify(name: str) -> str:
    s = name.lower()
    s = re.sub(r"[^a-z0-9]+", "-", s)
    return s.strip("-")


def load_annotations() -> dict[str, list[str]]:
    out: dict[str, list[str]] = {}
    for name, root_key in (
        ("cldr-annotations-en.json", "annotations"),
        ("cldr-annotations-derived-en.json", "annotationsDerived"),
    ):
        path = VENDOR / name
        data = json.loads(path.read_text(encoding="utf-8"))
        table = data[root_key]["annotations"]
        for glyph, payload in table.items():
            words = []
            words.extend(payload.get("default") or [])
            words.extend(payload.get("tts") or [])
            existing = out.setdefault(glyph, [])
            for word in words:
                w = word.strip().lower()
                if w and w not in existing:
                    existing.append(w)
    extra = json.loads((ROOT / "aliases.json").read_text(encoding="utf-8"))
    for glyph, words in extra.items():
        existing = out.setdefault(glyph, [])
        for word in words:
            w = word.strip().lower()
            if w and w not in existing:
                existing.append(w)
    return out


def parse_emoji_test() -> tuple[list[dict], dict[str, list[str | None]]]:
    group = ""
    subgroup = ""
    bases: list[dict] = []
    tones: dict[str, list[str | None]] = defaultdict(lambda: [None] * 5)
    by_name: dict[str, dict] = {}

    for raw in (VENDOR / "emoji-test.txt").read_text(encoding="utf-8").splitlines():
        if raw.startswith("# group:"):
            group = raw.split(":", 1)[1].strip()
            continue
        if raw.startswith("# subgroup:"):
            subgroup = raw.split(":", 1)[1].strip()
            continue
        if not raw or raw.startswith("#"):
            continue
        m = LINE_RE.match(raw)
        if not m:
            continue
        if m.group("status").strip() != "fully-qualified":
            continue
        if group not in GROUPS:
            continue
        glyph = m.group("glyph")
        name = m.group("name").strip()
        ver = m.group("ver")
        codes = [int(p, 16) for p in m.group("codes").split()]

        tone_key = None
        base_name = name
        if ": " in name:
            head, tail = name.rsplit(": ", 1)
            if tail in TONE_SUFFIX:
                tone_key = tail
                base_name = head

        if tone_key is not None:
            tones[base_name][TONE_SUFFIX[tone_key]] = glyph
            continue

        rec = {
            "g": glyph,
            "n": name,
            "s": slugify(name),
            "c": GROUPS.index(group),
            "u": subgroup,
            "v": ver,
            "p": codes,
        }
        bases.append(rec)
        by_name[name] = rec
    return bases, tones


def keywords_for(glyph: str, name: str, annotations: dict[str, list[str]]) -> list[str]:
    seen: set[str] = set()
    out: list[str] = []

    def add(word: str) -> None:
        w = word.strip().lower()
        if not w or w in seen or w == name.lower():
            return
        seen.add(w)
        out.append(w)

    for part in re.split(r"[^a-z0-9]+", name.lower()):
        if len(part) > 1:
            add(part)
    for src in (glyph, unicodedata.normalize("NFC", glyph)):
        for word in annotations.get(src, []):
            add(word)
    # CLDR often keys flags without VS16; try stripped variants
    stripped = glyph.replace("\ufe0f", "")
    if stripped != glyph:
        for word in annotations.get(stripped, []):
            add(word)
    return out[:24]


def main() -> None:
    annotations = load_annotations()
    bases, tones = parse_emoji_test()
    rows = []
    for rec in bases:
        row = {
            "g": rec["g"],
            "n": rec["n"],
            "s": rec["s"],
            "c": rec["c"],
            "u": rec["u"],
            "v": rec["v"],
            "p": rec["p"],
            "k": keywords_for(rec["g"], rec["n"], annotations),
        }
        variants = tones.get(rec["n"])
        if variants and all(variants):
            row["t"] = variants
        rows.append(row)

    payload = {
        "unicode": UNICODE_VERSION,
        "source": "Unicode emoji-test.txt + CLDR 48.2 English annotations",
        "groups": GROUPS,
        "emoji": rows,
    }
    OUT.write_text(json.dumps(payload, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    with_tones = sum(1 for r in rows if "t" in r)
    print(f"wrote {OUT} ({len(rows)} emoji, {with_tones} with skin tones, {OUT.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
