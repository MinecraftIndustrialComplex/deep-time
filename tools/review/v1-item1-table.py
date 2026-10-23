#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""V1 item 1's seven measures for two arms of tools/review/v1-item1-table.sh (plates.md, "V1 item
1's seven measures"): per run of each arm, from the census of its final frame
(continents_census.py fields: land, largest landmass, drowned continental crust, land above 2 km,
coast with land above 2 km within 200 km, land under ice) and its plate check (collisions per
100 Myr); per measure the arms' medians and a two-sided Mann-Whitney U test over the runs; the
criteria each run meets, with the same test; and how many runs of each arm meet each criterion.

    python3 -I tools/review/v1-item1-table.py <runs dir> <base arm> <arm> [<lo>-<hi> seeds]

LEN (default 1000) is the runs' length in the file names. Lines starting SUMMARY (per measure),
RUNS and COUNT (criteria met per run) are the summary.
"""
import os
import re
import statistics
import sys

from scipy.stats import mannwhitneyu

R, A0, A1 = sys.argv[1], sys.argv[2], sys.argv[3]
LO, HI = (int(x) for x in (sys.argv[4] if len(sys.argv) > 4 else "1-6").split("-"))
LEN = os.environ.get("LEN", "1000")
STARTS = ("supercontinent", "scattered_continents")
EARTH_MKM2 = 510.07
# V1 item 1's targets (STATUS.md; plates.md "V1 item 1's seven measures").
CRITERIA = [
    ("land", "land, share of the planet (0.26-0.32)", "{:.3f}", lambda v: 0.26 <= v <= 0.32),
    ("largest", "largest landmass, share of the planet (>= 0.12)", "{:.3f}", lambda v: v >= 0.12),
    ("drowned", "continental crust under the sea, share (<= 0.25)", "{:.3f}", lambda v: v <= 0.25),
    ("hi2", "land above 2 km, share of the land (<= 0.12)", "{:.3f}", lambda v: v <= 0.12),
    ("coast_hg", "coast with land above 2 km within 200 km (<= 0.15)", "{:.3f}", lambda v: v <= 0.15),
    ("ice", "land under ice, share of the land (<= 0.15)", "{:.3f}", lambda v: v <= 0.15),
    ("coll", "collisions per 100 Myr (2-6)", "{:.2f}", lambda v: 2.0 <= v <= 6.0),
]
EXTRA = [
    ("cont", "continental crust at the end, share of the planet", "{:.3f}"),
    ("sea_end", "sea at the end over the datum, m", "{:.0f}"),
    ("v1_met", "criteria met by the run (of 7)", "{:.0f}"),
]


def kv(line):
    return dict(re.findall(r"(\w+)=(-?[0-9.eE+-]+|NaN)", line))


def run(arm, start, seed):
    base = f"{R}/{arm}/{arm}-{start}-{seed}-{LEN}"
    row = {}
    try:
        txt = open(base + ".txt", encoding="utf-8").read().splitlines()
    except FileNotFoundError:
        return None
    ts = [kv(line) for line in txt if line.startswith("cb_t ")]
    if ts and "sea_m" in ts[-1]:
        row["sea_end"] = float(ts[-1]["sea_m"])
    try:
        for line in open(base + ".table.txt", encoding="utf-8"):
            if line.startswith("Collisions per 100 Myr"):
                cells = re.split(r"\s{2,}", line.strip())
                row["coll"] = float(cells[1])
    except (FileNotFoundError, ValueError, IndexError):
        pass
    try:
        cen = open(f"{R}/{arm}/census/{start}-{seed}.txt", encoding="utf-8").read().splitlines()
    except FileNotFoundError:
        cen = []
    coast_next = False
    for line in cen:
        m = re.search(r"largest ([\d.]+) Mkm2\)", line)
        if m and line.startswith("land "):
            row["largest"] = float(m.group(1)) / EARTH_MKM2
        if line.startswith("SERIES "):
            s = kv(line)
            row["land"] = float(s["land"])
            row["drowned"] = float(s["cont_wet"])
            row["hi2"] = float(s["hi2"])
            row["ice"] = float(s["glacier"])
            row["cont"] = float(s["cont"])
        if "coastal high ground on the grid's cells" in line:
            coast_next = True
            continue
        if coast_next:
            m = re.search(r"on the same landmass ([\d.]+)", line)
            if m:
                row["coast_hg"] = float(m.group(1))
            coast_next = False
    if all(k in row for k, *_ in CRITERIA):
        row["v1_met"] = sum(bool(f(row[k])) for k, _, _, f in CRITERIA)
    return row


keys = [(st, sd) for st in STARTS for sd in range(LO, HI + 1)]
data = {a: {key: run(a, *key) for key in keys} for a in (A0, A1)}
print(f"runs: {A0} {sum(v is not None for v in data[A0].values())}, "
      f"{A1} {sum(v is not None for v in data[A1].values())} (seeds {LO}-{HI}, both starts)")
for k, title, fmt, *_ in CRITERIA + EXTRA:
    print(f"\n## {title} [{k}]")
    print("run              " + " ".join(f"{a:>12s}" for a in (A0, A1)))
    vals = {A0: [], A1: []}
    for st, sd in keys:
        cells = []
        for a in (A0, A1):
            r = data[a][(st, sd)]
            v = None if r is None else r.get(k)
            if v is not None and v == v:
                vals[a].append(v)
            cells.append(f"{fmt.format(v):>12s}" if v is not None else f"{'-':>12s}")
        print(f"{st[:5]}-{sd:<10d} " + " ".join(cells))
    med = {a: statistics.median(v) if v else float("nan") for a, v in vals.items()}
    ok = len(vals[A0]) >= 3 and len(vals[A1]) >= 3
    p = mannwhitneyu(vals[A1], vals[A0], alternative="two-sided").pvalue if ok else float("nan")
    print("median           " + " ".join(f"{fmt.format(med[a]):>12s}" for a in (A0, A1))
          + f"   Mann-Whitney two-sided p={p:.3f}")
    tag = "RUNS" if k == "v1_met" else "SUMMARY"
    print(f"{tag} {k} n={len(vals[A0])}/{len(vals[A1])} {A0}={fmt.format(med[A0])} "
          f"{A1}={fmt.format(med[A1])} p={p:.3f}")
print("\n## runs meeting each criterion")
for k, _, _, f in CRITERIA:
    n = {a: sum(1 for key in keys if data[a][key] and k in data[a][key] and f(data[a][key][k]))
         for a in (A0, A1)}
    print(f"COUNT {k} {A0}={n[A0]} {A1}={n[A1]}")
