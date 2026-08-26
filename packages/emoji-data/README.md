# Emoji data package

Offline Unicode emoji catalog consumed by `clipl-emoji`.

## Pinned versions

| Dataset | Version |
| --- | --- |
| Unicode Emoji | **17.0** (`emoji-test.txt`, dated 2025-08-04) |
| CLDR annotations | **48.2.1** English (`annotations` + `annotationsDerived`) |

Source files (local copies):

- `vendor/emoji-test.txt` — Unicode emoji keyboard/display test data
- `vendor/cldr-annotations-en.json`
- `vendor/cldr-annotations-derived-en.json`
- `aliases.json` — ClipLinux extras (`linux` → 🐧, `india` → 🇮🇳, …)

Generated runtime file:

- `emoji.compact.json` — compact JSON included by `clipl-emoji` at compile time (`include_str!`). Parsed **once** with `OnceLock`.

## License / attribution

Emoji data © Unicode, Inc. Unicode® is a trademark of Unicode, Inc.

Terms: <https://www.unicode.org/terms_of_use.html>  
Unicode License v3: SPDX `Unicode-3.0`

ClipLinux does not ship a font. Rendering uses the host emoji font.

## Generate / update

From the repository root (no network if `vendor/` is present):

```bash
python3 packages/emoji-data/scripts/generate.py
```

To refresh vendor files:

```bash
curl -fsSL -o packages/emoji-data/vendor/emoji-test.txt \
  https://www.unicode.org/Public/emoji/17.0/emoji-test.txt
curl -fsSL -o packages/emoji-data/vendor/cldr-annotations-en.json \
  https://raw.githubusercontent.com/unicode-org/cldr-json/48.2.1/cldr-json/cldr-annotations-full/annotations/en/annotations.json
curl -fsSL -o packages/emoji-data/vendor/cldr-annotations-derived-en.json \
  https://raw.githubusercontent.com/unicode-org/cldr-json/48.2.1/cldr-json/cldr-annotations-derived-full/annotationsDerived/en/annotations.json
python3 packages/emoji-data/scripts/generate.py
```

Then run `cargo test -p clipl-emoji`.
