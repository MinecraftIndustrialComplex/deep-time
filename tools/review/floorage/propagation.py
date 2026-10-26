#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""propagation.py <arm folder>...: law 2's records per run (PLT-656) and their medians per arm.

Reads each run folder's summary.txt (the plate model's line: propagation_starts, propagations, propagation_stalls,
initiations_near_fronts, plates_final, plate_births), fields/globals.json (plates/propagation_km, the margin broken in
planet km, and plates/propagation_myr, the Myr summed over live propagations; their ratio is the mean speed) and
rec.json (the plates born with origin margin_propagation: how many, their fates, their lives). Measurement only."""
import json
import re
import statistics
import sys
from pathlib import Path

KEYS = ["propagation_starts", "propagations", "propagation_stalls", "initiations_near_fronts", "plates_final",
        "plate_births"]


def tokens(text):
    out = {}
    for line in text.splitlines():
        if "plates_final=" in line:
            for k, v in re.findall(r"(\w+)=([-0-9.e]+)", line):
                try:
                    out[k] = float(v)
                except ValueError:
                    pass
    return out


def run_part(run):
    s = tokens((run / "summary.txt").read_text()) if (run / "summary.txt").is_file() else {}
    row = {k: s.get(k) for k in KEYS}
    g = {}
    gp = run / "fields" / "globals.json"
    if gp.is_file():
        doc = json.loads(gp.read_text())
        g = doc.get("globals", doc)
    km = g.get("plates/propagation_km", 0.0)
    myr = g.get("plates/propagation_myr", 0.0)
    row["km"] = km
    row["myr"] = myr
    row["km_per_myr"] = km / myr if myr > 0 else None
    r = json.loads((run / "rec.json").read_text()) if (run / "rec.json").is_file() else {"plates": []}
    kp = [p for p in r["plates"] if p.get("origin") == "margin_propagation"]
    fates = {}
    lives = []
    for p in kp:
        f = p.get("fate") or "alive"
        fates[f] = fates.get(f, 0) + 1
        end = p["died_myr"] if p.get("died_myr") is not None else 0.0
        lives.append(end - p["born_myr"])
    row["kprime"] = len(kp)
    row["kprime_fates"] = fates
    row["kprime_absorbed"] = fates.get("absorbed", 0)
    row["kprime_life_p50"] = statistics.median(lives) if lives else None
    row["kprime_dead_within_20"] = sum(1 for x in lives if x <= 20.0) / len(lives) if lives else None
    return row


def main():
    for arm in sys.argv[1:]:
        rows = []
        for run in sorted(p for p in Path(arm).iterdir() if p.is_dir() and (p / "rec.json").is_file()):
            row = run_part(run)
            rows.append(row)
            print(json.dumps({"arm": arm, "run": run.name, **row}), flush=True)
        med = {}
        for k in KEYS + ["km", "myr", "km_per_myr", "kprime", "kprime_absorbed", "kprime_life_p50",
                         "kprime_dead_within_20"]:
            v = [r[k] for r in rows if r.get(k) is not None]
            med[k] = statistics.median(v) if v else None
        print(json.dumps({"arm": arm, "median": med, "runs": len(rows)}), flush=True)


main()
