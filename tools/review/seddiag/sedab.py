#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The sea's sediment, base against arm, run by run: sedab.py <base folder>[@N] <arm folder>[@N] [map, t0000.bin]

sedstock.py's stock and sedflow.py's flows for each run of two arm folders (pool run folders, <start>-<seed>), the
medians of each side, the arm's change and the two-sided Mann-Whitney p (exact for small samples) of the runs. `@N`
keeps seeds up to N. Each side's pelagic rain allowance for sedstock's pelagic/terrigenous split is set per side:
BASE_PELAGIC (default 5.0, PLT-103's rate, for runs of code before the pelagic lane) and ARM_PELAGIC (default 0.0:
since the pelagic lane the plates lay no rain and the surface stage's rain is part of its deposits). The in-place
piles use sedstock's PHI0 and LAM_KM (env), plus Kominz et al. 2011's ocean-floor average (0.66, 1.333 km) for the
totals.

"By share": the excess over GlobSed (Straume et al. 2019) is ln(model / GlobSed) on the ocean floor (GlobSed's two
ocean-floor regions, 1.89e8 km3, against the model's wet oceanic samples) and on everything below the sea (3.37e8 km3),
as the sediment-excess diagnosis measured it; the arm's share of that excess removed is ln(base / arm) / ln(base /
GlobSed), medians. Needs numpy and scipy."""
import math
import os
import statistics
import sys
from pathlib import Path

from scipy.stats import mannwhitneyu

import sedflow
import sedstock

GLOBSED_ALL = 3.37e8
GLOBSED_OCEAN = 7.6e7 + 1.13e8  # within and beyond 200 km of the COB (Table 3 and section 2.5)
KOMINZ = (0.66, 1.333)

STOCK = [("sea_sed", "the sea's sediment, solid, km3"), ("sea_sed_oc", "  on oceanic crust"),
         ("sea_sed_cont", "  on continental crust"), ("pel", "  pelagic allowance (sedstock's split)"),
         ("terr_oc", "  terrigenous on oceanic"), ("sea_sed_compacted", "in place (sedstock's PHI0, LAM_KM)"),
         ("kominz_all", "in place (Kominz 0.66, 1.333 km)"), ("kominz_oc", "  on oceanic crust, in place (Kominz)"),
         ("Gmargin", "GlobSed margin region, km3"), ("Gmargin:km2", "  its area, km2"), ("Gmargin:pile", "  its mean pile, m solid"),
         ("Gnear200", "ocean floor within 200 km, km3"), ("Gnear200:km2", "  its area, km2"),
         ("Gnear200:pile", "  its mean pile, m solid"), ("Gfar200", "ocean floor beyond 200 km, km3"), ("Gfar200:km2", "  its area, km2"), ("Gfar200:pile", "  mean pile beyond 200 km, m solid"),
         ("Gfar200:inplace", "  the same in place, m"), ("Gyoung82:pile", "young floor (eq. 2a's domain), m solid"),
         ("Gyoung82:inplace", "  the same in place, m"), ("Gyoung82:eq2a", "  eq. 2a at those samples, m"),
         ("young_ratio", "  in place over eq. 2a"),
         ("A0:pile", "pile on floor 0-20 Myr, m"), ("A20:pile", "  20-60 Myr"), ("A60:pile", "  60-120 Myr"),
         ("A120:pile", "  120-180 Myr"), ("A180:pile", "  180-300 Myr"), ("A300:pile", "  over 300 Myr"),
         ("Mpassive", "passive-margin floor, km3"), ("Mactive", "active-margin floor, km3"), ("Mopen", "open ocean, km3"),
         ("age_mean", "floor age mean, Myr"), ("oc", "oceanic share of the planet"), ("sea_m", "sea over the datum, m"),
         ("meta_wet_oc", "metamorphic on wet oceanic, km3"), ("x_ocean", "ocean floor over GlobSed (solid)"),
         ("x_all", "everything below the sea over GlobSed (solid)"), ("x_ocean_inplace", "ocean floor over GlobSed (Kominz in place)")]
FLOW = [("pelagic_rain", "plates' pelagic rain, km3/yr"), ("surface_net_ocean", "surface net onto oceanic, km3/yr"),
        ("surface_net_all", "surface net, all samples, km3/yr"), ("eroded", "eroded, km3/yr"),
        ("marine", "marine deposits, km3/yr"), ("to_sea", "to the sea, km3/yr"), ("carbonate", "carbonate, km3/yr"),
        ("chalk", "pelagic chalk, km3/yr"), ("dissolved_Gt", "dissolved load, last step, Gt/yr"),
        ("sed_subducted", "sediment subduction, km3/yr"), ("accreted", "accreted at resamplings, km3/yr"),
        ("accreted_ocean", "  onto oceanic tops"), ("forearc_o", "forearc erosion, oceanic"),
        ("collision_lost", "collisions, crossed - kept"), ("felsic_ocean_stock", "felsic rock on the ocean floor, km3"),
        ("ocean", "ocean share (crust budget)")]


def runs_of(spec):
    folder, _, top = spec.partition("@")
    runs = sorted(q for q in Path(folder).iterdir() if (q / "summary.txt").exists())
    if top:
        runs = [r for r in runs if int(r.name.rsplit("-", 1)[1]) <= int(top)]
    return runs


def measure(run, name, pelagic):
    out = {}
    if (run / "maps" / "pools" / name).exists() and (run / "maps" / "age" / name).exists():
        sedstock.PELAGIC = pelagic
        out.update(sedstock.one(run, name))
        phi0, lam = sedstock.PHI0, sedstock.LAM_KM
        sedstock.PHI0, sedstock.LAM_KM = KOMINZ
        k = sedstock.one(run, name)
        sedstock.PHI0, sedstock.LAM_KM = phi0, lam
        out["kominz_all"] = k["sea_sed_compacted"]
        # Oceanic part in place: the total in place less the continental part in place (one() keeps no split).
        out["kominz_oc"] = k["sea_sed_compacted"] - k["Gmargin:inplace"] * k["Gmargin:km2"] / 1e3 \
            if math.isfinite(k["Gmargin:inplace"]) else k["sea_sed_compacted"]
        out["young_ratio"] = out["Gyoung82:inplace"] / out["Gyoung82:eq2a"]
        out["x_ocean"] = out["sea_sed_oc"] / GLOBSED_OCEAN
        out["x_all"] = out["sea_sed"] / GLOBSED_ALL
        out["x_ocean_inplace"] = out["kominz_oc"] / GLOBSED_OCEAN
    if (run / "fields" / "globals.json").exists():
        f = sedflow.one(run, 1000.0)
        f.pop("series", None)
        out.update(f)
    return out


def med(xs):
    xs = [x for x in xs if isinstance(x, (int, float)) and math.isfinite(x)]
    return statistics.median(xs) if xs else float("nan")


def main():
    base, arm = runs_of(sys.argv[1]), runs_of(sys.argv[2])
    name = sys.argv[3] if len(sys.argv) > 3 else "t0000.bin"
    bp, ap = float(os.environ.get("BASE_PELAGIC", "5.0")), float(os.environ.get("ARM_PELAGIC", "0.0"))
    B = {r.name: measure(r, name, bp) for r in base}
    A = {r.name: measure(r, name, ap) for r in arm}
    print(f"base {sys.argv[1]}: {len(B)} runs ({', '.join(B)}); arm {sys.argv[2]}: {len(A)} runs ({', '.join(A)})")
    print(f"map {name}; base pelagic allowance {bp} m/Myr, arm {ap}; medians, arm - base, Mann-Whitney p (two-sided)")
    for title, rows in (("stock", STOCK), ("flows", FLOW)):
        print(title)
        for k, label in rows:
            b = [v[k] for v in B.values() if k in v and math.isfinite(v[k])]
            a = [v[k] for v in A.values() if k in v and math.isfinite(v[k])]
            if not b or not a:
                continue
            p = mannwhitneyu(a, b, alternative="two-sided").pvalue if len(set(a + b)) > 1 else 1.0
            mb, ma = med(b), med(a)
            print(f"  {label:<48} base {mb:>11.4g}  arm {ma:>11.4g}  change {ma - mb:>+11.4g}  p {p:.3g}")
    # The excess over GlobSed and the share of it the arm removed (medians).
    print("the excess over GlobSed, ln(model / GlobSed), and the arm's share of it removed")
    for k, g, label in (("sea_sed_oc", GLOBSED_OCEAN, "ocean floor, solid"),
                        ("kominz_oc", GLOBSED_OCEAN, "ocean floor, in place (Kominz)"),
                        ("sea_sed", GLOBSED_ALL, "everything below the sea, solid"),
                        ("kominz_all", GLOBSED_ALL, "everything below the sea, in place (Kominz)")):
        mb = med([v[k] for v in B.values() if k in v])
        ma = med([v[k] for v in A.values() if k in v])
        xb, xa = math.log(mb / g), math.log(ma / g)
        share = (xb - xa) / xb if xb else float("nan")
        print(f"  {label:<44} base x{mb / g:.2f} (ln {xb:.3f})  arm x{ma / g:.2f} (ln {xa:.3f})  share removed {share:.3f}")
    print("runs (sea's sediment on oceanic crust, km3; base | arm)")
    for r in sorted(set(B) | set(A)):
        f = lambda d: f"{d[r]['sea_sed_oc']:.4g}" if r in d and "sea_sed_oc" in d[r] else "-"
        print(f"  {r:<26} {f(B):>10} | {f(A):>10}")


if __name__ == "__main__":
    main()
