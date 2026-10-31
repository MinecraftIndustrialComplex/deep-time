#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The V1 census at another run's sea (no simulation): seashift.py <arm folder> <base folder> [--seeds N]

For each run of the arm (supercontinent and scattered_continents, seeds 1 to N), the pool's v1census
(tools/review/pool/measure_v1census.py, its own `census`, not a copy) on the run's final frame twice: as it stands, and
with the sea moved to the base arm's same run (start and seed): every cell's height over the sea raised by the arm's
sea less the base's (rec.json's last sea_m), i.e. the arm's ground under the base's sea. The checks the V1 table counts
(land 0.13 to 0.36, largest landmass >= 0.06, continental crust under the sea <= 0.56, land above 2 km <= 0.07, coast
with high ground <= 0.11; ice and collisions as recorded) are printed per run, so a check that fails at the arm's sea
and passes at the base's fails by the sea alone.

A static reading: had the sea stood lower through the run, the interiors would have eroded towards it (freeboard
thinning trace, sea-level.md), so this is the sea's first-order share, not a counterfactual run. Needs numpy and scipy
(the VM's .pyenv-numpy, run with -I).
"""
import contextlib
import io
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
sys.path.insert(0, str(HERE.parent / "pool"))

import continents_census as cc  # noqa: E402
import measure_v1census as mv  # noqa: E402

CHECKS = [
    ("land", lambda v: 0.13 <= v["land"] <= 0.36),
    ("largest", lambda v: v["largest"] >= 0.06),
    ("under_sea", lambda v: v["under_sea"] <= 0.56),
    ("above_2km", lambda v: v["above_2km"] <= 0.07),
    ("coast_hi", lambda v: v["coast_hi"] <= 0.11),
]


def sea_end(run):
    return float(json.loads((run / "rec.json").read_text())["steps"][-1]["sea_m"])


def census_shifted(run, shift_m):
    """measure_v1census.census with every height over the sea raised by shift_m (the sea lowered by it)."""
    load = cc.load_dump

    def shifted(d):
        f, g, e = load(d)
        f = dict(f)
        f["elevation"] = f["elevation"] + shift_m / cc.M_PER_BLOCK_V
        return f, g, e

    cc.load_dump = shifted
    try:
        with contextlib.redirect_stdout(io.StringIO()):
            return mv.census(run / "fields")
    finally:
        cc.load_dump = load


def main(argv):
    arm, base = Path(argv[0]), Path(argv[1])
    n = int(argv[argv.index("--seeds") + 1]) if "--seeds" in argv else 4
    print(f"# {arm} at its own sea and at {base}'s (same start and seed)")
    print(f"{'run':26s} {'dS m':>6s} | {'land':>6s} {'largest':>7s} {'u_sea':>6s} {'>2km':>6s} {'coast':>6s} fails |"
          f" {'land':>6s} {'largest':>7s} {'u_sea':>6s} {'>2km':>6s} {'coast':>6s} fails at the base's sea")
    for start in ("supercontinent", "scattered_continents"):
        for seed in range(1, n + 1):
            r, b = arm / f"{start}-{seed}", base / f"{start}-{seed}"
            if not (r / "fields").is_dir() or not (b / "rec.json").exists():
                continue
            ds = sea_end(r) - sea_end(b)
            own = census_shifted(r, 0.0)
            low = census_shifted(r, ds)
            f0 = [k for k, ok in CHECKS if not ok(own)]
            f1 = [k for k, ok in CHECKS if not ok(low)]
            row = lambda v: (f"{v['land']:6.3f} {v['largest']:7.3f} {v['under_sea']:6.3f} {v['above_2km']:6.3f}"
                             f" {v['coast_hi']:6.3f}")
            print(f"{r.name:26s} {ds:6.0f} | {row(own)} {','.join(f0) or '-'} | {row(low)} {','.join(f1) or '-'}")


if __name__ == "__main__":
    main(sys.argv[1:])
