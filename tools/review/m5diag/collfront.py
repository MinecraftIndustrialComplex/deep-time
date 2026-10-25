#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""What sets the crust added per km of collision front (front lane, 2026-10-25), on a pool arm's cached runs.

    BAKEY=<pool key[,key]> [BAARM=<preset-length/argkey>] python3 -I collfront.py <out.jsonl> <workers> [run,run,...]

Per run (measurement only, cached folders, nothing simulated):
1. The crust budget (summary.txt, DEEPTIME_CRUST_BUDGET): the cb_globals lines (cumulative km3, km2,
   km x Myr over the run) at every snapshot, so a window's flux is the difference of two lines: what
   crossed the collision fronts, what the wedges and belts kept, the no-room losses, the toe, and the
   km x Myr of front (belts.rs's counters, PLT-401, PLT-601); and the dock tally's lines (dock_tally.rs,
   PLT-515: the front km x Myr and the km x Myr-weighted convergence by the collision state's age).
2. The record (rec.json): the collision states that ended (transfers and welds, `collisions`, and
   partings): their duration, shortening, crossed area, front, the convergence at the onset and in the
   last step; the belts' convergence 20 Myr after the onset over the onset's (India's 0.27, Copley et
   al. 2010); per step after -900 Myr the boundary mix's collision front, and the force budget of the
   plates with a collision resistance (FRC-018, FRC-030): its torque over the slab pull's and over the
   driving terms', and those plates' speeds.
3. The final grid (fields/ at t0000; backarc.py's frame and front pairing): the collision front points
   (trench cells on continental crust, the overriding side's map sample), each cell's distance to the
   nearest and its side (the overriding plate, the incoming plate, another). Per km of the record's last
   collision front (mix_km[2]): the excess crust over the normal column, sum max(0, T - 35.1 km) x area
   of the continental cells, by distance band and side (km3 per km of front: the belt's cross-section);
   the area of crust 45 km or thicker, 55 km or thicker, of land above 2 km and of the young collision
   class's land above 2 km (m4split's coll_young) within 800 km (km2 per km of front: widths); the mean
   thickness of the continental cells by distance band and side; and the kinematic convergence at the
   front points (backarc.kinematics: Vc = Vup + Vsub, trench-normal)."""
import json
import sys
from multiprocessing import Pool
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
import backarc as ba  # noqa: E402
from m5lib import cc  # noqa: E402
from m4split import CLASSES  # noqa: E402

T_N = 35_100.0  # the normal continental column, m (collapse::normal_thickness_m)
D_EDGES = [0.0, 100.0, 200.0, 400.0, 800.0, 1500.0]
SIDES = ("over", "under", "other")
GKEYS = ("crossed", "kept", "crossed_km2", "front_km_myr", "crossed_above", "crossed_recent", "kept_wedge",
         "crossed_restack", "noroom_sliver", "noroom_capped", "noroom_clipped", "taper_toe", "collapse_moved",
         "collapse_held", "collapse_unplaced", "channel_flow", "foundered", "settle_rise", "settle_fall")


def summary_part(run):
    glob, dock = [], {}
    for line in (run / "summary.txt").read_text().splitlines():
        if line.startswith("cb_globals "):
            row = {}
            for tok in line.split()[1:]:
                k, _, v = tok.partition("=")
                if k == "t" or k in GKEYS:
                    try:
                        row[k] = float(v)
                    except ValueError:
                        pass
            glob.append(row)
        elif line.startswith("dock_age="):
            v = dict(tok.partition("=")[::2] for tok in line.split())
            age = v["dock_age"]
            km = [float(x) for x in v["km_myr"].split(",")]
            means = [float(x) for x in v["means"].split(",")]
            dock[age] = {"km_myr": km, "means": means}
    return {"globals": glob, "dock": dock}


def record_part(rec):
    dt = rec["dt_myr"]
    out = {"states": [], "partings": [], "belts": []}
    for c in rec["collisions"]:
        if not c["collision"] and not c["sutured"]:
            continue
        dur = c["end_myr"] - c["start_myr"] + dt
        out["states"].append({"coll": bool(c["collision"]), "weld": bool(c["sutured"]), "start": c["start_myr"],
                              "dur": dur, "short_km": c["front_shortening_km"], "area_km2": c["area_km2"],
                              "front_km": c["front_km"], "onset": c.get("onset_convergence_mm_yr"),
                              "last": c["convergence_mm_yr"], "contact_age": c["start_myr"] - c["contact_onset_myr"]})
    for p in rec.get("partings", []):
        out["partings"].append({"start": p["start_myr"], "dur": p["end_myr"] - p["start_myr"] + dt,
                                "short_km": p["shortening_km"], "area_km2": p["area_km2"], "front_km": p["front_km"],
                                "peak": p["peak_convergence_mm_yr"], "reason": p["reason"]})
    for b in rec.get("belts", []):
        out["belts"].append({k: b.get(k) for k in ("start_myr", "end_myr", "welded", "convergence_onset_mm_yr",
                                                    "convergence_20myr_mm_yr", "peak_height_m", "belt_width_km",
                                                    "belt_length_km", "wedge_width_km")})
    st = [s for s in rec["steps"] if s["t"] >= -900.0]
    out["coll_km_mean"] = float(np.mean([s["mix_km"][2] for s in st]))
    out["coll_km_last"] = float(rec["steps"][-1]["mix_km"][2])
    out["coll_km_50"] = float(np.mean([s["mix_km"][2] for s in rec["steps"] if s["t"] >= -50.0]))
    # The force budget of the plates with a collision resistance (torque magnitudes, 1e24 N m).
    cr_pull, cr_drive, spd_cr, spd_free, n_cr = [], [], [], [], 0
    for s in st:
        for f in s.get("forces", []):
            pull, suct, edge, flow, cr = f[1], f[2], f[3], f[4], f[8]
            drive = pull + suct + edge + flow
            if cr > 0:
                n_cr += 1
                cr_pull.append(cr / pull if pull > 0 else np.inf)
                cr_drive.append(cr / drive if drive > 0 else np.inf)
                spd_cr.append(f[11])
            else:
                spd_free.append(f[11])
    cp, cd = np.array(cr_pull), np.array(cr_drive)
    out["forces"] = {"n_cr": n_cr, "cr_over_pull": ba.pct(cp[np.isfinite(cp)]) if cp.size else None,
                     "cr_no_pull": float(np.mean(~np.isfinite(cp))) if cp.size else None,
                     "cr_over_drive": ba.pct(cd[np.isfinite(cd)]) if cd.size else None,
                     "cr_over_drive_ge1": float(np.mean(cd >= 1.0)) if cd.size else None,
                     "speed_cr": ba.pct(np.array(spd_cr)) if spd_cr else None,
                     "speed_free": ba.pct(np.array(spd_free)) if spd_free else None}
    return out


def grid_part(G, q, plate, P, k):
    """Absolute totals (km3, km2); the reader divides by a front length."""
    p, cell_km2, T, cont = G["p"], G["cell_km2"], G["T"], G["cont"]
    sub = k["sub"]
    fc = P["o_map"][~sub]
    oc, uc = P["o"][~sub], P["u"][~sub]
    out = {"front_points": int(fc.size)}
    if fc.size == 0:
        return out
    dc, jc = cKDTree(q[fc]).query(p)
    d = cc.arc_km(dc)
    side = np.where(plate == oc[jc], 0, np.where(plate == uc[jc], 1, 2))
    ex = np.where(cont, np.maximum(T - T_N, 0.0), 0.0) / 1000.0  # km
    bands = np.searchsorted(np.array(D_EDGES[1:]), d, side="right")  # 0..4 inside 1500 km, 5 beyond
    exb, tb, nb = {}, {}, {}
    for s, sn in enumerate(SIDES):
        for b in range(len(D_EDGES) - 1):
            m = (side == s) & (bands == b)
            key = f"{sn}_{D_EDGES[b]:.0f}_{D_EDGES[b + 1]:.0f}"
            exb[key] = float(ex[m].sum() * cell_km2)
            mc = m & cont
            tb[key] = float(T[mc].mean() / 1000.0) if mc.any() else None
            nb[key] = float(mc.sum() * cell_km2)
    out["excess_km3"] = exb  # excess crust over the normal column, km3, by side and distance band
    out["T_mean_km"] = tb
    out["cont_km2"] = nb
    near = d <= 800.0
    out["excess_le800_km3"] = float(ex[near].sum() * cell_km2)
    out["excess_all_km3"] = float(ex.sum() * cell_km2)
    # Over the planet's own continental crust's median (Earth's normal is CRUST1.0's, about 40 km).
    t_med = float(np.median(T[cont]))
    t_med_far = float(np.median(T[cont & (d > 1500.0)])) if (cont & (d > 1500.0)).any() else t_med
    out["T_cont_median_km"] = t_med / 1000.0
    out["T_cont_median_far_km"] = t_med_far / 1000.0
    for nm, ref in (("med", t_med), ("far", t_med_far)):
        exo = np.where(cont, np.maximum(T - ref, 0.0), 0.0) / 1000.0
        out[f"excess_le800_km3_{nm}"] = float(exo[near].sum() * cell_km2)
    yc = G["cls"] == CLASSES.index("coll_young")
    for nm, m in (("t45", cont & (T >= 45_000.0)), ("t55", cont & (T >= 55_000.0)), ("hi", G["hi"]),
                  ("hi_coll_young", G["hi"] & yc), ("coll_young_land", G["land"] & yc)):
        out[f"{nm}_le800_km2"] = float((m & near).sum() * cell_km2)
        out[f"{nm}_all_km2"] = float(m.sum() * cell_km2)
        out[f"{nm}_le800_over_km2"] = float((m & near & (side == 0)).sum() * cell_km2)
    out["vc_front"] = ba.pct(k["vc"][~sub])
    out["vc_front_mean"] = float(k["vc"][~sub].mean())
    out["land_km2"] = float(G["land"].sum() * cell_km2)
    return out


def one(run):
    a = np.fromfile(run / "maps" / "t0000.bin", "<f4").reshape(-1, ba.FIELDS).astype(np.float64)
    pid = np.fromfile(run / "maps" / "plate" / "t0000.bin", "<u4").astype(np.int64)
    rec = json.loads((run / "rec.json").read_text())
    G = ba.load_grid(run)
    _, ls, lsign, off = ba.align(G["p"], G["cont"], a)
    q = cc.xyz(ls * a[:, 0], lsign * a[:, 1] + off)
    qt = cKDTree(q)
    _, nn = qt.query(G["p"])
    plate = pid[nn]
    P = ba.pair_fronts(G, q, cc.xyz(a[:, 0], a[:, 1]), qt, pid, plate)
    sub = ~G["cont"][P["tr"]]
    k = ba.kinematics(a, pid, rec, P, sub)
    out = {"run": run.name, "summary": summary_part(run)}
    out["rec"] = record_part(rec)
    out["grid"] = grid_part(G, q, plate, P, k)
    return out


def main():
    dest = Path(sys.argv[1])
    workers = int(sys.argv[2]) if len(sys.argv) > 2 else 4
    seen = {}
    for key in ba.KEY.split(","):
        for p in sorted((ba.POOL / key / ba.ARM).glob("*-*")):
            if (p / "maps" / "t0000.bin").exists() and p.name not in seen:
                seen[p.name] = p
    runs = sorted(seen.values(), key=lambda p: p.name)
    if len(sys.argv) > 3:
        runs = [p for p in runs if p.name in sys.argv[3].split(",")]
    print(f"{len(runs)} runs", flush=True)
    with Pool(workers) as pool, dest.open("w") as fh:
        for res in pool.imap_unordered(one, runs):
            fh.write(json.dumps(res) + "\n")
            fh.flush()
            g = res["grid"]
            L = res["rec"]["coll_km_50"]
            print(f"{res['run']}: coll km (last 50 Myr) {L:.0f} excess<=800 {g.get('excess_le800_km3', 0) / L:.0f}"
                  f" km3/km hi<=800 {g.get('hi_le800_km2', 0) / L:.0f} km2/km", flush=True)
    print("CF_DONE")


if __name__ == "__main__":
    main()
