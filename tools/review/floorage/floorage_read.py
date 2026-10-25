#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""floorage_read.py <label>=<floorage.jsonl> ...: the floor-age diagnosis's tables, per arm (medians over runs, pooled
counts over the maps' snapshots)."""
import json
import statistics as st
import sys

MIX = ["ocean_ocean", "ocean_continent", "collision", "ridge", "rift", "other"]
DC = ["0-330", "330-1000", "1000-2000", ">2000"]
DT = ["0-500", "500-1000", "1000-1500", ">1500"]
DR = ["0-500", "500-1000", "1000-2000", ">2000"]
PS = ["<0.02", "0.02-0.05", "0.05-0.10", "0.10-0.20", ">0.20"]
PC = ["0", "<0.1", "0.1-0.3", ">0.3"]
TH = ["<10km", "10-25km", ">=25km"]
AB = ["0-10", "10-30", "30-60", "60-100", "100-150", "150-180", "180-250", ">250"]


def med(xs):
    xs = [x for x in xs if x is not None]
    return st.median(xs) if xs else None


def rng(xs):
    xs = [x for x in xs if x is not None]
    return (min(xs), max(xs)) if xs else (None, None)


def f(x, d=3):
    return "-" if x is None else f"{x:.{d}f}"


def load(path):
    return [json.loads(x) for x in open(path) if x.strip().startswith("{")]


def rec_table(label, runs):
    print(f"\n=== {label}: {len(runs)} runs (record) ===")
    R = [r["rec"] for r in runs]
    fin = [x["age_series"][-1] for x in R]
    late_mean = [st.mean(a["mean_lb"] for a in x["age_series"] if a["t"] >= -500) for x in R]
    print("final: mean_lb", f(med([a["mean_lb"] for a in fin]), 1), "range", rng([a["mean_lb"] for a in fin]),
          "| s150", f(med([a["s150"] for a in fin])), "s180", f(med([a["s180"] for a in fin])),
          "s250", f(med([a["s250"] for a in fin])), "y10", f(med([a["y10"] for a in fin])))
    print("late (t>=-500): mean_lb", f(med(late_mean), 1), "old180", f(med([x["old180_late"] for x in R])),
          "old250", f(med([x["old250_late"] for x in R])))
    ac = [x["A_over_C0_myr"] for x in R]
    print("production share/Myr", f(med([x["prod_share_per_myr"] for x in R]), 4), "A/C0 Myr", f(med(ac), 1),
          "range", rng(ac), "| shape mean/(A/C0)", f(med([m / a for m, a in zip(late_mean, ac) if a]), 3),
          "| ocean share", f(med([x["ocean_share_late"] for x in R])))
    if "resampling_interval_myr" in R[0]:
        print("resampling interval p50", f(med([x["resampling_interval_myr"] for x in R]), 1), "Myr; subduction front",
              f(med([x["sub_front_km"] for x in R]), 0), "km, flux", f(med([x["sub_flux_km2_yr"] for x in R]), 3),
              "km2/yr, convergence", f(med([x["sub_conv_mm_yr"] for x in R]), 1), "mm/yr; plate speed (area-weighted)",
              f(med([x["speed_aw_mm_yr"] for x in R]), 1), "mm/yr")
        # Area: Earth-normalised, 5.1e8 km2 over the samples.
        area = [5.1e8 * x["ocean_share_late"] for x in R]
        print("  turnover by the front's flux: flux / ocean area", f(med([x["sub_flux_km2_yr"] * 1e6 / a for x, a in zip(R, area)]), 4),
              "per Myr; production share per Myr", f(med([x["prod_share_per_myr"] for x in R]), 4))
        nb = [x["normal_bands_late"] for x in R if x.get("normal_bands_late")]
        if nb:
            print("  front km by convergence band (0,5,10,20,40,80 mm/yr), oo:",
                  [round(med([b[0][i] for b in nb])) for i in range(6)], "oc:", [round(med([b[1][i] for b in nb])) for i in range(6)])
        ip = [x["init_plates"] for x in R]
        print("  initiated plates (born after -900):", ", ".join(
            f"{k} {f(med([p[k] for p in ip]), 3)}" for k in ("n", "a0_p50", "front_within20_share", "front_ever_share",
                                                             "left_at_end_p50", "absorbed", "absorbed_into_parent",
                                                             "lost_samples_per_myr", "born_samples_per_myr")))
        # The hazard per 10-Myr bin, pooled over runs.
        import math
        den = [sum(x["bins_den"][b] for x in R) for b in range(39)]
        num = [sum(x["bins_num"][b] for x in R) for b in range(39)]
        row = []
        for b in range(0, 39):
            s = num[b] / den[b] if den[b] else float("nan")
            row.append(f"{b * 10}:{(-math.log(s) / 10 if s > 0 else float('nan')):.4f}")
        print("  hazard per Myr by 10-Myr bin:", " ".join(row))
    print("cohort survival per 10 Myr, pooled (S10, hazard/Myr, loss samples/Myr per run):")
    for k in range(len(R[0]["cohorts"])):
        num = sum(x["cohorts"][k]["S10"] * x["cohorts"][k]["den"] for x in R if x["cohorts"][k]["den"])
        den = sum(x["cohorts"][k]["den"] for x in R)
        loss = st.mean(x["cohorts"][k]["loss_per_myr"] for x in R)
        s = num / den if den else float("nan")
        import math
        print(f"  {R[0]['cohorts'][k]['band']:>8}: S10 {s:.4f} haz {(-math.log(s) / 10 if s > 0 else float('nan')):.5f}"
              f" loss {loss:8.1f} stock {den / len(R) / max(1, 1):10.0f}")
    print("plates alive late", f(med([x["plates_late"] for x in R]), 1), "range", rng([x["plates_late"] for x in R]),
          "| front_km late (sum of plates')", f(med([x["front_km_late"] for x in R]), 0))
    mix = [x["mix_km_late"] for x in R]
    print("mix km late:", ", ".join(f"{n} {med([m[i] for m in mix]):.0f}" for i, n in enumerate(MIX)))
    print("incipient km", [f(med([x["incipient_km_late"][i] for x in R]), 0) for i in range(2)],
          "benioff km", [f(med([x["benioff_km_late"][i] for x in R]), 0) for i in range(2)])
    for k in ("rho_old180_plates", "rho_old180_sub", "rho_old180_ridge", "rho_old180_plates_lag50", "rho_old180_sub_lag50"):
        print(f"  {k}: median {f(med([x.get(k) for x in R]), 2)} range {rng([x.get(k) for x in R])}")
    ca, va, ea = [], [], []
    for x in R:
        late = [c for c in x["resamplings"] if c["t"] >= -500 and c["consumed_ocean"] > 0]
        if late:
            ca.append(st.mean(c["consumed_age"] for c in late))
            va.append(st.mean(c["visible_age"] for c in late))
            ea.append(st.mean(c["escaped_age"] for c in late if c["escaped_ocean"] > 0) if any(c["escaped_ocean"] > 0 for c in late) else None)
    print("resamplings late: consumed oceanic floor mean age", f(med(ca), 1), "visible", f(med(va), 1),
          "escaped", f(med(ea), 1), "ratio consumed/visible", f(med([c / v for c, v in zip(ca, va)]), 3))
    origins = {}
    for x in R:
        for o, d in x["lives"].items():
            a = origins.setdefault(o, {"n": [], "fate": {}, "life": [], "d20": []})
            a["n"].append(d["n"])
            a["life"].append(d["life_p50"])
            a["d20"].append(d["dead_within_20"])
            for k, v in d["fate"].items():
                a["fate"][k] = a["fate"].get(k, 0) + v
    for o, a in origins.items():
        print(f"  plates born by {o}: per run median {med(a['n'])}, life p50 {f(med(a['life']), 0)} Myr, dead within"
              f" 20 Myr {f(med(a['d20']), 2)}, fates pooled {a['fate']}")
    print("initiations per 100 Myr: whole", f(med([x["init_per_100myr"] for x in R]), 2), "late",
          f(med([x["init_late_per_100myr"] for x in R]), 2))
    keys = R[0]["summary"].keys()
    print("summary:", ", ".join(f"{k} {f(med([x['summary'].get(k) for x in R]), 3)}" for k in keys))


def map_table(label, runs):
    snaps = [s for r in runs for s in r["maps"]]
    if not snaps:
        return
    print(f"\n=== {label}: {len(snaps)} snapshots (t >= -800, not at a resampling) ===")
    ro = sum(s["rec_old180"] for s in snaps)
    po = sum(s["proxy_old180_all"] for s in snaps)
    pb = sum(s["proxy_old180_bare"] for s in snaps)
    print(f"proxy check: record old180 {ro}, proxy over all oceanic {po} ({po / ro:.3f}), over bare floor {pb}"
          f" ({pb / ro:.3f}); map ocean / record ocean {sum(s['map_ocean'] for s in snaps) / sum(s['rec_ocean'] for s in snaps):.4f}")
    rb = [sum(s["rec_bands"][i] for s in snaps) for i in range(8)]
    pa = [sum(s["proxy_bands_all"][i] for s in snaps) for i in range(8)]
    pr = [sum(s["proxy_bands_bare"][i] for s in snaps) for i in range(8)]
    print("  bands   " + " ".join(f"{b:>8}" for b in AB))
    for name, v in (("record", rb), ("proxy", pa), ("bare", pr)):
        t = sum(v)
        print(f"  {name:7} " + " ".join(f"{x / t:8.3f}" for x in v))
    stn = snaps[0]["status_names"]
    stv = [sum(s["status_old180"][i] for s in snaps) for i in range(5)]
    print("old180 (proxy) by initiation status:", ", ".join(f"{n} {v / max(sum(stv), 1):.3f}" for n, v in zip(stn, stv)))
    for key, names in (("dc", DC), ("dco", DC), ("dt", DT), ("dr", DR), ("ps", PS), ("pc", PC), ("th", TH)):
        o = [sum(s["old180"][key][i] for s in snaps) for i in range(len(names))]
        a = [sum(s["ocean"][key][i] for s in snaps) for i in range(len(names))]
        to, ta = max(sum(o), 1), max(sum(a), 1)
        print(f"  {key:4} " + "  ".join(f"{n}: old {x / to:.3f} ocean {y / ta:.3f} (x{(x / to) / max(y / ta, 1e-9):.2f})"
                                       for n, x, y in zip(names, o, a)))
    o = [sum(s["old180"]["ptrench"][i] for s in snaps) for i in range(2)]
    a = [sum(s["ocean"]["ptrench"][i] for s in snaps) for i in range(2)]
    print(f"  plate with no trench on it: old {o[0] / sum(o):.3f} ocean {a[0] / sum(a):.3f}")
    for j, nm in ((0, "trench<=1000"), (1, "clear")):
        o = [sum(s["old180"]["dc_x_dt1000"][j][i] for s in snaps) for i in range(4)]
        tot = sum(sum(s["old180"]["dc_x_dt1000"][k][i] for s in snaps) for k in range(2) for i in range(4))
        print(f"  old180 {nm}: " + ", ".join(f"dc {n} {x / tot:.3f}" for n, x in zip(DC, o)))
    for k in ("plates", "plates_eligible", "plates_eligible_far", "lambda_per_myr", "lambda_far_per_myr",
              "births_si_pm25", "front_sub_km", "n_trench", "n_div"):
        print(f"  {k}: mean {st.mean(s[k] for s in snaps):.3f}")
    lf = st.mean(s["lambda_far_per_myr"] for s in snaps)
    ba = st.mean(s["births_si_pm25"] for s in snaps)
    print(f"  drawn-and-seeded expected per Myr {lf:.4f}, born per Myr {ba:.4f}: success {ba / lf if lf else float('nan'):.3f}")


def main():
    for arg in sys.argv[1:]:
        label, path = arg.split("=", 1)
        runs = load(path)
        if "@" in label:
            label, sel = label.split("@", 1)
            runs = [r for r in runs if sel in r["arm"]]
        rec_table(label, runs)
        map_table(label, runs)


main()
