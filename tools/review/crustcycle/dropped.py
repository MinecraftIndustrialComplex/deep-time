#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""What the resamplings drop, arm against arm (no simulation): dropped.py <base folder>[@N] <arm folder>[@N]

Reads each run's summary.txt (the crust budget, DEEPTIME_CRUST_BUDGET): the last `cb_globals` line (totals over the
run, km^3 Earth-normalized) and the last `cb_parts` line (each stock's change by part over the run, 10^6 km^3). Per run,
as km^3 a year over the run:

  dropped         visible continental crust of another plate under a resampling's top cover that neither accretes
                  nor is consumed (no contact puts it under the top's plate): dropped with its pools
                  (plates/resample_dropped_km3, ridges.rs; 0 since the consumer of last resort, PLT-812)
  last_resort_c   the consumer of last resort (PLT-812, ridges.rs): visible continental crust of a pair with no
                  contact under the other plate's top cover, accreted as a contact would
                  (plates/resample_last_resort_continent_km3)
  last_resort_o   the same for visible ocean floor, subducted with its sediment booked
                  (plates/resample_last_resort_ocean_km3)
  last_resort_sed of that floor, its sediment and sedimentary rock (plates/resample_last_resort_sediment_km3)
  last_resort_n   the new samples that took such a cover, a year (plates/resample_last_resort_samples; a count)
  last_resort_rise the largest rise the last resort's accretion gave one sample over the run, m (not a rate)
  mafic_dropped   the mafic pools of continental crust that accretes (plates/resample_mafic_dropped_km3)
  accreted        what accretes at resamplings (plates/resample_accreted_km3), visible crust of it (accreted_visible)
  sed_subducted   sediment that goes down with the slab at resamplings (plates/sediment_subducted_km3)
  residual        the sediment books' resampling residual: the change of the sediment on the visible and the hidden
                  ocean floor and on continental crust under resample_blend, plus sed_subducted (what the blend made or
                  lost; tools/review/crustcycle/steps.py's "blend residual" over the whole run), and its ocean-floor
                  and continental sides

and the medians of each arm with the two-sided Mann-Whitney p (scipy). Before the consumer of last resort, visible
ocean floor of another plate under the top cover with no contact vanished with its sediment and was in no global: its
sediment shows in the residual only (runs without the last-resort globals read 0 for them).
"""
import json
import re
import statistics
import sys
from pathlib import Path

from scipy.stats import mannwhitneyu

sys.path.insert(0, str(Path(__file__).resolve().parent))
import probe  # noqa: E402

KV = re.compile(r"(\w+)=([-+0-9.eE]+)")
PART = re.compile(r"(\w+):([-+0-9.eE]+)")


def last(lines, prefix):
    for line in reversed(lines):
        if line.startswith(prefix):
            return line
    return None


def run_row(run):
    lines = (run / "summary.txt").read_text().splitlines()
    g_line, p_line = last(lines, "cb_globals "), last(lines, "cb_parts ")
    if not g_line or not p_line:
        return None
    g = {k: float(v) for k, v in KV.findall(g_line)}
    parts = {}
    for name, body in re.findall(r"(\w+)=\[([^\]]*)\]", p_line):
        parts[name] = {k: float(v) for k, v in PART.findall(body)}
    # The run's start (rec.json), not the first snapshot's time.
    t0 = float(json.loads((run / "rec.json").read_text())["t0_myr"])
    years = (g["t"] - t0) * 1.0e6
    blend = lambda names: sum(parts.get(n, {}).get("resample_blend", 0.0) for n in names) * 1.0e6
    ocean, cont = blend(("sed_ov", "sed_oh")), blend(("sed_c",))
    sub = g.get("sed_subducted", 0.0)
    return {
        "dropped": g.get("dropped", 0.0) / years,
        "last_resort_c": g.get("last_resort_c", 0.0) / years,
        "last_resort_o": g.get("last_resort_o", 0.0) / years,
        "last_resort_sed": g.get("last_resort_sed", 0.0) / years,
        "last_resort_n": g.get("last_resort_n", 0.0) / years,
        "last_resort_rise": g.get("last_resort_rise", 0.0),
        "mafic_dropped": g.get("mafic_dropped", 0.0) / years,
        "accreted": g.get("accreted", 0.0) / years,
        "accreted_visible": g.get("accreted_visible", 0.0) / years,
        "sed_subducted": sub / years,
        "residual": (ocean + cont + sub) / years,
        "residual_ocean": (ocean + sub) / years,
        "residual_cont": cont / years,
    }


def main(argv):
    arms = []
    for arg in argv:
        rows = []
        for r in probe.runs_of(arg):
            try:
                row = run_row(r)
            except (FileNotFoundError, KeyError, IndexError, ValueError):
                row = None
            if row:
                rows.append((r.name, row))
        arms.append((arg, rows))
    keys = list(arms[0][1][0][1].keys()) if arms and arms[0][1] else []
    for arg, rows in arms:
        print(f"# {arg}: {len(rows)} runs, km^3 a year over the run")
        print("  " + " ".join(f"{k:>16s}" for k in ["run"] + keys))
        for name, row in rows:
            print("  " + f"{name:>16s} " + " ".join(f"{row[k]:16.4f}" for k in keys))
    print("# medians (Mann-Whitney p against the first)")
    base = arms[0][1]
    for arg, rows in arms:
        cells = []
        for k in keys:
            xs = [r[k] for _, r in rows]
            m = statistics.median(xs) if xs else float("nan")
            if rows is base or not base:
                cells.append(f"{k} {m:.4f}")
            else:
                p = mannwhitneyu([r[k] for _, r in base], xs, alternative="two-sided").pvalue
                cells.append(f"{k} {m:.4f} ({p:.2f})")
        print(f"  {Path(arg.split('@')[0]).parent.parent.name}: " + "  ".join(cells))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
