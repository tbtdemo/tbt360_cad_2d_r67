#!/usr/bin/env python3
"""Static audit of the CAD 2027 command surface.

This does not claim implementation parity: it only inventories CommandSpec names,
aliases, duplicate aliases, and the high-value commands tracked by the roadmap.
It is intentionally dependency-free so it can run on machines without Rust.
"""

from __future__ import annotations

import argparse
import re
from collections import defaultdict
from pathlib import Path

COMMAND = re.compile(r'CommandSpec::new\("([^"]+)"')
ALIASES = re.compile(r'\.alias\(&\[([^\]]*)\]\)')
STRING = re.compile(r'"([^"]+)"')

# Commands that must be present before the command-coverage phase can be closed.
TARGETS = {
    "draw": {"line", "pline", "circle", "arc", "ellipse", "spline", "rectang", "polygon", "donut", "xline", "ray", "point", "divide", "measure", "revcloud"},
    "modify": {"erase", "move", "copy", "rotate", "scale", "mirror", "offset", "trim", "extend", "fillet", "chamfer", "break", "breakatpoint", "join", "lengthen", "stretch", "explode", "align", "overkill"},
    "blocks": {"block", "insert", "attdef", "attedit", "bedit", "wblock", "xref", "xattach", "xopen", "xclip"},
    "annotation": {"text", "mtext", "textedit", "dimlinear", "dimaligned", "dimangular", "dimradius", "dimdiameter", "qdim", "mleader", "field", "updatefield"},
    "layout_plot": {"layout", "mview", "vpclip", "pagesetup", "plot", "exportpdf", "plottermanager"},
    "exchange": {"pdfattach", "imageattach", "xref", "exportpdf", "saveas"},
    "automation": {"lisp", "appload", "script", "smartblocks"},
    "3d": {"ucs", "3dorbit", "solid", "surface", "mesh", "sectionplane", "visualstyles"},
}


def audit(root: Path) -> int:
    commands: dict[str, str] = {}
    aliases: defaultdict[str, list[tuple[str, str]]] = defaultdict(list)
    for path in root.joinpath("crates/engine/src/cmd").rglob("*.rs"):
        text = path.read_text(encoding="utf-8", errors="replace")
        specs = list(COMMAND.finditer(text))
        for i, spec in enumerate(specs):
            name = spec.group(1).lower()
            commands[name] = str(path.relative_to(root))
            end = specs[i + 1].start() if i + 1 < len(specs) else min(len(text), spec.end() + 1400)
            chunk = text[spec.end():end]
            for block in ALIASES.findall(chunk):
                for alias in STRING.findall(block):
                    aliases[alias.lower()].append((name, str(path.relative_to(root))))

    collisions = {a: owners for a, owners in aliases.items() if len(owners) > 1}
    print(f"commands={len(commands)} aliases={len(aliases)} alias_collisions={len(collisions)}")
    if collisions:
        print("\nALIAS COLLISIONS")
        for alias, owners in sorted(collisions.items()):
            print(f"  {alias}: {owners}")

    missing_total = 0
    print("\nTARGET COVERAGE")
    for group, targets in TARGETS.items():
        missing = sorted(t for t in targets if t not in commands and t not in aliases)
        missing_total += len(missing)
        print(f"  {group}: missing={len(missing)}" + (f" -> {', '.join(missing)}" if missing else ""))
    return 1 if collisions else 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", nargs="?", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    return audit(args.root.resolve())


if __name__ == "__main__":
    raise SystemExit(main())
