#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The erosion recalibration of 2026-10-18: the summary of force-balance-ensemble.sh runs.

    erosion-recal.py <run folder> <length Myr> "<seeds>" "<starts>" <label> [<label> ...]

Per label (mean [min, max] over its runs; CO2 as a geometric mean): land share, land above 2 km
(share of the land), continental crust, surface temperature, CO2; the land's erosion by local
relief (`erosion_bands` line: the relief of a cell and its four edge neighbours, Montgomery &
Brandon's 10-km circle at 4.8-km cells) pooled over the runs with each run's land weighted
equally, against Montgomery & Brandon 2002 (ledger ERO-172): the power law E = 1.4e-6 Rz^1.8
(mm/yr, m), their eq. 2 E = 0.01 + 2.5e-4 Rz / (1 - (Rz/1500)^2) and the quiet-land line
E = 0.2 Rz (mm/yr, km); the share of land with relief above 1 and 1.5 km (Earth at most 0.05
and 0.02); the high ground's flat and steep wear (Tibet's interior under 10 m/Myr, ERO-174);
the carbon series of the first run (`carbon_series` line); the structure stage's arc rule
(DEF-026).
"""
import math
import re
import sys

RELIEF_LO = [0, 100, 250, 500, 1000, 1500, 2000, 3000]
RELIEF_MID = [50, 175, 375, 750, 1250, 1750, 2500, 3500]


def mb_power(rz):
    return 1.4e-6 * rz ** 1.8 * 1000.0  # m/Myr


def mb_eq2(rz):
    if rz >= 1500.0:
        return float("nan")
    return (0.01 + 2.5e-4 * rz / (1.0 - (rz / 1500.0) ** 2)) * 1000.0


def mb_linear(rz):
    return 0.2 * rz / 1000.0 * 1000.0


def num(line, key):
    m = re.search(r"(?:^|\s)" + re.escape(key) + r"=([-0-9.eE+]+|nan|NaN|inf)", line)
    return float(m.group(1)) if m else None


def parse(path):
    text = open(path).read()
    lines = text.splitlines()
    out = {}

    def line_with(tag):
        return next((ln for ln in lines if tag in ln), "")

    hy = line_with("hypsometry ")
    for k in ["land_frac_area", "land_above_2km", "continental_frac", "land_z_p50_m"]:
        out[k] = num(hy, k)
    cl = line_with("climate_final")
    for k in ["t_surface_c", "co2_ppm", "t_land_c"]:
        out[k] = num(cl, k)
    eb = line_with("erosion_bands ")
    if eb:
        bands = {}
        for name, vals in re.findall(r"(\w+):([-0-9./naN]+(?:/[-0-9.naN]+)*)", eb.split("erosion_bands", 1)[1]):
            v = [float(x) for x in vals.split("/")]
            bands[name] = v  # share, net, spl, threshold, diffusion, glacial
        out["bands"] = bands
    cs = line_with("carbon_series ")
    if cs:
        out["carbon"] = cs.split("carbon_series", 1)[1].strip()
    st = line_with("structure_final")
    for k in ["arc_cell_steps", "arc_mean_lift", "arc_lifted_share", "arc_mean_strain"]:
        out[k] = num(st, k)
    m = re.search(r"formed_by_arc=(\d+)/(\d+)", st)
    if m:
        out["arc_folds"], out["arc_thrusts"] = int(m.group(1)), int(m.group(2))
    m = re.search(r"formed_by_collision=(\d+)/(\d+)", st)
    if m:
        out["coll_folds"], out["coll_thrusts"] = int(m.group(1)), int(m.group(2))
    return out


def stat(xs, geo=False):
    xs = [x for x in xs if x is not None and not math.isnan(x)]
    if not xs:
        return "-"
    if geo:
        g = math.exp(sum(math.log(max(x, 1e-9)) for x in xs) / len(xs))
        return f"{g:.0f} [{min(xs):.0f}, {max(xs):.0f}]"
    mean = sum(xs) / len(xs)
    if abs(mean) >= 100:
        return f"{mean:.0f} [{min(xs):.0f}, {max(xs):.0f}]"
    return f"{mean:.3f} [{min(xs):.3f}, {max(xs):.3f}]"


def pooled(runs, name):
    rows = [r["bands"][name] for r in runs if "bands" in r and name in r["bands"]]
    w = sum(x[0] for x in rows)
    if w <= 0:
        return None
    return [w / len(rows)] + [sum(x[0] * x[i] for x in rows if not math.isnan(x[i])) / w for i in range(1, 6)]


def main():
    folder, length, seeds, starts = sys.argv[1], sys.argv[2], sys.argv[3].split(), sys.argv[4].split()
    for label in sys.argv[5:]:
        runs = []
        for start in starts:
            for seed in seeds:
                path = f"{folder}/{label}-{start}-{seed}-{length}.txt"
                try:
                    runs.append(parse(path))
                except FileNotFoundError:
                    print(f"{label} {start} {seed}: missing")
        if not runs:
            continue
        print(f"== {label} {length} Myr, {len(runs)} runs")
        print("  land " + stat([r["land_frac_area"] for r in runs])
              + "  >2km " + stat([r["land_above_2km"] for r in runs])
              + "  crust " + stat([r["continental_frac"] for r in runs])
              + "  T " + stat([r["t_surface_c"] for r in runs])
              + "  CO2 " + stat([r["co2_ppm"] for r in runs], geo=True))
        print("  relief class: land share | net, stream power (S_c cut), diffusion, glacial m/Myr | M&B power, eq.2, 0.2Rz")
        hi1 = hi15 = 0.0
        for lo, mid in zip(RELIEF_LO, RELIEF_MID):
            p = pooled(runs, f"rz{lo}")
            if p is None:
                continue
            if lo >= 1000:
                hi1 += p[0]
            if lo >= 1500:
                hi15 += p[0]
            print(f"    rz {lo:>4}+  {p[0]:.4f} | {p[1]:6.1f} {p[2]:6.1f} ({p[3]:5.1f}) {p[4]:5.1f} {p[5]:5.1f}"
                  f" | {mb_power(mid):6.0f} {mb_eq2(mid):6.0f} {mb_linear(mid):6.0f}")
        print(f"  land with relief above 1 km {hi1:.4f} (Earth <= 0.05), above 1.5 km {hi15:.4f} (Earth <= 0.02)")
        for name in ["z0", "z1000", "z2000", "z3000", "z4000", "flat", "steep"]:
            p = pooled(runs, name)
            if p is not None:
                print(f"    {name:>6}  {p[0]:.4f} | net {p[1]:6.1f}  spl {p[2]:6.1f} (S_c {p[3]:5.1f})  diff {p[4]:5.1f}  glac {p[5]:5.1f}")
        tot = pooled(runs, "z0")
        if tot is not None:
            net = sum((pooled(runs, f"z{z}") or [0, 0])[0] * (pooled(runs, f"z{z}") or [0, 0])[1]
                      for z in [0, 1000, 2000, 3000, 4000])
            share = sum((pooled(runs, f"z{z}") or [0])[0] for z in [0, 1000, 2000, 3000, 4000])
            print(f"  all land: net {net / max(share, 1e-9):.1f} m/Myr")
        if any(r.get("arc_cell_steps") is not None for r in runs):
            print("  arc rule (DEF-026): cell-steps " + stat([r.get("arc_cell_steps") for r in runs])
                  + "  mean lift " + stat([r.get("arc_mean_lift") for r in runs])
                  + "  at full lift " + stat([r.get("arc_lifted_share") for r in runs])
                  + "  mean strain " + stat([r.get("arc_mean_strain") for r in runs])
                  + "  arc folds/thrusts " + stat([r.get("arc_folds") for r in runs]) + " / "
                  + stat([r.get("arc_thrusts") for r in runs])
                  + "  collision folds " + stat([r.get("coll_folds") for r in runs]))
        if len(sys.argv) > 0 and runs[0].get("carbon"):
            print("  carbon series (first run; t:co2/T/T_land/D/W/W_land/W_kin/supply/kernel/kernel_sea_t/k_high/land_high/land/runoff/ridge/subd):")
            for part in runs[0]["carbon"].split():
                print("    " + part)


if __name__ == "__main__":
    main()
