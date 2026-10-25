#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Pools collfront.py's runs: collfront_read.py <cf-*.jsonl>. Medians over runs (and pooled sums where a
ratio of sums is the flux's own mean), for: the collision fronts' crust flux by window, the dock tally's
convergence by the collision state's age, the ended collision states and belts, the force budget of the
plates with a collision resistance, and the final grid's belt cross-section per km of front."""
import json
import statistics as st
import sys

import numpy as np

WINDOWS = ((-1000.0, 0.0), (-900.0, 0.0), (-300.0, 0.0), (-100.0, 0.0))


def med(xs):
    xs = [x for x in xs if x is not None and np.isfinite(x)]
    return st.median(xs) if xs else float("nan")


def q(xs, p):
    xs = [x for x in xs if x is not None and np.isfinite(x)]
    return float(np.percentile(xs, p)) if xs else float("nan")


def at(glob, t):
    best = None
    for g in glob:
        if g["t"] <= t + 1e-6:
            best = g
    return best


def main():
    rows = [json.loads(x) for x in open(sys.argv[1])]
    print(f"{len(rows)} runs")
    # 1. The crust flux into the collision fronts, by window (differences of the cumulative lines).
    print("\n1. crust flux into collision fronts (cb_globals differences); medians over runs [pooled ratio of sums]")
    for t0, t1 in WINDOWS:
        per = []
        for r in rows:
            g = r["summary"]["globals"]
            a, b = (at(g, t0) if t0 > -1000 else {k: 0.0 for k in g[-1]}), at(g, t1)
            if a is None or b is None:
                continue
            per.append({k: b.get(k, 0.0) - a.get(k, 0.0) for k in b if k != "t"})
        span = t1 - t0

        def show(name, f):
            vals = [f(d) for d in per]
            num = {k: sum(d.get(k, 0.0) for d in per) for k in per[0]}
            print(f"   {name:<46} {med(vals):10.4g}  [{f(num):.4g}]")
        print(f"  window {t0:.0f} to {t1:.0f} Myr ({len(per)} runs)")
        show("mean front in collision, km", lambda d: d["front_km_myr"] / span)
        show("convergence crossed_km2/front_km_myr, mm/yr", lambda d: d["crossed_km2"] / max(d["front_km_myr"], 1e-9))
        show("thickness crossing crossed/crossed_km2, km", lambda d: d["crossed"] / max(d["crossed_km2"], 1e-9))
        show("crossed per km of front per Myr, km3", lambda d: d["crossed"] / max(d["front_km_myr"], 1e-9))
        show("kept per km of front per Myr, km3", lambda d: d["kept"] / max(d["front_km_myr"], 1e-9))
        for k in ("kept", "kept_wedge", "upper", "noroom_sliver", "noroom_capped", "noroom_clipped", "taper_toe",
                  "crossed_restack", "crossed_above", "crossed_recent"):
            show(f"{k} / crossed", lambda d, k=k: d.get(k, 0.0) / max(d["crossed"], 1e-9))
        # PLT-652: kept at the front (the wedge and the belt) and the overriding interior's share.
        show("kept at the front (kept - upper) / crossed", lambda d: (d["kept"] - d.get("upper", 0.0)) / max(d["crossed"], 1e-9))
        for k in ("upper_km2", "upper_most_km2"):
            show(f"{k} / crossed_km2", lambda d, k=k: d.get(k, 0.0) / max(d["crossed_km2"], 1e-9))
        show("lost (crossed - kept) per km per Myr, km3", lambda d: (d["crossed"] - d["kept"]) / max(d["front_km_myr"], 1e-9))
        for k in ("collapse_moved", "channel_flow", "foundered", "collapse_unplaced"):
            show(f"{k} / kept", lambda d, k=k: d.get(k, 0.0) / max(d["kept"], 1e-9))
    # 2. The dock tally: front km x Myr and convergence by the collision state's age (from -900 Myr).
    print("\n2. collision front time and convergence by the collision state's age (dock tally, from -900 Myr)")
    ages = ["0", "10", "30", "60"]
    km = {a: sum(sum(r["summary"]["dock"][a]["km_myr"]) for r in rows if a in r["summary"]["dock"]) for a in ages}
    tot = sum(km.values())
    for a in ages:
        conv_w = sum(sum(r["summary"]["dock"][a]["km_myr"]) * r["summary"]["dock"][a]["means"][4]
                     for r in rows if a in r["summary"]["dock"] and np.isfinite(r["summary"]["dock"][a]["means"][4]))
        print(f"  age {a:>2}+ Myr: front time share {km[a] / tot:.3f}  convergence {conv_w / max(km[a], 1e-9):.1f} mm/yr"
              f"  (run medians {med([r['summary']['dock'][a]['means'][4] for r in rows if a in r['summary']['dock']]):.1f})"
              f"  under_cont {med([r['summary']['dock'][a]['means'][0] for r in rows if a in r['summary']['dock']]):.2f}")
    # 3. The ended collision states, partings and belts (pooled).
    print("\n3. ended collision states (transfers, welds), partings, belts: pooled p10/p50/p90")
    S = [s for r in rows for s in r["rec"]["states"] if s["coll"]]
    Pp = [p for r in rows for p in r["rec"]["partings"]]
    B = [b for r in rows for b in r["rec"]["belts"]]

    def line(name, xs, w=None):
        xs = [x for x in xs if x is not None and np.isfinite(x)]
        if not xs:
            print(f"   {name:<52} n=0")
            return
        print(f"   {name:<52} n={len(xs):<5} p10 {q(xs, 10):9.2f} p50 {q(xs, 50):9.2f} p90 {q(xs, 90):9.2f}"
              f" mean {np.mean(xs):9.2f}")
    print(f"  collision states ended by a transfer or weld: {len(S)} ({sum(1 for s in S if s['weld'])} welded)")
    line("duration, Myr", [s["dur"] for s in S])
    line("shortening, km", [s["short_km"] for s in S])
    line("crossed area, 1e3 km2", [s["area_km2"] / 1e3 for s in S])
    line("front, km", [s["front_km"] for s in S])
    line("mean convergence (shortening / duration), mm/yr", [s["short_km"] / s["dur"] for s in S])
    line("onset convergence, mm/yr", [s["onset"] for s in S])
    line("last-step convergence, mm/yr", [s["last"] for s in S])
    line("contact age at the state's start, Myr", [s["contact_age"] for s in S])
    aw = sum(s["area_km2"] for s in S)
    print(f"   area-weighted mean convergence {sum(s['short_km'] / s['dur'] * s['area_km2'] for s in S) / max(aw, 1):.1f} mm/yr,"
          f" area-weighted duration {sum(s['dur'] * s['area_km2'] for s in S) / max(aw, 1):.0f} Myr")
    print(f"  partings: {len(Pp)}")
    line("duration, Myr", [p["dur"] for p in Pp])
    line("shortening, km", [p["short_km"] for p in Pp])
    line("crossed area, 1e3 km2", [p["area_km2"] / 1e3 for p in Pp])
    line("peak convergence, mm/yr", [p["peak"] for p in Pp])
    reasons = {}
    for p in Pp:
        a = reasons.setdefault(p["reason"], [0, 0.0, 0.0])
        a[0] += 1
        a[1] += p["area_km2"]
        a[2] += p["short_km"] * p["area_km2"]
    for k_, (n_, a_, sw) in sorted(reasons.items(), key=lambda kv: -kv[1][1]):
        print(f"   parting reason {k_:<24} n={n_:<5} crossed {a_ / 1e6:8.1f} million km2, area-weighted shortening {sw / max(a_, 1):.0f} km")
    aw = sum(p["area_km2"] for p in Pp)
    print(f"   partings area-weighted duration {sum(p['dur'] * p['area_km2'] for p in Pp) / max(aw, 1):.0f} Myr,"
          f" area-weighted mean convergence {sum(p['short_km'] / p['dur'] * p['area_km2'] for p in Pp) / max(aw, 1):.1f} mm/yr")
    print(f"  crossed area: states {sum(s['area_km2'] for s in S) / 1e6:.1f}, partings {sum(p['area_km2'] for p in Pp) / 1e6:.1f} million km2")
    print(f"  belts: {len(B)}")
    line("convergence 20 Myr after onset / onset", [b["convergence_20myr_mm_yr"] / b["convergence_onset_mm_yr"]
                                                    for b in B if b["convergence_20myr_mm_yr"] is not None
                                                    and b["convergence_onset_mm_yr"]])
    line("onset convergence, mm/yr", [b["convergence_onset_mm_yr"] for b in B])
    line("convergence 20 Myr on, mm/yr", [b["convergence_20myr_mm_yr"] for b in B])
    line("belt width, km", [b["belt_width_km"] for b in B])
    line("belt length, km", [b["belt_length_km"] for b in B])
    line("wedge width, km", [b["wedge_width_km"] for b in B])
    line("peak height, m", [b["peak_height_m"] for b in B])
    # Force budget of the plates with a collision resistance.
    print("\n  plates with a collision resistance (FRC-018 + FRC-030), per step from -900 Myr: run medians of p50")
    F = [r["rec"]["forces"] for r in rows]
    print(f"   collision torque / slab pull torque p50 {med([f['cr_over_pull']['p50'] for f in F if f['cr_over_pull']]):.3f}"
          f" (p90 {med([f['cr_over_pull']['p90'] for f in F if f['cr_over_pull']]):.3f}); no pull {med([f['cr_no_pull'] for f in F]):.3f}")
    print(f"   collision torque / driving torques p50 {med([f['cr_over_drive']['p50'] for f in F if f['cr_over_drive']]):.3f}"
          f" (p90 {med([f['cr_over_drive']['p90'] for f in F if f['cr_over_drive']]):.3f}); share >= 1 {med([f['cr_over_drive_ge1'] for f in F]):.3f}")
    print(f"   RMS speed (forces f[11]) with collision resistance p50 {med([f['speed_cr']['p50'] for f in F if f['speed_cr']]):.1f} mm/yr,"
          f" without {med([f['speed_free']['p50'] for f in F if f['speed_free']]):.1f}")
    print(f"   record collision front: mean {med([r['rec']['coll_km_mean'] for r in rows]):.0f} km, last {med([r['rec']['coll_km_last'] for r in rows]):.0f} km")
    # 4. The final grid's belt per km of collision front.
    G = [(r["grid"], r["rec"]) for r in rows if "excess_km3" in r["grid"]]
    print("\n4. final grid, per km of the collision front (record mix_km[2], mean over the last 50 Myr): run medians")
    print(f"   front last/50 {med([rc['coll_km_last'] / rc['coll_km_50'] for g, rc in G]):.2f},"
          f" mean(-900..0)/50 {med([rc['coll_km_mean'] / rc['coll_km_50'] for g, rc in G]):.2f};"
          f" front last 50 Myr {med([rc['coll_km_50'] for g, rc in G]):.0f} km")
    print(f"   runs with fronts {len(G)}; front points p50 {med([g['front_points'] for g, rc in G]):.0f};"
          f" kinematic convergence at the front p50 {med([g['vc_front']['p50'] for g, rc in G]):.1f} mm/yr,"
          f" mean {med([g['vc_front_mean'] for g, rc in G]):.1f}")
    for norm in ("coll_km_50", "coll_km_mean"):
        print(f"   [per km of {norm}] excess crust (T - 35.1 km, continental) within 800 km:"
              f" {med([g['excess_le800_km3'] / rc[norm] for g, rc in G]):.0f} km3/km; planet's excess per km"
              f" {med([g['excess_all_km3'] / rc[norm] for g, rc in G]):.0f}; share within 800 km"
              f" {med([g['excess_le800_km3'] / g['excess_all_km3'] for g, rc in G]):.3f}")
    print(f"   the planet's continental crust median {med([g['T_cont_median_km'] for g, rc in G]):.1f} km, farther than"
          f" 1500 km from a collision front {med([g['T_cont_median_far_km'] for g, rc in G]):.1f} km; excess within 800 km"
          f" over them {med([g['excess_le800_km3_med'] / rc['coll_km_50'] for g, rc in G]):.0f} and"
          f" {med([g['excess_le800_km3_far'] / rc['coll_km_50'] for g, rc in G]):.0f} km3/km")
    keys = list(G[0][0]["excess_km3"].keys())
    print("   band (side_from_to km): excess km3/km | continental km2/km | mean T km   (per km of coll_km_50)")
    for kk in keys:
        print(f"     {kk:<16} {med([g['excess_km3'][kk] / rc['coll_km_50'] for g, rc in G]):8.0f}"
              f" | {med([g['cont_km2'][kk] / rc['coll_km_50'] for g, rc in G]):8.0f}"
              f" | {med([g['T_mean_km'][kk] for g, rc in G]):6.1f}")
    for nm in ("t45", "t55", "hi", "hi_coll_young", "coll_young_land"):
        print(f"   {nm:<16} within 800 km {med([g[nm + '_le800_km2'] / rc['coll_km_50'] for g, rc in G]):7.0f} km2/km"
              f" (overriding side {med([g[nm + '_le800_over_km2'] / rc['coll_km_50'] for g, rc in G]):7.0f});"
              f" planet {med([g[nm + '_all_km2'] / rc['coll_km_50'] for g, rc in G]):7.0f};"
              f" per km of coll_km_mean {med([g[nm + '_le800_km2'] / rc['coll_km_mean'] for g, rc in G]):7.0f}")
    print(f"   land {med([g['land_km2'] for g, rc in G]) / 1e6:.1f} million km2;"
          f" young collision high ground within 800 km / land {med([g['hi_coll_young_le800_km2'] / g['land_km2'] for g, rc in G]):.4f}")


if __name__ == "__main__":
    main()
