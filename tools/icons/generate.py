#!/usr/bin/env python3
"""Build the bundled Lucide icon font and its Rust glyph table.

Reads the icon names in tools/icons/icons.txt, downloads the pinned
lucide-static npm package, checks its SHA-512 integrity, subsets the Lucide
TTF to exactly those glyphs and writes:

  assets/fonts/lucide.ttf          subset font (rendered by theme::icon_font)
  assets/fonts/Lucide-LICENSE.txt  licence shipped with the package (ISC)
  src/icons.rs                     one `pub const` per icon, plus `ALL`

Requires fontTools (pip install fonttools). Run from anywhere:

  python3 tools/icons/generate.py

Unknown icon names, duplicate names, unsorted lists and names that share a
glyph (aliases such as `sort-asc` for `arrow-up-narrow-wide`) are errors.
"""

from __future__ import annotations

import base64
import hashlib
import json
import re
import shutil
import sys
import tarfile
import tempfile
import urllib.request
from pathlib import Path

try:
    from fontTools import subset
    from fontTools.pens.recordingPen import RecordingPen
    from fontTools.ttLib import TTFont
except ImportError:  # pragma: no cover - environment check
    sys.exit("generate.py needs fontTools: pip install fonttools")

# ---------------------------------------------------------------------------
# Pin. To bump Lucide, change the version and the integrity string together
# (take `dist.integrity` from https://registry.npmjs.org/lucide-static/<version>).
LUCIDE_STATIC_VERSION = "1.56.0"
LUCIDE_STATIC_INTEGRITY = (
    "sha512-R1W4B9jCNbHmqD2JnJW5+DF4JAecKP7RSd+eMqV2aQxLKcB0MRKz0QNHpMvJtE8EhFX3Qc5V6XmUpiXD2Vcc2w=="
)
# ---------------------------------------------------------------------------

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
ICON_LIST = HERE / "icons.txt"
OUT_FONT = REPO / "assets" / "fonts" / "lucide.ttf"
OUT_LICENSE = REPO / "assets" / "fonts" / "Lucide-LICENSE.txt"
OUT_RUST = REPO / "src" / "icons.rs"

TARBALL_URL = (
    f"https://registry.npmjs.org/lucide-static/-/lucide-static-{LUCIDE_STATIC_VERSION}.tgz"
)
NAME_RE = re.compile(r"^[a-z0-9]+(-[a-z0-9]+)*$")


def fail(message: str) -> None:
    sys.exit(f"generate.py: error: {message}")


def read_icon_names(path: Path) -> list[str]:
    names: list[str] = []
    for lineno, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        if not NAME_RE.match(line):
            fail(f"{path.name}:{lineno}: '{line}' is not a lucide icon name")
        names.append(line)
    if not names:
        fail(f"{path.name} lists no icons")
    duplicates = sorted({n for n in names if names.count(n) > 1})
    if duplicates:
        fail(f"{path.name} lists duplicates: {', '.join(duplicates)}")
    if names != sorted(names):
        fail(f"{path.name} must be sorted (one name per line)")
    return names


def download_and_verify(dest_dir: Path) -> Path:
    """Fetch the pinned tarball into dest_dir and check its SHA-512 integrity."""
    tarball = dest_dir / f"lucide-static-{LUCIDE_STATIC_VERSION}.tgz"
    print(f"downloading {TARBALL_URL}")
    with urllib.request.urlopen(TARBALL_URL, timeout=120) as response:
        tarball.write_bytes(response.read())
    digest = base64.b64encode(hashlib.sha512(tarball.read_bytes()).digest()).decode()
    actual = f"sha512-{digest}"
    if actual != LUCIDE_STATIC_INTEGRITY:
        fail(f"integrity mismatch for lucide-static {LUCIDE_STATIC_VERSION}: got {actual}")
    print(f"integrity ok ({LUCIDE_STATIC_INTEGRITY[:20]}...)")
    return tarball


def extract(tarball: Path, dest_dir: Path) -> Path:
    with tarfile.open(tarball, "r:gz") as archive:
        archive.extractall(dest_dir, filter="data")
    package = dest_dir / "package"
    if not package.is_dir():
        fail("tarball has no package/ directory")
    return package


def codepoint_map(package: Path) -> dict[str, int]:
    data = json.loads((package / "font" / "codepoints.json").read_text(encoding="utf-8"))
    return {name: int(value) for name, value in data.items()}


def check_names(names: list[str], codepoints: dict[str, int]) -> dict[str, int]:
    unknown = [n for n in names if n not in codepoints]
    if unknown:
        fail(
            "unknown lucide icon(s) for version "
            f"{LUCIDE_STATIC_VERSION}: {', '.join(unknown)} "
            "(check the name or use the current Lucide name)"
        )
    by_codepoint: dict[int, list[str]] = {}
    for name in names:
        by_codepoint.setdefault(codepoints[name], []).append(name)
    aliased = [group for group in by_codepoint.values() if len(group) > 1]
    if aliased:
        pairs = "; ".join(" = ".join(group) for group in aliased)
        fail(f"names share a glyph (aliases), keep one: {pairs}")
    return {name: codepoints[name] for name in names}


def load_font(path: Path) -> TTFont:
    # The package stores glyph bounding boxes that fontTools would recompute on
    # save, and recomputing them shifts the outlines (icons move off-centre).
    # Keep the stored tables as shipped, and keep head.modified for reproducible
    # output.
    return TTFont(str(path), recalcBBoxes=False, recalcTimestamp=False)


def outline(font: TTFont, codepoint: int) -> list:
    glyph_set = font.getGlyphSet()
    pen = RecordingPen()
    glyph_set[font.getBestCmap()[codepoint]].draw(pen)
    return pen.value


def subset_font(src: Path, out: Path, codepoints: set[int]) -> None:
    source = load_font(src)
    font = load_font(src)
    options = subset.Options()
    options.hinting = False
    options.glyph_names = True
    options.notdef_outline = True
    subsetter = subset.Subsetter(options=options)
    subsetter.populate(unicodes=sorted(codepoints))
    subsetter.subset(font)
    font.save(str(out))

    # The written font must map exactly the requested codepoints, and every
    # outline must be identical to the package's.
    written = load_font(out)
    if set(written.getBestCmap()) != codepoints:
        missing = sorted(codepoints - set(written.getBestCmap()))
        extra = sorted(set(written.getBestCmap()) - codepoints)
        fail(f"subset mismatch: missing {missing[:5]}, extra {extra[:5]}")
    for cp in sorted(codepoints):
        if outline(written, cp) != outline(source, cp):
            fail(f"outline of U+{cp:04X} changed while subsetting")


def rust_const(name: str) -> str:
    return name.replace("-", "_").upper()


def rust_source(entries: list[tuple[str, int]]) -> str:
    lines = [
        "//! Lucide icon glyphs (ISC licence, see assets/fonts/Lucide-LICENSE.txt), subset to the",
        "//! icons listed in tools/icons/icons.txt.",
        f"//! GENERATED by tools/icons/generate.py from lucide-static {LUCIDE_STATIC_VERSION}; do not edit",
        "//! by hand: change icons.txt and re-run the script. Render with [`crate::theme::icon_font`].",
        "",
    ]
    by_const = sorted(entries, key=lambda e: rust_const(e[0]))
    for name, cp in by_const:
        lines.append(f'pub const {rust_const(name)}: &str = "\\u{{{cp:X}}}";')
    lines.append("")
    lines.append("/// Every bundled icon as `(lucide name, glyph)`, for galleries.")
    lines.append("pub const ALL: &[(&str, &str)] = &[")
    for name, _ in sorted(entries, key=lambda e: e[0]):
        lines.append(f'    ("{name}", {rust_const(name)}),')
    lines.append("];")
    return "\n".join(lines) + "\n"


def main() -> None:
    names = read_icon_names(ICON_LIST)

    work = Path(tempfile.mkdtemp(prefix="lucide-static-"))
    try:
        download_dir = work / "download"
        download_dir.mkdir()
        extract_dir = work / "extract"
        extract_dir.mkdir()
        tarball = download_and_verify(download_dir)
        package = extract(tarball, extract_dir)

        codepoints = check_names(names, codepoint_map(package))
        src_font = package / "font" / "lucide.ttf"
        src_license = package / "LICENSE"
        for path in (src_font, src_license):
            if not path.is_file():
                fail(f"package is missing {path.relative_to(package)}")

        subset_font(src_font, OUT_FONT, set(codepoints.values()))
        shutil.copyfile(src_license, OUT_LICENSE)
        OUT_RUST.write_text(
            rust_source(list(codepoints.items())), encoding="utf-8"
        )
    finally:
        shutil.rmtree(work, ignore_errors=True)

    print(
        f"wrote {len(codepoints)} icons, "
        f"{OUT_FONT.relative_to(REPO)} ({OUT_FONT.stat().st_size} bytes), "
        f"{OUT_RUST.relative_to(REPO)}"
    )


if __name__ == "__main__":
    main()
