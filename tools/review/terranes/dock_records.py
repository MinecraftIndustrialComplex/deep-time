#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""How small terranes dock (the docking lane, 2026-10-30): the collision states of a pool arm's cached
runs by the terrane's size, from each run's dynamics record and crust budget. Measurement only.

    python3 -I dock_records.py <arm folder> [--json OUT]

<arm folder> is a pool folder holding <start>-<seed>/ runs (rec.json, summary.txt), e.g.
/data/ban/deeptime-runs/pool/31c7c70bd7c7/earthlike_quick_16k-1000/cd665cef (base33).

Over each run after its first 100 Myr (the pool's measures' window), every collision state that ended:
  hand-offs (`collisions`: a terrane transferred or two plates welded, `collision::collide`) by the
  terrane's samples at the hand-off (lt50, 50_200, 200_1000, ge1000; about 5,100 km2 each on the quick
  planet, Earth-normalized), and partings (`partings`) by their reason. Per class, pooled over the runs:
  the count per 100 Myr (the median over runs), the duration (end - start, Myr), the shortening (km), the
  front's longest length (km), the crust that crossed the front (km2, the record's `area_km2`) and its
  share of all the crossing, the terrane's area at the hand-off, the share of what reached the front that
  went across it (crossed / (crossed + terrane area): the share stacked as thickness or lost rather than
  moved as area), the volume kept (crossed area x the run's mean crossing thickness, `crossed` /
  `crossed_km2`, x the run's kept share, `kept` / `crossed`, from the last cb_globals line), the uplift
  (`peak_uplift_m`: the front's largest uplift of each step summed; partings: `peak_height_m`, the belt's
  highest mean height), the belt's footprint, and how the hand-off ended (stalled, sutured, transference:
  a continent that did not suture).
Quartiles are p25 / p50 / p75 over the pooled states; counts are medians over runs.
"""
import json
import re
import sys
from pathlib import Path

import numpy as np

SIZES = (("lt50", 0, 50), ("50_200", 50, 200), ("200_1000", 200, 1000), ("ge1000", 1000, 10**12))
REASONS = ("parted", "reversed", "closed", "emptied", "open")
SAMPLE_KM2 = 510.07e6 / 100_000  # Earth-normalized area of a quick-planet sample


def globals_of(summary):
    last = None
    for line in summary.read_text().splitlines():
        if line.startswith("cb_globals"):
            last = line
    if last is None:
        return {}
    return {k: float(v) for k, v in re.findall(r"(\w+)=([-+0-9.eE]+)", last)}


def q(xs):
    xs = [x for x in xs if x is not None and np.isfinite(x)]
    if not xs:
        return None
    return [float(np.percentile(xs, p)) for p in (25, 50, 75)]


def one(run):
    rec = json.loads((run / "rec.json").read_text())
    g = globals_of(run / "summary.txt")
    h_km = g.get("crossed", 0.0) / max(g.get("crossed_km2", 0.0), 1e-9)
    kept = g.get("kept", 0.0) / max(g.get("crossed", 0.0), 1e-9)
    t0 = rec["t0_myr"] + 100.0
    t_end = rec["steps"][-1]["t"]
    span = max(t_end - t0, 1e-9)
    hands = [c for c in rec["collisions"] if c["end_myr"] >= t0]
    parts = [p for p in rec["partings"] if p["end_myr"] >= t0]
    rows = []
    for c in hands:
        size = next(n for n, lo, hi in SIZES if lo <= c["terrane_samples"] < hi)
        terr = c["terrane_samples"] * SAMPLE_KM2
        a = float(c.get("area_km2") or 0.0)
        rows.append({
            "cls": size, "dur": c["end_myr"] - c["start_myr"], "short": c["shortening_km"],
            "front": c["front_km"], "area": a, "terr": terr, "share_x": a / max(a + terr, 1e-9),
            "vol": a * h_km * kept, "uplift": c["peak_uplift_m"],
            # The terrane's extent across the front, km: what crossed (the shortening) and the
            # rest's area over the front's length.
            "extent": c["shortening_km"] + terr / max(c["front_km"], 1e-9),
            "bw": c.get("belt", {}).get("width_km"), "bl": c.get("belt", {}).get("length_km"),
            "lift100": c.get("lifted100", {}).get("samples"),
            "stalled": bool(c.get("stalled")), "sutured": bool(c.get("sutured")),
            "transf": bool(c["frac"] >= 0.1 and not c.get("sutured")),
            "conv_end": c.get("convergence_mm_yr"), "conv_on": c.get("onset_convergence_mm_yr"),
            "pair": (c["under"], c["over"]),
        })
    for p in parts:
        a = float(p.get("area_km2") or 0.0)
        rows.append({
            "cls": "part_" + p.get("reason", "?"), "dur": p["end_myr"] - p["start_myr"],
            "short": p["shortening_km"], "front": p["front_km"], "area": a, "terr": None,
            "share_x": None, "vol": a * h_km * kept, "uplift": p.get("peak_height_m"),
            "stacked": p.get("stacked_km2_m"), "pair": (p["under"], p["over"]),
        })
    return {"run": run.name, "span": span, "h_km": h_km, "kept": kept, "rows": rows}


def table(runs):
    classes = [n for n, _, _ in SIZES] + ["part_" + r for r in REASONS]
    out = {}
    total_area = {r["run"]: sum(x["area"] for x in r["rows"]) or 1e-9 for r in runs}
    total_n = {r["run"]: len(r["rows"]) for r in runs}
    for cls in classes:
        sel = [(r, x) for r in runs for x in r["rows"] if x["cls"] == cls]
        per100 = [100.0 * sum(1 for x in r["rows"] if x["cls"] == cls) / r["span"] for r in runs]
        crossed_share = [sum(x["area"] for x in r["rows"] if x["cls"] == cls) / total_area[r["run"]]
                         for r in runs]
        vol_share = crossed_share  # the kept share is per run, so the volume share is the area's
        xs = [x for _, x in sel]
        o = {
            "per100_median": float(np.median(per100)),
            "share_of_states": float(np.median([sum(1 for x in r["rows"] if x["cls"] == cls)
                                                / max(total_n[r["run"]], 1) for r in runs])),
            "share_of_crossed_median": float(np.median(crossed_share)),
            "n_pooled": len(xs),
            "dur_myr": q([x["dur"] for x in xs]),
            "shortening_km": q([x["short"] for x in xs]),
            "front_km": q([x["front"] for x in xs]),
            "crossed_km2": q([x["area"] for x in xs]),
            "kept_km3": q([x["vol"] for x in xs]),
            "uplift_m": q([x["uplift"] for x in xs]),
        }
        if not cls.startswith("part_"):
            o.update({
                "terrane_km2": q([x["terr"] for x in xs]),
                "extent_km": q([x["extent"] for x in xs]),
                "extent_p10_p90": [float(np.percentile([x["extent"] for x in xs], p)) for p in (10, 90)]
                if xs else None,
                "extent_under": {str(e): float(np.mean([x["extent"] <= e for x in xs])) if xs else None
                                 for e in (150, 220, 270, 300, 350, 400)},
                "share_crossed_of_reached": q([x["share_x"] for x in xs]),
                "share_crossed_of_reached_pooled": float(sum(x["area"] for x in xs)
                                                         / max(sum(x["area"] + x["terr"] for x in xs), 1e-9)),
                "belt_width_km": q([x["bw"] for x in xs if x["bw"]]),
                "belt_length_km": q([x["bl"] for x in xs if x["bl"]]),
                "lifted100": q([x["lift100"] for x in xs]),
                "stalled": float(np.mean([x["stalled"] for x in xs])) if xs else None,
                "sutured": float(np.mean([x["sutured"] for x in xs])) if xs else None,
                "transference": float(np.mean([x["transf"] for x in xs])) if xs else None,
                "conv_end_mm_yr": q([x["conv_end"] for x in xs]),
                "conv_onset_mm_yr": q([x["conv_on"] for x in xs]),
            })
        else:
            o["stacked_km2_m"] = q([x.get("stacked") for x in xs])
        out[cls] = o
    # How often one contact hands off again and again: states per (under, over) pair, by the
    # pair's smallest-class majority.
    reps = []
    for r in runs:
        pairs = {}
        for x in r["rows"]:
            if not x["cls"].startswith("part_"):
                pairs.setdefault(x["pair"], []).append(x["cls"])
        reps += [len(v) for v in pairs.values()]
    out["handoffs_per_pair"] = q(reps)
    out["h_km_median"] = float(np.median([r["h_km"] for r in runs]))
    out["kept_share_median"] = float(np.median([r["kept"] for r in runs]))
    return out


def main():
    arm = Path(sys.argv[1])
    dest = Path(sys.argv[sys.argv.index("--json") + 1]) if "--json" in sys.argv else None
    folders = sorted(p for p in arm.glob("*-*") if p.is_dir() and (p / "rec.json").exists())
    runs = [one(p) for p in folders]
    out = {"arm": str(arm), "runs": len(runs), "table": table(runs)}
    text = json.dumps(out, indent=1)
    if dest:
        dest.write_text(text)
    print(text)


if __name__ == "__main__":
    main()
