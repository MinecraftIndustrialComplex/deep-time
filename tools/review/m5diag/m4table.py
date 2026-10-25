#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The tables of m4split.py's runs: m4table.py <m4split.jsonl> [start]

Parts of M4 are means over the runs (so they add up to the mean M4; the median M4 beside them);
every other figure is the median over the runs of each run's value. `start` keeps one start."""
import json
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
from m4split import AGE_EDGES, CLASSES, EARTH_BAND_P50, EARTH_BAND_SHARE, GROUPS  # noqa: E402


def mean(v):
    v = [x for x in v if x is not None]
    return float(np.mean(v)) if v else float("nan")


def p50(v):
    v = [x for x in v if x is not None]
    return float(np.median(v)) if v else float("nan")


def f(x, nd=3):
    return "n/a" if x is None or (isinstance(x, float) and np.isnan(x)) else f"{x:.{nd}f}"


def main():
    rows = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines() if line.strip()]
    if len(sys.argv) > 2:
        rows = [r for r in rows if r["start"] == sys.argv[2]]
    n = len(rows)
    print(f"{n} runs ({sys.argv[1]})")
    for k in ("M4", "cached_M4", "M4_plate", "M4_ice_kept_out", "hi_iced_share", "land_iced_share", "M4_airy",
              "M4_airy_hi_cells", "M4_earth_line", "M4_no_excess", "sea_m"):
        print(f"  {k:18s} p50 {f(p50([r.get(k) for r in rows]), 4)}  mean {f(mean([r.get(k) for r in rows]), 4)}")

    print("\nM4 by class (share of the land; mean over runs, p50 in brackets): all, not iced, iced; Airy >= 2 km,"
          " Airy < 2 km (of it not iced); class p50s of T km, and the means of the census height = Airy over sea"
          " + plate over Airy + census over plate, m; stamp age p50")
    hdr = (f"  {'class':17s} {'m4':>13s} {'noice':>6s} {'iced':>6s} {'airy>=2':>7s} {'airy<2':>6s} {'(noice)':>7s}"
           f" {'T':>5s} {'z':>5s} {'=airy':>6s} {'+exc':>5s} {'+srf':>5s} {'age':>5s} {'lat':>4s} {'land':>6s}")
    print(hdr)

    def line(name, keys):
        def s(field):
            return mean([sum(r["by_class"][k][field] for k in keys) for r in rows])
        m4s = [sum(r["by_class"][k]["m4"] for k in keys) for r in rows]

        def wavg(field):
            # Weighted by each class's high cells, then the median over runs.
            vals = []
            for r in rows:
                w = [r["by_class"][k]["m4"] for k in keys if r["by_class"][k].get(field) is not None]
                v = [r["by_class"][k][field] for k in keys if r["by_class"][k].get(field) is not None]
                if sum(w) > 0:
                    vals.append(float(np.average(v, weights=w)))
            return p50(vals)
        print(f"  {name:17s} {s('m4'):.4f} ({p50(m4s):.4f}) {s('m4_noice'):.4f} {s('m4_iced'):.4f}"
              f" {s('m4_airy_ge2km'):.4f}  {s('m4_airy_lt2km'):.4f}  {s('m4_airy_lt2km_noice'):.4f}"
              f" {wavg('T_km_p50'):5.1f} {wavg('z_mean'):5.0f} {wavg('airy_mean'):6.0f} {wavg('exc_mean'):5.0f}"
              f" {wavg('srf_mean'):5.0f} {f(wavg('age_p50'), 0):>5s} {f(wavg('lat_abs_p50'), 0):>4s} {s('land'):.4f}")
    for gname, keys in GROUPS.items():
        line(gname, keys)
        if len(keys) > 1:
            for k in keys:
                line("  " + k, [k])
    line("all", CLASSES)

    print("\nThe crust's thickness bands over the land: share of the land (Earth's, HYP-040), M4 from the band, rock"
          " height p50 m (ice-free; Earth's), Airy over sea p50, plate over Airy p50, untagged and collision shares")
    for b in rows[0]["bands"]:
        def v(field):
            return p50([r["bands"][b][field] for r in rows])
        ew = EARTH_BAND_SHARE.get(b)
        ez = EARTH_BAND_P50.get(b)
        print(f"  {b:>9s} land {v('land'):.4f} (Earth {f(ew, 3)})  m4 {mean([r['bands'][b]['m4'] for r in rows]):.4f}"
              f"  z {f(v('z_p50'), 0)} ({f(v('z_p50_icefree'), 0)}; Earth {f(ez, 0)})  airy {f(v('airy_p50'), 0)}"
              f"  exc {f(v('exc_p50'), 0)}  untagged {v('untagged'):.2f}  coll {v('coll'):.2f}")
    sl = [r["icefree_line"]["slope_m_per_km"] for r in rows if "icefree_line" in r]
    ic = [r["icefree_line"]["intercept_m"] for r in rows if "icefree_line" in r]
    print(f"  ice-free continental land: rock height = {p50(ic):.0f} m + {p50(sl):.0f} m/km x T"
          f" (Earth -2,559 + 81, HYP-040; the model's Airy line 142 m/km)")

    print("\nThe high ground's plate height against the Airy height averaged over a disc of R km (continental cells):"
          " plate - regional Airy p50 (|.| p50, p90), the high cells whose regional Airy is 2 km or more")
    for R in rows[0]["discs"]:
        def v(field):
            return p50([r["discs"][R][field] for r in rows])
        print(f"  R {R:>4s} km: {f(v('plate_minus_regional_airy_p50'), 0)} m ({f(v('abs_p50'), 0)}, {f(v('p90'), 0)})"
              f"  regional Airy >= 2 km {f(v('share_regional_airy_ge2km'), 3)}")

    mrows = [r for r in rows if "decay" in r]
    if mrows:
        print("\nAfter the stamp (crust-budget maps, continental samples, snapshots from -800 Myr, pooled per run,"
              " p50 over runs): samples, height over the sea p50 / p90, dry, above 2 km, T p50 / p90 km,"
              " Airy >= 2 km, plate over Airy p50")
        for gn in ("coll", "coll_arc_crust", "coll_no_arc_crust", "andean", "untagged_thick", "untagged_thin"):
            bins = sorted({b for r in mrows for b in r["decay"].get(gn, {})}, key=lambda s: float(s.split("-")[0]))
            print(f"  {gn}")
            for b in bins:
                rr = [r["decay"][gn][b] for r in mrows if b in r["decay"].get(gn, {})]

                def v(field):
                    return p50([x[field] for x in rr])
                print(f"    {b:>9s} n {sum(x['n'] for x in rr):8d}  z {v('zs_p50'):6.0f} / {v('zs_p90'):5.0f}"
                      f"  dry {v('dry'):.2f}  >2km {v('above_2km'):.3f}  T {v('T_km_p50'):5.1f} / {v('T_km_p90'):5.1f}"
                      f"  airy>=2km {v('airy_ge2km'):.3f}  exc {v('exc_p50'):5.0f}")
        print("\nThe high ground through time (maps, share of the land; p50 over runs): M4, collision (by stamp age),"
              " Andean, untagged thick / thin, oceanic, Airy >= 2 km")
        times = sorted({s["t"] for r in mrows for s in r["snaps"]})
        for t in times:
            ss = [s for r in mrows for s in r["snaps"] if s["t"] == t]

            def v(field):
                return p50([s.get(field, 0.0) for s in ss])
            ages = " ".join(f"{v(f'coll_{a:g}_{b:g}'):.3f}" for a, b in zip(AGE_EDGES[:-1], AGE_EDGES[1:]))
            print(f"  t {t:6.0f}: M4 {v('M4'):.3f}  coll {v('coll'):.3f} [{ages}]  andean {v('andean'):.3f}"
                  f"  untagged {v('untagged_thick'):.3f} / {v('untagged_thin'):.3f}  oceanic {v('oceanic'):.3f}"
                  f"  airy>=2km {v('airy_ge2km'):.3f}")
        print(f"  (collision age bins, Myr: {AGE_EDGES})")


if __name__ == "__main__":
    main()
