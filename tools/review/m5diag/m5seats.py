#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The sea, the floor's age and the continental area through the run: m5seats.py <name>=<arm folder> ...

For the drowning diagnosis (m5sea.py reads the final frame). From each run's rec.json (the plate check's record): at
t0 + 100, 300, 500, 700, 900 Myr (900 is the last step before steering's tail, `steering.tail_fraction` 0.1; STE-006
moves the sea there by at most 2 m/Myr to at most 200 m) and at the end: the sea over the datum, the continental and
land shares of the visible samples, the ocean floor's mean age and p50 (the `ages` record, 10-Myr bins, the last bin
holding everything older), its GDH1 depth (PLT-352: 2,600 + 365 sqrt(t) under 20 Myr, 5,651 - 2,473 exp(-0.0278 t)
after) averaged over the histogram, and the share of the floor older than 180 Myr. Medians over the runs."""
import json
import math
import statistics
import sys
from pathlib import Path

TIMES = (100.0, 300.0, 500.0, 700.0, 900.0, None)
BIN = 10.0


def gdh1(t):
    """GDH1's depth below the datum, m (PLT-352; Stein & Stein 1992)."""
    return 2600.0 + 365.0 * math.sqrt(t) if t < 20.0 else 5651.0 - 2473.0 * math.exp(-0.0278 * t)


def ages_at(rec, t):
    rows = rec["ages"]
    return min(rows, key=lambda r: abs(r["t"] - t))


def floor(row):
    h = row["hist"]
    n = max(sum(h), 1)
    mids = [(i + 0.5) * BIN for i in range(len(h))]
    return {"age_mean": sum(m * c for m, c in zip(mids, h)) / n, "age_p50": row["p50_myr"],
            "gdh1_mean": sum(gdh1(m) * c for m, c in zip(mids, h)) / n,
            "old180": sum(c for m, c in zip(mids, h) if m >= 180.0) / n}


def step_at(rec, t):
    return min(rec["steps"], key=lambda s: abs(s["t"] - t))


def main():
    for arg in sys.argv[1:]:
        name, folder = arg.split("=", 1)
        rows = {t: [] for t in TIMES}
        for run in sorted(p for p in Path(folder).iterdir() if (p / "rec.json").exists()):
            rec = json.loads((run / "rec.json").read_text())
            t0 = rec["t0_myr"]
            for t in TIMES:
                st = rec["steps"][-1] if t is None else step_at(rec, t0 + t)
                vis = max(st["visible"], 1)
                o = {"sea": st["sea_m"], "cont": st["continental"] / vis, "land": st["land"] / vis}
                o.update(floor(ages_at(rec, st["t"])))
                rows[t].append(o)
        print(f"{name} ({len(rows[TIMES[0]])} runs): time, sea m, cont, land, floor age mean / p50 Myr,"
              " GDH1 depth over the age histogram m, floor older than 180 Myr")
        for t in TIMES:
            r = rows[t]
            m = {k: statistics.median(x[k] for x in r) for k in r[0]}
            label = "end" if t is None else f"{t:.0f}"
            print(f"  {label:>4s}: sea {m['sea']:6.0f}  cont {m['cont']:.3f}  land {m['land']:.3f}"
                  f"  age {m['age_mean']:5.1f} / {m['age_p50']:5.1f}  GDH1 {m['gdh1_mean']:5.0f}  old180 {m['old180']:.3f}")


if __name__ == "__main__":
    main()
