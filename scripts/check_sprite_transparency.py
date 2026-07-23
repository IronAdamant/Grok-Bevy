#!/usr/bin/env python3
"""Audit sprites: zero true-magenta plates (R>200,G<40,B>200,A>200). Exit 1 on failure."""
from __future__ import annotations

import sys
from pathlib import Path

try:
    from PIL import Image
except ImportError:
    print("Pillow required", file=sys.stderr)
    sys.exit(2)


def true_magenta_count(path: Path) -> int:
    im = Image.open(path).convert("RGBA")
    n = 0
    for r, g, b, a in im.getdata():
        if a > 200 and r > 200 and g < 40 and b > 200:
            n += 1
    return n


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(f"usage: {argv[0]} <sprites_dir|png...>", file=sys.stderr)
        return 2
    paths: list[Path] = []
    for a in argv[1:]:
        p = Path(a)
        if p.is_dir():
            paths.extend(sorted(p.glob("*.png")))
        else:
            paths.append(p)
    bad = 0
    for p in paths:
        c = true_magenta_count(p)
        status = "OK" if c == 0 else f"FAIL magenta={c}"
        print(f"{p}: {status}")
        if c:
            bad += 1
    return 1 if bad else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
