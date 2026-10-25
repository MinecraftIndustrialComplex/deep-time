#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""What slows the keel-less collision belts: m4decay.py <run or arm folder> ...

The thin sheet (PLT-414, PLT-415; collapse.rs `Sheet`) brings a hot belt's column under 2 km within about
10 Myr of the quiet time (PLT-259, 10 Myr after the sample's last stamp), yet collision belts 10 to 50 Myr
past their stamp hold about 0.02 of the land above 2 km at the end of base13. This reads the cached pool runs:

A. The model's collisions (rec.json): a train is the collision states on one contact (its two plates and its
   onset) with gaps of at most 6 Myr (shortening::GRACE_MYR, PLT-225), as the `sutures` measure groups them.
   Over the run after its first 100 Myr: trains, their duration (first start to last end, plus a step) p50/p90
   by count and by longest front, the states' convergence, and the trains open at the end (duration,
   convergence, front).
B. The end snapshot's collision-belt high ground (tag 2, Myr since the stamp in [10, 50), more than 2 km over
   the sea, continental): (i) within 400 km (`collision_belt_km`) of a collision front of the end step (the
   crust-budget map's MAP_FRONT_COLLISION flag, overriding side) on a contact whose collision state is live at
   the end, (ii) the rest. Per group: the land share, T, the height over the sea, and with the front trace
   the lithosphere L, the keel K, the sheet's viscosity factor eta/eta0 (ISO-112: the thermal age of L in
   place of the orogeny's, e-fold 62.8 Myr), the sheet's thinning rate now (per Myr) and at eta0, and the
   trace's gains since the column crossed 45 km by group. For (i), the live train's duration and convergence.
C. With the front trace, every snapshot pooled: keel-less (K < 100 km) collision-belt samples by Myr since
   the stamp (2-Myr bins to 60): samples, the share above 2 km, T, height, L, eta/eta0, the sheet's rate, and
   the share within 400 km of a live collision front; then the path the collapse's own laws give a column
   from the 10-12 Myr bin's median T and L (threshold law PLT-254 to 256 above T_c, the sheet at the thermal
   age's eta, the lithosphere cooling as thermal.rs's law, the median sea), against the observed medians.

The sheet here is collapse.rs's: eps = dU / (4 eta L_s), L_s 100 km (FRC-010), dU the column energy
(forces::column_energy, FRC-014) at its Airy height over the sea less the normal column's (room_column_m,
PLT-412/413: T_n + 7.02 S within [T_n/2, 41.1 km]), the Airy height z_0 + (T - T_n) / 7.02 less the eclogite
drop with the mafic pools taken as the normal column's share (0.128 of T; pools are not dumped).
Needs numpy only."""
import json
import math
import sys
from collections import defaultdict
from pathlib import Path

import numpy as np

ATT = 27
GRACE = 6.0
R_KM = 6371.0
REACH_KM = (200.0, 400.0)
GROUPS = ["founder", "collapse", "flow", "stack", "collision", "arc", "rift", "surface", "other"]
# collapse.rs, thermal.rs, forces.rs, pools.rs constants (ledger ids in those files)
T_N, Z0, F_ISO = 35100.0, 525.0, 3300.0 / 470.0
ECL_DEPTH, ECL_SHARE, MAFIC = 1.5e9 / (2830.0 * 9.81), 0.3, 4500.0 / 35100.0
RHO_C, RHO_M, RHO_W, CRUST, G = 2750.0, 3300.0, 1000.0, 35000.0, 9.81
ETA0, TAU_S, L_SHEET, YEAR = 1.0e22, 62.8, 100e3, 3.15576e7
L_HOT, L_COLD, TAU_C = 60.0, 225.0, 62.8 * (225.0 / 125.0) ** 2
T_C, COL_MAX, COLLAPSE_MYR, DT = 50000.0, 41100.0, 15.0, 2.0


def iso_z(T):
    T = np.asarray(T, float)
    e = ECL_SHARE * np.minimum(MAFIC * T, np.maximum(T - ECL_DEPTH, 0.0))
    return Z0 + (T - T_N) / F_ISO - e * (3450.0 - 2830.0) / 3300.0


def energy(z, sea):
    h = np.asarray(z, float) - sea
    rho = np.where(h >= 0.0, RHO_C, RHO_C - RHO_W)
    crust = CRUST + sea * RHO_M / (RHO_M - RHO_C)
    return rho * G * h * (crust + 0.5 * h * (1.0 + rho / (RHO_M - RHO_C)))


def normal_col(sea):
    return float(np.clip(T_N + F_ISO * sea, 0.5 * T_N, max(COL_MAX, T_N)))


def thermal_age(L):
    L = np.asarray(L, float)
    with np.errstate(divide="ignore", invalid="ignore"):
        a = TAU_C * np.log((L_COLD - L_HOT) / (L_COLD - L))
    return np.where(L <= L_HOT, 0.0, np.where(L >= L_COLD, np.inf, a))


def eta_factor(L):
    return np.exp(np.minimum(thermal_age(L) / TAU_S, 50.0))


def sheet_rate(T, L, sea, eta_f=None):
    n = normal_col(sea)
    du = energy(iso_z(T), sea) - energy(iso_z(np.array(n)), sea)
    f = eta_factor(L) if eta_f is None else eta_f
    eps = du / (4.0 * ETA0 * f * L_SHEET) * YEAR * 1e6
    return np.where((du > 0) & (np.asarray(T) > n), eps, 0.0)


def unit(lat, lon):
    la, lo = np.radians(lat), np.radians(lon)
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], axis=1)


def near_km(pts, front):
    """Great-circle distance of each point to the nearest front point, km (inf without a front)."""
    if len(front) == 0 or len(pts) == 0:
        return np.full(len(pts), np.inf)
    out = np.empty(len(pts))
    for i in range(0, len(pts), 2048):
        d = np.clip(pts[i:i + 2048] @ front.T, -1.0, 1.0)
        out[i:i + 2048] = np.arccos(d.max(axis=1)) * R_KM
    return out


K0, H0, H_MAX, HEX = 1500.0, 15.0, 30.0, 2.0 / 3.0  # flow.rs (PLT-300, PLT-301), crust_channel_km2_myr
NEIGHBOUR_KM = 105.0  # the 100,000-sample lattice's spacing is 77 km; the second ring stands at 133
TAG = 2
# gap_budget::Part, in order
PARTS = ["birth", "erosion", "rebound", "water", "flexure", "roots", "released", "flex_rest", "spread", "later",
         "carry", "subduction", "shortening", "belts", "arcs", "forearc", "founder", "collapse", "flow",
         "transition", "collision", "rifting", "hotspots", "settle", "rate", "plate_surface", "finish", "bury",
         "resample_blend", "resample_accrete", "other"]


def flow_inflow(a, idx):
    """Lower-crustal flow into the samples `idx` of the trace snapshot `a` (all its continental rows), km/Myr:
    flow.rs's explicit rule on the lattice edges (same plate), K of the thicker column's channel, the hexagonal
    Laplacian's weight 2/3 per edge over the edge's length squared. Also the part from columns in their quiet
    time (tag 2, under 10 Myr since the stamp)."""
    pts = unit(a[:, 0], a[:, 1])
    T = a[:, 2] / 1000.0
    held = (np.rint(a[:, 4]) >= 1) & (a[:, 5] < 10)
    plate = a[:, 8]
    out, out_h = np.zeros(len(idx)), np.zeros(len(idx))
    cos_r = math.cos(NEIGHBOUR_KM / R_KM)
    for c in range(0, len(idx), 512):
        ii = idx[c:c + 512]
        dots = pts[ii] @ pts.T
        rk, j = np.nonzero(dots > cos_r)
        i = ii[rk]
        keep = (plate[j] == plate[i]) & (j != i)
        rk, i, j = rk[keep], i[keep], j[keep]
        d = np.arccos(np.clip(dots[rk, j], -1.0, 1.0)) * R_KM
        h = np.clip(np.maximum(T[j], T[i]) - T_C / 1000.0, 0.0, H_MAX)
        f = HEX * K0 * (h / H0) ** 3 * (T[j] - T[i]) / d ** 2
        out[c:c + len(ii)] = np.bincount(rk, weights=f, minlength=len(ii))
        out_h[c:c + len(ii)] = np.bincount(rk, weights=f * held[j], minlength=len(ii))
    return out, out_h


def runs_of(args):
    out = []
    for a in args:
        p = Path(a)
        if (p / "rec.json").exists():
            out.append(p)
        else:
            out += sorted(q for q in p.iterdir() if (q / "rec.json").exists())
    return out


def trains(rec):
    by = defaultdict(list)
    for c in rec["collisions"]:
        by[(c["under"], c["over"], round(c["contact_onset_myr"], 3))].append(
            (c["start_myr"], c["end_myr"], c["front_km"], c["convergence_mm_yr"], "hand-off"))
    for p in rec["partings"]:
        by[(p["under"], p["over"], round(p["contact_onset_myr"], 3))].append(
            (p["start_myr"], p["end_myr"], p["front_km"], p["peak_convergence_mm_yr"], p["reason"]))
    out = []
    for k, v in by.items():
        v.sort()
        cur = None
        for s, e, f, cv, why in v:
            if cur is None or s - cur["e"] > GRACE + 1e-6:
                if cur:
                    out.append(cur)
                cur = {"key": k, "s": s, "e": e, "f": f, "cv": [cv], "end": why, "onset": k[2]}
            else:
                cur["e"] = max(cur["e"], e)
                cur["f"] = max(cur["f"], f)
                cur["cv"].append(cv)
                cur["end"] = why
        if cur:
            out.append(cur)
    return out


def load_maps(run, t):
    name = f"t{round(t):04d}.bin"
    m, p = run / "maps" / name, run / "maps" / "plate" / name
    if not m.exists():
        return None
    a = np.fromfile(m, "<f4").reshape(-1, 11).astype(np.float64)
    plate = np.fromfile(p, "<u4").astype(np.int64) if p.exists() else None
    return a, plate


def load_att(run):
    idx = run / "trace" / "att_index.txt"
    if not idx.exists():
        return {}
    out = {}
    for line in idx.read_text().splitlines():
        kv = dict(x.split("=", 1) for x in line.split())
        f = run / "trace" / kv["file"]
        if f.exists():
            out[float(kv["t"])] = f
    return out


def read_att(f):
    return np.fromfile(f, "<f4").reshape(-1, ATT).astype(np.float64)


def q(x, p=50):
    x = np.asarray(x, float)
    x = x[np.isfinite(x)]
    return float(np.percentile(x, p)) if x.size else float("nan")


def live_fronts(rec, maps, t):
    """Unit vectors of the end step's collision-front samples on contacts with a live collision state at t, and
    per front sample its live train (for duration and convergence)."""
    if maps is None:
        return np.zeros((0, 3)), []
    a, plate = maps
    flags = a[:, 10].astype(np.int64)
    fr = (flags & 512) != 0
    if plate is None:
        return unit(a[fr, 0], a[fr, 1]), []
    live = {}
    for tr in trains(rec):
        if tr["s"] <= t + 1e-6 and tr["e"] >= t - DT - 1e-6:
            live.setdefault(tr["key"][1], []).append(tr)
            live.setdefault(tr["key"][0], []).append(tr)
    keep = fr & np.isin(plate, list(live.keys()))
    return unit(a[keep, 0], a[keep, 1]), [live[int(p)] for p in plate[keep]]


def main():
    global TAG
    args = sys.argv[1:]
    if args and args[0].startswith("tag"):
        # the orogeny the class takes: tag2 (collision, the default) or tag1 (Andean)
        TAG = int(args.pop(0)[3:])
    print(f"the class: orogeny tag {TAG}")
    runs = runs_of(args)
    print(f"{len(runs)} runs")
    # A. collision trains
    durs, fronts, cvs, opens, contact = [], [], [], [], []
    for run in runs:
        rec = json.loads((run / "rec.json").read_text())
        t0, t_end, dt = rec["t0_myr"] + 100.0, rec["steps"][-1]["t"], rec["dt_myr"]
        for tr in trains(rec):
            if tr["e"] < t0:
                continue
            d = tr["e"] - tr["s"] + dt
            durs.append(d)
            fronts.append(tr["f"])
            cvs += [c for c in tr["cv"] if c is not None]
            contact.append(tr["e"] - tr["onset"] + dt)
            if tr["end"] == "open":
                opens.append((d, tr["f"], max(tr["cv"]), tr["e"] - tr["onset"] + dt))
    durs, fronts = np.array(durs), np.array(fronts)
    o = np.argsort(durs)
    cw = np.cumsum(fronts[o]) / max(fronts.sum(), 1e-9)
    wq = lambda p: float(durs[o][np.searchsorted(cw, p)])  # noqa: E731
    print("\nA. collision trains after the first 100 Myr (states on one contact, gaps <= 6 Myr)")
    print(f"  trains {len(durs)} ({len(durs) / len(runs):.1f} a run); duration p50 {q(durs):.0f}, p90 {q(durs, 90):.0f},"
          f" max {durs.max():.0f} Myr; by longest front p50 {wq(0.5):.0f}, p90 {wq(0.9):.0f} Myr")
    print(f"  trains of 40 Myr or more {np.mean(durs >= 40):.2f}, of 100 or more {np.mean(durs >= 100):.2f};"
          f" contact age at the train's end (subduction before included) p50 {q(contact):.0f}, p90 {q(contact, 90):.0f}")
    print(f"  states' convergence (hand-off: at the hand-off; parting: peak) p10/p50/p90 {q(cvs, 10):.1f}/"
          f"{q(cvs):.1f}/{q(cvs, 90):.1f} mm/yr")
    if opens:
        od = np.array(opens)
        print(f"  open at the end: {len(od)} ({len(od) / len(runs):.1f} a run), duration p50 {q(od[:, 0]):.0f}, max"
              f" {od[:, 0].max():.0f} Myr, front p50 {q(od[:, 1]):.0f} km, peak convergence p50 {q(od[:, 2]):.1f}"
              f" mm/yr, contact age p50 {q(od[:, 3]):.0f} Myr")

    # B. the end snapshot's belt high ground, aged 10 to 50 Myr
    print("\nB. end snapshot: collision-belt high ground (tag 2, 10 <= Myr since the stamp < 50, > 2 km over the sea)")
    groups = defaultdict(lambda: defaultdict(list))
    ends = defaultdict(int)
    land_n = 0
    for run in runs:
        rec = json.loads((run / "rec.json").read_text())
        t_end = rec["steps"][-1]["t"]
        sea = float(rec["steps"][-1].get("sea_m") or 0.0)
        maps = load_maps(run, t_end)
        att = load_att(run).get(t_end)
        att = read_att(att) if att is not None else None
        if att is not None:
            lat, lon, T, z, tag, age, L, K = (att[:, i] for i in range(8))
            gains = att[:, 18:18 + len(GROUPS)]
            peak = att[:, 17]
        elif maps is not None:
            a = maps[0]
            cont = a[:, 4] > 0.5
            lat, lon, z, T, tag, age = (a[cont, i] for i in (0, 1, 2, 3, 5, 6))
            L = K = peak = np.full(len(T), np.nan)
            gains = np.full((len(T), len(GROUPS)), np.nan)
        else:
            continue
        # the land: every visible sample over the sea (the maps), else the continental ones (the trace)
        land_n += int(np.sum(maps[0][:, 2] > sea)) if maps is not None else int(np.sum(z - sea > 0))
        cls = (np.rint(tag) == TAG) & (age >= 10) & (age < 50) & (z - sea > 2000)
        fr, live = live_fronts(rec, maps, t_end)
        if maps is not None:
            ends["front samples"] += int(((maps[0][:, 10].astype(np.int64) & 512) != 0).sum())
        ends["on live contacts"] += len(fr)
        ends["live trains"] += sum(1 for tr in trains(rec) if tr["e"] >= t_end - DT - 1e-6)
        pts = unit(lat[cls], lon[cls])
        dkm = near_km(pts, fr)
        hm = (np.rint(tag) >= 1) & (age < 10) & (T > T_C)
        dheld = near_km(pts, unit(lat[hm], lon[hm]))
        nearest = None
        if len(fr) and len(pts):
            nearest = np.argmax(np.clip(pts @ fr.T, -1, 1), axis=1)
        for j, i in enumerate(np.flatnonzero(cls)):
            g = ("(i-a) <= 400 km of a live collision front" if dkm[j] <= 400 else "(i-b) <= 200 km of a held column"
                 " (quiet time, over T_c), no live front" if dheld[j] <= 200 else "(ii) neither")
            for name, v in (("T", T[i]), ("zs", z[i] - sea), ("L", L[i]), ("K", K[i]), ("age", age[i]),
                            ("d", dkm[j]), ("peak", peak[i])):
                groups[g][name].append(v)
            groups[g]["eps"].append(float(sheet_rate(np.array([T[i]]), np.array([L[i]]), sea)[0]) if np.isfinite(L[i])
                                    else float("nan"))
            groups[g]["eps0"].append(float(sheet_rate(np.array([T[i]]), np.array([L[i]]), sea, eta_f=1.0)[0]))
            groups[g]["gains"].append(gains[i])
            if att is not None:
                p_ = int(att[i, 14])
                groups[g]["touch"].append((PARTS[p_] if 0 <= p_ < len(PARTS) else "none", att[i, 15]))
            if g.startswith("(i-a)") and nearest is not None:
                trs = live[nearest[j]]
                groups[g]["train"].append(max(t_end - tr["s"] + DT for tr in trs))
                groups[g]["cv"].append(max(max(tr["cv"]) for tr in trs))
    print("  at the end, all runs: " + ", ".join(f"{k} {v}" for k, v in ends.items())
          + " (the maps' collision-front flags; none at a resampling step)")
    for g in sorted(groups):
        v = groups[g]
        n = len(v["T"])
        kl = np.array(v["K"])
        print(f"  {g}: n {n}, share of the land {n / max(land_n, 1):.4f}; Myr since the stamp p50 {q(v['age']):.0f};"
              f" T p50 {q(v['T']) / 1000:.1f} km, over the sea p50 {q(v['zs']):.0f} m; keel K >= 100 "
              f"{np.mean(kl >= 100) if np.isfinite(kl).any() else float('nan'):.2f}")
        print(f"      L p50 {q(v['L']):.0f} km (p10/p90 {q(v['L'], 10):.0f}/{q(v['L'], 90):.0f}), eta/eta0 p50 "
              f"{q(eta_factor(np.array(v['L']))):.1f} (p90 {q(eta_factor(np.array(v['L'])), 90):.1f}); sheet now p50"
              f" {100 * q(v['eps']):.2f} %/Myr, at eta0 {100 * q(v['eps0']):.2f} %/Myr; peak since 45 km less T p50"
              f" {q(np.array(v['peak']) - np.array(v['T'])) / 1000:.1f} km")
        if v["gains"] and np.isfinite(np.array(v["gains"])).any():
            gm = np.nanmean(np.array(v["gains"]), axis=0) / 1000
            print("      gains since crossing 45 km, mean km: " + ", ".join(f"{a} {b:.1f}" for a, b in zip(GROUPS, gm)))
        if v.get("touch"):
            tl = defaultdict(list)
            for nm, dm in v["touch"]:
                tl[nm].append(dm)
            top = sorted(tl.items(), key=lambda x: -len(x[1]))[:5]
            print("      last part to change T: " + "; ".join(f"{nm} {len(x) / n:.2f} (dm p50 {q(x):+.0f} m)"
                                                       for nm, x in top))
        if v.get("train"):
            print(f"      the live train: duration p50 {q(v['train']):.0f}, p90 {q(v['train'], 90):.0f} Myr;"
                  f" peak convergence p50 {q(v['cv']):.1f} mm/yr; distance to the front p50 {q(v['d']):.0f} km")


    # C. the pooled decay by Myr since the stamp, keel-less belts (the trace), else every belt sample (the maps)
    rows = []
    seas = []
    src = "trace"
    for run in runs:
        rec = json.loads((run / "rec.json").read_text())
        ts = np.array([s["t"] for s in rec["steps"]])
        sv = np.array([s.get("sea_m", 0.0) or 0.0 for s in rec["steps"]])
        atts = load_att(run)
        times = sorted(atts) if atts else sorted(
            float(p.stem[1:]) for p in (run / "maps").glob("t*.bin")) if (run / "maps").exists() else []
        for t in times:
            sea = float(sv[int(np.argmin(np.abs(ts - t)))])
            seas.append(sea)
            maps = load_maps(run, t)
            if atts:
                a = read_att(atts[t])
                m = (np.rint(a[:, 4]) == TAG) & (a[:, 7] < 100) & (a[:, 5] < 60)
                lat, lon, age, T, zs, L = a[m, 0], a[m, 1], a[m, 5], a[m, 2], a[m, 3] - sea, a[m, 6]
                gain = a[m][:, [18 + GROUPS.index(g) for g in ("flow", "stack", "collision", "other")]]
                held = (np.rint(a[:, 4]) >= 1) & (a[:, 5] < 10) & (a[:, 2] > T_C)
                young = unit(a[held, 0], a[held, 1])
                fin, fin_h = flow_inflow(a, np.flatnonzero(m))
                touch = np.column_stack([a[m, 14], a[m, 15], t - a[m, 16]])
            elif maps is not None:
                src = "maps (no keel or lithosphere: every belt sample, the sheet at eta0)"
                a = maps[0]
                m = (a[:, 4] > 0.5) & (np.rint(a[:, 5]) == TAG) & (a[:, 6] < 60)
                lat, lon, age, T, zs = a[m, 0], a[m, 1], a[m, 6], a[m, 3], a[m, 2] - sea
                L = np.full(len(T), L_HOT)
                gain = np.full((len(T), 4), np.nan)
                held = (a[:, 4] > 0.5) & (np.rint(a[:, 5]) >= 1) & (a[:, 6] < 10) & (a[:, 3] > T_C)
                young = unit(a[held, 0], a[held, 1])
                fin = fin_h = np.full(len(T), np.nan)
                touch = np.full((len(T), 3), np.nan)
            else:
                continue
            fr, _ = live_fronts(rec, maps, t)
            pts = unit(lat, lon)
            d = near_km(pts, fr)
            # the nearest column still in its quiet time and over T_c: lower-crustal flow's source (flow.rs)
            dy = near_km(pts, young)
            rows.append(np.column_stack([age, T, zs, L, np.full(len(T), sea), d, gain, dy, fin, fin_h, touch]))
    if not rows:
        return
    r = np.concatenate(rows)
    sea = float(np.median(seas))
    n_col = normal_col(sea)
    tc = max(T_C, n_col)
    rate = 1.0 - math.exp(-DT / COLLAPSE_MYR)

    def pace(T, L):
        """The collapse's thinning, km per Myr, of a column T (m) with lithosphere L (km): the larger of the
        threshold law and the sheet, as collapse.rs applies them (per step of DT)."""
        qt = rate * (T - tc) if T > tc else 0.0
        e = float(sheet_rate(np.array([T]), np.array([L]), sea)[0])
        qs = min(T * (1.0 - math.exp(-e * DT)), max(T - n_col, 0.0))
        return max(qt, qs) / DT / 1000.0

    print(f"\nC. collision-belt samples, every snapshot pooled ({len(r)}, from the {src}), by Myr since the stamp;"
          f" median sea {sea:.0f} m")
    print("  age: n, >2 km share, T p50 km, over the sea p50 m, L p50 km, eta/eta0 p50, sheet p50 %/Myr,"
          " share <= 400 km of a live front;")
    print("       the median T's fall, km/Myr (from the bins either side) / the collapse's pace at the bin's median T"
          " and L; mean gains since 45 km (flow, stack, collision, other), km")
    obs = {}
    for a0 in range(0, 60, 2):
        k = (r[:, 0] >= a0 - 0.5) & (r[:, 0] < a0 + 1.5)
        if k.sum() < 10:
            continue
        obs[a0] = (q(r[k, 1]), q(r[k, 2]), q(r[k, 3]), int(k.sum()), np.mean(r[k, 2] > 2000), np.mean(r[k, 5] <= 400),
                   np.nanmean(r[k, 6:10], axis=0) / 1000 if np.isfinite(r[k, 6:10]).any() else None)
    for a0, (T, zs, L, n, hi, near, g) in obs.items():
        eps = q(sheet_rate(r[:, 1][(r[:, 0] >= a0 - 0.5) & (r[:, 0] < a0 + 1.5)],
                           r[:, 3][(r[:, 0] >= a0 - 0.5) & (r[:, 0] < a0 + 1.5)], sea))
        fall = (obs[a0 - 2][0] - obs[a0 + 2][0]) / 4000.0 if a0 - 2 in obs and a0 + 2 in obs else float("nan")
        gs = "" if g is None else "  gains " + " ".join(f"{x:5.1f}" for x in g)
        print(f"  {a0:3d}: n {n:6d}  >2km {hi:.2f}  T {T / 1000:5.1f}  zs {zs:5.0f}  L {L:4.0f}"
              f"  eta {float(eta_factor(np.array(L))):5.2f}  sheet {100 * eps:5.2f}  near {near:.2f}"
              f"  fall {fall:5.2f} / {pace(T, L):5.2f}{gs}")
    # the observed medians' time from the 10-12 bin to each threshold against the collapse's own path
    k = (r[:, 0] >= 9.5) & (r[:, 0] < 13.5)
    T, L = q(r[k, 1]), q(r[k, 3])
    print(f"\n  the collapse's own path from age 10-12 (T {T / 1000:.1f} km, L {L:.0f} km held at each bin's observed"
          f" median), sea {sea:.0f} m: age: model T km / zs m  vs  observed p50")
    path = {}
    for a0 in range(10, 60, 2):
        path[a0] = T
        if a0 in obs and a0 % 4 == 2:
            zs = float(iso_z(np.array(T))) - sea
            print(f"  {a0:3d}: {T / 1000:5.1f} / {zs:5.0f}   vs  {obs[a0][0] / 1000:5.1f} / {obs[a0][1]:5.0f}")
        L = obs[a0][2] if a0 in obs else L
        T -= pace(T, L) * DT * 1000.0
    for thr in (50e3, 47e3, 45e3, 43e3, 41e3):
        tm = next((a for a in sorted(path) if path[a] <= thr), None)
        to = next((a for a in sorted(obs) if a >= 10 and obs[a][0] <= thr), None)
        if tm is not None and to is not None:
            print(f"  T to {thr / 1000:.0f} km: the path at age {tm}, observed at {to}: {(to - 10) / max(tm - 10, 1):.1f}"
                  f" times the path's time")

    # D. the samples still above 2 km against those under, past the quiet time
    print("\nD. past the quiet time, above 2 km over the sea against under: n, T p50 km, L p50 km, eta/eta0 p50,"
          " sea p50 m, sheet p50 %/Myr, <= 400 km of a live front, km to the nearest held column (quiet time, over"
          " T_c) p50, share within 200 km of one, mean gains since 45 km (flow, stack, collision, other)")
    for lo, hi in ((10, 20), (20, 30), (30, 50)):
        for up in (True, False):
            k = (r[:, 0] >= lo) & (r[:, 0] < hi) & ((r[:, 2] > 2000) == up)
            if k.sum() < 10:
                continue
            g = np.nanmean(r[k, 6:10], axis=0) / 1000 if np.isfinite(r[k, 6:10]).any() else None
            gs = "" if g is None else "  gains " + " ".join(f"{x:5.1f}" for x in g)
            eps = np.array([float(sheet_rate(np.array([a]), np.array([b]), c)[0]) for a, b, c in
                            zip(r[k, 1], r[k, 3], r[k, 4])])
            print(f"  {lo:2d}-{hi:2d} {'above' if up else 'under'}: n {int(k.sum()):6d}  T {q(r[k, 1]) / 1000:5.1f}"
                  f"  L {q(r[k, 3]):4.0f}  eta {q(eta_factor(r[k, 3])):5.2f}  sea {q(r[k, 4]):4.0f}"
                  f"  sheet {100 * q(eps):5.2f}  near {np.mean(r[k, 5] <= 400):.2f}  held km {q(r[k, 10]):5.0f}"
                  f"  <=200 {np.mean(r[k, 10] <= 200):.2f}{gs}")
            if np.isfinite(r[k, 11]).any():
                pc = np.array([pace(a, b) for a, b in zip(r[k, 1], r[k, 3])])
                fin, fh = r[k, 11], r[k, 12]
                print(f"         flow in p50 {q(fin):5.2f} km/Myr (mean {np.mean(fin):5.2f}; from held columns mean"
                      f" {np.mean(fh):5.2f}), the collapse's pace p50 {q(pc):5.2f} (mean {np.mean(pc):5.2f}); flow in"
                      f" >= half the pace {np.mean(fin >= 0.5 * pc):.2f}, >= the pace {np.mean(fin >= pc):.2f}")
            if np.isfinite(r[k, 13]).any():
                tp, tdm, tago = r[k, 13], r[k, 14], r[k, 15]
                parts = defaultdict(list)
                for p_, d_, a_ in zip(tp, tdm, tago):
                    parts[PARTS[int(p_)] if 0 <= p_ < len(PARTS) else "none"].append((d_, a_))
                top = sorted(parts.items(), key=lambda x: -len(x[1]))[:5]
                print("         last part to change T (>= 1 m while over 45 km): " + "; ".join(
                    f"{nm} {len(v) / k.sum():.2f} (dm p50 {q([x[0] for x in v]):+.0f} m, {q([x[1] for x in v]):.0f}"
                    f" Myr ago)" for nm, v in top))


if __name__ == "__main__":
    main()
