# SPDX-License-Identifier: LGPL-3.0-or-later
"""The runoff lane: what the continents' export to the sea follows from run to run (VM, cached runs, no simulation).

Usage: python3 -I export_corr.py <label>=<arm folder> [...]

Per run: the surface's export to the sea (`surface_final` `to_sea_km3`, Earth-normalized as bands.py does), its gross
erosion, and every number of the run's cached measures (`measures/*.json`) and of the summary's surface line. Printed:
Spearman's rank correlation of the export with each of them, over the runs of every arm pooled after removing each
arm's median from both (so an arm's own shift does not count), for those with |rho| >= 0.35 and at least 16 runs; and
the arms' medians of the export, the gross erosion and the ratio between them.
"""
import json
import statistics
import sys
from pathlib import Path

SCALE = 5.10e8 / (1024 * 328 * 4.8 * 4.8) / 1.0e9


def ranks(v):
    order = sorted(range(len(v)), key=lambda i: v[i])
    r = [0.0] * len(v)
    i = 0
    while i < len(v):
        j = i
        while j + 1 < len(v) and v[order[j + 1]] == v[order[i]]:
            j += 1
        for k in range(i, j + 1):
            r[order[k]] = (i + j) / 2.0
        i = j + 1
    return r


def spearman(a, b):
    ra, rb = ranks(a), ranks(b)
    ma, mb = statistics.mean(ra), statistics.mean(rb)
    num = sum((x - ma) * (y - mb) for x, y in zip(ra, rb))
    den = (sum((x - ma) ** 2 for x in ra) * sum((y - mb) ** 2 for y in rb)) ** 0.5
    return num / den if den > 0 else float("nan")


def run_values(r):
    out = {}
    for line in open(r / "summary.txt", errors="replace"):
        if "surface_final=" in line:
            for t in line.split():
                if "=" not in t:
                    continue
                k, v = t.split("=", 1)
                try:
                    out["summary." + k] = float(v.split("/")[0])
                except ValueError:
                    pass
            break
    m = r / "measures"
    if m.is_dir():
        for f in sorted(m.glob("*.json")):
            try:
                vals = json.load(open(f)).get("values", {})
            except (OSError, ValueError):
                continue
            for k, v in vals.items():
                if isinstance(v, (int, float)) and v == v:
                    out[f"{f.stem}.{k}"] = float(v)
    return out


def main():
    rows = []
    for a in sys.argv[1:]:
        label, folder = a.split("=", 1)
        runs = [run_values(r) for r in sorted(Path(folder).iterdir()) if r.is_dir() and (r / "summary.txt").exists()]
        runs = [x for x in runs if "summary.to_sea_km3" in x]
        sea = [x["summary.to_sea_km3"] * SCALE for x in runs]
        ero = [x["summary.eroded_km3"] * SCALE for x in runs]
        print(f"== {label}: {len(runs)} runs; export {statistics.median(sea):.2f} km3/yr (per run "
              + ", ".join(f"{v:.2f}" for v in sea) + f"), gross erosion {statistics.median(ero):.2f}, "
              f"export over gross {statistics.median([s / e for s, e in zip(sea, ero)]):.3f}")
        rows.append((label, runs))
    keys = set()
    for _, runs in rows:
        for x in runs:
            keys |= set(x)
    out = []
    for k in sorted(keys):
        if k == "summary.to_sea_km3":
            continue
        a, b = [], []
        for _, runs in rows:
            pair = [(x["summary.to_sea_km3"], x[k]) for x in runs if k in x]
            if len(pair) < 4:
                continue
            ms = statistics.median(p[0] for p in pair)
            mk = statistics.median(p[1] for p in pair)
            a += [p[0] - ms for p in pair]
            b += [p[1] - mk for p in pair]
        if len(a) >= 16 and len(set(b)) > 2:
            rho = spearman(a, b)
            if abs(rho) >= 0.35:
                out.append((abs(rho), rho, k, len(a)))
    print("Spearman's rho of the export with (within-arm deviations, pooled):")
    for _, rho, k, n in sorted(out, reverse=True):
        print(f"  {rho:+.2f}  {k}  (n {n})")


if __name__ == "__main__":
    main()
