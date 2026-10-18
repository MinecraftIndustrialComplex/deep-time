#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The continental crust budget of runs against Stern & Scholl 2010 (plate task 8).

  python3 tools/review/crust-budget.py <length Myr> <summary.txt> ...

Reads the plates line of each run summary (`deeptime run`'s `collapse_km3=... continental_volume_km3=`)
and its `hypsometry` and `crust_bins` lines, and prints per run and as the mean [lowest, highest]
over the runs:
  land, >2km        land share of the planet and land above 2 km of the land (Earth 0.289, 0.102)
  crust, dry        continental crust's share of the planet and the dry share of it
  vol               continental crust at the end, 10^9 km^3
  z40-55            median elevation of the 40-55 km crust (`crust_bins`)
and the crust budget in km^3 a year over the whole run (Earth-normalized; Stern & Scholl 2010,
Table 1: continental arcs about 1.0, oceanic arcs 1.5, losses at collision zones about 0.7):
  arc_c, arc_o      crust added at continental and island arcs (arc magma, cones counted inside)
  cord              crust the cordillera's shortening moved into the range (moved, not added)
  found             crust foundered beyond the thickest column (lost)
  accr, accr_vis    a resampling's accretion: all of it, and of it visible crust (moved, not added)
  rise              surface the accretion raised, 10^6 km^3 (not a crust volume)
  hot               the hotspots' own basalt; cone_f, cone_m the volcano stage's cones
and per km of arc and Myr (Stern & Scholl: continental arcs 35, island arcs about 100):
  c/km, o/km        arc crust over the length of oceanic-slab front times the time, by side
"""
import re
import sys

NUM = r"(-?[0-9.]+(?:e[+-]?[0-9]+)?)"


def kv(line):
    return {k: float(v) for k, v in re.findall(r"([A-Za-z0-9_]+)=" + NUM + r"(?=\s|$)", line)}


def read(path):
    plates, hyps, z4055 = None, None, None
    with open(path) as f:
        for line in f:
            if "collapse_km3=" in line and "continental_volume_km3=" in line:
                plates = kv(line)
            elif " hypsometry " in line and "land_frac_area=" in line:
                hyps = kv(line)
            elif line.startswith("crust_bins"):
                m = re.search(r"\[40-55km [^\]]*z=(-?\d+)", line)
                if m:
                    z4055 = float(m.group(1))
    return plates, hyps, z4055


def main():
    length = float(sys.argv[1])
    years = length * 1.0e6
    cols = [
        ("land", lambda p, h, z: h.get("land_frac_area"), "{:.3f}"),
        (">2km", lambda p, h, z: h.get("land_above_2km"), "{:.3f}"),
        ("crust", lambda p, h, z: h.get("continental_frac"), "{:.3f}"),
        ("dry", lambda p, h, z: h.get("continental_dry"), "{:.2f}"),
        ("vol", lambda p, h, z: p.get("continental_volume_km3", 0) / 1e9, "{:.2f}"),
        ("z40-55", lambda p, h, z: z, "{:.0f}"),
        ("arc_c", lambda p, h, z: p.get("arc_continental_km3", 0) / years, "{:.2f}"),
        ("arc_o", lambda p, h, z: p.get("arc_oceanic_km3", 0) / years, "{:.2f}"),
        ("cord", lambda p, h, z: p.get("cordillera_shortened_km3", 0) / years, "{:.2f}"),
        ("found", lambda p, h, z: p.get("foundered_km3", 0) / years, "{:.2f}"),
        ("accr", lambda p, h, z: p.get("resample_accreted_km3", 0) / years, "{:.1f}"),
        ("accr_vis", lambda p, h, z: p.get("resample_accreted_visible_km3", 0) / years, "{:.1f}"),
        ("rise", lambda p, h, z: p.get("resample_accreted_rise_km3", 0) / 1e6, "{:.1f}"),
        ("hot", lambda p, h, z: p.get("hotspot_magma_km3", 0) / years, "{:.3f}"),
        ("cone_f", lambda p, h, z: p.get("cone_felsic_km3", 0) / years, "{:.3f}"),
        ("cone_m", lambda p, h, z: p.get("cone_mafic_km3", 0) / years, "{:.3f}"),
        ("c/km", lambda p, h, z: per_km(p, "arc_continental_km3", "arc_front_continental_km_myr"), "{:.1f}"),
        ("o/km", lambda p, h, z: per_km(p, "arc_oceanic_km3", "arc_front_oceanic_km_myr"), "{:.1f}"),
    ]
    rows = []
    for path in sys.argv[2:]:
        p, h, z = read(path)
        if p is None or h is None:
            print(f"{path}: no plates or hypsometry line", file=sys.stderr)
            continue
        vals = [fn(p, h, z) for _, fn, _ in cols]
        rows.append(vals)
        name = path.rsplit("/", 1)[-1]
        print(name + "  " + "  ".join(f"{c}=" + fmt(v, f) for (c, _, f), v in zip(cols, vals)))
    if rows:
        print("mean [lowest, highest] over %d runs:" % len(rows))
        for k, (c, _, f) in enumerate(cols):
            xs = [r[k] for r in rows if r[k] is not None]
            if not xs:
                print(f"  {c:9s} n/a")
                continue
            print(f"  {c:9s} {fmt(sum(xs) / len(xs), f)} [{fmt(min(xs), f)}, {fmt(max(xs), f)}]")


def per_km(p, km3, front):
    f = p.get(front, 0.0)
    return p.get(km3, 0.0) / f if f > 0 else None


def fmt(v, f):
    return "n/a" if v is None else f.format(v)


if __name__ == "__main__":
    main()
