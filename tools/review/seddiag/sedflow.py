#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The flows into and out of the sea's sediment over a run, km3 a year: sedflow.py <arm folder> [length Myr, 1000]

The sediment-excess diagnosis (2026-10-25). Reads each run's summary.txt (the crust budget's last `cb_t`, `cb_parts`
and `cb_globals` lines, cumulative over the run, `DEEPTIME_CRUST_BUDGET=1`; and `surface_final`) and
fields/globals.json. Everything is Earth-normalised km3 a year over the run's length:
  - `cb_parts` stocks are in 1e6 km3 (loss-budget.py): `felsic` is pools 0 to 4 everywhere (sediment, sedimentary,
    metamorphic, felsic plutonic and volcanic), `felsic_ocean` the same on oceanic-labelled samples, by the part of
    the step that moved it;
  - `surface_final` totals are in the surface stage's block metric (surface/mod.rs Totals: cell area (r x 300 m)^2);
    the L1 raster is Lambert equal-area over the whole planet (1024 x 328 cells of 4.8 km on the quick preset,
    7.74e6 km2 against the 16,384-block sphere's C^2/pi = 7.69e6 km2), so x 5.1007e8 / (C x 0.3 km)^2 x pi gives
    Earth-normalised km3;
  - `climate/dissolved_mol` (globals) is the last climate step's dissolved Ca + Mg (ERO-205), per 2-Myr step.
Supply: the plates' pelagic rain (`felsic` part plate_surface, PLT-103) and the surface stage's net material onto
oceanic samples (`felsic_ocean` part erosion: deposits less what it stripped there), with the surface stage's own
gross totals beside. Removal: sediment subduction (`sed_subducted`, PLT-440), what resamplings accreted (all
conserved pools: onto continental tops `accreted_continent`, the rest onto oceanic tops), the forearcs' erosion
(`forearc_o`, `forearc_c`), collisions (crossed - kept, all crust), and the `felsic_ocean` parts that took felsic rock
off the ocean floor. Medians [lowest, highest] over the runs."""
import json
import math
import re
import statistics
import sys
from pathlib import Path

NUM = r"(-?[0-9.]+(?:e[-+]?[0-9]+)?)"
EARTH_KM2 = 5.1007e8
C_BLOCKS = 16384.0
M_PER_BLOCK = 300.0
BLOCK_TO_EARTH = EARTH_KM2 / ((C_BLOCKS * M_PER_BLOCK / 1e3) ** 2 / math.pi)
KG_PER_MOL = 0.26  # ERO-206
RHO_ROCK = 2.7  # t/m3


def last(lines, tag):
    for line in reversed(lines):
        if line.startswith(tag + " "):
            return line
    return None


def kv(text):
    return {k: float(v) for k, v in re.findall(r"([A-Za-z0-9_/]+)=" + NUM + r"(?=[\s\]]|$)", text)}


def one(run, length):
    lines = (run / "summary.txt").read_text().splitlines()
    yrs = length * 1e6
    parts = {name: {p: float(x) for p, x in re.findall(r"(\w+):" + NUM, inner)}
             for name, inner in re.findall(r"(\w+)=\[([^\]]*)\]", last(lines, "cb_parts"))}
    g = kv(last(lines, "cb_globals"))
    t = kv(last(lines, "cb_t"))
    sf = next((kv(l[l.index("surface_final="):]) for l in lines if "surface_final=" in l), {})
    glob = json.loads((run / "fields" / "globals.json").read_text())["globals"]
    p = lambda stock, name: parts.get(stock, {}).get(name, 0.0) * 1e6 / yrs
    s = lambda name: sf.get(name, float("nan")) * BLOCK_TO_EARTH / yrs
    out = {
        "pelagic_rain": p("felsic", "plate_surface"),
        "surface_net_ocean": p("felsic_ocean", "erosion"),
        "surface_net_all": p("felsic", "erosion"),
        "arcs_ocean": p("felsic_ocean", "arcs"),
        "eroded": s("eroded_km3"), "marine": s("marine_km3"), "to_sea": s("to_sea_km3"),
        "carbonate": s("carbonate_km3"), "chalk": s("pelagic_km3"), "periplatform": s("periplatform_km3"),
        "evaporite": s("evaporite_km3"), "fluvial": s("fluvial_km3"), "lake": s("lake_km3"),
        "surface_sediment_stock": sf.get("sediment_km3", float("nan")) * BLOCK_TO_EARTH,
        "dissolved_Gt": glob.get("climate/dissolved_mol", 0.0) / 2e6 * KG_PER_MOL / 1e12,
        "sed_subducted": g.get("sed_subducted", 0.0) / yrs,
        "accreted": g.get("accreted", 0.0) / yrs,
        "accreted_continent": g.get("accreted_continent", 0.0) / yrs,
        "accreted_ocean": (g.get("accreted", 0.0) - g.get("accreted_continent", 0.0)) / yrs,
        "stacked": g.get("stacked", 0.0) / yrs,
        "slab": g.get("slab", 0.0) / yrs,
        "forearc_o": g.get("forearc_o", 0.0) / yrs, "forearc_c": g.get("forearc_c", 0.0) / yrs,
        "collision_lost": (g.get("crossed", 0.0) - g.get("kept", 0.0)) / yrs,
        "felsic_ocean_stock": t.get("felsic_ocean_km3", float("nan")),
        "felsic_stock": t.get("felsic_km3", float("nan")),
        "ocean": 1.0 - t.get("cont", float("nan")),
    }
    out["dissolved_km3"] = out["dissolved_Gt"] / RHO_ROCK
    for k, v in parts.get("felsic_ocean", {}).items():
        out[f"fo:{k}"] = v * 1e6 / yrs
    # The felsic-on-ocean stock over time (cb_t lines every 50 Myr).
    series = {}
    for line in lines:
        if line.startswith("cb_t "):
            d = kv(line)
            series[d["t"]] = d.get("felsic_ocean_km3", float("nan"))
    out["series"] = series
    return out


def main():
    folder = Path(sys.argv[1])
    length = float(sys.argv[2]) if len(sys.argv) > 2 else 1000.0
    runs = sorted(q for q in folder.iterdir() if (q / "summary.txt").exists() and (q / "fields" / "globals.json").exists())
    rows = [(r.name, one(r, length)) for r in runs]
    if not rows:
        print("no runs")
        return
    keys = [k for k in rows[0][1] if k != "series"]
    allk = sorted({k for _, r in rows for k in r if k != "series"})
    def med(k):
        xs = [r[k] for _, r in rows if k in r and not math.isnan(r[k])]
        return (statistics.median(xs), min(xs), max(xs)) if xs else (float("nan"),) * 3
    f = lambda k: "{:.3g} [{:.3g}, {:.3g}]".format(*med(k))
    print(f"{len(rows)} runs of {folder}; km3 a year over {length:.0f} Myr, Earth-normalised (block x {BLOCK_TO_EARTH:.2f})")
    print("supply")
    print(f"  plates' pelagic rain on the visible ocean floor (PLT-103, 5 m/Myr)  {f('pelagic_rain')}")
    print(f"  surface stage's net material onto oceanic samples                  {f('surface_net_ocean')}")
    print(f"  surface stage's net material, all samples (production less loss)   {f('surface_net_all')}")
    print(f"  island-arc felsic rock onto the ocean floor (not sediment)         {f('arcs_ocean')}")
    print("  surface stage, gross: eroded " + f("eroded") + "; marine deposits " + f("marine") + "; to the sea " +
          f("to_sea") + "; fluvial " + f("fluvial") + "; lakes " + f("lake"))
    print("    carbonate " + f("carbonate") + "; pelagic chalk " + f("chalk") + "; periplatform " + f("periplatform") +
          "; evaporite " + f("evaporite") + "; standing surface sediment km3 " + f("surface_sediment_stock"))
    print(f"  dissolved load (ERO-205, last step), Gt a year {f('dissolved_Gt')}, km3 of rock {f('dissolved_km3')}: leaves the budget")
    print("removal")
    print(f"  sediment subduction (PLT-440's 0.12 at resamplings)               {f('sed_subducted')}")
    print(f"  accreted at resamplings, all conserved pools                      {f('accreted')}")
    print(f"    onto continental tops {f('accreted_continent')}; onto oceanic tops {f('accreted_ocean')}; stacked {f('stacked')}")
    print(f"  slab (one-way held crust's conserved pools down)                  {f('slab')}")
    print(f"  forearc erosion: oceanic {f('forearc_o')}, continental {f('forearc_c')}")
    print(f"  collisions (crossed - kept, all crust)                            {f('collision_lost')}")
    print("felsic rock onto (+) or off (-) the ocean floor by part (pools 0 to 4 on oceanic samples):")
    fo = [k for k in allk if k.startswith("fo:")]
    print("  " + "; ".join(f"{k[3:]} {med(k)[0]:.3f}" for k in fo if abs(med(k)[0]) > 0.005))
    print(f"  felsic rock on the ocean floor at the end, km3 {f('felsic_ocean_stock')}; ocean share {f('ocean')}")
    ts = sorted(rows[0][1]["series"])
    print("  felsic on the ocean floor over time, km3 (median): " + ", ".join(
        f"{t:.0f} {statistics.median(r['series'].get(t, float('nan')) for _, r in rows):.3g}" for t in ts[::2]))


if __name__ == "__main__":
    main()
