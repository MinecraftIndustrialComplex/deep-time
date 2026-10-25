#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The M5 diagnosis's tables: m5table.py <fields.jsonl> <maps.jsonl> <earth.json>

Medians over the 24 runs and by start (Mann-Whitney p between the starts), Spearman of M5 against
the factors and the class parts over the runs, the time series of the maps, and Earth's control
beside each (earth.json: the census grid and Fibonacci points, with and without the two ice
sheets)."""
import json
import sys
from collections import defaultdict
from pathlib import Path

import numpy as np
from scipy.stats import mannwhitneyu, spearmanr

sys.path.insert(0, str(Path(__file__).resolve().parent))
from m5fields import CLASSES  # noqa: E402


def med(xs):
    xs = [x for x in xs if x is not None and np.isfinite(x)]
    return float(np.median(xs)) if xs else float("nan")


def iqr(xs):
    xs = [x for x in xs if x is not None and np.isfinite(x)]
    return (float(np.percentile(xs, 25)), float(np.percentile(xs, 75))) if xs else (float("nan"),) * 2


def row(name, rows, get, earth=None, fmt="{:.4f}"):
    allv = [get(r) for r in rows]
    a = [get(r) for r in rows if r["start"] == "supercontinent"]
    b = [get(r) for r in rows if r["start"] != "supercontinent"]
    try:
        p = mannwhitneyu(a, b).pvalue
    except ValueError:
        p = float("nan")
    lo, hi = iqr(allv)
    s = (f"{name:<38} all {fmt.format(med(allv))} [{fmt.format(lo)}, {fmt.format(hi)}]"
         f"  sc {fmt.format(med(a))}  scat {fmt.format(med(b))}  MW p {p:.3f}")
    if earth is not None:
        s += f"  | Earth {earth}"
    print(s)


def main():
    F = [json.loads(x) for x in Path(sys.argv[1]).read_text().splitlines() if x.strip()]
    M = [json.loads(x) for x in Path(sys.argv[2]).read_text().splitlines() if x.strip()]
    E = json.loads(Path(sys.argv[3]).read_text())
    eg, eg_all, ef = E["grid"]["no_sheets"], E["grid"]["all"], E[[k for k in E if k.startswith("fib") and k != "fib100000"][0]]["no_sheets"]
    e100 = E["fib100000"]["no_sheets"]
    print(f"runs {len(F)}; maps rows {len(M)}")
    print("Earth: grid = the census's Lambert grid (332,800 cells), fib = Fibonacci 332,800, f100k = Fibonacci 100,000;"
          " no_sheets = Antarctica and Greenland kept out of the shares")

    def ew(key, sub=None):
        out = []
        for nm, e in (("grid", eg), ("fib", ef), ("f100k", e100), ("grid_all", eg_all)):
            v = e[key] if sub is None else e[key][sub]
            out.append(f"{nm} {v:.4f}")
        return " ".join(out)

    print("\n== Headline (census grid)")
    row("M5", F, lambda r: r["M5"], ew("M5"))
    row("M4", F, lambda r: r["M4"], ew("M4"))
    row("cached M5 agrees", F, lambda r: abs(r["M5"] - r["cached_M5"]))
    row("M5/M4", F, lambda r: r["factors"]["ratio"], ew("factors", "ratio"), "{:.3f}")
    for k in ("f_near", "kappa", "s", "land_within_50", "land_within_100", "land_within_200"):
        row(k, F, lambda r, k=k: r["factors"][k], ew("factors", k))
    row("map align (kind agreement)", F, lambda r: r["map_align"]["agree_kind"])
    row("orogeny agree on hi (map vs field)", F, lambda r: r["oro_agree_hi"])

    print("\n== Q1: the qualified coast by the class of its nearest qualifying high ground (share of M5)")
    for k in CLASSES:
        row(f"by_nearest/M5 {k}", F, lambda r, k=k: r["m5_by_nearest"][k] / max(r["M5"], 1e-9))
    print("-- M5 left without the class (absolute)")
    for k in CLASSES:
        row(f"without {k}", F, lambda r, k=k: r["m5_without"][k])
    print("-- M4 by class (share of M4)")
    for k in CLASSES:
        row(f"m4_by/M4 {k}", F, lambda r, k=k: r["m4_by"][k] / max(r["M4"], 1e-9))
    print("-- the young collision part by stamp age (absolute, part of M5)")
    for k in ("-1e+09-2", "2-5", "5-10"):
        row(f"coll_young age {k}", F, lambda r, k=k: r["coll_young_by_age"][k])
    print("-- the parked untagged columns (u_Tc) on high ground")
    row("u_Tc hi island arc share", F, lambda r: r["u_Tc_hi_island_arc"])
    row("u_Tc hi regime back_arc", F, lambda r: r["u_Tc_hi_regime"]["4"])
    row("u_Tc hi regime craton", F, lambda r: r["u_Tc_hi_regime"]["8"])
    print("-- the coast by the nearest plate front (map flags)")
    for k in ("sub200", "coll200", "neither"):
        row(f"{k} coast share", F, lambda r, k=k: r["coast_by_front"][k]["coast_share"])
        row(f"{k} M5 inside", F, lambda r, k=k: r["coast_by_front"][k]["m5_in"])
        row(f"{k} part of M5 (abs)", F, lambda r, k=k: r["coast_by_front"][k]["m5_part"])
    print("-- the qualifying high point's own setting (fields' regime; 3 arc, 4 back_arc, 5 collision, 8 craton,"
          " 9 passive_margin)")
    for c in ("coll_young", "coll_old", "andean_young", "u_other"):
        for rc in ("2", "3", "4", "5", "6", "8", "9"):
            row(f"{c} hi regime {rc}", F, lambda r, c=c, rc=rc: r["hi_setting_by_class"].get(c, {}).get("regime", {})
                .get(rc, np.nan))
        row(f"{c} hi on active collision", F, lambda r, c=c: r["hi_setting_by_class"].get(c, {})
            .get("active_collision", np.nan))
    for rc in ("2", "3", "4", "5", "6", "8", "9"):
        row(f"all hi regime {rc}", F, lambda r, rc=rc: r["hi_regime_all"][rc])
    row("all hi on active collision", F, lambda r: r["hi_active_collision_all"])
    print("-- the coast by landmass size (Mkm2)")
    for k in ("0-1", "1-10", "10-30", "30-1e+09"):
        row(f"landmass {k} coast share", F, lambda r, k=k: r["coast_by_landmass_Mkm2"][k]["coast_share"])
        row(f"landmass {k} M5 inside", F, lambda r, k=k: r["coast_by_landmass_Mkm2"][k]["m5_in"])

    print("\n== Q2: distance from the qualified coast to its nearest qualifying high point (share of M5)")
    for k in ("self", "0-50", "50-100", "100-150", "150-200"):
        row(f"dist {k}", F, lambda r, k=k: r["dist_hist"][k], ew("dist_hist", k))
    print("-- by class (share of that class's part)")
    for c in ("coll_young", "andean_young", "coll_old"):
        for k in ("self", "0-50", "50-100", "100-150", "150-200"):
            row(f"{c} dist {k}", F, lambda r, k=k, c=c: r["dist_hist_by"][c][k])
    print("-- the qualifying high point's own distance to the sea")
    for k in ("-1-50", "50-100", "100-200", "200-400", "400-1e+09"):
        row(f"hi own sea {k}", F, lambda r, k=k: r["hi_own_sea_km_hist"][k], ew("hi_own_sea_km_hist", k))

    print("\n== Matched M4: M5 with the level at the quantile putting the target share of land above it")
    tg = [c[0] for c in F[0]["curve"][:-1]]
    for i, t in enumerate(tg):
        ec = [c for c in eg["curve"] if abs(c[0] - t) < 1e-9]
        efc = [c for c in ef["curve"] if abs(c[0] - t) < 1e-9]
        row(f"M5 at M4={t:g}", F, lambda r, i=i: r["curve"][i][2],
            f"grid {ec[0][2]:.4f} fib {efc[0][2]:.4f}" if ec else None)
        row(f"  level m at M4={t:g}", F, lambda r, i=i: r["curve"][i][1], f"grid {ec[0][1]:.0f}" if ec else None, "{:.0f}")
        row(f"  f_near at M4={t:g}", F, lambda r, i=i: r["curve"][i][3], f"grid {ec[0][3]:.4f}" if ec else None)
        row(f"  s at M4={t:g}", F, lambda r, i=i: r["curve"][i][4], f"grid {ec[0][4]:.4f}" if ec else None)
    row("fib M5 at M4=0.055", F, lambda r: r["checks"]["fib_curve"][1][2])
    print("-- Earth with the level lowered to put the model's share of land above it")
    for nm, e in (("grid no_sheets", eg), ("fib no_sheets", ef), ("grid all", eg_all)):
        print(f"   Earth {nm}: " + " ".join(f"{c[0]:g}:{c[2]:.4f}@{c[1]:.0f}m(f_near {c[3]:.3f}, s {c[4]:.3f})"
                                           for c in e["curve"]))
    row("levels 2500 m M5", F, lambda r: r["curve_levels"]["2500"])
    row("levels 3000 m M5", F, lambda r: r["curve_levels"]["3000"])
    print(f"   Earth grid no_sheets at 1500 m {eg['curve_levels']['1500']:.4f}, 1000 m {eg['curve_levels']['1000']:.4f}")

    print("\n== Q5: measure checks")
    C = lambda k: (lambda r: r["checks"][k])  # noqa: E731
    row("fib M5 (same count)", F, C("fib_M5"), ew("M5"))
    row("fib M5 / grid M5", F, lambda r: r["checks"]["fib_M5"] / r["M5"], f"{ef['M5'] / eg['M5']:.3f}")
    row("fib M4", F, C("fib_M4"))
    for k in ("f_near", "kappa", "s", "ratio"):
        row(f"fib {k}", F, lambda r, k=k: r["checks"]["fib_factors"][k], f"{ef['factors'][k]:.4f}")
    row("n_comp census", F, C("n_comp_census"), f"{eg['n_comp_census']} grid8 {eg['n_comp_grid8']}", "{:.0f}")
    row("n_comp grid8", F, C("n_comp_grid8"), None, "{:.0f}")
    row("grid8 M5", F, C("grid8_M5"), f"grid {eg['grid8_M5']:.4f}")
    for b in F[0]["checks"]["lat_bands"]:
        for k in ("land_share", "coast_per_land_grid", "coast_per_land_fib", "m5_grid", "m5_fib", "coast_share_grid"):
            ek = {"coast_per_land_grid": "coast_per_land", "m5_grid": "m5"}.get(k)
            row(f"lat {b} {k}", F, lambda r, b=b, k=k: r["checks"]["lat_bands"][b][k],
                f"{eg['lat_bands'][b][ek]:.4f} fib {ef['lat_bands'][b][ek]:.4f}" if ek else
                (f"{eg['lat_bands'][b]['land_share']:.4f}" if k == "land_share" else None))
    print(f"   Earth grid sea bodies: {json.dumps(E['grid'].get('sea_bodies'))[:400]}")
    for k in ("n_ge1", "n_lt1", "area_lt1_share_of_sea", "largest_share_of_sea"):
        row(f"sea bodies {k}", F, lambda r, k=k: r["checks"]["sea_bodies"][k])
    for lim in ("0.1", "1"):
        row(f"coast on body < {lim} Mkm2", F, C(f"coast_on_body_lt{lim}"))
        row(f"M5 inside coast on body < {lim}", F, C(f"m5_coast_on_body_lt{lim}"))
        row(f"M5 with bodies < {lim} as land", F, C(f"m5_bodies_lt{lim}_as_land"),
            f"grid {eg.get(f'm5_bodies_lt{lim}_as_land', float('nan')):.4f}")
        row(f"M4 with bodies < {lim} as land", F, C(f"m4_bodies_lt{lim}_as_land"))
    row("line crosses sea", F, C("line_crosses_sea"), f"grid {eg.get('line_crosses_sea', float('nan')):.4f}"
        f" fib {ef.get('line_crosses_sea', float('nan')):.4f}")
    row("M5 if crossing dropped", F, C("m5_if_crossing_dropped_upper"),
        f"grid {eg.get('m5_if_crossing_dropped_upper', float('nan')):.4f}")
    for k in ("fib", "fib_bodies", "fib_bodies_inland"):
        efm = eg.get("fixed_measure", {}).get(k, (np.nan, np.nan))
        efa = eg_all.get("fixed_measure", {}).get(k, (np.nan, np.nan))
        row(f"fixed measure {k} M5", F, lambda r, k=k: r["checks"]["fixed_measure"][k][0],
            f"grid no_sheets {efm[0]:.4f} all {efa[0]:.4f}")
        row(f"fixed measure {k} M4", F, lambda r, k=k: r["checks"]["fixed_measure"][k][1],
            f"grid no_sheets {efm[1]:.4f} all {efa[1]:.4f}")
    row("elev - surface p50 land", F, C("elev_minus_surface_p50_land"))
    row("hi iced share", F, C("hi_iced_share"))
    row("M4 rock + ice", F, C("m4_by_surface_with_ice"))
    for lim in ("1", "100"):
        row(f"M5 with ice >= {lim} m kept out", F, lambda r, lim=lim: r[f"m5_ice_lt{lim}_kept_out"])
        row(f"M4 with ice >= {lim} m kept out", F, lambda r, lim=lim: r[f"m4_ice_lt{lim}_kept_out"])
    row("seam equal cols", F, C("seam_equal_cols"))
    for c in F[0]["checks"]["seam_cols"]:
        row(f"seam col {c} coast/mean", F, lambda r, c=c: r["checks"]["seam_cols"][c]["coast_over_mean"])
        row(f"seam col {c} land/mean", F, lambda r, c=c: r["checks"]["seam_cols"][c]["land_over_mean"])
    row("seam land flips / mean flips", F, C("seam_land_flips_over_mean"))
    row("plate_z M5", F, C("plate_z_M5"))
    row("plate_z M4", F, C("plate_z_M4"))

    print("\n== Spearman over the 24 runs: M5 against")
    m5 = np.array([r["M5"] for r in F])
    cand = {"M4": lambda r: r["M4"], "f_near": lambda r: r["factors"]["f_near"],
            "kappa": lambda r: r["factors"]["kappa"], "s": lambda r: r["factors"]["s"],
            "land_within_200": lambda r: r["factors"]["land_within_200"],
            "coll_young part": lambda r: r["m5_by_nearest"]["coll_young"],
            "andean_young part": lambda r: r["m5_by_nearest"]["andean_young"],
            "coll200 coast share": lambda r: r["coast_by_front"]["coll200"]["coast_share"],
            "sub200 coast share": lambda r: r["coast_by_front"]["sub200"]["coast_share"],
            "largest landmass coast share": lambda r: r["coast_by_landmass_Mkm2"]["30-1e+09"]["coast_share"],
            "M5 at M4=0.055": lambda r: r["curve"][1][2]}
    for k, g in cand.items():
        x = np.array([g(r) for r in F])
        rho, p = spearmanr(m5, x)
        print(f"   {k:<32} rho {rho:+.3f} p {p:.4f}")
    x = np.array([r["curve"][1][2] for r in F])
    for k in ("f_near", "kappa", "s"):
        y = np.array([r["factors"][k] for r in F])
        rho, p = spearmanr(x, y)
        print(f"   M5@0.055 vs {k:<20} rho {rho:+.3f} p {p:.4f}")

    print("\n== Q4: the maps (100,000 Fibonacci points, plate elevation before the surface stage)")
    print(f"   Earth f100k no_sheets: M5 {e100['M5']:.4f} M4 {e100['M4']:.4f} ratio {e100['factors']['ratio']:.3f}"
          f" f_near {e100['factors']['f_near']:.3f} kappa {e100['factors']['kappa']:.4f} s {e100['factors']['s']:.3f}"
          f" M5@0.055 {[c[2] for c in e100['curve'] if abs(c[0] - 0.055) < 1e-9][0]:.4f}")
    by_t = defaultdict(list)
    for r in M:
        by_t[r["t"]].append(r)
    for t in sorted(by_t):
        rs = by_t[t]
        print(f"-- t {t}: {len(rs)} runs")
        row("M5", rs, lambda r: r["M5"])
        row("M4", rs, lambda r: r["M4"])
        row("M5/M4", rs, lambda r: r["factors"]["ratio"], None, "{:.3f}")
        row("M5 at M4=0.055", rs, lambda r: r["curve"][0][2])
        row("M5 at M4=0.07", rs, lambda r: r["curve"][1][2])
        for k in ("f_near", "kappa", "s", "land_within_200"):
            row(k, rs, lambda r, k=k: r["factors"][k])
        row("land", rs, lambda r: r["land"])
        row("largest landmass of land", rs, lambda r: r["largest_landmass_of_land"])
        for k in ("coll_young", "coll_old", "andean_young", "andean_old", "u_Tc", "u_volc"):
            row(f"by_nearest/M5 {k}", rs, lambda r, k=k: r["m5_by_nearest"][k] / max(r["M5"], 1e-9))
        row("without coll_young", rs, lambda r: r["m5_without"]["coll_young"])
    # Spearman of the map M5 at 0 against the field M5.
    f_by = {r["run"]: r for r in F}
    m0 = [r for r in M if r["t"] == 0 and r["run"] in f_by]
    if m0:
        rho, p = spearmanr([r["M5"] for r in m0], [f_by[r["run"]]["M5"] for r in m0])
        print(f"   map M5 at 0 vs field M5: rho {rho:+.3f} p {p:.4f}; map/field median "
              f"{med([r['M5'] / f_by[r['run']]['M5'] for r in m0]):.3f}; map / field plate_z "
              f"{med([r['M5'] / f_by[r['run']]['checks']['plate_z_M5'] for r in m0]):.3f}")


if __name__ == "__main__":
    main()
