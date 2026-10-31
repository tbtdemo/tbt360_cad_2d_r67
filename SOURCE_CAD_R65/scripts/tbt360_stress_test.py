#!/usr/bin/env python3
"""Generate deterministic large-coordinate CAD stress fixtures for CAD 2D - TBT 360.

The fixtures use projected metre coordinates representative of Europe and Asia:
one 20,000 ha site boundary and one 1,000 km transport corridor per region.
If --cli is supplied, the built CLI is also asked to inspect each file.
"""
from __future__ import annotations

import argparse
import math
import pathlib
import subprocess
import time


def pair(x: float, y: float) -> str:
    return f"{x:.3f},{y:.3f}"


def line(x1: float, y1: float, x2: float, y2: float) -> list[str]:
    return ["0", "LINE", "8", "INFRA", "10", f"{x1:.3f}", "20", f"{y1:.3f}", "11", f"{x2:.3f}", "21", f"{y2:.3f}"]


def polyline(points: list[tuple[float, float]], layer: str) -> list[str]:
    out = ["0", "LWPOLYLINE", "8", layer, "90", str(len(points)), "70", "1"]
    for x, y in points:
        out += ["10", f"{x:.3f}", "20", f"{y:.3f}"]
    return out


def fixture(name: str, easting: float, northing: float) -> pathlib.Path:
    # 20,000 ha = 200,000,000 m²; a square makes the area explicit and stable.
    side = math.sqrt(200_000_000.0)
    x0, y0 = easting, northing
    boundary = [(x0, y0), (x0 + side, y0), (x0 + side, y0 + side), (x0, y0 + side)]
    # 1,000 km centreline, split into 1 km segments to exercise large entity counts.
    road = [(x0 - 500_000 + i * 1_000, y0 - 200_000 + 0.04 * i * 1_000) for i in range(1_001)]
    records = ["0", "SECTION", "2", "HEADER", "0", "ENDSEC", "0", "SECTION", "2", "ENTITIES"]
    records += polyline(boundary, "SITE_20000HA")
    records += ["0", "LWPOLYLINE", "8", "ROAD_1000KM", "90", str(len(road)), "70", "0"]
    for x, y in road:
        records += ["10", f"{x:.3f}", "20", f"{y:.3f}"]
    # Cross sections every 10 km keep the fixture useful for pan/zoom and selection tests.
    for i in range(0, 1_001, 10):
        x, y = road[i]
        records += line(x, y - 30, x, y + 30)
    records += ["0", "ENDSEC", "0", "EOF", ""]
    path = pathlib.Path(name)
    path.write_text("\n".join(records), encoding="ascii")
    return path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=pathlib.Path, default=pathlib.Path("stress-fixtures"))
    parser.add_argument("--cli", type=pathlib.Path, help="optional tbt360-cli/cadcraft-cli executable")
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    fixtures = [
        fixture(str(args.out / "europe_utm_large_site.dxf"), 500_000, 5_500_000),
        fixture(str(args.out / "asia_utm_large_site.dxf"), 400_000, 1_500_000),
    ]
    for path in fixtures:
        started = time.perf_counter()
        text = path.read_text(encoding="ascii")
        numbers = [float(v) for v in text.split() if v.replace(".", "", 1).replace("-", "", 1).isdigit()]
        assert numbers and all(math.isfinite(v) for v in numbers)
        print(f"generated {path} size={path.stat().st_size:,} bytes elapsed={time.perf_counter() - started:.3f}s")
        if args.cli:
            result = subprocess.run([str(args.cli), "info", str(path)], text=True, capture_output=True, check=False)
            print(f"cli_exit={result.returncode} {result.stdout.strip()[:240]}")
            if result.returncode:
                print(result.stderr.strip())
                return result.returncode
    print("PASS: Europe and Asia projected coordinates remain finite; 20,000 ha and 1,000 km fixtures generated.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
