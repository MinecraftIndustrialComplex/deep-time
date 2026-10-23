#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's `guard` measure: how close a run's surface came to the surface guard (surface/guard.rs,
ISO-072) and how many plate columns end the run far above their Airy height (the flexure guard's lane,
docs/science/flexure.md "The plates held over their Airy height"), from one pool run folder, printed as a
JSON object on stdout.

    python3 -I measure_guard.py <run folder>

From the stage's summary (`summary.txt`, the surface line):

  guard_worst_m      the farthest any L1 cell stood from the sea after a step over the run, m
  guard_over_1 .. 6  steps on which that cell stood more than 1, 2, 3 and 6 Earth reliefs (19.8 km) away
  step_rise_max_m, step_fall_max_m
                     the largest rise and fall the surface stage gave one cell in one step (polar rows
                     and the rest together), m
  plate_cap_share    the share of continental sample-steps standing at `max_elevation_m`

From the final frame's dump (`fields/`, the L1 grid):

  over_airy_5km, over_airy_10km
                     continental cells whose plate elevation stands more than 5 and 10 km over the Airy
                     height of their plate column (z_0 525 m + (T - 35.1 km) / 7.02, PLT-281; no eclogite
                     root, so thick crust reads a little high), a share of the continental cells
  over_airy_max_m    the largest such excess, m
  plate_peak_m       the highest plate elevation, m

Version 1.
"""
import json
import re
import sys
from pathlib import Path

import numpy as np

Z_0 = 525.0  # the normal column's Airy height over the datum, m (PLT-412's T_n at z_0)
T_N = 35_100.0  # the normal continental column, m
FACTOR = 3300.0 / (3300.0 - 2830.0)  # COLUMN_ISOSTATIC_FACTOR, PLT-281


def summary_values(text):
    out = {}
    m = re.search(r"guard_worst_m=([-0-9.eE+]+)", text)
    out["guard_worst_m"] = float(m.group(1)) if m else float("nan")
    m = re.search(r"guard_steps_over_1/2/3/6_reliefs=(\d+)/(\d+)/(\d+)/(\d+)", text)
    for k, v in zip((1, 2, 3, 6), m.groups() if m else (float("nan"),) * 4):
        out[f"guard_over_{k}"] = float(v)
    m = re.search(r"step_rise/fall_polar_m=([-0-9.]+)/([-0-9.]+)", text)
    n = re.search(r"step_rise/fall_rest_m=([-0-9.]+)/([-0-9.]+)", text)
    if m and n:
        out["step_rise_max_m"] = max(float(m.group(1)), float(n.group(1)))
        out["step_fall_max_m"] = max(float(m.group(2)), float(n.group(2)))
    m = re.search(r"plate_cap_bound_share=([-0-9.eE+]+)", text)
    out["plate_cap_share"] = float(m.group(1)) if m else float("nan")
    return out


def fields_values(d):
    idx = json.loads((d / "index.json").read_text())

    def get(name):
        e = idx[name]
        return np.fromfile(d / e["file"], "<f4").reshape(e["h"], e["w"]).astype(np.float64)

    z = get("plates/elevation_m")
    t = get("plates/thickness_m")
    cont = (get("plates/crust_type") > 0.5) & np.isfinite(z) & np.isfinite(t)
    over = z - (Z_0 + (t - T_N) / FACTOR)
    n = max(int(cont.sum()), 1)
    o = over[cont]
    return {
        "over_airy_5km": float((o > 5000.0).sum()) / n,
        "over_airy_10km": float((o > 10000.0).sum()) / n,
        "over_airy_max_m": float(o.max()) if o.size else float("nan"),
        "plate_peak_m": float(np.nanmax(z)),
    }


def main():
    rd = Path(sys.argv[1])
    values = summary_values((rd / "summary.txt").read_text())
    values.update(fields_values(rd / "fields"))
    print(json.dumps(values, sort_keys=True))


if __name__ == "__main__":
    main()
