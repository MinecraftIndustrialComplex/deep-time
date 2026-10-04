#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Print a compact summary of survey.py / moser.py output (one or more JSON streams)."""
import json
import sys


def blocks(txt):
    dec = json.JSONDecoder()
    i = 0
    while True:
        j = txt.find("{", i)
        if j < 0:
            return
        try:
            obj, end = dec.raw_decode(txt[j:])
            i = j + end
            yield obj
        except ValueError:
            i = j + 1


for path in sys.argv[1:]:
    for obj in blocks(open(path).read()):
        for k, v in obj.items():
            d = v.get("distortion")
            c = v.get("creases")
            if d is None:
                print(k, json.dumps(v)[:1500])
                continue
            r = lambda x: round(x, 3)
            print(k, "cover", r(d["planet_area_covered"]), "orient", r(d["orientation_preserving_share"]),
                  "cv", round(d["area_cv"], 4), v.get("build", ""))
            print("  bpa q", {q: r(x) for q, x in d["blocks_per_planet_area_rel_q"].items()})
            print("  shape q", {q: r(x) for q, x in d["shape_ratio_q"].items()},
                  "2w mean/max", round(d["two_omega_mean_deg"], 1), round(d["two_omega_max_deg"], 1))
            print("  le1.5", r(d["share_shape_le_1.5"]), "le2", r(d["share_shape_le_2"]),
                  "area1.25", r(d["share_area_within_x1.25"]), "area2", r(d["share_area_within_x2"]),
                  "drift", [round(x, 1) for x in d["walk_drift_deg_per_1000_blocks_q50_q90_q99"]])
            print("  seams gap %.2e" % c["seam_gap_max"], "seam bend", round(c["seam_bend_max_deg"], 2),
                  "eps", r(c["seam_eps_max"]), "int bend", round(c["interior_bend_max_deg"], 2),
                  "eps", r(c["interior_eps_max"]), "perp seam/int",
                  round(c.get("seam_perp_bend_max_deg", float("nan")), 2),
                  round(c.get("interior_perp_bend_max_deg", float("nan")), 2))
            for line in c["creased_lines"]:
                print("    ", line)
