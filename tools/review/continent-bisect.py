#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The continent bisect's table (build log 2026-10-21): per commit, start and stop, the land and
the continents of runs of that commit's own binary and shipped preset, read from `deeptime
dump-fields` of each run's checkpoint (the same reading for every commit, so the numbers compare).

Runs are written by tools/review/continent-bisect.sh into <root>/runs/<commit>/<start>-<seed>-<stop>.txt
(the stage's stdout) with the fields of its checkpoint in <root>/fields/<commit>/<start>-<seed>-<stop>/
(`dump-fields`: index.json, globals.json, <field>.f32 raw little-endian). Stdlib only (it reads
f32 arrays with `array`), so any python3 runs it.

    python3 -I tools/review/continent-bisect.py <root> <commit,...> <seeds,> <starts,> <stops,> [--csv F] [--md F] [--runs]

Metrics per run, on the L1 grid (a Lambert cylindrical equal-area raster: every cell the same
area except the rows that reach past a pole; `GridDesc::area_share`), true-area weighted; land is
the elevation above the climate stage's sea level (`climate/sea_level_m`):

  land      land share of the planet (the build log's `hypsometry land_frac_area`)
  cont      continental crust's share of the planet (`plates/crust_type` >= 0.5)
  drown     the share of the continental crust under the sea (Earth about 0.19; 1 - `continental_dry`)
  hi2       land above 2 km, as a share of the land (Earth 0.102 of the land by the build log)
  L1, L2    the largest and the second-largest landmass, shares of the planet's surface
            (Earth: Afro-Eurasia 0.167, the Americas 0.083, Antarctica 0.027, Australia 0.015)
  n1        landmasses of at least 0.01 of the planet (Earth 4)
  in1       the share of all land that sits in landmasses of at least 0.01 of the planet (Earth 0.95)
  arc       the island-arc share of the land (the crust budget's `cb_origin`, DEEPTIME_CRUST_BUDGET=1,
            the L0 samples; only where the commit has it: 4b615b24 on)

The sea level is `climate/sea_level_m` of `globals.json` where `dump-fields` writes one (a later
commit), else the `sea_level_m=` of the stage's printed `surface_final` line (one decimal: under
0.05 m, which moves no cell count past the printed `check`). A run's time is its printed `t_years`
(0 at the end of the planned run, -500000000 for a stop at 500 Myr of 1000).

A landmass is a set of land cells joined by an edge or a corner (8-neighbours), wrapped east-west
across the seam. The cross-check against the stage's own printed `hypsometry` line (land_frac_area,
continental_frac, continental_dry, land_above_2km) is reported as `check` (largest difference).
"""
import array
import json
import math
import re
import sys
from collections import deque
from pathlib import Path


def read_f32(path):
    a = array.array("f")
    with open(path, "rb") as f:
        a.frombytes(f.read())
    if sys.byteorder != "little":
        a.byteswap()
    return a


def area_shares(w, h, r):
    """`GridDesc::area_share` of every row of an L1 grid of w x h cells of r blocks on the planet
    of circumference w*r (Lambert: rows uniform in sin(lat), the edge rows take what lies beyond
    them, a row wholly past a pole 0). Zo is the pole's |z| rounded up to the 64-block quantum."""
    radius = w * r / (2.0 * math.pi)
    zo = math.ceil(radius / 64.0) * 64
    if abs(2 * zo - h * r) > r:
        raise ValueError(f"grid {w}x{h} r={r}: half extent {zo} does not give {h} rows")
    out = []
    for j in range(h):
        z = -zo + j * r
        half = 0.5 * r
        lo = -math.inf if j == 0 else z - half
        hi = math.inf if j + 1 >= h else z + half
        out.append(min(max((min(hi, radius) - max(lo, -radius)) / r, 0.0), 1.0))
    return out


def landmasses(land, w, h):
    """The 8-connected components of a boolean raster (flat, row-major, w x h), wrapped east-west
    across the seam, rows clamped at the ends: a list of cell-index lists."""
    seen = bytearray(w * h)
    comps = []
    for start in range(w * h):
        if not land[start] or seen[start]:
            continue
        seen[start] = 1
        comp = [start]
        queue = deque([start])
        while queue:
            c = queue.popleft()
            j, i = divmod(c, w)
            for dj in (-1, 0, 1):
                jj = j + dj
                if jj < 0 or jj >= h:
                    continue
                base = jj * w
                for di in (-1, 0, 1):
                    if dj == 0 and di == 0:
                        continue
                    n = base + (i + di) % w
                    if land[n] and not seen[n]:
                        seen[n] = 1
                        comp.append(n)
                        queue.append(n)
        comps.append(comp)
    return comps


def kv_line(text, tag):
    """The last line starting with `tag ` as a dict of its key=value numbers."""
    lines = [l for l in text.splitlines() if l.startswith(tag + " ") or (" " + tag + " ") in l[:80]]
    if not lines:
        return {}
    return {k: float(v) for k, v in re.findall(r"(\w+)=(-?[0-9.]+(?:e[-+]?[0-9]+)?)(?=[\s\]]|$)", lines[-1])}


def hypsometry(text):
    m = re.search(r"hypsometry land_frac_area=\S+.*", text)
    return kv_line(m.group(0), "hypsometry") if m else {}


def run_time_myr(text):
    """The run's final time in Myr from the stage's `t_years=` (0: the end of the planned run)."""
    m = re.search(r"stage volcanoes: steps=\d+ t_years=(-?[0-9.e+]+)", text)
    return float(m.group(1)) / 1.0e6 if m else None


def island_arc_share(text):
    """The island-arc share of the land at the run's final time: the `cb_origin` snapshot at that
    time (DEEPTIME_CRUST_BUDGET=1 writes one at the end); None when the commit has no such line."""
    snaps = {}
    for m in re.finditer(r"^cb_origin t=(-?[0-9.]+) island_arc=\[land=([0-9.]+)", text, re.M):
        snaps[float(m.group(1))] = float(m.group(2))
    t_end = run_time_myr(text)
    if not snaps or t_end is None:
        return None
    t = min(snaps, key=lambda x: abs(x - t_end))
    return snaps[t] if abs(t - t_end) <= 5 else None


def printed_sea_level(text):
    m = re.search(r"stage volcanoes: surface_final=\d+ .*?sea_level_m=(-?[0-9.]+)", text)
    return float(m.group(1)) if m else None


def measure(folder, txt):
    folder = Path(folder)
    text = Path(txt).read_text(errors="replace") if Path(txt).exists() else ""
    index = json.loads((folder / "index.json").read_text())
    if (folder / "globals.json").exists():
        sea = json.loads((folder / "globals.json").read_text())["globals"]["climate/sea_level_m"]
    else:
        sea = printed_sea_level(text)
        if sea is None:
            raise ValueError(f"{folder}: no sea level (no globals.json, no surface_final line)")
    g = index["surface/elevation_m"]
    w, h, r = g["w"], g["h"], g["r"]
    elev = read_f32(folder / g["file"])
    crust = read_f32(folder / index["plates/crust_type"]["file"])
    rows = area_shares(w, h, r)
    total = w * sum(rows)
    land = bytearray(w * h)
    land_area = cont = cont_dry = hi2 = 0.0
    for j in range(h):
        a = rows[j]
        for k in range(j * w, (j + 1) * w):
            z = elev[k] - sea
            if z > 0.0:
                land[k] = 1
                land_area += a
                if z > 2000.0:
                    hi2 += a
            if crust[k] >= 0.5:
                cont += a
                if z > 0.0:
                    cont_dry += a
    comps = landmasses(land, w, h)
    sizes = sorted((sum(rows[c // w] for c in comp) for comp in comps), reverse=True)
    big = [s for s in sizes if s >= 0.01 * total]
    out = {
        "land": land_area / total,
        "cont": cont / total,
        "drown": 1.0 - cont_dry / cont if cont > 0 else float("nan"),
        "hi2": hi2 / land_area if land_area > 0 else float("nan"),
        "L1": sizes[0] / total if sizes else 0.0,
        "L2": sizes[1] / total if len(sizes) > 1 else 0.0,
        "n1": float(len(big)),
        "in1": sum(big) / land_area if land_area > 0 else float("nan"),
        "pieces": float(len(sizes)),
        "sea_m": sea,
    }
    arc = island_arc_share(text)
    if arc is not None:
        out["arc"] = arc
    hyp = hypsometry(text)
    if hyp:
        # The printed line counts the same cells with the same weights: the readings must agree.
        diffs = [
            abs(out["land"] - hyp.get("land_frac_area", out["land"])),
            abs(out["cont"] - hyp.get("continental_frac", out["cont"])),
            abs(out["drown"] - (1.0 - hyp["continental_dry"])) if "continental_dry" in hyp else 0.0,
            abs(out["hi2"] - hyp.get("land_above_2km", out["hi2"])),
        ]
        out["check"] = max(diffs)
    return out


KEYS = ["land", "cont", "drown", "hi2", "L1", "L2", "n1", "in1", "arc", "pieces", "check"]
HEAD = {
    "land": "land", "cont": "cont", "drown": "drown", "hi2": "hi2", "L1": "L1", "L2": "L2",
    "n1": "n>=.01", "in1": "in>=.01", "arc": "arc", "pieces": "pieces", "check": "check",
}


def fmt(k, v):
    if v is None or v != v:
        return "-"
    if k in ("n1", "pieces"):
        return f"{v:.1f}"
    if k == "check":
        return f"{v:.4f}"
    return f"{v:.3f}"


def mean(vals):
    ok = [v for v in vals if v is not None and v == v]
    return sum(ok) / len(ok) if ok else None


def sd(vals):
    ok = [v for v in vals if v is not None and v == v]
    if len(ok) < 2:
        return None
    m = sum(ok) / len(ok)
    return math.sqrt(sum((v - m) ** 2 for v in ok) / (len(ok) - 1))


def main():
    args = sys.argv[1:]
    csv_path = None
    md_path = None
    md = []
    per_run = False
    if "--csv" in args:
        k = args.index("--csv")
        csv_path = args[k + 1]
        del args[k : k + 2]
    if "--md" in args:
        k = args.index("--md")
        md_path = args[k + 1]
        del args[k : k + 2]
    if "--runs" in args:
        args.remove("--runs")
        per_run = True
    root = Path(args[0])
    commits, seeds, starts, stops = (a.split(",") for a in args[1:5])
    rows_csv = ["commit,start,seed,stop," + ",".join(KEYS + ["sea_m"])]
    pooled = {}  # (stop, commit) -> every run's result over the starts and seeds
    for stop in stops:
        for start in starts:
            print(f"\n== {start}, t = {stop} Myr, seeds {','.join(seeds)} (mean over the runs present)")
            print(f"{'commit':10}{'runs':>5}" + "".join(f"{HEAD[k]:>9}" for k in KEYS))
            md.append(f"\n### {start}, {stop} Myr (mean over seeds {','.join(seeds)})\n")
            md.append("| commit | runs | " + " | ".join(HEAD[k] for k in KEYS) + " |")
            md.append("|---|---|" + "---|" * len(KEYS))
            for c in commits:
                results = []
                for seed in seeds:
                    name = f"{start}-{seed}-{stop}"
                    fdir = root / "fields" / c / name
                    if not (fdir / "index.json").exists():
                        continue
                    res = measure(fdir, root / "runs" / c / f"{name}.txt")
                    results.append((seed, res))
                    pooled.setdefault((stop, c), []).append(res)
                    rows_csv.append(
                        f"{c},{start},{seed},{stop},"
                        + ",".join("" if res.get(k) is None else f"{res[k]:.6g}" for k in KEYS + ["sea_m"])
                    )
                    if per_run:
                        print(f"  {c:8}s{seed:>3}" + "".join(f"{fmt(k, res.get(k)):>9}" for k in KEYS))
                cells = "".join(f"{fmt(k, mean([r.get(k) for _, r in results])):>9}" for k in KEYS)
                print(f"{c:10}{len(results):>5}{cells}")
                md.append(f"| {c} | {len(results)} | "
                          + " | ".join(fmt(k, mean([r.get(k) for _, r in results])) for k in KEYS) + " |")
    if len(starts) > 1:
        for stop in stops:
            title = f"both starts, t = {stop} Myr, seeds {','.join(seeds)} (mean over every run present)"
            print(f"\n== {title}")
            print(f"{'commit':10}{'runs':>5}" + "".join(f"{HEAD[k]:>9}" for k in KEYS) + f"{'sd land':>9}{'sd L1':>9}{'sd n':>9}")
            md.append(f"\n### {title}\n")
            md.append("| commit | runs | " + " | ".join(HEAD[k] for k in KEYS) + " | sd land | sd L1 | sd n>=.01 |")
            md.append("|---|---|" + "---|" * (len(KEYS) + 3))
            for c in commits:
                rs = pooled.get((stop, c), [])
                spread = [fmt(k, sd([r.get(k) for r in rs])) for k in ("land", "L1", "n1")]
                print(f"{c:10}{len(rs):>5}" + "".join(f"{fmt(k, mean([r.get(k) for r in rs])):>9}" for k in KEYS)
                      + "".join(f"{x:>9}" for x in spread))
                md.append(f"| {c} | {len(rs)} | " + " | ".join(fmt(k, mean([r.get(k) for r in rs])) for k in KEYS)
                          + " | " + " | ".join(spread) + " |")
    if csv_path:
        Path(csv_path).write_text("\n".join(rows_csv) + "\n")
    if md_path:
        Path(md_path).write_text("\n".join(md) + "\n")


if __name__ == "__main__":
    main()
